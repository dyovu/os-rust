// ================================================================
// @file usb/device.rs
//
// 全てのusbデバイスに共通の振る舞いの定義
// C++の仮想関数はトレイトとして実装する
// 全てのusb規格に共通のフィールドは構造体として定義する
// ================================================================

use alloc::boxed::Box;

use crate::usb::classdriver::base::ClassDriver;
use crate::usb::setupdata::{SetupData, request_type, request, descriptor_type};
use crate::usb::memory_alloc::ArrayMap;
use crate::usb::endpoint::{EndpointConfig, EndpointID};
use crate::usb::descriptor::{descriptor_dynamic_cast, DeviceDescriptor, ConfigurationDescriptor};

// 全ての規格のUSBが実装するべきメソッド
pub trait UsbDevice {
    fn control_in(&mut self, ep_id: EndpointID, setup_data: SetupData, buf: Option<&mut [u8]>) -> Result<(), ()>;
    fn control_out(&mut self, ep_id: EndpointID, setup_data: SetupData, buf: Option<&mut [u8]>) -> Result<(), ()>;
    fn interrupt_in(&mut self, ep_id: EndpointID, buf: &mut [u8]) -> Result<(), ()>;
    fn interrupt_out(&mut self, ep_id: EndpointID, buf: &mut [u8]) -> Result<(),  ()>;
}

// コントローラ転送のTransferEvent処理が終わった際にcontroller側に返すenum
// device固有の処理は各device構造体にやらせて、controllerの処理はcontroller側で行う
pub enum TransferEventResult {
    InterruptCompleted {
        ep_id: EndpointID,
        buffer: usize,
        transfer_length: u32,
    },
    ControlCompleted {
        ep_id: EndpointID,
        setup_data: SetupData,
        data_stage_buffer: usize,
        transfer_length: usize,
    },
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
    pub event_waiters: ArrayMap<SetupData, usize, 4>, // usizeはクラスドライバのアドレス
    pub num_configurations: u8,
    pub config_index: u8,
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

            // ゼロで初期化するだけで、実際の値はinitialize_phase1()の中でセットする
            num_configurations: 0,
            config_index: 0
        }
    }

    pub fn control_in(&mut self, ep_id: EndpointID, setup_data: SetupData, buf: Option<&mut [u8]>, issuer: Option<usize>){
        if let Some(class_driver_addr) = issuer{
            self.event_waiters.put(setup_data, class_driver_addr);
        }
        self.controller.control_in(ep_id, setup_data, buf);
    }

    pub fn control_out(&mut self, ep_id: EndpointID, setup_data: SetupData, buf: Option<&mut [u8]>, issuer: Option<usize>){
        if let Some(class_driver_addr) = issuer{
            self.event_waiters.put(setup_data, class_driver_addr);
        }
        self.controller.control_out(ep_id, setup_data, buf);
    }

    pub fn interrupt_in(&mut self, ep_id: EndpointID, buf: &mut [u8]){
        self.controller.interrupt_in(ep_id, buf);
    }

    pub fn interrupt_out(&mut self, ep_id: EndpointID, buf: &mut [u8]){
        self.controller.interrupt_out(ep_id, buf);
    }

    pub fn on_control_completed(&mut self, ep_id: EndpointID, setup_data: SetupData, data_stage_buffer: usize, transfer_length: usize){
        // log
        // err
        if self.is_initialized {
            if let Some(w) = self.event_waiters.get(&setup_data){
                todo!("クラスドライバを実装したら呼び出す")
            }
        }

        let buf_u8 = unsafe { core::slice::from_raw_parts(data_stage_buffer as *const u8, transfer_length) };

        // 初期化が終わってない場合、初期化していく
        match self.initialize_phase{
            1 => {
                if setup_data.request == request::GET_DESCRIPTOR{
                    if let Some(buf) = descriptor_dynamic_cast::<DeviceDescriptor>(buf_u8){
                        self.initialize_phase1(buf, buf_u8.len());
                    }
                }
            }
            2 => {
                if setup_data.request == request::GET_DESCRIPTOR{
                    if let Some(buf) = descriptor_dynamic_cast::<ConfigurationDescriptor>(buf_u8){
                        self.initialize_phase2(buf, buf_u8.len());
                    }
                }
            }
            3 => {
                if setup_data.request == request::SET_CONFIGURATION{
                    self.initialize_phase3(setup_data.value as u8); // 下位8bitだけ残す
                }
            }
            _ => {
                todo!("エラーハンドリングする")
            }
        }
    }

    pub fn on_interrupt_completed(&mut self, ep_id: EndpointID, data_stage_buffer: usize, transfer_length: u32){
        // log
        // err
        if let Some(w) = self.class_drivers[ep_id.number() as usize].as_deref() {
            // w.on_interrupt_completed(ep_id, data_stage_buffer, transfer_length);
            todo!("クラスドライバを実装したら呼び出す")
        }   
    }

    fn initialize_phase1(&mut self, device_desc: &DeviceDescriptor, len: usize){
        self.num_configurations = device_desc.num_configurations;
        self.config_index = 0;
        self.initialize_phase = 2;
        // log
        // err
    }

    fn initialize_phase2(&self, buf: &ConfigurationDescriptor, len: usize){
        
    }

    fn initialize_phase3(&self, config_value: u8){
        
    }
}