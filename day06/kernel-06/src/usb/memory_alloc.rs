// ================================================================
// @file usb/ring.hpp
//
// USB ドライバ用の動的メモリ管理機能
// 64バイト境界とうの制約を厳格に定める
// ================================================================

use core::mem::size_of;

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

struct MemoryPool {
    pool: [u8; MEMORY_POOL_SIZE],
    alloc_ptr: usize,
}

impl MemoryPool{
    fn alloc_mem(&mut self, size: usize, alignment: usize, boundary: usize) -> usize {
        if alignment > 0{
            self.alloc_ptr = ceil(self.alloc_ptr, alignment);
        }
        if boundary > 0 {
            let next_boundary = ceil(self.alloc_ptr, boundary);
            if next_boundary < self.alloc_ptr + size {
                self.alloc_ptr = next_boundary;
            }
        }

        let pool_start = self.pool.as_ptr() as usize;
        if pool_start + MEMORY_POOL_SIZE < self.alloc_ptr + size {
            return 0;
        }

        let p = self.alloc_ptr;
        self.alloc_ptr += size;
        p
    }

    pub fn alloc_array<T>(&mut self, num_obj: usize, alignment: usize, boundary: usize) -> *const T {
        self.alloc_mem(size_of::<T>()*num_obj, alignment, boundary) as *const T
    }

}
