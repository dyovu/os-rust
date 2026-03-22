#![no_main]
#![no_std]

use core::panic::PanicInfo;

mod serial;
use serial::{serial_print_str, print_decimal, print_hex};

mod graphics;
use graphics::framebuffer::{FrameBufferInfo, fill_screen_blue, draw_gradient};
use graphics::font::draw_string;
use graphics::console;


// ================================================================
// 構造体定義
// ================================================================

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
// パニックハンドラ
// ================================================================

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// ================================================================
// デバッグ用出力ヘルパー
// ================================================================

fn print_memory_size(bytes: u64) {
    if bytes >= 1024 * 1024 * 1024 {
        print_decimal(bytes / (1024 * 1024 * 1024));
        serial_print_str(" GB");
        let remainder = (bytes % (1024 * 1024 * 1024)) / (1024 * 1024);
        if remainder > 0 { serial_print_str(" "); print_decimal(remainder); serial_print_str(" MB"); }
    } else if bytes >= 1024 * 1024 {
        print_decimal(bytes / (1024 * 1024));
        serial_print_str(" MB");
        let remainder = (bytes % (1024 * 1024)) / 1024;
        if remainder > 0 { serial_print_str(" "); print_decimal(remainder); serial_print_str(" KB"); }
    } else if bytes >= 1024 {
        print_decimal(bytes / 1024);
        serial_print_str(" KB");
        let remainder = bytes % 1024;
        if remainder > 0 { serial_print_str(" "); print_decimal(remainder); serial_print_str(" bytes"); }
    } else {
        print_decimal(bytes);
        serial_print_str(" bytes");
    }
}

fn print_memory_type(memory_type: u32) {
    match memory_type {
        0  => serial_print_str("Reserved           "),
        1  => serial_print_str("LoaderCode         "),
        2  => serial_print_str("LoaderData         "),
        3  => serial_print_str("BootServicesCode   "),
        4  => serial_print_str("BootServicesData   "),
        5  => serial_print_str("RuntimeServicesCode"),
        6  => serial_print_str("RuntimeServicesData"),
        7  => serial_print_str("ConventionalMemory "),
        8  => serial_print_str("UnusableMemory     "),
        9  => serial_print_str("ACPIReclaimMemory  "),
        10 => serial_print_str("ACPIMemoryNVS      "),
        11 => serial_print_str("MemoryMappedIO     "),
        12 => serial_print_str("MemoryMappedIOPort "),
        13 => serial_print_str("PalCode            "),
        _ => {
            serial_print_str("Unknown(");
            print_decimal(memory_type as u64);
            serial_print_str(")        ");
        }
    }
}

fn print_pixel_format(format: u32) {
    match format {
        0 => serial_print_str("RGB"),
        1 => serial_print_str("BGR"),
        2 => serial_print_str("Bitmask"),
        3 => serial_print_str("BltOnly"),
        _ => {
            serial_print_str("Unknown(");
            print_decimal(format as u64);
            serial_print_str(")");
        }
    }
}


// ================================================================
// エントリーポイント
// ================================================================

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text._start")]
pub extern "sysv64" fn _start(
    framebuffer_info: &FrameBufferInfo,
    mmap_ptr: *const RawMemoryDescriptor,
    mmap_len: usize,
) -> ! {
    serial_print_str("\n");
    serial_print_str("=====================================\n");
    serial_print_str("    KERNEL BOOT INFORMATION\n");
    serial_print_str("=====================================\n");
    serial_print_str("Hello, world from kernel\n");

    serial_print_str("Memory entries pointer: 0x");
    print_hex(mmap_ptr as u64);
    serial_print_str("\n");

    serial_print_str("Framebuffer pointer: 0x");
    print_hex(framebuffer_info.buffer as u64);
    serial_print_str("\n");

    serial_print_str("Memory entries: ");
    print_decimal(mmap_len as u64);
    serial_print_str("\n");

    serial_print_str("Resolution: ");
    print_decimal(framebuffer_info.width as u64);
    serial_print_str(" x ");
    print_decimal(framebuffer_info.height as u64);
    serial_print_str("\n");

    fill_screen_blue(
        framebuffer_info.buffer as u64,
        framebuffer_info.stride as u64,
        framebuffer_info.height as u64,
    );

    draw_string(
        framebuffer_info.buffer as u64,
        framebuffer_info.stride as u64,
        10, 10,
        "Hello, World!",
        0xFF_FF_FF_FF, // 白
    );

    draw_string(
        framebuffer_info.buffer as u64,
        framebuffer_info.stride as u64,
        10, 30,
        "OS Development!",
        0xFF_00_FF_00, // 緑
    );

    loop {}
}