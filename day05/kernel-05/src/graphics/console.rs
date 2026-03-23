// ================================================================
// 文字を描画する際の、改行、スクロールのプログラム
// ================================================================
use core::ptr::{copy_nonoverlapping, write_bytes};

use crate::graphics::font::draw_char;
use crate::graphics::framebuffer::set_pixel;

const ROWS: usize = 25;
const COLUMNS: usize = 80;

#[repr(C)]
pub struct Console{
    buffer: [[char; COLUMNS + 1]; ROWS],
    cursor_row: usize,
    cursor_column: usize,
}

impl Console{
    pub fn new() -> Self{
        Self{
            buffer: [[' '; COLUMNS + 1]; ROWS],
            cursor_row: 0,
            cursor_column: 0,
        }
    }

    pub fn put_string(&mut self, text: &str, fb_buffer: u64, stride: u64, color: u32) {
        for c in text.chars(){
            if c == '\n'{
                self.new_line(fb_buffer, stride, color);
            }else if self.cursor_column < COLUMNS{
                draw_char(fb_buffer, stride, 8*self.cursor_column as u64, 16*self.cursor_row as u64, c, color);
                self.buffer[self.cursor_row][self.cursor_column] = c;
                self.cursor_column += 1;
            }
        }
    }

    fn new_line(&mut self, fb_buffer: u64, stride: u64, color: u32){
        self.cursor_column = 0;
        if self.cursor_row < ROWS-1{
            self.cursor_row += 1;
        }else{
            for y in 0..16 * ROWS {
                for x in 0..8 * COLUMNS {
                    set_pixel(fb_buffer, x as u64, y as u64, stride, 0x00_00_00_FF);
                }
            }

            // 配列の要素を指定した分シフトするメソッド
            // 基本的にこれを使えばいいけど、memcopyとmemsetのような形で実装するために
            // 生ポインタで実装する
            // 
            // self.buffer.rotate_left(1);
            // 

            for i in 0..ROWS -1{
                let tmp = self.buffer[i+1].as_ptr();
                let fil = self.buffer[i].as_mut_ptr();
                unsafe{
                    copy_nonoverlapping(
                        tmp,
                        fil,
                        COLUMNS+1,
                    )
                }

                for (col, c) in self.buffer[i].iter().enumerate() {
                    draw_char(fb_buffer, stride, (8 * col) as u64, (16 * i) as u64, *c, color);
                }
            }

            unsafe{
                write_bytes(
                    self.buffer[ROWS-1].as_mut_ptr(),
                    0, 
                    COLUMNS+1,
                );
            }
        }
    }
}