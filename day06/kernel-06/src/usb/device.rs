// ================================================================
// @file usb/device.rs
//
// 全てのusbデバイスに共通の振る舞いの定義
// C++の仮想関数はトレイトとして実装する
// 全てのusb規格に共通のフィールドは構造体として定義する
// ================================================================

use alloc::boxed::Box;

use crate::usb::classdriver::base::ClassDriver;
use crate::usb::setupdata::SetupData;
use crate::usb::memory_alloc::ArrayMap;
use crate::usb::endpoint::{EndpointConfig, EndpointID};

// 全ての規格のUSBが実装するべきメソッド
pub trait UsbDevice {
    fn control_in(&mut self, ep_id: EndpointID, setup_data: SetupData, buf: Option<&mut [u8]>) -> Result<(), ()>;
    fn control_out(&mut self, ep_id: EndpointID, setup_data: SetupData, buf: Option<&mut [u8]>) -> Result<(), ()>;
    fn interrupt_in(&mut self, ep_id: EndpointID, buf: &mut [u8]) -> Result<(), ()>;
    fn interrupt_out(&mut self, ep_id: EndpointID, buf: &mut [u8]) -> Result<(),  ()>;
}

// 共通フィールドの構造体
// 上記のUsbDeviceトレイトを実装した型をフィールドに持つ
// これによりxHCIなどに固有な操作を行う
pub struct Device<C: UsbDevice> {
    pub controller: C,
    pub initialize_phase: u8,
    pub is_initialized: bool,
    pub class_drivers: [Option<Box<dyn ClassDriver>>; 16],
    pub buf: [u8; 256],
    pub ep_configs: [EndpointConfig; 16],
    pub num_ep_configs: usize,
    pub event_waiters: ArrayMap<SetupData, usize, 4>, // usizeはep_idのNumber()
}

impl <C: UsbDevice> Device<C>{
    pub fn new(controller: C) -> Self{
        Self{
            controller,
            initialize_phase: 1,
            is_initialized: false,
            class_drivers: core::array::from_fn(|_| None),
            buf: [0; 256],
            ep_configs: core::array::from_fn(|_| EndpointConfig::default()),
            num_ep_configs: 0,
            event_waiters: ArrayMap::new(),
        }
    }

    pub fn control_in(&mut self, ep_id: EndpointID, setup_data: SetupData, buf: &mut [u8], issuer: Option<usize>){
        if let Some(class_driver_addr) = issuer{
            self.event_waiters.put(setup_data, class_driver_addr);
        }
        self.controller.control_in(ep_id, setup_data, buf);
    }
}