// ================================================================
// @file usb/xhci/device_manager.rs
//
// USB デバイスの管理機能．
// ================================================================

use crate::usb::xhci::device::Dvice;

struct DviceManager{
    max_slots: usize,
    devices:[Dvice; max_slots + 1],
}