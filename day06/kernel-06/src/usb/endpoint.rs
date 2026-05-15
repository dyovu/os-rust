// ================================================================
// @file usb/endpoint.rs
//
// エンドポイント設定に関する機能
// ================================================================

use core::default;


// デフォルトコントロールパイプ（エンドポイント0, IN）
pub const DEFAULT_CONTROL_PIPE_ID: EndpointID = EndpointID::from_parts(0, true);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndpointType{
    Control = 0,
    Isochronous = 1,
    Bulk = 2,
    Interrupt = 3,
}

impl Default for EndpointType {
    // usb/deviceの初期化の際に呼ばれる
    fn default() -> Self {
        EndpointType::Control
    }
}

#[derive(Debug, Copy, Clone)]
#[derive(Default)]
pub struct EndpointID{
    addr: u8,
}

impl EndpointID{
    pub fn new() -> Self{
        Self { addr: 0 }
    }

    pub fn from_addr(addr: u8) -> Self{
        Self{ addr }
    }

    /** エンドポイント番号と入出力方向から ID を構成する．
     *
     * ep_num は 0..15 の整数．
     * dir_in は Control エンドポイントでは常に true にしなければならない．
     */
    pub const fn from_parts(ep_num: u8, dir_in: bool) -> Self{
        Self{
            addr: ep_num << 1 | dir_in as u8
        }
    }

    pub fn address(&self) -> u8{
        self.addr
    }

    pub fn number(&self) -> u8{
        self.addr >> 1
    }

    pub fn is_in(&self) -> bool{
        self.addr & 1 != 0
    }
    
}

#[derive(Default, Clone, Copy)]
pub struct EndpointConfig{
    pub ep_id: EndpointID,
    pub ep_type: EndpointType,
    pub max_packet_size: u16,  // エンドポイントの最大パケットサイズ（バイト）
    pub interval: u8, // このエンドポイントの制御周期（125*2^(interval-1) マイクロ秒）
}

impl EndpointConfig{

}