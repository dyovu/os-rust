#![no_main]
#![no_std]

use core::arch::asm;
use core::panic::PanicInfo;


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
    pub memory_type: u32,
    pub physical_start: u64,
    pub virtual_start: u64,
    pub number_of_pages: u64,
    pub attribute: u64,
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

fn serial_print_str(s: &str) {
    unsafe {
        for byte in s.bytes() {
            loop {
                let status: u8;
                asm!("in al, dx", out("al") status, in("dx") 0x3fdu16);
                if (status & 0x20) != 0 {
                    break;
                }
            }
            asm!("out dx, al", in("dx") 0x3f8u16, in("al") byte);
        }
    }
}

fn serial_print_num(digits: &[u8]) {
    unsafe{
        for i in (0..digits.len()).rev() { // サイズの異なる配列を扱うためにスライスで受け取って、要素数でfor回す
            let digit = digits[i];
            loop {
                let status: u8;
                asm!("in al, dx", out("al") status, in("dx") 0x3fdu16);
                if (status & 0x20) != 0 {
                    break;
                }
            }
            asm!("out dx, al", in("dx") 0x3f8u16, in("al") digit);
        }
    }
}

fn print_decimal(mut num: u64) {
    if num == 0 {
        serial_print_str("0");
        return;
    }
    
    let mut digits = [0u8; 20];
    let mut count = 0;
    
    while num > 0 {
        let digit = num % 10;
        digits[count] = b'0' + digit as u8;
        num = num / 10;
        count = count + 1;
    }
    
    serial_print_num(&digits[0..count]);
}

fn print_hex(mut value: u64) {
    let hex_chars = b"0123456789abcdef";
    let mut buffer = [0u8; 16];
    let mut count = 0;
    
    if value == 0 {
        serial_print_str("0");
        return;
    }
    
    while value > 0 {
        buffer[count] = hex_chars[(value & 0xf) as usize];
        value >>= 4;
        count += 1;
    }
    
    serial_print_num(&buffer[0..count]);
}

fn print_memory_size(bytes: u64) {
    if bytes >= 1024 * 1024 * 1024 {
        print_decimal(bytes / (1024 * 1024 * 1024));
        serial_print_str(" GB");
        let remainder = (bytes % (1024 * 1024 * 1024)) / (1024 * 1024);
        if remainder > 0 {
            serial_print_str(" ");
            print_decimal(remainder);
            serial_print_str(" MB");
        }
    } else if bytes >= 1024 * 1024 {
        print_decimal(bytes / (1024 * 1024));
        serial_print_str(" MB");
        let remainder = (bytes % (1024 * 1024)) / 1024;
        if remainder > 0 {
            serial_print_str(" ");
            print_decimal(remainder);
            serial_print_str(" KB");
        }
    } else if bytes >= 1024 {
        print_decimal(bytes / 1024);
        serial_print_str(" KB");
        let remainder = bytes % 1024;
        if remainder > 0 {
            serial_print_str(" ");
            print_decimal(remainder);
            serial_print_str(" bytes");
        }
    } else {
        print_decimal(bytes);
        serial_print_str(" bytes");
    }
}

fn print_memory_type(memory_type: u32) {
    match memory_type {
        0 => serial_print_str("Reserved          "),
        1 => serial_print_str("LoaderCode        "),
        2 => serial_print_str("LoaderData        "),
        3 => serial_print_str("BootServicesCode  "),
        4 => serial_print_str("BootServicesData  "),
        5 => serial_print_str("RuntimeServicesCode"),
        6 => serial_print_str("RuntimeServicesData"),
        7 => serial_print_str("ConventionalMemory"),
        8 => serial_print_str("UnusableMemory    "),
        9 => serial_print_str("ACPIReclaimMemory "),
        10 => serial_print_str("ACPIMemoryNVS     "),
        11 => serial_print_str("MemoryMappedIO    "),
        12 => serial_print_str("MemoryMappedIOPort"),
        13 => serial_print_str("PalCode           "),
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

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text._start")]
pub extern "C" fn _start(
    // mem_ptr: u64,
    // mem_len: u64, 
    // desc_size: u64,
    // fb_width: u64,
    // fb_height: u64,
    // fb_buffer: u64
) -> ! {
    // 0x200000番地から全ての引数を読み取り
    let args_address = 0x80000 as *const u64;
    let (mem_ptr, mem_len, desc_size, fb_width, fb_height, fb_buffer) = unsafe {
        (
            core::ptr::read(args_address.offset(0)), // mem_ptr
            core::ptr::read(args_address.offset(1)), // mem_len
            core::ptr::read(args_address.offset(2)), // desc_size
            core::ptr::read(args_address.offset(3)), // fb_width
            core::ptr::read(args_address.offset(4)), // fb_height
            core::ptr::read(args_address.offset(5)), // fb_buffer
        )
    };


    // let args_address_safe = 0x91100 as *const u64;    // 安全な場所
    // let args_address_stable = 0x80000 as *const u64;  // 上書きされる場所


    // let mem_len_raw = unsafe { core::ptr::read(args_address_safe.offset(2)) };
    // let desc_size_raw = unsafe { core::ptr::read(args_address_safe.offset(3)) };

    // // デバッグ出力を追加
    // serial_print_str("DEBUG - Raw mem_len from 0x91110: ");
    // print_decimal(mem_len_raw);
    // serial_print_str("\n");

    // serial_print_str("DEBUG - Raw desc_size from 0x91118: ");
    // print_decimal(desc_size_raw);
    // serial_print_str("\n");



    // let (mem_len, desc_size, mem_ptr, fb_width, fb_height, fb_buffer) = unsafe {
    //     (
            
    //         core::ptr::read(args_address_safe.offset(2)),   // mem_len
    //         core::ptr::read(args_address_safe.offset(3)),   // desc_size

    //         core::ptr::read(args_address_stable.offset(0)),   // mem_ptr
    //         core::ptr::read(args_address_stable.offset(1)), // fb_width
    //         core::ptr::read(args_address_stable.offset(2)), // fb_height
    //         core::ptr::read(args_address_stable.offset(3)), // fb_buffer
    //     )
    // };

    serial_print_str("\n");
    serial_print_str("=====================================\n");
    serial_print_str("    KERNEL BOOT INFORMATION\n");
    serial_print_str("=====================================\n");

    serial_print_str("Hello, world from kernel\n");
    
    serial_print_str("Memory entries pointer: 0x");
    print_hex(mem_ptr);
    serial_print_str("\n");

    serial_print_str("Framebuffer pointer: 0x");
    print_hex(fb_buffer);
    serial_print_str("\n");
    
    serial_print_str("Memory entries: ");
    print_decimal(mem_len);
    serial_print_str("\n");
    
    serial_print_str("Descriptor size: ");
    print_decimal(desc_size);
    serial_print_str("\n");
    
    serial_print_str("Resolution: ");
    print_decimal(fb_width);
    serial_print_str(" x ");
    print_decimal(fb_height);
    serial_print_str("\n");

    serial_print_str("\n=== KERNEL READY ===\n");
    serial_print_str("System initialization completed successfully!\n");
    serial_print_str("Kernel is now running...\n\n");
    
    loop {}
}