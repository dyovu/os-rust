// ================================================================
// @file usb/xhci/trb.rs
//
// エンドポイント設定に関する機能
// C++では配列と構造体を持つunionを定義していたが
// rustではunionのフィールドへのアクセスがunsafeになる
// そのため構造体のみ定義して、必要な時にinto_bytes()で変換する
// ================================================================

use modular_bitfield::prelude::*;

use crate::usb::xhci::context::{InputContext};
use crate::usb::endpoint::EndpointID;


/**
 * TRBの基本形
 */
#[bitfield(bits = 128)]
#[derive(Debug, Copy, Clone)]
pub struct TRB {
    parameter: u64,
    
    status: u32,

    pub cycle_bit: B1,
    evaluate_next_trb: B1,
    #[skip] __: u8,
    trb_type : B6,
    control: B16
}


/**
 * NormalTRB
 */
#[bitfield(bits = 128)]
#[derive(Debug, Copy, Clone)]
pub struct NormalTRB {
    data_buffer_pointer: u64,

    trb_transfer_length: B17,
    td_size: B5,
    interrupter_target: B10,

    cycle_bit: B1,
    evaluate_next_trb: B1,
    interrupt_on_short_packet: B1,
    no_snoop: B1,
    chain_bit: B1,
    interrupt_on_completion: B1,
    immediate_data: B1,
    #[skip] __: B2,
    block_event_interrupt: B1,
    trb_type : B6,
    #[skip] __: B16
}

impl NormalTRB{
    pub const TYPE: u8 = 1;

    pub fn initialize() -> Self {
        let mut trb = NormalTRB::new();
        trb.set_trb_type(NormalTRB::TYPE);
        trb
    }

    pub fn pointer(&self) -> *const TRB {
        self.data_buffer_pointer() as *const TRB
    }

    pub fn set_pointer(&mut self, p: *const TRB) {
        self.set_data_buffer_pointer(p as u64);
    }
}


/**
 * SetupStageTRB
 */
#[bitfield(bits = 128)]
#[derive(Debug, Copy, Clone)]
pub struct SetupStageTRB {
    request_type: u8,
    request: u8,
    value: u16,

    index: u16,
    length: u16,

    trb_transfer_length: B17,
    #[skip] __ : B5,
    interrupter_target: B10,

    cycle_bit: B1,
    #[skip] __: B4,
    interrupt_on_completion: B1,
    immediate_data: B1,
    #[skip] __: B3,
    trb_type : B6,
    transfer_type: B2,
    #[skip] __: B14,
}

impl SetupStageTRB{
    pub const TYPE: u8 = 2;
    const NO_DATA_STAGE:u8 = 0;
    const OUT_DATA_STAGE:u8 = 2;
    const IN_DATA_STAGE:u8 = 3;
    
    pub fn initialize() -> Self {
        let mut trb = SetupStageTRB::new();
        trb.set_trb_type(SetupStageTRB::TYPE);
        trb.set_immediate_data(1);
        trb.set_trb_transfer_length(8);
        trb
    }
}


/**
 * DataStageTRB
 */
#[bitfield(bits = 128)]
#[derive(Debug, Copy, Clone)]
pub struct DataStageTRB {
    data_buffer_pointer: u64,

    trb_transfer_length: B17,
    td_size : B5,
    interrupter_target: B10,

    cycle_bit: B1,
    evaluate_next_trb: B1,
    interrupt_on_short_packet: B1,
    no_snoop: B1,
    chain_bit: B1,
    pub interrupt_on_completion: B1,
    immediate_data: B1,
    #[skip] __: B3,
    trb_type : B6,
    direction: B1,
    #[skip] __: B15,
}

impl DataStageTRB{
    pub const TYPE: u8 = 3;

    pub fn initialize() -> Self {
        let mut trb = DataStageTRB::new();
        trb.set_trb_type(DataStageTRB::TYPE);
        trb
    }

    pub fn pointer(&self) -> *const TRB {
        self.data_buffer_pointer() as *const TRB
    }

    pub fn set_pointer(&mut self, p: *const TRB) {
        self.set_data_buffer_pointer(p as u64);
    }
}


/**
 * StatusStageTRB
 */
#[bitfield(bits = 128)]
#[derive(Debug, Copy, Clone)]
pub struct StatusStageTRB {
    #[skip] __: u64,

    #[skip] __: B22,
    interrupter_target: B10,

    cycle_bit: B1,
    evaluate_next_trb: B1,
    #[skip] __: B2,
    chain_bit: B1,
    interrupt_on_completion: B1,
    #[skip] __: B4,
    trb_type : B6,
    direction: B1,
    #[skip] __: B15,
}

impl StatusStageTRB{
    pub const TYPE: u8 = 4;
    
    pub fn initialize() -> Self {
        let mut trb = StatusStageTRB::new();
        trb.set_trb_type(StatusStageTRB::TYPE);
        trb
    }
}


/**
 * LinkTRB
 */
#[bitfield(bits = 128)]
#[derive(Debug, Copy, Clone)]
pub struct LinkTRB {
    #[skip] __: B4,
    ring_segment_pointer: B60,

    #[skip] __: B22,
    interrupter_target: B10,

    cycle_bit: B1,
    pub toggle_cycle: B1,
    #[skip] __: B2,
    chain_bit: B1,
    interrupt_on_completion: B1,
    #[skip] __: B4,
    trb_type : B6,
    #[skip] __: B16,
}

impl LinkTRB{
    pub const TYPE: u8 = 6;

    pub fn initialize(p: *const TRB) -> Self {
        let mut trb = LinkTRB::new();
        trb.set_trb_type(LinkTRB::TYPE);
        trb.set_pointer(p);
        trb
    }

    pub fn pointer(&self) -> *const TRB {
        (self.ring_segment_pointer() << 4) as *const TRB
    }

    pub fn set_pointer(&mut self, p: *const TRB) {
        self.set_ring_segment_pointer(p as u64 >> 4);
    }
}


/**
 * NoOpTRB
 */
#[bitfield(bits = 128)]
#[derive(Debug, Copy, Clone)]
pub struct NoOpTRB {
    #[skip] __: u64,

    #[skip] __: B22,
    interrupter_target: B10,

    cycle_bit: B1,
    evaluate_next_trb: B1,
    #[skip] __: B2,
    chain_bit: B1,
    interrupt_on_completion: B1,
    #[skip] __: B4,
    trb_type : B6,
    #[skip] __: B16,
}

impl NoOpTRB{
    pub const TYPE: u8 = 8;
    
    pub fn initialize() -> Self {
        let mut trb = NoOpTRB::new();
        trb.set_trb_type(NoOpTRB::TYPE);
        trb
    }
}


/**
 * EnableSlotCommandTRB
 */
#[bitfield(bits = 128)]
#[derive(Debug, Copy, Clone)]
pub struct EnableSlotCommandTRB {
    #[skip] __: u32,

    #[skip] __: u32,

    #[skip] __: u32,

    cycle_bit: B1,
    #[skip] __: B9,
    trb_type : B6,
    slot_type: B5,
    #[skip] __: B11,
}

impl EnableSlotCommandTRB{
    pub const TYPE: u8 = 9;
    
    pub fn initialize() -> Self {
        let mut trb = EnableSlotCommandTRB::new();
        trb.set_trb_type(EnableSlotCommandTRB::TYPE);
        trb
    }
}


/**
 * AddressDeviceCommandTRB
 */
#[bitfield(bits = 128)]
#[derive(Debug, Copy, Clone)]
pub struct AddressDeviceCommandTRB {
    #[skip] __: B4,
    input_context_pointer: B60,

    #[skip] __: u32,

    cycle_bit: B1,
    #[skip] __: B8,
    block_set_address_request: B1,
    trb_type: B6,
    #[skip] __: B8,
    slot_id: u8,
}

impl AddressDeviceCommandTRB {
    pub const TYPE: u8 = 11;

    pub fn initialize(input_context: *const InputContext, slot_id: u8) -> Self {
        let mut trb = AddressDeviceCommandTRB::new();
        trb.set_trb_type(AddressDeviceCommandTRB::TYPE);
        trb.set_slot_id(slot_id);
        trb.set_pointer(input_context);
        trb
    }

    pub fn pointer(&self) -> *const InputContext {
        (self.input_context_pointer() << 4) as *const InputContext
    }

    pub fn set_pointer(&mut self, p: *const InputContext) {
        self.set_input_context_pointer(p as u64 >> 4);
    }
}


/**
 * ConfigureEndpointCommandTRB
 */
#[bitfield(bits = 128)]
#[derive(Debug, Copy, Clone)]
pub struct ConfigureEndpointCommandTRB {
    #[skip] __: B4,
    input_context_pointer: B60,

    #[skip] __: u32,

    cycle_bit: B1,
    #[skip] __: B8,
    deconfigure: B1,
    trb_type: B6,
    #[skip] __: B8,
    slot_id: u8,
}

impl ConfigureEndpointCommandTRB {
    pub const TYPE: u8 = 12;

    pub fn initialize(input_context: *const InputContext, slot_id: u8) -> Self {
        let mut trb = ConfigureEndpointCommandTRB::new();
        trb.set_trb_type(ConfigureEndpointCommandTRB::TYPE);
        trb.set_slot_id(slot_id);
        trb.set_pointer(input_context);
        trb
    }

    pub fn pointer(&self) -> *const InputContext {
        (self.input_context_pointer() << 4) as *const InputContext
    }

    pub fn set_pointer(&mut self, p: *const InputContext) {
        self.set_input_context_pointer(p as u64 >> 4);
    }
}


/**
 * StopEndpointCommandTRB
 */
#[bitfield(bits = 128)]
#[derive(Debug, Copy, Clone)]
pub struct StopEndpointCommandTRB {
    #[skip] __: u32,

    #[skip] __: u32,

    #[skip] __: u32,

    cycle_bit: B1,
    #[skip] __: B9,
    trb_type: B6,
    endpoint_id: B5,
    #[skip] __: B2,
    suspend: B1,
    slot_id: u8,
}

impl StopEndpointCommandTRB {
    pub const TYPE: u8 = 15;

    pub fn initialize(endpoint_id: EndpointID, slot_id: u8) -> Self {
        let mut trb = StopEndpointCommandTRB::new();
        trb.set_trb_type(StopEndpointCommandTRB::TYPE);
        trb.set_endpoint_id(endpoint_id.address() as u8);
        trb.set_slot_id(slot_id);
        trb
    }

    pub fn get_endpoint_id(&self) -> EndpointID {
        EndpointID::from_addr(self.endpoint_id())
    }
}


/**
 * NoOpCommandTRB
 */
#[bitfield(bits = 128)]
#[derive(Debug, Copy, Clone)]
pub struct NoOpCommandTRB {
    #[skip] __: u32,

    #[skip] __: u32,

    #[skip] __: u32,

    cycle_bit: B1,
    #[skip] __: B9,
    trb_type: B6,
    #[skip] __: B16,
}

impl NoOpCommandTRB {
    pub const TYPE: u8 = 23;

    pub fn initialize() -> Self {
        let mut trb = NoOpCommandTRB::new();
        trb.set_trb_type(NoOpCommandTRB::TYPE);
        trb
    }
}


/**
 * TransferEventTRB
 */
#[bitfield(bits = 128)]
#[derive(Debug, Copy, Clone)]
pub struct TransferEventTRB {
    trb_pointer: u64,

    trb_transfer_length: B24,
    completion_code: u8,

    cycle_bit: B1,
    #[skip] __: B1,
    event_data: B1,
    #[skip] __: B7,
    trb_type: B6,
    endpoint_id: B5,
    #[skip] __: B3,
    pub slot_id: u8,
}

impl TransferEventTRB {
    pub const TYPE: u8 = 32;

    pub fn initialize() -> Self {
        let mut trb = TransferEventTRB::new();
        trb.set_trb_type(TransferEventTRB::TYPE);
        trb
    }

    pub fn pointer(&self) -> *const TRB {
        self.trb_pointer() as *const TRB
    }

    pub fn set_pointer(&mut self, p: *const TRB) {
        self.set_trb_pointer(p as u64);
    }

    pub fn get_endpoint_id(&self) -> EndpointID {
        EndpointID::from_addr(self.endpoint_id())
    }
}


/**
 * CommandCompletionEventTRB
 */
#[bitfield(bits = 128)]
#[derive(Debug, Copy, Clone)]
pub struct CommandCompletionEventTRB {
    #[skip] __: B4,
    command_trb_pointer: B60,

    command_completion_parameter: B24,
    completion_code: u8,

    cycle_bit: B1,
    #[skip] __: B9,
    trb_type: B6,
    vf_id: u8,
    slot_id: u8,
}

impl CommandCompletionEventTRB {
    pub const TYPE: u8 = 33;

    pub fn initialize() -> Self {
        let mut trb = CommandCompletionEventTRB::new();
        trb.set_trb_type(CommandCompletionEventTRB::TYPE);
        trb
    }

    pub fn pointer(&self) -> *const TRB {
        (self.command_trb_pointer() << 4) as *const TRB
    }

    pub fn set_pointer(&mut self, p: *mut TRB) {
        self.set_command_trb_pointer(p as u64 >> 4);
    }
}


/**
 * PortStatusChangeEventTRB
 */
#[bitfield(bits = 128)]
#[derive(Debug, Copy, Clone)]
pub struct PortStatusChangeEventTRB {
    #[skip] __: B24,
    port_id: u8,

    #[skip] __: u32,

    #[skip] __: B24,
    completion_code: u8,

    cycle_bit: B1,
    #[skip] __: B9,
    trb_type: B6,
    #[skip] __: B16,
}

impl PortStatusChangeEventTRB {
    pub const TYPE: u8 = 34;

    pub fn initialize() -> Self {
        let mut trb = PortStatusChangeEventTRB::new();
        trb.set_trb_type(PortStatusChangeEventTRB::TYPE);
        trb
    }
}

pub trait TrbTrait {
    const TYPE: u8;
}

macro_rules! impl_trb_trait {
    ($($t:ty),*) => {
        $(impl TrbTrait for $t { const TYPE: u8 = <$t>::TYPE; })*
    }
}

impl_trb_trait!(
    NormalTRB,
    SetupStageTRB,
    DataStageTRB,
    StatusStageTRB,
    LinkTRB,
    NoOpTRB,
    EnableSlotCommandTRB,
    AddressDeviceCommandTRB,
    ConfigureEndpointCommandTRB,
    StopEndpointCommandTRB,
    NoOpCommandTRB,
    TransferEventTRB,
    CommandCompletionEventTRB,
    PortStatusChangeEventTRB
);

pub unsafe fn trb_dynamic_cast<T: TrbTrait>(trb: *mut TRB) -> Option<&'static T> {
    if (*trb).trb_type() == T::TYPE {
        Some(&*(trb as *const T))
    } else {
        None
    }
}