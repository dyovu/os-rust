// ================================================================
// @file usb/xhci/register.rs
//
// xHCIのMMRの定義に基づいた構造体の定義
// ================================================================

use core::ptr::{read_volatile, write_volatile};

#[repr(C, packed)]
pub struct MemMapRegister<T: Copy> {
    value: T,
}

impl<T: Copy> MemMapRegister<T> {
    pub fn read(&self) -> T {
        // selfのアドレスをそのままMMIOアドレスとして扱い、volatile読み込み
        unsafe { read_volatile(&raw const self.value) }
    }

    pub fn write(&mut self, value: T) {
        unsafe { write_volatile(&raw mut self.value, value) }
    }
}

#[repr(C)]
#[allow(non_snake_case)] // xHCIの仕様と同じ名前にするため
pub struct CapabilityRegisters {
    CAPLENGTH: MemMapRegister<u8>,
    _reserved: MemMapRegister<u8>,
    HCIVERSION: MemMapRegister<u16>,
    HCSPARAMS1: MemMapRegister<u32>,
    HCSPARAMS2: MemMapRegister<u32>,
    HCSPARAMS3: MemMapRegister<u32>,
    HCCPARAMS1: MemMapRegister<u32>,
    DBOFF: MemMapRegister<u32>,
    RTSOFF: MemMapRegister<u32>,
    HCCPARAMS2: MemMapRegister<u32>,
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct OperationalRegisters{
    USBCMD: MemMapRegister<u32>,
    USBSTS: MemMapRegister<u32>,
    PAGESIZE: MemMapRegister<u32>,
    _reserved1: [u8; 8],
    DNCTRL: MemMapRegister<u32>,
    CRCR: MemMapRegister<u64>,
    _reserved2: [u8; 16],
    DCBAAP: MemMapRegister<u64>,
    CONFIG: MemMapRegister<u32>,
}

pub struct InterrupterRegisterSet{

}