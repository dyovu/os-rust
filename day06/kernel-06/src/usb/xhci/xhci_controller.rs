// ================================================================
// @file usb/xhci/xhci_controller.rs
//
// xHCI ホストコントローラ制御用クラス．
// ================================================================

use crate::usb::xhci::registers::{CapabilityRegisters, OperationalRegisters, InterrupterRegisterSet, ExtendedRegisterList, UsblegsupRegister, ArrayWrapper};
use crate::usb::xhci::device_manager::{DeviceManager};
use crate::usb::xhci::ring::{Ring, EventRing};

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
    const DEVICE_SIZE: usize = 8;

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

    pub fn initialize(&self) -> Result<(), ()>{
        self.request_HC_ownership();
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