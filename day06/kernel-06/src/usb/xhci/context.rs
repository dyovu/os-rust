// ================================================================
// @file usb/xhci/context.rs
//
// xHCIのcontextの構造体
// ================================================================

use modular_bitfield::prelude::*;

// ================================================================
// SlotContext
// ================================================================
#[bitfield(bits = 128)]
#[derive(Debug, Copy, Clone)]
pub struct SlotContextBits {
    pub route_string: B20,
    pub speed: B4,
    #[skip] __: B1,
    pub mtt: B1,
    pub hub: B1,
    pub context_entries: B5,

    pub max_exit_latency: B16,
    pub root_hub_port_num: B8,
    pub num_ports: B8,

    pub tt_hub_slot_id: B8,
    pub tt_port_num: B8,
    pub ttt: B2,
    #[skip] __: B4,
    pub interrupter_target: B10,

    pub usb_device_address: B8,
    #[skip] __: B19,
    pub slot_state: B5,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub union SlotContext {
    pub dwords: [u32; 8],
    pub bits: SlotContextBits,
}

// ================================================================
// EndPointContext
// ================================================================
#[bitfield(bits = 160)]
#[derive(Debug, Copy, Clone)]
pub struct EndpointContextBits {
    pub ep_state: B3,
    #[skip] __: B5,
    pub mult: B2,
    pub max_primary_streams: B5,
    pub linear_stream_array: B1,
    pub interval: B8,
    pub max_esit_payload_hi: B8,

    #[skip] __: B1,
    pub error_count: B2,
    pub ep_type: B3,
    #[skip] __: B1,
    pub host_initiate_disable: B1,
    pub max_burst_size: B8,
    pub max_packet_size: B16,

    pub dequeue_cycle_state: B1,
    #[skip] __: B3,
    pub tr_dequeue_pointer: B60,

    pub average_trb_length: B16,
    pub max_esit_payload_lo: B16,
}

impl EndpointContextBits{
    pub fn TransferRingBuffer(){

    }

    pub fn SetTransferRingBuffer(buffer: &TBR){

    }
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub union EndpointContext {
    pub dwords: [u32; 8],
    pub bits: EndpointContextBits,
}

// ================================================================
// DeviceContext
// ================================================================
struct DeviceContextIndex{
    value: i32,
}

impl DeviceContextIndex{

}

#[repr(C, align(64))]
#[derive(Copy, Clone)]
pub struct DeviceContext{
    slot_context: SlotContext,
    ep_contexts: [EndpointContext; 31],
}

impl DeviceContext{
    
}

// ================================================================
// InputContext
// ================================================================
#[repr(C, align(64))]
#[derive(Copy, Clone)]
pub struct InputControlContext {
    drop_context_flags: u32,
    add_context_flags: u32,
    _reserved1 : [u32; 5],
    configuration_value: u8,
    interface_number: u8,
    alternate_setting: u8,
    _reserved2: u8,
}

#[repr(C, align(64))]
#[derive(Copy, Clone)]
pub struct InputContext{
    input_controll_context: InputControlContext,
    slot_context: SlotContext,
    ep_contexts: [EndpointContext; 31],
}

impl InputContext{
    

}