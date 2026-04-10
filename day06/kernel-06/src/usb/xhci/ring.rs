// ================================================================
// @file usb/xhci/ring.hpp
//
// Event Ring, Command Ring, Transfer Ring のクラスや関連機能．
// ================================================================

use crate::usb::memory_alloc::MEMORY_POOL;
use crate::usb::xhci::trb::{TRB, LinkTRB};
use crate::usb::xhci::registers::{InterrupterRegisterSet};


struct Ring{
    buf: *mut TRB,
    buf_size: usize,
    cycle_bit: bool,
    write_index: usize, // リングの中で次に書き込む位置 (offset)
}

impl Ring{
    pub fn new(buf_size: usize) -> Self{
        // boundaryの指定はTRBのbufferが64KBの境界を跨いじゃダメという規定がある
        let buf: *mut TRB = match MEMORY_POOL.lock().alloc_array::<TRB>(buf_size, 64, 64*1024){
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
    pub fn push(&mut self, trb: &[u8; 16]) -> *mut TRB {
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

struct EventRing {
    buf_addr: usize,
    buf_size: usize,
    cycle_bit: bool,
    erste_addr: usize,
    interrupter_addr: usize,
}

impl EventRing{
    pub fn new(buf_size: usize, interrupter: *mut InterrupterRegisterSet) -> Self{
        let cycle_bit = true;

        let buf: *mut TRB = match MEMORY_POOL.lock().alloc_array::<TRB>(buf_size, 64, 64*1024){
            Some(pool) => {
                pool
            }
            None => {
                loop{}
            }
        };

        let erste = match MEMORY_POOL.lock().alloc_array::<EventRingSegmentTableEntry>(1, 64, 64*1024){
            Some(pool) => {
                pool
            }
            None => {
                loop{}
            }
        };

        let this = Self{
            buf_addr: buf as usize,
            buf_size,
            cycle_bit,
            erste_addr: erste as usize,
            interrupter_addr: interrupter as usize,
        };

        unsafe {
            // erstの0番目のエントリにbufの先頭アドレスとサイズを書き込む
            (*erste).ring_segment_base_address = buf as u64;
            (*erste).ring_segment_size = buf_size as u16;
        }

        unsafe {
            let mut erstsz = (*interrupter).ERSTSZ.read();
            erstsz.set_event_ring_segment_table_size(1);
            (*interrupter).ERSTSZ.write(erstsz);
        }

        // DequeuePointerをbufの先頭に設定
        this.write_deque_pointer(this.buf_addr as *mut TRB);

        // ERSTBAにerstのアドレスを書き込む
        unsafe {
            let mut erstba = (*interrupter).ERSTBA.read();
            erstba.set_event_ring_segment_table_base_address(erste as u64);
            (*interrupter).ERSTBA.write(erstba);
        }

        this
    }

    // usizeからInterrupterRegisterSetポインタへの変換
    fn interrupter_ptr(&self) -> *mut InterrupterRegisterSet {
        self.interrupter_addr as *mut InterrupterRegisterSet
    }

    pub fn read_deque_pointer(&self) -> *mut TRB { 
        let erdp = unsafe{ (*self.interrupter_ptr()).ERDP.read() };
        (erdp.event_ring_dequeue_pointer() << 4) as *mut TRB
    }

    pub fn write_deque_pointer(&self, trb: *mut TRB){
        let mut erdp = unsafe{ (*self.interrupter_ptr()).ERDP.read() };
        erdp.set_event_ring_dequeue_pointer(trb as u64);
        unsafe { (*self.interrupter_ptr()).ERDP.write(erdp); }
    }

    pub fn has_pending_event(&self) -> bool {
        let trb = self.read_deque_pointer();
        unsafe { (*trb).cycle_bit() == self.cycle_bit as u8 }
    }
}
