// ================================================================
// フレームバッファ描画
// ================================================================

#[repr(C)]
pub struct FrameBufferInfo {
    pub buffer: *mut u8,
    pub buffer_size: usize,
    pub width: usize,
    pub height: usize,
    pub stride: usize,
    pub pixel_format: u32,
}

#[derive(Debug, Copy, Clone)]
pub struct PixelColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Copy, Clone)]
pub enum PixelFormat {
    Rgb,
    Bgr,
}

pub struct PixelWriter {
    pub fb_buffer: u64,
    pub stride: u64,
    pub width: u64,
    pub height: u64,
    pub format: PixelFormat,
}

impl PixelWriter {
    pub fn new(info: &FrameBufferInfo) -> Self {
        let format = match info.pixel_format {
            0 => PixelFormat::Rgb,
            _ => PixelFormat::Bgr,
        };
        Self {
            fb_buffer: info.buffer as u64,
            stride: info.stride as u64,
            width: info.width as u64,
            height: info.height as u64,
            format,
        }
    }

    // color は BGR 32bit フォーマット 基本的に(0x00_RR_GG_BB)
    pub fn write(&self, x: u64, y: u64, color: PixelColor) {
        let c = match self.format {
            // byte[0]=R, byte[1]=G, byte[2]=B
            PixelFormat::Rgb => {
                ((color.b as u32) << 16) | ((color.g as u32) << 8) | (color.r as u32)
            }
            // byte[0]=B, byte[1]=G, byte[2]=R
            PixelFormat::Bgr => {
                ((color.r as u32) << 16) | ((color.g as u32) << 8) | (color.b as u32)
            }
        };
        let framebuffer = self.fb_buffer as *mut u32;
        let offset = (y * self.stride + x) as isize;
        unsafe { core::ptr::write(framebuffer.offset(offset), c); }
    }

    pub fn fill(&self, color: PixelColor) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.write(x, y, color);
            }
        }
    }
}