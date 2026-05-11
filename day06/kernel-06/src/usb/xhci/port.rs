// ================================================================
// @file usb/xhci/port.rs
//
// xHCI の各ポートを表すクラスと周辺機能．
// ================================================================

use crate::usb::xhci::registers::PortRegisterSet;
pub struct Port{
    port_id: u8,
    port_reg_set_addr: usize,
}

impl Port{
    pub fn new(port_id: u8, port_reg_set_addr: usize) -> Self{
        Port{
            port_id,
            port_reg_set_addr,
        }
    }

    pub fn port_speed(&self) -> u8{
        let port_reg_set = self.port_reg_set_addr as *const PortRegisterSet;
        unsafe{
            (*port_reg_set).PORTSC.read().port_speed()
        }
    }
}