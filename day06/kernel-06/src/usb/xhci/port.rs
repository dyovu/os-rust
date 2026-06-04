// ================================================================
// @file usb/xhci/port.rs
//
// xHCI の各ポートを表すクラスと周辺機能．
// ================================================================

use crate::usb::xhci::registers::PortRegisterSet;

pub struct Port {
    pub port_id: u8,
    pub port_reg_set_addr: usize,
}

impl Port {
    pub fn new(port_id: u8, port_reg_set_addr: usize) -> Self {
        Port {
            port_id,
            port_reg_set_addr,
        }
    }

    pub fn port_speed(&self) -> u8 {
        let port_reg_set = self.port_reg_set_addr as *const PortRegisterSet;
        unsafe { (*port_reg_set).PORTSC.read().port_speed() }
    }

    pub fn is_connected(&self) -> bool {
        let port_reg_set = self.port_reg_set_addr as *const PortRegisterSet;
        unsafe { (*port_reg_set).PORTSC.read().current_connect_status() == 1 }
    }

    pub fn is_enabled(&self) -> bool {
        let port_reg_set = self.port_reg_set_addr as *const PortRegisterSet;
        unsafe { (*port_reg_set).PORTSC.read().port_enabled_disabled() == 1 }
    }

    pub fn is_port_reset_changed(&self) -> bool {
        let port_reg_set = self.port_reg_set_addr as *const PortRegisterSet;
        unsafe { (*port_reg_set).PORTSC.read().port_reset_change() == 1 }
    }

    pub fn reset(&self) {
        let port_reg_set = self.port_reg_set_addr as *mut PortRegisterSet;
        unsafe {
            let mut portsc = (*port_reg_set).PORTSC.read();
            portsc.set_port_reset(1);
            portsc.set_connect_status_change(1);
            portsc.set_port_enabled_disabled_change(0);
            portsc.set_warm_port_reset_change(0);
            portsc.set_over_current_change(0);
            portsc.set_port_reset_change(0);
            portsc.set_port_link_state_change(0);
            portsc.set_port_config_error_change(0);
            (*port_reg_set).PORTSC.write(portsc);
            while (*port_reg_set).PORTSC.read().port_reset() != 0 {}
        }
    }

    pub fn clear_connect_status_changed(&self) {
        let port_reg_set = self.port_reg_set_addr as *mut PortRegisterSet;
        unsafe {
            let mut portsc = (*port_reg_set).PORTSC.read();
            portsc.set_connect_status_change(1);
            portsc.set_port_enabled_disabled_change(0);
            portsc.set_warm_port_reset_change(0);
            portsc.set_over_current_change(0);
            portsc.set_port_reset_change(0);
            portsc.set_port_link_state_change(0);
            portsc.set_port_config_error_change(0);
            (*port_reg_set).PORTSC.write(portsc);
        }
    }

    pub fn clear_port_reset_change(&self) {
        let port_reg_set = self.port_reg_set_addr as *mut PortRegisterSet;
        unsafe {
            let mut portsc = (*port_reg_set).PORTSC.read();
            portsc.set_connect_status_change(0);
            portsc.set_port_enabled_disabled_change(0);
            portsc.set_warm_port_reset_change(0);
            portsc.set_over_current_change(0);
            portsc.set_port_reset_change(1);
            portsc.set_port_link_state_change(0);
            portsc.set_port_config_error_change(0);
            (*port_reg_set).PORTSC.write(portsc);
        }
    }
}