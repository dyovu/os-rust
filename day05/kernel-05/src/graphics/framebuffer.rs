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

// color は BGR 32bit フォーマット (0x00_RR_GG_BB)
pub fn set_pixel(fb_buffer: u64, x: u64, y: u64, stride: u64, color: u32) {
    let framebuffer = fb_buffer as *mut u32;
    let offset = (y * stride + x) as isize;
    unsafe { core::ptr::write(framebuffer.offset(offset), color); }
}

pub fn fill_screen_blue(fb_buffer: u64, stride: u64, height: u64) {
    let framebuffer = fb_buffer as *mut u32;
    let total_pixels = (stride * height) as isize;
    unsafe {
        for i in 0..total_pixels {
            core::ptr::write(framebuffer.offset(i), 0x000000FF); // 青 (BGR)
        }
    }
}

pub fn draw_gradient(fb_buffer: u64, stride: u64, height: u64) {
    for y in 0..height {
        for x in 0..stride {
            let red   = (x * 255 / stride) as u32;
            let green = (y * 255 / height) as u32;
            let blue  = 128u32;
            let color = (red << 16) | (green << 8) | blue;
            set_pixel(fb_buffer, x, y, stride, color);
        }
    }
}