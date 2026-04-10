// ================================================================
// @file usb/xhci/xhci_controller.rs
//
// xHCI ホストコントローラ制御用クラス．
// ================================================================

use crate::usb::xhci::registers::{CapabilityRegisters, OperationalRegisters};
use crate::usb::xhci::device_manager::{DeviceManager};

/*
 * 生ポインタを構造体に持たせるのは避ける
 * 生ポインタはライフタイムを持たない
 * usizeで持つことでunsafeの範囲を最小限にできる
 */ 
pub struct Controller {
    mmio_base: usize,
    op_base: usize,
    max_ports: u8,
    device_manager: DeviceManager
}

impl Controller {
    pub fn new(mmio_base: usize) -> Self{
        let max_slots:usize = 8;
        let (max_ports, cap_len) = unsafe {
            let cap = &*(mmio_base as *const CapabilityRegisters);
            (cap.HCSPARAMS1.read().max_ports(), cap.CAPLENGTH.read())
        };
        Self{
            mmio_base,
            op_base: mmio_base + cap_len as usize,
            max_ports,
            device_manager: DeviceManager::new(max_slots),
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