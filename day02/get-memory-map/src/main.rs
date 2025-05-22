#![no_main]
#![no_std]

extern crate alloc;
use uefi::allocator::Allocator;

// #[global_allocator]でUEFI専用アロケータを指定
// format!マクロなどのalloc機能が使用可能になる
#[global_allocator]
static ALLOCATOR: Allocator = Allocator;

use log::info;
use alloc::format;
use uefi::prelude::*;
use uefi::boot::{self, memory_map, MemoryType};
use uefi::mem::memory_map::MemoryMap;
use uefi::proto::media::file::{self, File, FileMode, FileAttribute}; 


#[entry]
fn main() -> Status {
    // ログ初期化（システムテーブルは内部で管理される）
    uefi::helpers::init().unwrap();
    
    // メモリマップ取得
    let mt = MemoryType::LOADER_DATA;
    let memory_map = match memory_map(mt) {
        Ok(map) => map,
        Err(e) => {
            info!("Failed to get memory map: {:?}", e);
            return Status::ABORTED;
        }
    };
    
    info!("Memory map retrieved with {} entries", memory_map.entries().count());
    
    // ファイル保存
    match save_memory_map_to_file(&memory_map) {
        Ok(_) => info!("Memory map saved successfully"),
        Err(e) => info!("Failed to save memory map: {:?}", e),
    }
    
    Status::SUCCESS
}


fn save_memory_map_to_file(
    memory_map: &uefi::mem::memory_map::MemoryMapOwned
) -> uefi::Result<()> {
    // 現在のイメージハンドルを取得
    let image_handle = boot::image_handle();
    
    // ファイルシステム取得
    let mut file_system = boot::get_image_file_system(image_handle)?;
    let mut root:file::Directory = file_system.open_volume()?;
    
    // rootディレクトリに対してファイルモードをCreateで指定しているからファイルが作成できる
    let file_handle:file::FileHandle = root.open(
        cstr16!("memory_map.txt"),
        FileMode::CreateReadWrite,
        FileAttribute::empty(),
    )?;

    let mut file = file_handle.into_regular_file()
        .ok_or(uefi::Status::INVALID_PARAMETER)?;
    
    // ヘッダー書き込み
    let header = format!("Memory Map - {} entries\n\n", memory_map.entries().count());
    let _ = file.write(header.as_bytes());
    
    // エントリ書き込み
    for (i, desc) in memory_map.entries().enumerate() {
        let entry = format!(
            "Entry {}: Type={:?}, Start=0x{:016x}, Pages={}, Attr=0x{:x}\n",
            i, desc.ty, desc.phys_start, desc.page_count, desc.att.bits()
        );
        let _ = file.write(entry.as_bytes());
    }
    
    file.flush()?;
    Ok(())
}
