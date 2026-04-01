// ================================================================
// @file usb/xhci/context.rs
//
// xHCIのcontextの構造体
// ================================================================

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct SlotContextBits {
    // dword 0
    // [0:19]   route_string
    // [20:23]  speed
    // [24]     reserved
    // [25]     mtt
    // [26]     hub
    // [27:31]  context_entries
    pub dword0: u32,

    // dword 1
    // [0:15]   max_exit_latency
    // [16:23]  root_hub_port_num
    // [24:31]  num_ports
    pub dword1: u32,

    // dword 2
    // [0:7]    tt_hub_slot_id
    // [8:15]   tt_port_num
    // [16:17]  ttt
    // [18:21]  reserved
    // [22:31]  interrupter_target
    pub dword2: u32,

    // dword 3
    // [0:7]    usb_device_address
    // [8:26]   reserved
    // [27:31]  slot_state
    pub dword3: u32,

    pub dword4: u32,
    pub dword5: u32,
    pub dword6: u32,
    pub dword7: u32,
}

#[repr(C, packed)]
pub union SlotContext {
    pub dwords: [u32; 8],
    pub bits: SlotContextBits,
}

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct EndpointContextBits {
    // dword 0
    // [0:2]    ep_state
    // [3:7]    reserved
    // [8:9]    mult
    // [10:14]  max_primary_streams
    // [15]     linear_stream_array
    // [16:23]  interval
    // [24:31]  max_esit_payload_hi
    pub dword0: u32,

    // dword 1
    // [0]      reserved
    // [1:2]    error_count
    // [3:5]    ep_type
    // [6]      reserved
    // [7]      host_initiate_disable
    // [8:15]   max_burst_size
    // [16:31]  max_packet_size
    pub dword1: u32,

    // dword 2-3: tr_dequeue_pointer (64bit)
    // [0]      dequeue_cycle_state
    // [1:3]    reserved
    // [4:63]   tr_dequeue_pointer
    pub dequeue: u64,

    // dword 4
    // [0:15]   average_trb_length
    // [16:31]  max_esit_payload_lo
    pub dword4: u32,

    pub dword5: u32,
    pub dword6: u32,
    pub dword7: u32,
}

#[repr(C, packed)]
pub union EndpointContext {
    pub dwords: [u32; 8],
    pub bits: EndpointContextBits,
}

#[derive(Debug, Copy, Clone)]
#[repr(C, align(64))]
pub struct DeviceContext{

}

impl DeviceContext{

}