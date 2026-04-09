// ================================================================
// @file usb/xhci/register.rs
//
// xHCIのMMRの定義に基づいた構造体の定義
// ================================================================

#[repr(C)]
#[allow(non_snake_case)] // xHCIの仕様と同じ名前にするため
pub struct CapabilityRegisters {
    pub CAPLENGTH: u8,
    _reserved: u8,
    HCIVERSION: u16,
    pub HCSPARAMS1: u32,
    HCSPARAMS2: u32,
    HCSPARAMS3: u32,
    HCCPARAMS1:u32,
    DBOFF: u32,
    RTSOFF: u32,
    HCCPARAMS2: u32,
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct OperationalRegisters{
    USBCMD: u32,
    USBSTS: u32,
    PAGESIZE: u32,
    _reserved1: [u8; 8],
    DNCTRL: u32,
    CRCR: u64,
    _reserved2: [u8; 16],
    DCBAAP: u64,
    CONFIG: u32,
}