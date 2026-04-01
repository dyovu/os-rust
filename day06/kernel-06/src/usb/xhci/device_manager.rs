// ================================================================
// @file usb/xhci/device_manager.rs
//
// USB デバイスの管理機能．
// ================================================================


use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::vec;

use crate::usb::xhci::device::Device;
use crate::usb::xhci::context::DeviceContext;

pub struct DeviceManager{
    max_slots: usize,
    devices:Vec<Option<Box<Device>>>,
    device_context: Vec<Option<Box<DeviceContext>>>,
}

impl DeviceManager{
    pub fn new(max_slots: usize) -> Self{
        let mut devices = Vec::new();
        devices.resize_with(max_slots + 1, || None);

        let mut device_context = Vec::new();
        device_context.resize_with(max_slots + 1, || None);
        Self { 
            max_slots,
            devices,
            device_context,
        }
    }
}

