#![no_std]
#![no_main]

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;

use log::info;
use uefi::prelude::*;
use uefi::boot::{memory_map, exit_boot_services, MemoryType};
use uefi::proto::media::file::{File, FileMode, FileAttribute};

// グローバルアロケータ追加
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

    // ブートサービス終了
    let image_handle = uefi::boot::image_handle();
    let (_runtime_table, memory_map_final) = exit_boot_services(image_handle, mt);

    // カーネルを1MBにコピーしてジャンプ
    let kernel_entry = 0x100000 as *mut u8;
    unsafe {
        core::ptr::copy_nonoverlapping(
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
    let bs = uefi::boot::boot_services();
    let image_handle = uefi::boot::image_handle();
    
    let fs = bs.get_image_file_system(image_handle)?;
    let mut root = fs.open_volume()?;
    
    let mut kernel_file = root
        .open(cstr16!("kernel.bin"), FileMode::Read, FileAttribute::empty())?
        .into_regular_file()
        .expect("Kernel file is not regular");

    let mut info_buffer = vec![0u8; 1024];
    let file_info = kernel_file
        .get_info::<uefi::proto::media::file::FileInfo>(&mut info_buffer)?;

    let mut buffer = vec![0u8; file_info.file_size() as usize];
    kernel_file.read(&mut buffer)?;

    Ok(buffer)
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}