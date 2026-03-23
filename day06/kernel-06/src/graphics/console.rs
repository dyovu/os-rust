// ================================================================
// 文字を描画する際の、改行、スクロールのプログラム
// ================================================================
use core::ptr::{copy_nonoverlapping, write_bytes};
use core::fmt;
use spin::Mutex;

use crate::graphics::font::draw_char;
use crate::graphics::framebuffer::{PixelWriter, PixelColor};

const ROWS: usize = 25;
const COLUMNS: usize = 80;

#[repr(C)]
pub struct Console {
    writer: &'static Mutex<Option<PixelWriter>>,
    fg_color: PixelColor,
    bg_color: PixelColor,
    buffer: [[char; COLUMNS + 1]; ROWS],
    cursor_row: usize,
    cursor_column: usize,
}

impl Console {
    pub fn new(writer: &'static Mutex<Option<PixelWriter>>, fg_color: PixelColor, bg_color: PixelColor) -> Self {
        Self {
            writer,
            fg_color,
            bg_color,
            buffer: [[' '; COLUMNS + 1]; ROWS],
            cursor_row: 0,
            cursor_column: 0,
        }
    }

    pub fn put_string(&mut self, text: &str) {
        for c in text.chars() {
            if c == '\n' {
                self.new_line();
            } else if self.cursor_column < COLUMNS {
                if let Some(w) = self.writer.lock().as_ref() {
                    draw_char(w, 8 * self.cursor_column as u64, 16 * self.cursor_row as u64, c, self.fg_color);
                }
                self.buffer[self.cursor_row][self.cursor_column] = c;
                self.cursor_column += 1;
            }
        }
    }

    fn new_line(&mut self) {
        self.cursor_column = 0;
        if self.cursor_row < ROWS - 1 {
            self.cursor_row += 1;
        } else {
            let guard = self.writer.lock();
            if let Some(w) = guard.as_ref() {
                for y in 0..16 * ROWS {
                    for x in 0..8 * COLUMNS {
                        w.write(x as u64, y as u64, self.bg_color);
                    }
                }

                for i in 0..ROWS - 1 {
                    unsafe {
                        copy_nonoverlapping(
                            self.buffer[i + 1].as_ptr(),
                            self.buffer[i].as_mut_ptr(),
                            COLUMNS + 1,
                        );
                    }
                    for (col, c) in self.buffer[i].iter().enumerate() {
                        draw_char(w, (8 * col) as u64, (16 * i) as u64, *c, self.fg_color);
                    }
                }
            }
            // guardがここでドロップされロック解放

            unsafe {
                write_bytes(
                    self.buffer[ROWS - 1].as_mut_ptr(),
                    0,
                    COLUMNS + 1,
                );
            }
        }
    }
}

impl fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.put_string(s);
        Ok(())
    }
}