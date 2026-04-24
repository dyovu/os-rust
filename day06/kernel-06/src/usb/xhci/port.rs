// ================================================================
// @file usb/xhci/port.rs
//
// xHCI の各ポートを表すクラスと周辺機能．
// ================================================================

use crate::usb::xhci::registers::PortRegisterSet;
pub struct Port{
    port_id: u8,
    port_reg_set: &PortRegisterSet,
}

impl Port{
    pub fn new(port_id: u8, port_reg_set: &PortRegisterSet) -> Self{
        Port{
            port_id,
            port_reg_set,
        }
    }
}