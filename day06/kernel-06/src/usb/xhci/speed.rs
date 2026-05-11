// ================================================================
// @file usb/xhci/Speed: usize.rs
//
// Protocol Speed: usize ID のデフォルト定義．PSIC == 0 のときのみ有効．
// ================================================================

pub const FullSpeed: u8 = 1;
pub const LowSpeed: u8 = 2;
pub const HighSpeed: u8 = 3;
pub const SuperSpeed: u8 = 4;
pub const SuperSpeedPlus: u8= 5;