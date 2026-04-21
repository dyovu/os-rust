// ================================================================
// @file usb/xhci/device.rs
//
// xHCI固有のデバイスを表すクラスと関連機能．
// ================================================================

use crate::usb::device::UsbDevice;
use crate::usb::xhci::context::{DeviceContext, DeviceContextIndex, InputContext};
use crate::usb::xhci::ring::Ring;
use crate::usb::memory_alloc::ArrayMap;
use crate::usb::endpoint::EndpointID;
use crate::usb::setupdata::SetupData;
use crate::usb::xhci::trb::{SetupStageTRB, DataStageTRB, StatusStageTRB, trb_dynamic_cast};
use crate::usb::xhci::registers::{DoorbellRegister};

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
        Self{
            slot_id,
            state: State::Blank,
            transfer_rings: core::array::from_fn(|_| None),
            ctx: DeviceContext::new(),
            input_ctx: InputContext::new(),
            dbreg_addr,
            setup_stage_map: ArrayMap::new(),
        }
    }

    pub fn on_transfer_event_received(&self) {
        
    }

    fn make_SetupStageTRB() -> SetupStageTRB {
        SetupStageTRB::initialize()
    }

    fn make_DataStageTRB() -> DataStageTRB {
        DataStageTRB::initialize()
    }
}

impl UsbDevice for XhciDevice{
    fn control_in(&mut self, ep_id: EndpointID, setup_data: SetupData, buf: &mut [u8]) -> Result<(), ()>{
        let len = buf.len(); // Deviceのbufの長さなので=256
        if ep_id.number() < 0 || 15 < ep_id.number(){
            return Err(())
        }

        let dci = DeviceContextIndex::new(ep_id.address());

        let ring = match &mut self.transfer_rings[dci.value() as usize]{
            Some(ring) => {
                ring
            }
            None => { return Err(()) }
        };

        // C++の方ではbufの存在によって分岐してるけど、
        // ControlInがそもそもusbデバイスからデータを受け取る操作だから不要

        // 3段階
        // 
        let setup_stage_trb: SetupStageTRB = XhciDevice::make_SetupStageTRB();
        let tup_stage_trb_bit:[u8; 16] = setup_stage_trb.into_bytes();
        let setup_trb = ring.push(&tup_stage_trb_bit);
        let setup_trb_addr = unsafe { trb_dynamic_cast::<SetupStageTRB>(setup_trb) }.ok_or(())?;

        // 
        let mut data_stage_trb: DataStageTRB = XhciDevice::make_DataStageTRB();
        data_stage_trb.set_interrupt_on_completion(true as u8);
        let data_stage_trb_bit:[u8; 16] = data_stage_trb.into_bytes();
        let data_trb = ring.push(&data_stage_trb_bit) as usize;

        // 
        let status_stage_trb = StatusStageTRB::initialize();
        let status_stage_trb_bit:[u8; 16] = status_stage_trb.into_bytes();
        let _ = ring.push(&status_stage_trb_bit);

        // subデバイス側から応答があった時に対応するsetup_trb_addrを特定するため
        self.setup_stage_map.put(data_trb, *setup_trb_addr);

        // 
        let door_reg = unsafe{ &mut *(self.dbreg_addr as *mut DoorbellRegister) };
        door_reg.ring(dci.value(), 0);

        Ok(())
    }

    fn control_out(&mut self, ep_id: EndpointID, setup_data: SetupData, buf: &[u8]) -> Result<(), ()>{

        Ok(())
    }

    fn interrupt_in(&mut self, ep_id: EndpointID, buf: &mut [u8]) -> Result<(), ()>{

        Ok(())
    }

    fn interrupt_out(&mut self, ep_id: EndpointID, buf: &mut [u8]) -> Result<(),  ()>{

        Ok(())
    }
}