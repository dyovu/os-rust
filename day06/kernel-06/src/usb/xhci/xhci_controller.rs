// ================================================================
// @file usb/xhci/xhci_controller.rs
//
// xHCI ホストコントローラ制御用クラス．
// ================================================================

#[repr(C)]
#[allow(non_snake_case)] // xHCIの仕様と同じ名前にするため
struct CapabilityRegisters {
    CAPLENGTH: u8,
    Rsvd: u8,
    HCIVERSION: u16,
    HCSPARAMS1: u32,
    HCSPARAMS2: u32,
    HCSPARAMS3: u32,
    HCCPARAMS1:u32,
    DBOFF: u32,
    RTSOFF: u32,
    HCCPARAMS2: u32,
}

#[repr(C)]
#[allow(non_snake_case)]
struct OperationalRegisters{
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


/*
 *生ポインタを構造体に持たせるのは避ける
 * 生ポインタはライフタイムを持たない
 * usizeで持つことでunsafeの範囲を最小限にできる
 */ 
pub struct Controller {
    mmio_base: usize,
    pub max_ports: u8,
}

impl Controller {
    pub fn new(mmio_base: usize) -> Self{
        let max_ports = unsafe {
            let hcsparams1 = (*(mmio_base as *const CapabilityRegisters)).HCSPARAMS1;
            (hcsparams1 >> 24) as u8
        };
        Self{
            mmio_base,
            max_ports,
        }
    }

    fn cap_regs(&self) -> *const CapabilityRegisters{
        self.mmio_base as *const CapabilityRegisters
    }

    fn op_regs(&self) -> *mut OperationalRegisters {
        let cap_len = unsafe { (*self.cap_regs()).CAPLENGTH };
        (self.mmio_base + cap_len as usize) as *mut OperationalRegisters
    }
}