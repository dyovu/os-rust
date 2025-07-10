

#![no_main]
#![no_std]

use core::arch::asm;
use core::panic::PanicInfo;

/// パニックしたらマジでやばいからloopさせる
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! { // '!'は絶対に呼び出し元の関数に返らないことを意味する
    loop {}
}

// VGAバッファに直接書き込
// 設定的にVGAバッファがなくてできなそう
fn _print_hello() {
    let vga_buffer = 0xb8000 as *mut u8;
    let message = b"Hello, kernel!";
    
    unsafe {
        for (i, &byte) in message.iter().enumerate() {
            *vga_buffer.offset(i as isize * 2) = byte;
            *vga_buffer.offset(i as isize * 2 + 1) = 0x0f; // 白文字
        }
    }
}

// シリアルポートに文字列をそうしんする, 
fn serial_print(s: &str) {
    unsafe {
        for byte in s.bytes() {
            // 送信準備完了まで待機
            loop {
                let status: u8;
                asm!("in al, dx", out("al") status, in("dx") 0x3fdu16);
                if (status & 0x20) != 0 {
                    break;
                }
            }
            // データ送信
            asm!("out dx, al", in("dx") 0x3f8u16, in("al") byte);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    print_hello();
    serial_print("Hello from the kernel!\n");
    loop {}
}