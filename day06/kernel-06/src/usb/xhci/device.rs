// ================================================================
// @file usb/xhci/device.rs
//
// xHCI固有のデバイスを表すクラスと関連機能．
// ================================================================

use crate::usb::device::UsbDevice;
use crate::usb::xhci::context::{DeviceContext, InputContext};
use crate::usb::xhci::ring::Ring;
use crate::usb::xhci::trb::SetupStageTRB;
use crate::usb::memory_alloc::ArrayMap;

#[derive(Debug, Clone, Copy)]
enum State {
    Invalid,
    Blank,
    SlotAssigning,
    SlotAssigned,
}
pub struct XhciDevice{
    slot_id: u8,
    state: State,
    transfer_rings: [Option<Ring>; 31],
    ctx: DeviceContext,
    input_ctx: InputContext,
    dbreg_addr: usize, // ドアベルレジスタのアドレス
    setup_stage_map: ArrayMap<usize, SetupStageTRB, 16>,
}

impl XhciDevice{
    pub fn new(slot_id: u8, dbreg_addr: usize) -> Self{
        let state = State::Blank;
        let  transfer_rings = core::array::from_fn(|_| None);
        let setup_stage_map = ArrayMap::new();
        let ctx = DeviceContext::new();
        let input_ctx = InputContext::new();
        
        Self{
            slot_id,
            state,
            transfer_rings,
            ctx,
            input_ctx,
            dbreg_addr,
            setup_stage_map,
        }
    }

    pub fn on_transfer_event_received(&self) {
        
    }
}

impl UsbDevice for XhciDevice{
    fn control_in(&mut self, ep_id: EndpointID, setup_data: SetupData, buf: &mut [u8]) -> Result<(), ()>{

    }

    fn control_out(&mut self, ep_id: EndpointID, setup_data: SetupData, buf: &[u8]) -> Result<(), ()>{

    }

    fn interrupt_in(&mut self, ep_id: EndpointID, buf: &mut [u8]) -> Result<(), ()>{

    }

    fn interrupt_out(&mut self, ep_id: EndpointID, buf: &mut [u8]) -> Result<(),  ()>{

    }
}