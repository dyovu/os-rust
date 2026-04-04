// ================================================================
// @file usb/endpoint.rs
//
// エンドポイント設定に関する機能
// ================================================================


// デフォルトコントロールパイプ（エンドポイント0, IN）
const DEFAULT_CONTROL_PIPE_ID: EndpointID = EndpointID::from_parts(0, true);

// #[repr(u8)]
enum EndpointType{
    Control = 0,
    Isochronous = 1,
    Bulk = 2,
    Interrupt = 3,
}

#[derive(Debug, Copy, Clone)]
pub struct EndpointID{
    addr: i32,
}

impl EndpointID{
    pub fn default() -> Self{
        Self { addr: 0 }
    }
    pub fn from_addr(addr: i32) -> Self{
        Self{ addr }
    }

    /** エンドポイント番号と入出力方向から ID を構成する．
     *
     * ep_num は 0..15 の整数．
     * dir_in は Control エンドポイントでは常に true にしなければならない．
     */
    pub const fn from_parts(ep_num: i32, dir_in: bool) -> Self{
        Self{
            addr: ep_num << 1 | dir_in as i32
        }
    }

    pub fn address(&self) -> i32{
        self.addr
    }

    pub fn number(&self) -> i32{
        self.addr >> 1
    }

    pub fn is_in(&self) -> bool{
        self.addr & 1 != 0
    }
    
}

pub struct EndpointConfig{
    ep_id: EndpointID,
    ep_type: EndpointType,
    max_packet_size: i32,  // エンドポイントの最大パケットサイズ（バイト）
    interval: i32, // このエンドポイントの制御周期（125*2^(interval-1) マイクロ秒）
}