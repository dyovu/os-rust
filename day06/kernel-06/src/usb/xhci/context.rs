// ================================================================
// @file usb/xhci/context.rs
//
// xHCIのcontextの構造体
// ================================================================

use modular_bitfield::prelude::*;

use crate::usb::endpoint::EndpointID;
use crate::usb::xhci::trb::TRB;

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

impl EndpointContextBits {
    pub fn transfer_ring_buffer(&self) -> *mut TRB {
        (self.tr_dequeue_pointer() << 4) as *mut TRB
    }

    pub fn set_transfer_ring_buffer(&mut self, buffer: *const TRB) {
        self.set_tr_dequeue_pointer((buffer as u64) >> 4);
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
#[derive(Copy, Clone)]
pub struct DeviceContextIndex {
    pub value: u8,
}

impl DeviceContextIndex {
    pub fn new(dci: u8) -> Self {
        Self { value: dci }
    }

    pub fn from_endpoint_id(ep_id: EndpointID) -> Self {
        Self { value: ep_id.address() }
    }

    pub fn from_endpoint_num(ep_num: u8, dir_in: bool) -> Self {
        let direction = if ep_num == 0 { 1 } else if dir_in { 1 } else { 0 };
        Self { value: 2 * ep_num + direction }
    }
}

#[repr(C, align(64))]
#[derive(Copy, Clone)]
pub struct DeviceContext {
    pub slot_context: SlotContext,
    pub ep_contexts: [EndpointContext; 31],
}

impl DeviceContext {
    
}

// ================================================================
// InputContext
// ================================================================
#[repr(C, align(64))]
#[derive(Copy, Clone)]
pub struct InputControlContext {
    pub drop_context_flags: u32,
    pub add_context_flags: u32,
    pub _reserved1: [u32; 5],
    pub configuration_value: u8,
    pub interface_number: u8,
    pub alternate_setting: u8,
    pub _reserved2: u8,
}

#[repr(C, align(64))]
#[derive(Copy, Clone)]
pub struct InputContext {
    pub input_control_context: InputControlContext,
    pub slot_context: SlotContext,
    pub ep_contexts: [EndpointContext; 31],
}

impl InputContext {
    pub fn enable_slot_context(&mut self) -> &mut SlotContext {
        self.input_control_context.add_context_flags |= 1;
        &mut self.slot_context
    }

    pub fn enable_endpoint(&mut self, dci: DeviceContextIndex) -> &mut EndpointContext {
        self.input_control_context.add_context_flags |= 1 << dci.value;
        &mut self.ep_contexts[(dci.value - 1) as usize]
    }
}