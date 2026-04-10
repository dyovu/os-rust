// ================================================================
// @file usb/xhci/register.rs
//
// xHCIのMMRの定義に基づいた構造体の定義
// ================================================================

use core::ptr::{read_volatile, write_volatile};
use core::marker::PhantomData;

use modular_bitfield::prelude::*;

// アクセス権限を表すマーカー型
pub struct ReadOnly;
pub struct ReadWrite;

// 全てのregisterフィールドをラップする構造体
// publicなフィールドに対してRWの制限
// volatileなアクセスを保証するため
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

// ================================================================
// CapabilityRegisters
// ================================================================

#[repr(C)]
#[allow(non_snake_case)]
pub struct CapabilityRegisters {
    pub CAPLENGTH:  MemMapRegister<u8,  ReadOnly>,
    _reserved:      u8,  
    pub HCIVERSION: MemMapRegister<u16, ReadOnly>,
    pub HCSPARAMS1: MemMapRegister<u32, ReadOnly>,
    pub HCSPARAMS2: MemMapRegister<u32, ReadOnly>,
    pub HCSPARAMS3: MemMapRegister<u32, ReadOnly>,
    pub HCCPARAMS1: MemMapRegister<u32, ReadOnly>,
    pub DBOFF:      MemMapRegister<u32, ReadOnly>,
    pub RTSOFF:     MemMapRegister<u32, ReadOnly>,
    pub HCCPARAMS2: MemMapRegister<u32, ReadOnly>,
}

// ================================================================
// OperationalRegisters
// ================================================================

#[repr(C)]
#[allow(non_snake_case)]
pub struct OperationalRegisters {
    pub USBCMD:   MemMapRegister<u32, ReadWrite>,
    pub USBSTS:   MemMapRegister<u32, ReadWrite>,
    pub PAGESIZE: MemMapRegister<u32, ReadOnly>,  // read-only
    _reserved1:   [u8; 8],
    pub DNCTRL:   MemMapRegister<u32, ReadWrite>,
    pub CRCR:     MemMapRegister<u64, ReadWrite>,
    _reserved2:   [u8; 16],
    pub DCBAAP:   MemMapRegister<u64, ReadWrite>,
    pub CONFIG:   MemMapRegister<u32, ReadWrite>,
}

// ================================================================
// PortRegisterSet
// ================================================================

#[repr(C)]
#[allow(non_snake_case)]
pub struct PortRegisterSet {
    pub PORTSC:    MemMapRegister<u32, ReadWrite>,
    pub PORTPMSC:  MemMapRegister<u32, ReadWrite>,
    pub PORTLI:    MemMapRegister<u32, ReadOnly>,
    pub PORTHLPMC: MemMapRegister<u32, ReadWrite>,
}

// ================================================================
// InterrupterRegisterSet
// ================================================================

#[repr(C)]
#[allow(non_snake_case)]
pub struct InterrupterRegisterSet {
    pub IMAN:   MemMapRegister<u32, ReadWrite>,
    pub IMOD:   MemMapRegister<u32, ReadWrite>,
    pub ERSTSZ: MemMapRegister<u32, ReadWrite>,
    _reserved:  u32,  
    pub ERSTBA: MemMapRegister<u64, ReadWrite>,
    pub ERDP:   MemMapRegister<u64, ReadWrite>,
}

// ================================================================
// DoorbellRegister
// ================================================================

#[repr(C)]
pub struct DoorbellRegister {
    reg: MemMapRegister<u32, ReadWrite>,
}

impl DoorbellRegister {
    pub fn ring(&mut self, target: u8, stream_id: u16) {
        let value = (target as u32) | ((stream_id as u32) << 16);
        self.reg.write(value);
    }
}

// ================================================================
// ExtendedRegister
// ================================================================

// 拡張レジスタの共通ヘッダ
// capability_id, next_pointer, valueのフィールドを持つ
#[repr(C)]
pub struct ExtendedRegister {
    pub reg: MemMapRegister<u32, ReadWrite>,
}

impl ExtendedRegister {
    pub fn capability_id(&self) -> u8 {
        (self.reg.read() & 0xFF) as u8
    }

    pub fn next_pointer(&self) -> u8 {
        ((self.reg.read() >> 8) & 0xFF) as u8
    }
}