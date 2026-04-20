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

pub trait UsbDevice {
    fn control_in(&mut self, ep_id: EndpointID, setup_data: SetupData, buf: &mut [u8]) -> Result<(), ()>;
    fn control_out(&mut self, ep_id: EndpointID, setup_data: SetupData, buf: &[u8]) -> Result<(), ()>;
    fn interrupt_in(&mut self, ep_id: EndpointID, buf: &mut [u8]) -> Result<(), ()>;
    fn interrupt_out(&mut self, ep_id: EndpointID, buf: &mut [u8]) -> Result<(),  ()>;
}

// 共通フィールドの構造体
pub struct CommonDevice {
    pub initialize_phase: u8,
    pub is_initialized: bool,
    pub class_drivers: [Option<Box<dyn ClassDriver>>; 16],
    pub buf: [u8; 256],
    pub ep_configs: [EndpointConfig; 16],
    pub num_ep_configs: usize,
    pub event_waiters: ArrayMap<SetupData, Box<dyn ClassDriver>, 4>,
}