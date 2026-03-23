#![no_main]
#![no_std]

use core::panic::PanicInfo;
use core::fmt::Write;
use spin::Mutex;

mod serial;
use serial::{serial_print_str, print_decimal, print_hex};
mod graphics;
use graphics::framebuffer::{FrameBufferInfo, PixelWriter, PixelColor};
use graphics::console::Console;
use graphics::mouse_cursor::{MOUSE_CURSOR_WIDTH, MOUSE_CURSOR_HEIGHT, MOUSE_CURSOR_SHAPE};

// ================================================================
// 色
// ================================================================
const BLACK:PixelColor = PixelColor{r:0, g:0, b:0};
const WHITE:PixelColor = PixelColor{r:255, g:255, b:255};
const RED:PixelColor =  PixelColor{r:255, g:0, b:0};
const GREEN:PixelColor =  PixelColor{r:0, g:255, b:0};
const BLUE:PixelColor =  PixelColor{r:0, g:0, b:255};


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
// グローバルのPixelWriter
// ================================================================

static PIXEL_WRITER: Mutex<Option<PixelWriter>> = Mutex::new(None);

// ================================================================
// コンソール出力のためのグローバル変数
// ================================================================

static CONSOLE: Mutex<Option<Console>> = Mutex::new(None);

pub fn printk(args: core::fmt::Arguments) {
    if let Some(c) = CONSOLE.lock().as_mut() {
        c.write_fmt(args).ok();
    }
}

#[macro_export]
macro_rules! printk {
    ($($arg:tt)*) => {
        $crate::printk(format_args!($($arg)*))
    };
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

    *PIXEL_WRITER.lock() = Some(PixelWriter::new(framebuffer_info));

    // 青で塗りつぶし
    if let Some(w)  = PIXEL_WRITER.lock().as_ref(){
        w.fill(BLUE); 
    }

    *CONSOLE.lock() = Some(Console::new(
        &PIXEL_WRITER,
        GREEN,  // fg: 緑
        BLUE, // bg: 青
    ));

    for i in 0..27 {
        printk!("printk: {}\n", i);
    }

    if let Some(w) = PIXEL_WRITER.lock().as_ref(){
        for dy in 0..MOUSE_CURSOR_HEIGHT{
            for dx in 0..MOUSE_CURSOR_WIDTH{
                match MOUSE_CURSOR_SHAPE[dy].as_bytes()[dx]{
                    b'@' =>  {
                        w.write(200+dx as u64, 100+dy as u64, BLACK);
                    }
                    b'.' => {
                        w.write(200+dx as u64, 100+dy as u64, WHITE);
                    }
                    _ => {}
                }
            }
        }
    }

    loop {}
}