// ================================================================
// @file usb/xhci/device.rs
//
// xHCI固有のデバイスを表すクラスと関連機能．
// ================================================================


enum State{

}
pub struct Device{
    slot_id: u8,

}

impl Device{
    pub fn new(slot_id: u8) -> Self{
        Self{
            slot_id,
        }
    }
}