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

    fn make_SetupStageTRB(setup_data: SetupData, transfer_type: u8) -> SetupStageTRB {
        let mut setup: SetupStageTRB = SetupStageTRB::initialize();
        setup.set_request_type(setup_data.request_type);
        setup.set_request(setup_data.request);
        setup.set_value(setup_data.value);
        setup.set_index(setup_data.index);
        setup.set_length(setup_data.length);
        setup.set_transfer_type(transfer_type);
        setup
    }

    fn make_DataStageTRB(buf: *const [u8], dir_in: bool) -> DataStageTRB {
        let mut data: DataStageTRB = DataStageTRB::initialize();
        data.set_pointer(buf);
        data.set_td_size(0);
        data.set_direction(dir_in as u8);
        data
    }
}

impl UsbDevice for XhciDevice{
    // USBデバイスとホスト間の4つの転送方式のうち、コントロール転送（デバイスからホストへの通信）を行う
    fn control_in(&mut self, ep_id: EndpointID, setup_data: SetupData, buf: &mut [u8]) -> Result<(), ()>{
        // ホスト側で用意したデータ受け取り用バッファのサイズ
        let len = buf.len(); 
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

        // ControlInはデバイスからデータを受け取る仕様上、Data Stageが必須となるためバッファの存在確認による分岐は不要

        // 1. Setup Stage
        // ホストからデバイスへ、これからどのような処理を行いたいかという8バイトの要求データ（SetupData）を送信する
        let setup_stage_trb: SetupStageTRB = XhciDevice::make_SetupStageTRB(setup_data, SetupStageTRB::IN_DATA_STAGE);
        let tup_stage_trb_bit:[u8; 16] = setup_stage_trb.into_bytes();
        let setup_trb = ring.push(&tup_stage_trb_bit);
        let setup_trb_addr = unsafe { trb_dynamic_cast::<SetupStageTRB>(setup_trb) }.ok_or(())?;

        // 2. Data Stage
        // Setup Stageの要求に基づき、デバイスから実際のデータを受信する
        let mut data_stage_trb: DataStageTRB = XhciDevice::make_DataStageTRB(buf as *const [u8],  true);
        data_stage_trb.set_interrupt_on_completion(true as u8); // 完了時に割り込みを発生させる設定
        let data_stage_trb_bit:[u8; 16] = data_stage_trb.into_bytes();
        let data_trb = ring.push(&data_stage_trb_bit) as usize;

        // 3. Status Stage
        // ホストからデバイスへ、一連のデータ受信が正常に完了したことを通知する
        let status_stage_trb = StatusStageTRB::initialize();
        let status_stage_trb_bit:[u8; 16] = status_stage_trb.into_bytes();
        let _ = ring.push(&status_stage_trb_bit);

        // 後でデバイスから応答（割り込みイベント）が来た際、誰が待っていた情報かを照合できるように台帳へ登録する
        self.setup_stage_map.put(data_trb, *setup_trb_addr);

        // xHCIハードウェアのドアベルレジスタを叩き、メモリ上にTRBを積んだことを通知して実際の通信処理を開始させる
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