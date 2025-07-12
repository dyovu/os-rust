#![no_std]
#![no_main]

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;

use log::info;
use uefi::prelude::*;
use uefi::boot::{memory_map, exit_boot_services, MemoryType, image_handle}; // UEFIのブートサービスを使うためのもの、メモリマップ取得したり
use uefi::mem::memory_map::{MemoryMap, MemoryMapOwned}; 
use uefi::proto::media::file::{File, FileMode, FileAttribute}; // ファイルシステムからカーネルを読み込むためのもの
use uefi::proto::media::fs::SimpleFileSystem; // カーネルを呼び出すためにESPにアクセスするためのもの
use uefi::proto::console::gop::GraphicsOutput; // グラフィック出力プロトコルを使うためのもの

#[repr(C)] // 構造体のメモリレイアウトをC言語と同じにするやつらしい
struct FrameBufferInfo {
    buffer: *mut u8, // フレームバッファのメモリ領域の先頭アドレス
    buffer_size: usize,
    width: usize,
    height: usize,
    pixels_per_scan_line: usize,
    pixel_format: u32, // PixelFormat情報
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct RawMemoryDescriptor {
    pub memory_type: u32, // メモリタイプ（MemoryTypeの値）
    pub physical_start: u64, // 物理アドレスの開始位置
    pub virtual_start: u64, // 仮想アドレスの開始位置
    pub number_of_pages: u64,
    pub attribute: u64, 
}


// uefiクレートが提供するメモリ管理機能を使う
#[global_allocator]
static ALLOCATOR: uefi::allocator::Allocator = uefi::allocator::Allocator;

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();

    let mt: MemoryType = MemoryType::LOADER_DATA;
    let memory_map_result: MemoryMapOwned = match memory_map(mt) {
        Ok(map) => map,
        Err(e) => {
            info!("Failed to get memory map: {:?}", e);
            return Status::ABORTED;
        }
    };
    info!("Memory map retrieved with {} entries", memory_map_result.entries().count());

    
    info!("About to get framebuffer info...");
    // カーネルに渡すためのGraphicsBufferを取得する
    let framebuffer_info: FrameBufferInfo = match get_framebuffer_info() {
        Ok(info) => info,
        Err(e) => {
            info!("Failed to get framebuffer info: {:?}", e);
            return Status::ABORTED;
        }
    };
    info!("Framebuffer info: width={}, height={}, pixel_format={}",
        framebuffer_info.width, framebuffer_info.height, framebuffer_info.pixel_format);

    // カーネルファイルを読み込み
    let kernel_data = load_kernel().expect("Failed to load kernel");
    info!("Kernel loaded, size: {} bytes", kernel_data.len());
    info!("Kernel first bytes: {:02x} {:02x} {:02x} {:02x}", 
      kernel_data[0], kernel_data[1], kernel_data[2], kernel_data[3]);

    // ブートサービス終了
    info!("Getting fresh memory map for exit...");
    let pre_exit_memory_map = match memory_map(mt) {
        Ok(map) => map,
        Err(e) => {
            info!("Failed to get pre-exit memory map: {:?}", e);
            return Status::ABORTED;
        }
    };
    info!("Pre-exit memory map has {} entries", pre_exit_memory_map.entries().count());

    // エントリー情報を事前に収集（ブートサービス終了前）
    let mut memory_entries = Vec::new();
    for entry in pre_exit_memory_map.entries() {
        memory_entries.push(RawMemoryDescriptor {
            memory_type: entry.ty.0,
            physical_start: entry.phys_start,
            virtual_start: entry.virt_start,
            number_of_pages: entry.page_count,
            attribute: entry.att.bits(),
        });
    }
    let meta_desc_size = pre_exit_memory_map.meta().desc_size;


    let memory_map_final: MemoryMapOwned = unsafe { exit_boot_services(Some(mt)) };


    // カーネルを1MBにコピーしてジャンプ
    let kernel_entry = 0x100000 as *mut u8;
    unsafe {
        core::ptr::copy_nonoverlapping( // カーネルデータを1MBのアドレスにコピー
            kernel_data.as_ptr(),
            kernel_entry,
            kernel_data.len()
        );
        
        let kernel_main: extern "C" fn(
            memory_entries_ptr: *const RawMemoryDescriptor,
            memory_entries_len: usize,
            descriptor_size: usize,
            framebuffer_info: FrameBufferInfo
        ) -> ! = 
            core::mem::transmute(kernel_entry);
        kernel_main(
            memory_entries.as_ptr(),
            memory_entries.len(),
            meta_desc_size,
            framebuffer_info
        );
    }
}

fn get_framebuffer_info() -> Result<FrameBufferInfo, uefi::Error> {
    let handle = uefi::boot::get_handle_for_protocol::<GraphicsOutput>()?;
    let mut graphics_output = uefi::boot::open_protocol_exclusive::<GraphicsOutput>(handle)?;
    
    let mode_info = graphics_output.current_mode_info();
    let mut framebuffer = graphics_output.frame_buffer();
    
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