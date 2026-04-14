// ================================================================
// @file usb/ring.hpp
//
// USB ドライバ用の動的メモリ管理機能
// 64バイト境界とうの制約を厳格に定める
// ================================================================

use core::mem::size_of;
use spin::Mutex;

// 動的メモリ確保のためのメモリプールの最大容量（バイト）
const MEMORY_POOL_SIZE:usize = 4096 * 32;

fn ceil(ptr: usize, alignment: usize) -> usize{
    let addr = ptr as usize;
    (addr + alignment - 1) & !((alignment - 1) as usize)
}

fn mask_bits(ptr: usize , mask: usize) -> usize{
    let addr = ptr as usize;
    addr & !(mask -1)
}

// USB関連のためのヒープ領域
// static 変数はメモリ上に固定配置される
pub static MEMORY_POOL:Mutex<MemoryPool> = Mutex::new(MemoryPool::new());

pub struct MemoryPool {
    pool: [u8; MEMORY_POOL_SIZE],
    alloc_ptr: usize,
}

impl MemoryPool{
    const fn new() -> Self {
        Self {
            pool: [0u8; MEMORY_POOL_SIZE],
            alloc_ptr: 0, // オフセット0から開始
        }
    }

    pub fn alloc_mem(&mut self, size: usize, alignment: usize, boundary: usize) -> Option<usize> {
        // 絶対アドレスはここで初めて計算する
        let pool_start = self.pool.as_ptr() as usize;
        let mut current = pool_start + self.alloc_ptr;

        // alignmentの調整
        if alignment > 0{
            current = ceil(current, alignment);
        }
        // 4KBのページ境界を調整
        if boundary > 0 {
            let next_boundary = ceil(current, boundary);
            if next_boundary < self.alloc_ptr + size {
                current = next_boundary;
            }
        }

        // 確保したpoolの範囲外に出ないかチェック
        if pool_start + MEMORY_POOL_SIZE < current + size {
            return None
        }

        // 調整されたアドレスを返し、オフセットとして保存
        self.alloc_ptr = current - pool_start + size;
        Some(current)
    }

    pub fn alloc_array<T>(&mut self, num_obj: usize, alignment: usize, boundary: usize) -> Option<*mut T> {
        if let Some(ptr) = self.alloc_mem(size_of::<T>()*num_obj, alignment, boundary) {
            return Some(ptr as *mut T)
        }else{
            return None
        }
    }
}
