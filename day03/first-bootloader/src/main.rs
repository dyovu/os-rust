#![no_std]
#![no_main]

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;

use log::info;
use uefi::prelude::*;
use uefi::boot::{memory_map, exit_boot_services, MemoryType, image_handle};
use uefi::proto::media::file::{File, FileMode, FileAttribute};
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::mem::memory_map::MemoryMap;


// uefiクレートが提供するメモリ管理機能を使う
#[global_allocator]
static ALLOCATOR: uefi::allocator::Allocator = uefi::allocator::Allocator;

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();

    let mt = MemoryType::LOADER_DATA;
    let memory_map_result = match memory_map(mt) {
        Ok(map) => map,
        Err(e) => {
            info!("Failed to get memory map: {:?}", e);
            return Status::ABORTED;
        }
    };

    info!("Memory map retrieved with {} entries", memory_map_result.entries().count());
    
    // カーネルファイルを読み込み
    let kernel_data = load_kernel().expect("Failed to load kernel");
    info!("Kernel loaded, size: {} bytes", kernel_data.len());
    info!("Kernel first bytes: {:02x} {:02x} {:02x} {:02x}", 
      kernel_data[0], kernel_data[1], kernel_data[2], kernel_data[3]);

    // ブートサービス終了
    let memory_map_final = unsafe { exit_boot_services(Some(mt)) };

    // カーネルを1MBにコピーしてジャンプ
    let kernel_entry = 0x100000 as *mut u8;
    unsafe {
        core::ptr::copy_nonoverlapping( // カーネルデータを1MBのアドレスにコピー
            kernel_data.as_ptr(),
            kernel_entry,
            kernel_data.len()
        );
        
        let kernel_main: extern "C" fn() -> ! = 
            core::mem::transmute(kernel_entry);
        kernel_main();
    }
}



fn load_kernel() -> Result<Vec<u8>, uefi::Error> {
    let image_handle = image_handle();

    // 本当は boot services を使ってフィルシステムとか持ってくるらしいけど、
    // ここでは直接取得する
    // UEFIがファイルシステムにアクセスできないとカーネルファイルが見つけられない
    // rootディレクトリまで取得する
    let mut fs_protocol = uefi::boot::get_image_file_system(image_handle)?; 
    let fs: &mut SimpleFileSystem = fs_protocol.get_mut().expect("Failed to get file system");
    let mut root = fs.open_volume()?;
    
    let mut kernel_file = root
        // 
        // ここを実際にファイル名に変更する
        // ↓ ↓ ↓ ↓ ↓ ↓ ↓ ↓
        .open(cstr16!("kernel.bin"), FileMode::Read, FileAttribute::empty()).expect("Failed to open kernel file")
        .into_regular_file()
        .expect("Kernel file is not regular");

    let mut info_buffer = vec![0u8; 1024];
    let file_info = kernel_file
        .get_info::<uefi::proto::media::file::FileInfo>(&mut info_buffer).expect("Failed to get file info");

    let mut buffer = vec![0u8; file_info.file_size() as usize];
    kernel_file.read(&mut buffer)?;

    Ok(buffer)
}