// ================================================================
// @file usb/xhci/xhci_controller.rs
//
// xHCI ホストコントローラ制御用クラス．
// ================================================================

use crate::usb::xhci::registers::{CapabilityRegisters, OperationalRegisters, InterrupterRegisterSet, ArrayWrapper};
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
    const DeviceSize: usize = 8;

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
        
        Ok(())
    }

    fn cap_regs(&self) -> *const CapabilityRegisters{
        self.mmio_base as *const CapabilityRegisters
    }

    fn op_regs(&self) -> *mut OperationalRegisters {
        self.op_base as *mut OperationalRegisters
    }
}