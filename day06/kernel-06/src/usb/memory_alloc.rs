// ================================================================
// @file usb/ring.hpp
//
// USB ドライバ用の動的メモリ管理機能
// 64バイト境界とうの制約を厳格に定める
// ================================================================

use core::mem::size_of;

// 動的メモリ確保のためのメモリプールの最大容量（バイト）
const MEMORY_POOL_SIZE:usize = 4096 * 32;


fn ceil<T>(ptr: *const T , alignment: usize) -> *const T{
    let addr = ptr as usize;
    let aligned_addr = (addr + alignment - 1) & !((alignment - 1) as usize);
    aligned_addr as *const T
}

fn mask_bits<T>(ptr: *const T , mask: usize) -> *const T{
    let addr = ptr as usize;
    let masked_addr = addr & !(mask -1);
    masked_addr as *const T
}

struct MemoryPool {
    pool: [u8; MEMORY_POOL_SIZE],
    alloc_ptr: usize,
}

impl MemoryPool{
    fn alloc_mem(&self, size: usize, alignment: usize, boundary: usize) -> usize {
        0
    }

    pub fn alloc_array<T>(&self, num_obj: usize, alignment: usize, boundary: usize) -> *const T {
        self.alloc_mem(size_of::<T>()*num_obj, alignment, boundary) as *const T
    }

}
