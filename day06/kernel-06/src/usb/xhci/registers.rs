// ================================================================
// @file usb/xhci/register.rs
//
// xHCIのMMRの定義に基づいた構造体の定義
// ================================================================

use core::ptr::{read_volatile, write_volatile};
use core::marker::PhantomData;

// アクセス権限を表すマーカー型
pub struct ReadOnly;
pub struct ReadWrite;

#[repr(C, packed)]
pub struct MemMapRegister<T: Copy, Access> {
    value: T,
    // 実際のメモリは使わないが、型としてAccessを保持するために必要
    _marker: PhantomData<Access>,
}

// readはどちらの権限でも使える
impl<T: Copy, Access> MemMapRegister<T, Access> {
    pub fn read(&self) -> T {
        unsafe { read_volatile(&raw const self.value) }
    }
}

// writeはReadWriteの時だけ使える
impl<T: Copy> MemMapRegister<T, ReadWrite> {
    pub fn write(&mut self, value: T) {
        unsafe { write_volatile(&raw mut self.value, value) }
    }
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct CapabilityRegisters {
    pub CAPLENGTH:  MemMapRegister<u8,  ReadOnly>,
    _reserved:      u8,  // MMIOアクセス不要なのでMemMapRegisterでラップしない
    pub HCIVERSION: MemMapRegister<u16, ReadOnly>,
    pub HCSPARAMS1: MemMapRegister<u32, ReadOnly>,
    pub HCSPARAMS2: MemMapRegister<u32, ReadOnly>,
    pub HCSPARAMS3: MemMapRegister<u32, ReadOnly>,
    pub HCCPARAMS1: MemMapRegister<u32, ReadOnly>,
    pub DBOFF:      MemMapRegister<u32, ReadOnly>,
    pub RTSOFF:     MemMapRegister<u32, ReadOnly>,
    pub HCCPARAMS2: MemMapRegister<u32, ReadOnly>,
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct OperationalRegisters {
    pub USBCMD:     MemMapRegister<u32, ReadWrite>,
    pub USBSTS:     MemMapRegister<u32, ReadWrite>,
    pub PAGESIZE:   MemMapRegister<u32, ReadOnly>,  // read-only
    _reserved1:     [u8; 8],
    pub DNCTRL:     MemMapRegister<u32, ReadWrite>,
    pub CRCR:       MemMapRegister<u64, ReadWrite>,
    _reserved2:     [u8; 16],
    pub DCBAAP:     MemMapRegister<u64, ReadWrite>,
    pub CONFIG:     MemMapRegister<u32, ReadWrite>,
}

pub struct InterrupterRegisterSet{

}