// ================================================================
// @file usb/xhci/xhci_controller.rs
//
// xHCI ホストコントローラ制御用クラス．
// ================================================================

use crate::usb::xhci::registers::{CapabilityRegisters, OperationalRegisters, InterrupterRegisterSet, ExtendedRegisterList, UsblegsupRegister, Dcbaap, ArrayWrapper};
use crate::usb::xhci::device_manager::{DeviceManager};
use crate::usb::xhci::ring::{Ring, EventRing};
use crate::usb::xhci::context::DeviceContext;
use crate::usb::memory_alloc::MEMORY_POOL;

/*
 * 生ポインタを構造体に持たせるのは避ける
 * 生ポインタはライフタイムを持たない
 * usizeで持つことでunsafeの範囲を最小限にできる
 */ 
pub struct Controller {
    mmio_base: usize,
    op_base: usize,
    max_ports: u8,
    device_manager: DeviceManager,
    cr: Ring,
    er: EventRing,
}

impl Controller {
    const DEVICE_SIZE: u8 = 8;

    pub fn new(mmio_base: usize) -> Self {
        let (max_ports, cap_len, rtsoff) = unsafe {
            let cap = &*(mmio_base as *const CapabilityRegisters);
            (
                cap.HCSPARAMS1.read().max_ports(),
                cap.CAPLENGTH.read() as usize,
                cap.RTSOFF.read().runtime_register_space_offset() as usize,
            )
        };

        let primary_interrupter = unsafe {
            ArrayWrapper::<InterrupterRegisterSet>::new(mmio_base + rtsoff + 0x20, 1024).get_mut(0)
        };

        let er = EventRing::new(32, primary_interrupter);

        Self {
            mmio_base,
            op_base: mmio_base + cap_len,
            max_ports,
            device_manager: DeviceManager::new(8),
            cr: Ring::new(32),
            er,
        }
    }

    pub fn initialize(&mut self) -> Result<(), ()>{
        self.request_HC_ownership();

        // usbコマンドの設定
        let mut usbcmd = unsafe{ (*self.op_regs()).USBCMD.read() };
        usbcmd.set_interrupter_enable(false as u8);
        usbcmd.set_host_system_error_enable(false as u8);
        usbcmd.set_enable_wrap_event(false as u8);

        // usbcmdを書き込む前に、ホストコントローラが停止してなかったら止める
        if unsafe{ (*self.op_regs()).USBSTS.read().host_controller_halted() } == 0{
            usbcmd.set_run_stop(false as u8);
        }

        // 設定を書き込む
        unsafe{ (*self.op_regs()).USBCMD.write(usbcmd) };
        // 動き出すまで待つ
        while unsafe{ (*self.op_regs()).USBSTS.read().host_controller_halted() } == 0{
            continue
        }

        // 
        let mut usbcmd = unsafe{ (*self.op_regs()).USBCMD.read() };
        usbcmd.set_host_controller_reset(true as u8);
        unsafe{ (*self.op_regs()).USBCMD.write(usbcmd) };

        while unsafe{ (*self.op_regs()).USBCMD.read().host_controller_reset() } == 1 
            || unsafe{ (*self.op_regs()).USBSTS.read().controller_not_ready() } == 1 {
            continue
        }

        // MaxSlotsの設定
        let mut config = unsafe{ (*self.op_regs()).CONFIG.read() };
        config.set_max_device_slots_enabled(Controller::DEVICE_SIZE);
        unsafe{ (*self.op_regs()).CONFIG.write(config) };

        // xHCIコントローラが内部処理のために使うプライベートなメモリ領域の確保と、レジスタへの割り当て
        let mut hcsparams2 = unsafe{ (*self.cap_regs()).HCSPARAMS2.read() };
        let max_scratchpad_buffers = hcsparams2.max_scratchpad_buffers_low() | ((hcsparams2.max_scratchpad_buffers_high() << 5));
        if (max_scratchpad_buffers > 0) {
            let scratchpad_buf_arr = match MEMORY_POOL.lock().alloc_array::<usize>(max_scratchpad_buffers as  usize, 64, 4*1024){
                Some(t) => {
                    t
                }
                None => {
                    loop{}
                }
            };
            // 2. 各バッファ本体を確保して配列に書き込む
            for i in 0..max_scratchpad_buffers as usize {
                let buf = match MEMORY_POOL.lock().alloc_mem(4096, 4096, 4096) {
                    Some(t) => t,
                    None => loop {},
                };
                unsafe { *(scratchpad_buf_arr as *mut usize).add(i) = buf };
            }

            // 3. DCBAA[0]にポインタ配列のアドレスを書く
            unsafe {
                self.device_manager.set_device_context_addr(*scratchpad_buf_arr);
            };

        }

        // DCBAAPにdevice contextのアドレスを設定する
        let mut dcbaap = Dcbaap::new();
        let addr = self.device_manager.device_context_addr() as u64;
        dcbaap.set_device_context_base_address_array_pointer((addr >> 6) as u32);
        unsafe { (*self.op_regs()).DCBAAP.write(dcbaap) };

        //
        let rtsoff = unsafe{ (*self.cap_regs()).RTSOFF.read().runtime_register_space_offset() as usize };
        let primary_interrupter = unsafe {
            ArrayWrapper::<InterrupterRegisterSet>::new(self.mmio_base + rtsoff + 0x20, 1024).get_mut(0)
        };

        let mut iman = unsafe{ (*primary_interrupter).IMAN.read() };
        iman.set_interrupt_pending(true as u8);
        iman.set_interrupt_enable(true as u8);
        unsafe{ (*primary_interrupter).IMAN.write(iman) };

        let mut usbcmd = unsafe{ (*self.op_regs()).USBCMD.read() };
        usbcmd.set_interrupter_enable(true as u8);
        unsafe{ (*self.op_regs()).USBCMD.write(usbcmd) };

        Ok(())
    }

    fn cap_regs(&self) -> *const CapabilityRegisters{
        self.mmio_base as *const CapabilityRegisters
    }

    fn op_regs(&self) -> *mut OperationalRegisters {
        self.op_base as *mut OperationalRegisters
    }

    fn request_HC_ownership(&self){
        let hccp = unsafe{ (*self.cap_regs()).HCCPARAMS1.read().xhci_extended_capabilities_pointer() as usize };
        let ext_regs = unsafe{ ExtendedRegisterList::new(self.mmio_base, hccp) };

        let ext_usblegsup = match ext_regs.iter().find(|&x| unsafe{ (*x).reg.read().capability_id() ==1 }){
            Some(t) => t,
            None => {
                loop{}
            }
        };

        let usb_leg_reg_ptr  = (ext_usblegsup as usize) as * mut UsblegsupRegister;
        let usb_leg_reg = unsafe{ &*usb_leg_reg_ptr };
        let mut r = usb_leg_reg.reg.read();

        if r.hc_os_owned_semaphore() == 1{
            return
        }

        // OSがホストコントローラの権限を持つように設定
        r.set_hc_os_owned_semaphore(1);
        unsafe{ (*usb_leg_reg_ptr).reg.write(r) };
        
        // BIOS
        while r.hc_os_owned_semaphore() == 0 || r.hc_bios_owned_semaphore() == 1{
            r = unsafe { (*usb_leg_reg_ptr).reg.read() };
        }
    }
}