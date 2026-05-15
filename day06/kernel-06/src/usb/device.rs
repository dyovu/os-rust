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
use crate::usb::endpoint::{EndpointConfig, EndpointID, DEFAULT_CONTROL_PIPE_ID};
use crate::usb::descriptor::{descriptor_dynamic_cast, Descriptor, DeviceDescriptor, ConfigurationDescriptor, InterfaceDescriptor, EndpointDescriptor, HIDDescriptor};

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

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct ConfigurationDescriptorReader<'a> {
    // 「現在位置から末尾まで」を表すスライス
    // p_ と (desc_buf_ + desc_buf_len_ - p_) の情報が1つに収まっている
    remaining: &'a [u8],
}

impl<'a> Iterator for ConfigurationDescriptorReader<'a>{
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        // current が None なら即終了（? がNoneを返す）
        if self.remaining.is_empty() {
            return None;
        }
        let len = self.remaining[0] as usize; // 現在のディスクリプタの長さ
        let current = &self.remaining[..len]; // 現在のディスクリプタ
        self.remaining = &self.remaining[len..]; // remainingを1ディスクリプタ分進める
        Some(current)
    }
}

impl<'a> ConfigurationDescriptorReader<'a>{
    pub fn new(remaining: &'a [u8]) -> Self {
        Self { remaining }
    }
    // find_mapは指定したdescriptorの最初の要素を返す、なければNoneを返す
    // ConfigurationDescriptorReaderがiteratorトレイトを実装しているから使える
    pub fn next_typed<T: Descriptor>(&mut self) -> Option<&T> {
        self.find_map(|buf| descriptor_dynamic_cast::<T>(buf))
    }
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

        let buf_u8: &[u8] = unsafe { core::slice::from_raw_parts(data_stage_buffer as *const u8, transfer_length) };

        // 初期化が終わってない場合、初期化していく
        match self.initialize_phase{
            1 => {
                if setup_data.request == request::GET_DESCRIPTOR{
                    if let Some(buf) = descriptor_dynamic_cast::<DeviceDescriptor>(buf_u8){
                        self.initialize_phase1(buf);
                    }
                }
            }
            2 => {
                if setup_data.request == request::GET_DESCRIPTOR{
                    if let Some(buf) = descriptor_dynamic_cast::<ConfigurationDescriptor>(buf_u8){
                        self.initialize_phase2(buf_u8);
                    }
                }
            }
            3 => {
                if setup_data.request == request::SET_CONFIGURATION{
                    self.initialize_phase3(setup_data.value as u8); // 下位8bitだけ残す
                }
            }
            _ => {
                // err
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

    fn initialize_phase1(&mut self, device_desc: &DeviceDescriptor) -> Result<(), ()> {
        self.num_configurations = device_desc.num_configurations;
        self.config_index = 0;
        self.initialize_phase = 2;

        return self.get_descriptor(ConfigurationDescriptor::TYPE);
        // log
        // err
    }

    fn initialize_phase2(&self, buf_u8: &[u8]){
        let conf_desc = descriptor_dynamic_cast::<ConfigurationDescriptor>(buf_u8).unwrap_or_else(return);
        let mut config_reader = ConfigurationDescriptorReader::new(buf_u8);
        let mut class_driver = None;

        while let Some(if_desc) = config_reader.next_typed::<InterfaceDescriptor>(){
            class_driver = self.new_class_driver(if_desc);

            let num_ep_configs = 0;
            while num_ep_configs < if_desc.num_endpoints{
                let desc = config_reader.next().unwrap_or_else(break); // なんでこれunreachable?
                if let Some(ep_desc) = descriptor_dynamic_cast::<EndpointDescriptor>(desc){
                    let config = self.make_config(ep_desc);
                    self.ep_configs[num_ep_configs as usize] = config;
                    num_ep_configs += 1;
                    self.class_drivers[config.ep_id.number() as usize] = class_driver;
                }else if let Some(hid_desc) = descriptor_dynamic_cast::<HIDDescriptor>(desc){
                    // log
                }
            }

            // 最初に対応したインターフェースが見つかったらbreakする
            break;
        }

        if class_driver == None {
            // log
            // err
            return 
        }
        self.initialize_phase = 3;
        self.set_configuration(conf_desc.configuration_value);
    }

    fn initialize_phase3(&self, config_value: u8){
        for i in 0..self.num_ep_configs {
            let index = self.ep_configs[i].ep_id.number() as usize;
            if let  Some(class_driver) = self.class_drivers[index].as_deref(){
                class_driver.set_endpoint(self.ep_configs[i]);
            }
        }
        self.initialize_phase = 4;
        self.is_initialized = true;
        // err
    }

    fn new_class_driver(&self, if_desc: &InterfaceDescriptor) -> Option<> {

        None
    }

    fn make_config(&self, ep_desc: &EndpointDescriptor) -> EndpointConfig {
        let mut ep_conf = EndpointConfig::default();
        ep_conf.ep_id = EndpointID::from_parts(ep_desc.endpoint_address.number(), ep_desc.endpoint_address.dir_in() == 1);
        ep_conf.max_packet_size = ep_desc.max_packet_size;
        ep_conf.interval = ep_desc.interval;
        ep_conf
    }

    fn get_descriptor(&mut self, desc_type: u8) -> Result<(), ()> {
        let mut setup_data = SetupData::default();
        setup_data.request_type.set_direction(request_type::DIR_IN);
        setup_data.request_type.set_ty(request_type::TYPE_STANDARD);
        setup_data.request_type.set_recipient(request_type::RECIPIENT_DEVICE);
        setup_data.request = request::GET_DESCRIPTOR;
        setup_data.value = ((desc_type as u16) << 8) | self.config_index as u16;
        setup_data.index = 0;
        setup_data.length = self.buf.len() as u16;
        
        return Self::control_in_raw(
            &mut self.controller,
            &mut self.event_waiters,
            DEFAULT_CONTROL_PIPE_ID,
            setup_data,
            Some(&mut self.buf),
            None,
        );
    }

    fn control_in_raw(
        controller: &mut C,
        event_waiters: &mut ArrayMap<SetupData, usize, 4>,
        ep_id: EndpointID,
        setup_data: SetupData,
        buf: Option<&mut [u8]>,
        issuer: Option<usize>,
    ) -> Result<(), ()> {
        if let Some(addr) = issuer {
            event_waiters.put(setup_data, addr);
        }
        return controller.control_in(ep_id, setup_data, buf);
    }

    fn set_configuration(&mut self, config_value: u8) {
        let mut setup_data = SetupData::default();
        setup_data.request_type.set_direction(request_type::DIR_OUT);
        setup_data.request_type.set_ty(request_type::TYPE_STANDARD);
        setup_data.request_type.set_recipient(request_type::RECIPIENT_DEVICE);
        setup_data.request = request::SET_CONFIGURATION;
        setup_data.value = config_value as u16;
        setup_data.index = 0;
        setup_data.length = 0;

        Self::control_out_raw(
            &mut self.controller,
            &mut self.event_waiters,
            DEFAULT_CONTROL_PIPE_ID,
            setup_data,
            None,
            None,
        );
    }

    fn control_out_raw(
        controller: &mut C,
        event_waiters: &mut ArrayMap<SetupData, usize, 4>,
        ep_id: EndpointID,
        setup_data: SetupData,
        buf: Option<&mut [u8]>,
        issuer: Option<usize>,
    ) {
        if let Some(addr) = issuer {
            event_waiters.put(setup_data, addr);
        }
        controller.control_out(ep_id, setup_data, buf);
    }
}