// ================================================================
// @file usb/xhci/ring.hpp
//
// Event Ring, Command Ring, Transfer Ring のクラスや関連機能．
// ================================================================

use crate::usb::memory_alloc::MEMORY_POOL;
use crate::usb::xhci::trb::{TRB, LinkTRB};


 struct Ring{
    buf: *mut TRB,
    buf_size: usize,
    cycle_bit: bool,
    write_index: usize, // リングの中で次に書き込む位置
 }

 impl Ring{
    pub fn new(buf_size: usize) -> Self{
        let buf: *mut TRB = match MEMORY_POOL.lock().alloc_array::<TRB>(buf_size, 64, 32*1024){
            Some(pool) => {
                pool
            }
            None => {
                loop{ }
            }
        };

        Self{
            buf,
            buf_size,
            cycle_bit: true,  // 1つ前のbitと反転させることで、contollerが新しい書き込みかどうかを判断できるようにする
            write_index: 0,
        }
    }

    fn copy_to_last(&mut self, data: &[u8; 16]){
        unsafe {
            // TRBポインタをu8ポインタとして解釈し直す
            let dest = self.buf.add(self.write_index) as *mut u8;
            
            // 最初の12バイトをそのままコピー
            core::ptr::copy_nonoverlapping(data.as_ptr(), dest, 12);

            // dataの12〜15バイト目を取り出してu32として解釈する
            // [u8; 4]からu32への変換はtry_into()でやる
            // ベアメタルではリトルエンディアンを明示するためfrom_le_bytesを使う
            let last_word = u32::from_le_bytes(data[12..16].try_into().unwrap());
            // まずANDで既存のcycle bitを消し、ORで新しいcycle bitを書き込む
            let with_cycle = (last_word & 0xFFFF_FFFE) | (self.cycle_bit as u32);
            let dest_last = dest.add(12) as *mut u32;
            dest_last.write_volatile(with_cycle);
        }
    }

    // ジェネリクスで任意の型のTRBを受け取り、ここでバイト列に変換するのではなく
    // 引数としてバイト列を受け取るようにする
    // 
    // 任意の型を受け取ってバイト列にしようとすると
    // トレイトを定義して全てのTRB型にそれを実装しなキュいけないから
    pub fn push<TRBType>(&mut self, trb: &[u8; 16]) -> *mut TRB {
        let trb_ptr: *mut TRB = unsafe { self.buf.add(self.write_index) };
        
        self.copy_to_last(&trb);
        self.write_index += 1;

        if self.write_index == self.buf_size - 1{
            let mut link_trb = LinkTRB::initialize(self.buf);
            link_trb.set_toggle_cycle(true as u8);
            self.copy_to_last(&link_trb.into_bytes());

            self.write_index = 0;
            self.cycle_bit = !self.cycle_bit;
        }
        trb_ptr
    }
 }

#[repr(C, align(64))]
#[derive(Copy, Clone)]
 struct EventRingSegmentTableEntry{
    ring_segment_base_address: u64,

    ring_segment_size: u16,
    _reserved1: u16,

    _reserved2: u32,
 }

struct EventRing{

}
