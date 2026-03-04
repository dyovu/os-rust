#![no_main]
#![no_std]

use core::arch::asm; // インラインアセンブリ用のライブラリ
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



// 与えられたフレームバッファのアドレスに与えられた色でピクセルを設定
fn set_pixel(fb_buffer: u64, x: u64, y: u64, width: u64, color: u32) {
    let framebuffer = fb_buffer as *mut u32;
    let offset = (y * width + x) as isize;
    unsafe {
        core::ptr::write(framebuffer.offset(offset), color);
    }
}

// ブルスク
fn fill_screen_blue(fb_buffer: u64, width: u64, height: u64) {
    let framebuffer = fb_buffer as *mut u32;
    let total_pixels = (width * height) as isize;
    
    unsafe {
        for i in 0..total_pixels {
            // BGR形式なので：0x00_RR_GG_BB
            // 赤色 = 0x000000FF
            core::ptr::write(framebuffer.offset(i), 0x000000FF);
        }
    }
}
// グラデーションを描画する関数
fn draw_gradient(fb_buffer: u64, width: u64, height: u64) {
    for y in 0..height {
        for x in 0..width {
            let red = (x * 255 / width) as u32;
            let green = (y * 255 / height) as u32;
            let blue = 128u32;
            let color = (red << 16) | (green << 8) | blue; // BGR形式
            set_pixel(fb_buffer, x, y, width, color);
        }
    }
}

/*
* 以下与えられたフォントデータから文字列を描画するための関数たち
* あとで別ファイルとか分けて構造体に対してimplするようなじっそうにする　
*/

static FONT_8X16: &[u8] = include_bytes!("../assets/font.psf");
#[repr(C, packed)]
struct Psf1Header {
    magic: [u8; 2],
    mode: u8,
    char_size: u8, // 文字の高さ
}


fn draw_char(
    fb_buffer: u64, 
    fb_width: u64, 
    x: u64, 
    y: u64, 
    ch: char, 
    color: u32
) {
    let char_code = ch as usize;
    if char_code >= 256 { return; } // 範囲外チェック

    const HEADER_SIZE: usize = 4;
    let header = unsafe { &*(FONT_8X16.as_ptr() as *const Psf1Header) };

    if header.magic[0] != 0x36 || header.magic[1] != 0x04 {
        return;
    }

    let char_height = header.char_size as usize;
    let char_width = 8; // PSF v1は8px幅で固定
    
    let font_char_data_start = FONT_8X16.as_ptr() as usize + HEADER_SIZE;
    let char_offset = char_code * char_height;
    
    let font_glyph_ptr = (font_char_data_start + char_offset) as *const u8;

    for row in 0..char_height {
        let byte = unsafe { *font_glyph_ptr.add(row) };
        for col in 0..char_width {
            // ビットが1ならピクセルを描画
            if (byte & (0x80 >> col)) != 0 {
                set_pixel(fb_buffer, x + col as u64, y + row as u64, fb_width, color);
            }
        }
    }
    serial_print_str("display char: ");
    // let mut buffer = [0u8; 4]; // UTF-8の1文字は最大4バイト
    
    // // 2. バッファにcharを書き込み、&strとして受け取る
    // let s = ch.encode_utf8(&mut buffer);

    // // 3. &strをシリアルポートに出力
    // serial_print_str(s);
    // serial_print_str("\n"); // 改行を追加すると見やすい
}

// 文字列を描画する関数
fn draw_string(
    fb_buffer: u64,
    fb_width: u64,
    mut x: u64,
    y: u64,
    text: &str,
    color: u32
) {
    let char_width = 8; // 8x16フォントなので文字の幅は8px
    for ch in text.chars() {
        if ch == '\n' {
            // 改行処理は後で実装
            continue;
        }
        draw_char(fb_buffer, fb_width, x, y, ch, color);
        x += char_width; // 次の文字位置に移動
    }
    serial_print_str(text);
}



#[unsafe(no_mangle)]
#[unsafe(link_section = ".text._start")]
pub extern "C" fn _start() -> ! {

    // ポインタのアドレスを指定してブートローダーから引数を受け取る
    let args_address = 0x80000 as *const u64;
    let (mem_ptr, mem_len, desc_size, framebuffer_info_ptr) = unsafe {
        (
            core::ptr::read(args_address.offset(0)), // mem_ptr
            core::ptr::read(args_address.offset(1)), // mem_len
            core::ptr::read(args_address.offset(2)), // desc_size
            core::ptr::read(args_address.offset(3)), // framebuffer_info_ptr
        )
    };

    // FrameBufferInfo構造体を読み取り
    let framebuffer_info = unsafe {
        core::ptr::read(framebuffer_info_ptr as *const FrameBufferInfo)
    };

    serial_print_str("\n");
    serial_print_str("=====================================\n");
    serial_print_str("    KERNEL BOOT INFORMATION\n");
    serial_print_str("=====================================\n");

    serial_print_str("Hello, world from kernel\n");

    serial_print_str("Memory entries pointer: 0x");
    print_hex(mem_ptr);
    serial_print_str("\n");

    serial_print_str("Framebuffer pointer: 0x");
    print_hex(framebuffer_info.buffer as u64);
    serial_print_str("\n");

    serial_print_str("Memory entries: ");
    print_decimal(mem_len);
    serial_print_str("\n");

    serial_print_str("Descriptor size: ");
    print_decimal(desc_size);
    serial_print_str("\n");

    serial_print_str("Resolution: ");
    print_decimal(framebuffer_info.width as u64);
    serial_print_str(" x ");
    print_decimal(framebuffer_info.height as u64);
    serial_print_str("\n");

    fill_screen_blue(framebuffer_info.buffer as u64, framebuffer_info.width as u64, framebuffer_info.height as u64);

    // draw_gradient(framebuffer_info.buffer as u64, framebuffer_info.width as u64, framebuffer_info.height as u64);

    draw_string(
        framebuffer_info.buffer as u64,
        framebuffer_info.width as u64,
        10,  // x座標
        10,  // y座標
        "Hello, World!",
        0xFF_FF_FF_FF  // 白色（ARGB）
    );

    draw_string(
        framebuffer_info.buffer as u64,
        framebuffer_info.width as u64,
        10,
        30,
        "OS Development!",
        0xFF_00_FF_00  // 緑色
    );
    
    loop {}
}