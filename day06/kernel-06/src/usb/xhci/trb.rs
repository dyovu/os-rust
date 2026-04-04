// ================================================================
// @file usb/xhci/trb.rs
//
// エンドポイント設定に関する機能
// ================================================================

use modular_bitfield::prelude::*;


/**
 * TRBの基本形
 */
#[bitfield(bits = 128)]
#[derive(Debug, Copy, Clone)]
struct TRB {
    parameter: u64,
    status: u32,
    cycle_bit: B1,
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
struct NormalTRB {
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
    pub trb_type : B6,
    #[skip] __: B16
}

impl NormalTRB{
    pub fn initialize() -> Self {
        let mut trb = NormalTRB::new();
        trb.set_trb_type(1);
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
struct SetupStageTRB {
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
    pub trb_type : B6,
    transfer_type: B2,
    #[skip] __: B14,
}

impl SetupStageTRB{
    pub fn initialize() -> Self {
        let mut trb = SetupStageTRB::new();
        trb.set_trb_type(2);
        trb.set_immediate_data(1);
        trb.set_trb_transfer_length(8);
        trb
    }
}


