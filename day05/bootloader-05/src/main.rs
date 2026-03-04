#![no_std]
#![no_main]

extern crate alloc;
use alloc::format;
use alloc::vec;
use alloc::vec::Vec;
use core::arch::asm;
use log::info;

use uefi::boot::{exit_boot_services, memory_map, image_handle, MemoryType};
use uefi::mem::memory_map::{MemoryMap, MemoryMapOwned};
use uefi::prelude::*;
use uefi::proto::console::gop::GraphicsOutput;
use uefi::proto::media::file::{self, File, FileAttribute, FileMode};

// ================================================================
// 構造体定義
// ================================================================

#[repr(C)]
struct FrameBufferInfo {
    buffer: *mut u8,
    buffer_size: usize,
    width: usize,
    height: usize,
    pixels_per_scan_line: usize,
    pixel_format: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct RawMemoryDescriptor {
    pub memory_type: u32,
    pub physical_start: u64,
    pub virtual_start: u64,
    pub number_of_pages: u64,
    pub attribute: u64,
}

// ================================================================
// アロケータ
// ================================================================

#[global_allocator]
static ALLOCATOR: uefi::allocator::Allocator = uefi::allocator::Allocator;

// ================================================================
// ユーティリティ
// ================================================================

fn halt() -> ! {
    loop {
        unsafe { asm!("hlt") }
    }
}

// ================================================================
// メモリマップをファイルに保存（デバッグ用）
// ================================================================

fn save_memory_map_to_file(
    image_handle: uefi::Handle,
    memory_map: &uefi::mem::memory_map::MemoryMapOwned
) -> uefi::Result<()> {
    let mut file_system = boot::get_image_file_system(image_handle)?;
    let mut root: file::Directory = file_system.open_volume()?;

    let file_handle = root.open(
        cstr16!("memory_map.txt"),
        FileMode::CreateReadWrite,
        FileAttribute::empty(),
    )?;
    let mut file = file_handle
        .into_regular_file()
        .ok_or(uefi::Status::INVALID_PARAMETER)?;

    let header = format!("Memory Map - {} entries\n\n", memory_map.entries().count());
    let _ = file.write(header.as_bytes());

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

// ================================================================
// フレームバッファ情報の取得
// ================================================================

fn get_framebuffer_info() -> Result<FrameBufferInfo, uefi::Error> {
    let handle = uefi::boot::get_handle_for_protocol::<GraphicsOutput>()?;
    let mut gop = uefi::boot::open_protocol_exclusive::<GraphicsOutput>(handle)?;

    let mode_info = gop.current_mode_info();
    let mut framebuffer = gop.frame_buffer();

    Ok(FrameBufferInfo {
        buffer: framebuffer.as_mut_ptr(),
        buffer_size: framebuffer.size(),
        width: mode_info.resolution().0,
        height: mode_info.resolution().1,
        pixels_per_scan_line: mode_info.stride(),
        pixel_format: match mode_info.pixel_format() {
            uefi::proto::console::gop::PixelFormat::Rgb => 0,
            uefi::proto::console::gop::PixelFormat::Bgr => 1,
            _ => 2,
        },
    })
}

// ================================================================
// カーネルファイルの読み込み
// ================================================================

fn load_kernel(image_handle: uefi::Handle) -> Result<Vec<u8>, uefi::Error> {
    let mut file_system = boot::get_image_file_system(image_handle)?;
    let mut root: file::Directory = file_system.open_volume()?;

    let mut kernel_file = root
        .open(cstr16!("kernel.bin"), FileMode::Read, FileAttribute::empty())
        .expect("Failed to open kernel file")
        .into_regular_file()
        .expect("Kernel file is not a regular file");

    let mut info_buffer = vec![0u8; 1024];
    let file_info = kernel_file
        .get_info::<uefi::proto::media::file::FileInfo>(&mut info_buffer)
        .expect("Failed to get file info");

    let mut buffer = vec![0u8; file_info.file_size() as usize];
    kernel_file.read(&mut buffer)?;

    Ok(buffer)
}

// ================================================================
// エントリーポイント
// ================================================================

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    let image_handle = image_handle();

    // フレームバッファ情報の取得
    let framebuffer_info = match get_framebuffer_info() {
        Ok(info) => {
            info!("Framebuffer: {}x{}, format={}", info.width, info.height, info.pixel_format);
            info
        }
        Err(e) => {
            info!("Failed to get framebuffer: {:?}", e);
            return Status::ABORTED;
        }
    };

    // メモリマップの取得
    let mt = MemoryType::LOADER_DATA;
    let memory_map = match memory_map(mt) {
        Ok(map) => {
            info!("Memory map: {} entries", map.entries().count());
            map
        }
        Err(e) => {
            info!("Failed to get memory map: {:?}", e);
            return Status::ABORTED;
        }
    };

    // メモリマップをファイルに保存（デバッグ用）
    save_memory_map_to_file(image_handle, &memory_map).unwrap();

    // メモリマップをRawMemoryDescriptor配列に変換
    let mut memory_entries = [RawMemoryDescriptor {
        memory_type: 0,
        physical_start: 0,
        virtual_start: 0,
        number_of_pages: 0,
        attribute: 0,
    }; 200];

    let mut entry_count = 0;
    for entry in memory_map.entries() {
        if entry_count >= 200 { break; }
        memory_entries[entry_count] = RawMemoryDescriptor {
            memory_type: entry.ty.0,
            physical_start: entry.phys_start,
            virtual_start: entry.virt_start,
            number_of_pages: entry.page_count,
            attribute: entry.att.bits(),
        };
        entry_count += 1;
    }

    let desc_size = memory_map.meta().desc_size as u64;
    info!("Memory entries: {}, descriptor size: {}", entry_count, desc_size);

    // カーネルの読み込み
    let kernel_data = load_kernel(image_handle).expect("Failed to load kernel");
    info!("Kernel: {} bytes, magic={:02x}{:02x}{:02x}{:02x}",
        kernel_data.len(),
        kernel_data[0], kernel_data[1], kernel_data[2], kernel_data[3]
    );

    // ブートサービス終了
    let _memory_map_final: MemoryMapOwned = unsafe { exit_boot_services(Some(mt)) };

    // カーネルを0x100000にコピーしてジャンプ
    let kernel_entry = 0x100000 as *mut u8;
    unsafe {
        core::ptr::copy_nonoverlapping(kernel_data.as_ptr(), kernel_entry, kernel_data.len() + 12);

        let kernel_main: extern "C" fn() -> ! = core::mem::transmute(kernel_entry);
        kernel_main();
    }
}