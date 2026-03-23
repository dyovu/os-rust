#![no_main]
#![no_std]

use core::panic::PanicInfo;
use spin::Mutex;

mod serial;
use serial::{serial_print_str, print_decimal, print_hex};
mod graphics;
use graphics::framebuffer::{FrameBufferInfo, fill_screen_blue, draw_gradient};
use graphics::font::draw_string;
use graphics::console::Console;


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
// コンソール出力のためのグローバル変数
// ================================================================

static CONSOLE: Mutex<Option<Console>> = Mutex::new(None);

pub fn printk(text: &str, fb: u64, stride: u64, color: u32) {
    if let Some(c) = CONSOLE.lock().as_mut() {
        c.put_string(text, fb, stride, color);
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
    fill_screen_blue(
        framebuffer_info.buffer as u64,
        framebuffer_info.stride as u64,
        framebuffer_info.height as u64,
    );

    *CONSOLE.lock() = Some(Console::new());

    for _ in 0..27 {
        printk("printk: \n", framebuffer_info.buffer as u64, framebuffer_info.stride as u64, 0xFF_00_FF_00,);
    }


    // draw_string(
    //     framebuffer_info.buffer as u64,
    //     framebuffer_info.stride as u64,
    //     10, 10,
    //     "Hello, World!",
    //     0xFF_FF_FF_FF, // 白
    // );
    // draw_string(
    //     framebuffer_info.buffer as u64,
    //     framebuffer_info.stride as u64,
    //     10, 30,
    //     "OS Development!",
    //     0xFF_00_FF_00, // 緑
    // );

    loop {}
}