#![no_std]
#![no_main]

extern crate alloc;
use alloc::format;
use alloc::vec;
use alloc::vec::Vec;
use core::arch::asm;
use core::ptr::{copy_nonoverlapping, write_bytes};
use log::info;

use uefi::boot::{image_handle, exit_boot_services, memory_map, MemoryType, allocate_pages};
use uefi::mem::memory_map::{MemoryMap, MemoryMapOwned};
use uefi::prelude::*;
use uefi::proto::console::gop::GraphicsOutput;
use uefi::proto::media::file::{self, File, FileAttribute, FileMode, FileInfo};

// ================================================================
// 構造体定義
// ================================================================

#[repr(C)]
struct FrameBufferInfo {
    buffer: *mut u8,
    buffer_size: usize,
    width: usize,
    height: usize,
    stride: usize,
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

// ELFファイルを解析するための構造体
// ファイルヘッダの構造体
#[repr(C)]
pub struct Elf64_Ehdr {
    pub e_ident: [u8; 16],
    pub e_type: u16,
    pub e_machine: u16,
    pub e_version: u32,
    pub e_entry: u64,
    pub e_phoff: u64,
    pub e_shoff: u64,
    pub e_flags: u32,
    pub e_ehsize: u16,
    pub e_phentsize: u16,
    pub e_phnum: u16,
    pub e_shentsize: u16,
    pub e_shnum: u16,
    pub e_shstrndx: u16,
}

// プログラムヘッダの構造体
#[repr(C)]
pub struct Elf64_Phdr {
    pub p_type: u32,
    pub p_flags: u32,
    pub p_offset: u64,
    pub p_vaddr: u64,
    pub p_paddr: u64,
    pub p_filesz: u64,
    pub p_memsz: u64,
    pub p_align: u64,
}

// ================================================================
// グローバル変数
// ================================================================

#[global_allocator]
static ALLOCATOR: uefi::allocator::Allocator = uefi::allocator::Allocator;

const PT_LOAD: u32 = 1;

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
    memory_map_data: &uefi::mem::memory_map::MemoryMapOwned
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

    let header = format!("Memory Map - {} entries\n\n", memory_map_data.entries().count());
    let _ = file.write(header.as_bytes());

    for (i, desc) in memory_map_data.entries().enumerate() {
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
        stride: mode_info.stride(),
        pixel_format: match mode_info.pixel_format() {
            uefi::proto::console::gop::PixelFormat::Rgb => 0,
            uefi::proto::console::gop::PixelFormat::Bgr => 1,
            _ => 2,
        },
    })
}

// ================================================================
// カーネルファイルの読み込み、コピー
// ================================================================

fn load_kernel(image_handle: uefi::Handle) -> Result<Vec<u8>, uefi::Error> {
    let mut file_system = boot::get_image_file_system(image_handle)?;
    let mut root: file::Directory = file_system.open_volume()?;

    let mut kernel_file = root
        .open(cstr16!("kernel"), FileMode::Read, FileAttribute::empty())
        .expect("Failed to open kernel file")
        .into_regular_file()
        .expect("Kernel file is not a regular file");

    // Cではsize_ofでファイル名の長さに応じたサイズを動的に決められるが、rustではできない
    // 一度サイズ0でget_infoをよびだすか、下記のようにある程度大きなbufを用意しておく
    let mut info_buffer = vec![0u8; 1024];
    // ファイルのメタ情報を読み込む
    let file_info = kernel_file
        .get_info::<FileInfo>(&mut info_buffer)
        .expect("Failed to get file info");

    let mut buffer = vec![0u8; file_info.file_size() as usize];
    kernel_file.read(&mut buffer)?;

    Ok(buffer)
}

// 読み込むカーネルのアドレス範囲を計算する
fn calc_load_address_range(elf_header:*const Elf64_Ehdr) -> Result<(u64, u64),uefi::Error >{
    let ehdr_ref = unsafe { &*elf_header };
    let phdr =(elf_header as usize + ehdr_ref.e_phoff as usize) as *const Elf64_Phdr;
    // Rustではポインタに対してC言語のように直接 [i] でアクセスできないため、ポインタからスライス（配列）を作る
    let phdr_slice = unsafe { core::slice::from_raw_parts(phdr, ehdr_ref.e_phnum as usize) };

    let mut first = u64::MAX;
    let mut last:u64 =  0;

    for i in 0..ehdr_ref.e_phnum as usize{
        // PT_LOAD の値は通常 1 らしい
        if phdr_slice[i].p_type != PT_LOAD {continue}
        first = first.min(phdr_slice[i].p_vaddr);
        last = last.max(phdr_slice[i].p_vaddr + phdr_slice[i].p_memsz);
    }
    Ok((first, last))
}

// 実際にカーネルのロードセクションをメモリの指定の位置にコピーする
fn copy_load_segments(elf_header:*const Elf64_Ehdr) {
    let ehdr_ref = unsafe { &*elf_header };
    let phdr =(elf_header as usize + ehdr_ref.e_phoff as usize) as *const Elf64_Phdr;
    let phdr_slice = unsafe { core::slice::from_raw_parts(phdr, ehdr_ref.e_phnum as usize) };

    for i in 0..ehdr_ref.e_phnum as usize{
        if phdr_slice[i].p_type != PT_LOAD {continue}
        let phdr_ref = &phdr_slice[i];
        let segm_in_file = elf_header as usize+ phdr_ref.p_offset as usize;
        unsafe {
            copy_nonoverlapping(
                segm_in_file as *const u8, 
                phdr_ref.p_vaddr as *mut u8, 
                phdr_ref.p_filesz as usize
            );

            write_bytes(
                (phdr_ref.p_vaddr + phdr_ref.p_filesz) as *mut u8,
                0,
                (phdr_ref.p_memsz - phdr_ref.p_filesz) as usize
            );
        }

    }
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
    let memory_map_data = match memory_map(mt) {
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
    // save_memory_map_to_file(image_handle, &memory_map_data).unwrap();

    // メモリマップをRawMemoryDescriptor配列に変換
    let mut memory_entries = [RawMemoryDescriptor {
        memory_type: 0,
        physical_start: 0,
        virtual_start: 0,
        number_of_pages: 0,
        attribute: 0,
    }; 200];

    let mut entry_count = 0;
    for entry in memory_map_data.entries() {
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

    let desc_size = memory_map_data.meta().desc_size as u64;
    info!("Memory entries: {}, descriptor size: {}", entry_count, desc_size);


    // カーネルの読み込み
    let mut kernel_data = load_kernel(image_handle).expect("Failed to load kernel");
    let elf_header = kernel_data.as_mut_ptr() as *const Elf64_Ehdr;
    let (kernel_first_addr, kernel_last_addr) = calc_load_address_range(elf_header).expect("Failed calculate address range");

    let page_num = ((kernel_last_addr - kernel_first_addr + 0xfff) / 0x1000 )as usize;
    allocate_pages(boot::AllocateType::Address(kernel_first_addr), MemoryType::LOADER_DATA, page_num).expect("Failed to allocate kernel address");
    copy_load_segments(elf_header);
    
    info!("Kernel: {} bytes, magic={:02x}{:02x}{:02x}{:02x}",
        kernel_data.len(),
        kernel_data[0], kernel_data[1], kernel_data[2], kernel_data[3]
    );

    let ehdr_ref = unsafe { &*elf_header };

    // ブートサービス終了
    let _memory_map_final: MemoryMapOwned = unsafe { exit_boot_services(Some(mt)) };

    // kernelのエントリーポイントと関数のシグネチャを指定
    let entry_point = ehdr_ref.e_entry as usize;
    type KernelMain = extern "sysv64" fn(info: &FrameBufferInfo, mmap_ptr: *const RawMemoryDescriptor, mmap_len: usize) -> !;
    unsafe {
        let kernel_main: KernelMain = core::mem::transmute(entry_point);

        kernel_main(&framebuffer_info, memory_entries.as_ptr(), entry_count);
    }
    
}