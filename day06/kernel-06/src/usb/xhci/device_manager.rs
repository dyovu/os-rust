// ================================================================
// @file usb/xhci/device_manager.rs
//
// USB デバイスの管理機能．
// ================================================================


use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::usb::xhci::device::XhciDevice;
use crate::usb::device::Device;
use crate::usb::xhci::context::DeviceContext;
use crate::usb::memory_alloc::MEMORY_POOL;

pub struct DeviceManager{
    max_slots: usize,
    devices: Vec<Option<Box<Device<XhciDevice>>>>,
    device_context_addr: usize, // DCBAAの先頭のアドレス
}

impl DeviceManager{
    pub fn new(max_slots: usize) -> Self{
        // アラインメントとかの制約がないからvecで確保しちゃう
        let mut devices = Vec::new();
        devices.resize_with(max_slots + 1, || None);

        // 4kb境界とかのアラインメントを満たした領域を確保する
        // DCBAAPレジスタに書き込むアドレス（DeviceContext
        let device_context_addr  = match MEMORY_POOL.lock().alloc_array::<*mut DeviceContext>(max_slots + 1, 64, 4*1024){
            Some(t) => t as usize,
            None => {
                loop{}
            },
        };

        Self { 
            max_slots,
            devices,
            device_context_addr,
        }
    }

    pub fn device_context_addr(&self) -> usize{
        self.device_context_addr
    }

    pub fn find_by_slot(&self, slot_id: usize) -> Option<&Device<XhciDevice>>{
        if slot_id > self.max_slots {
            return None
        }
        let dev = self.devices[slot_id].as_deref();
        dev
    }

    pub fn find_by_slot_mut(&mut self, slot_id: usize) -> Option<&mut Device<XhciDevice>>{
        if slot_id > self.max_slots {
            return None
        }
        let dev = self.devices[slot_id].as_deref_mut();
        dev
    }
}