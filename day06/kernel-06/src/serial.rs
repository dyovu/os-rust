// ================================================================
// シリアルポート出力
// ================================================================

use core::arch::asm;

const SERIAL_PORT:   u16 = 0x3F8; // COM1 データレジスタ
const SERIAL_STATUS: u16 = 0x3FD; // COM1 ステータスレジスタ

// 送信バッファが空くまで待ってから1バイト送信する
fn serial_write_byte(byte: u8) {
    unsafe {
        loop {
            let status: u8;
            asm!("in al, dx", out("al") status, in("dx") SERIAL_STATUS);
            if (status & 0x20) != 0 { break; }
        }
        asm!("out dx, al", in("dx") SERIAL_PORT, in("al") byte);
    }
}

pub fn serial_print_str(s: &str) {
    for byte in s.bytes() {
        serial_write_byte(byte);
    }
}

// digits は小さい桁が先頭のため逆順に出力する
fn serial_print_num(digits: &[u8]) {
    for i in (0..digits.len()).rev() {
        serial_write_byte(digits[i]);
    }
}

pub fn print_decimal(mut num: u64) {
    if num == 0 { serial_print_str("0"); return; }
    let mut digits = [0u8; 20];
    let mut count = 0;
    while num > 0 {
        digits[count] = b'0' + (num % 10) as u8;
        num /= 10;
        count += 1;
    }
    serial_print_num(&digits[0..count]);
}

pub fn print_hex(mut value: u64) {
    let hex_chars = b"0123456789abcdef";
    let mut buffer = [0u8; 16];
    let mut count = 0;
    if value == 0 { serial_print_str("0"); return; }
    while value > 0 {
        buffer[count] = hex_chars[(value & 0xf) as usize];
        value >>= 4;
        count += 1;
    }
    serial_print_num(&buffer[0..count]);
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