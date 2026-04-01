// ================================================================
// @file usb/xhci/xhci_controller.rs
//
// xHCI ホストコントローラ制御用クラス．
// ================================================================

use crate::usb::xhci::registers::{CapabilityRegisters, OperationalRegisters};

/*
 * 生ポインタを構造体に持たせるのは避ける
 * 生ポインタはライフタイムを持たない
 * usizeで持つことでunsafeの範囲を最小限にできる
 */ 
pub struct Controller {
    mmio_base: usize,
    pub max_ports: u8,
}

impl Controller {
    pub fn new(mmio_base: usize) -> Self{
        let max_ports = unsafe {
            let hcsparams1 = (*(mmio_base as *const CapabilityRegisters)).HCSPARAMS1;
            (hcsparams1 >> 24) as u8
        };
        Self{
            mmio_base,
            max_ports,
        }
    }

    pub fn initialize() -> Result<(), ()>{

        Ok(())
    }

    fn cap_regs(&self) -> *const CapabilityRegisters{
        self.mmio_base as *const CapabilityRegisters
    }

    fn op_regs(&self) -> *mut OperationalRegisters {
        let cap_len = unsafe { (*self.cap_regs()).CAPLENGTH };
        (self.mmio_base + cap_len as usize) as *mut OperationalRegisters
    }
}