// ================================================================
// フォント・テキスト描画
// ================================================================

use crate::graphics::framebuffer::{PixelWriter, PixelColor};

static FONT_8X16: &[u8] = include_bytes!("../../assets/font.psf");

// packed: フィールドをパディングなしで詰めてレイアウトする
#[repr(C, packed)]
struct Psf2Header {
    magic: u32,       // マジックナンバー: 0x864AB572
    version: u32,
    header_size: u32, // フォントデータの開始オフセット
    flags: u32,
    length: u32,      // グリフ数
    char_size: u32,   // 1グリフあたりのバイト数
    height: u32,
    width: u32,
}

// ASCII (0–255) のみ対応
pub fn draw_char(writer: &PixelWriter, x: u64, y: u64, ch: char, color: PixelColor) {
    let char_code = ch as usize;
    if char_code >= 256 { return; }

    let header = unsafe { &*(FONT_8X16.as_ptr() as *const Psf2Header) };
    if header.magic != 0x864ab572 { return; }

    let char_height    = header.height as usize;
    let char_width     = header.width as usize;
    let bytes_per_line = (char_width + 7) / 8;

    // ヘッダ直後がフォントデータの先頭
    let font_data_start = FONT_8X16.as_ptr() as usize + header.header_size as usize;
    let glyph_ptr = (font_data_start + char_code * header.char_size as usize) as *const u8;

    for row in 0..char_height {
        for col in 0..char_width {
            let byte_index = col / 8;
            let bit_index  = 7 - (col % 8); // MSBが左端
            let byte = unsafe { *glyph_ptr.add(row * bytes_per_line + byte_index) };
            if (byte & (1 << bit_index)) != 0 {
                writer.write(x + col as u64, y + row as u64, color);
            }
        }
    }
}

// 現在は改行非対応（'\n' は無視）
pub fn draw_string(writer: &PixelWriter, mut x: u64, y: u64, text: &str, color: PixelColor) {
    const CHAR_WIDTH: u64 = 8;
    for ch in text.chars() {
        if ch == '\n' { continue; } // TODO: 改行処理
        draw_char(writer, x, y, ch, color);
        x += CHAR_WIDTH;
    }
}