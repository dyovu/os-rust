#![no_main]
#![no_std]

extern crate alloc;
use core::panic::PanicInfo;
use core::fmt::Write;
use core::ops::AddAssign;
use spin::Mutex;
use linked_list_allocator::LockedHeap;

mod serial;
use serial::{serial_print_str, print_decimal, print_hex};
mod graphics;
use graphics::framebuffer::{FrameBufferInfo, PixelWriter, PixelColor};
use graphics::console::Console;
use graphics::mouse_cursor::{MOUSE_CURSOR_WIDTH, MOUSE_CURSOR_HEIGHT, MOUSE_CURSOR_SHAPE};
mod pci;
use pci::{Device, DEVICES, NUM_DEVICE};
mod usb;
use usb::xhci::xhci_controller::Controller;

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
// パニックハンドラ / グローバルアロケータ
// ================================================================

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// usbドライバのデバイス管理、ring bufのための一時的なアロケータ
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();
const HEAP_SIZE: usize = 100 * 1024;
// 配列の実体サイズと、アロケータに伝えるサイズを合わせる
static mut HEAP:[u8; HEAP_SIZE] = [0; HEAP_SIZE];

pub fn init_heap() {
    unsafe {
        let heap_ptr = core::ptr::addr_of_mut!(HEAP) as *mut u8;
        ALLOCATOR.lock().init(heap_ptr, HEAP_SIZE);
    }
}

// ================================================================
// グローバルのPixelWriter
// ================================================================

static PIXEL_WRITER: Mutex<Option<PixelWriter>> = Mutex::new(None);

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Vector2D<T> {
    pub x: T,
    pub y: T,
}

impl<T> Vector2D<T> {
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T, U> AddAssign<Vector2D<U>> for Vector2D<T>
where
    T: AddAssign<U>,
{
    fn add_assign(&mut self, rhs: Vector2D<U>) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

pub fn fill_rectangle(
    writer: &Mutex<Option<PixelWriter>>,
    pos: Vector2D<u64>,
    size: Vector2D<u64>,
    col: PixelColor,
) {
    if let Some(w) = writer.lock().as_ref(){
        for dy in 0..size.y {
            for dx in 0..size.x {
                w.write(pos.x + dx, pos.y + dy, col);
            }
        }
    }
}

pub fn draw_rectangle(
    writer: &Mutex<Option<PixelWriter>>,
    pos: Vector2D<u64>,
    size: Vector2D<u64>,
    col: PixelColor,
) {
    if let Some(w) = writer.lock().as_ref(){
        for dx in 0..size.x {
            w.write(pos.x + dx, pos.y, col);
            w.write(pos.x + dx, pos.y + size.y - 1, col);
        }
        for dy in 1..(size.y - 1) {
            w.write(pos.x, pos.y + dy, col);
            w.write(pos.x + size.x - 1, pos.y + dy, col);
        }
    }
}

pub fn draw_desktop(writer: &Mutex<Option<PixelWriter>>, frame_width: u64, frame_height: u64) {
    let bg_color = PixelColor { r: 30, g: 30, b: 46 };
    fill_rectangle(
        writer,
        Vector2D::new(0, 0),
        Vector2D::new(frame_width, frame_height),
        bg_color,
    );

    let topbar_color = PixelColor { r: 17, g: 17, b: 27 };
    fill_rectangle(
        writer,
        Vector2D::new(0, 0),
        Vector2D::new(frame_width, 30),
        topbar_color,
    );

    let dock_width: u64 = 300;
    let dock_height: u64 = 50;
    let dock_x = (frame_width - dock_width) / 2;
    let dock_y = frame_height - dock_height - 10;

    let dock_bg_color = PixelColor { r: 69, g: 71, b: 90 };
    fill_rectangle(
        writer,
        Vector2D::new(dock_x, dock_y),
        Vector2D::new(dock_width, dock_height),
        dock_bg_color,
    );

    let dock_border_color = PixelColor { r: 147, g: 153, b: 178 };
    draw_rectangle(
        writer,
        Vector2D::new(dock_x, dock_y),
        Vector2D::new(dock_width, dock_height),
        dock_border_color,
    );
}


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
    init_heap();

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

    draw_desktop(&PIXEL_WRITER, framebuffer_info.width as u64, framebuffer_info.height as u64);

    // マウスカーソルの描画
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

    pci::scan_all_bus().expect("PCI scan failed");

    {
        let num_device = *NUM_DEVICE.lock();
        let devices = DEVICES.lock();
        for i in 0..num_device {
            if let Some(dev) = devices[i] {
                let vendor_id = pci::read_vendor_id_from_dev(&dev);
                printk!("{}.{}.{}: vend {}, class {:?}, head {}\n",
                    dev.bus, dev.device, dev.function,
                    vendor_id, dev.class_code, dev.header_type
                );
            }
        }
    }
    

    let mut xhc_device: Option<Device> = None;
    {
        let num_device = *NUM_DEVICE.lock();
        let devices = DEVICES.lock();
        for i in 0..num_device{
            if let Some(dev) = devices[i]{
                if dev.class_code.match_all(0x0c, 0x03, 0x30){
                    xhc_device = Some(dev);

                    if 0x8086 == pci::read_vendor_id_from_dev(&dev){
                        break
                    }
                }
            }
        }
    }

    let xhc_device = xhc_device.unwrap();

    printk!("xHC has been found: {}, {}, {} \n",
        xhc_device.bus, xhc_device.device, xhc_device.function, 
    );

    let xhc_mmio_base = match pci::read_bar(&xhc_device, 0){
        Ok(base_addr) => {
            (base_addr & !(0xfu64)) as usize
        }
        Err(e) => {
            printk!("failed to read BAR: {:?}", e);
            loop {

            }
        }
    };

    let xhc_controller = Controller::new(xhc_mmio_base);
    printk!("max_ports: {}", xhc_controller.max_ports);

    if 0x8086 == pci::read_vendor_id_from_dev(&xhc_device){
        pci::switch_ehci2xhci(&xhc_device);
    }

    match xhc_controller.initialize(){
        Ok(()) => {

        }
        Err(e) => {

        }
    }

    loop {}
}