// pci.rs

use core::arch::asm;
use spin::Mutex;

const CONFIG_ADDRESS: u16 = 0x0cf8;
const CONFIG_DATA: u16 = 0x0cfc;

pub static DEVICES: Mutex<[Option<Device>; 32]> = Mutex::new([None; 32]);
pub static NUM_DEVICE: Mutex<usize> = Mutex::new(0);

// -----------------------------------------------
// Error 型
// -----------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PciError {
    Full,
}

#[derive(Debug, Clone, Copy)]
struct ClassCode{
    base: u8,
    sub: u8,
    interface: u8,
}

impl ClassCode{
    fn match_base(&self, b: u8) -> bool{
        self.base == b
    }

    fn match_base_sub(&self, b: u8, s:u8) -> bool{
        self.base == b && self.sub ==  s
    }

    fn match_all(&self, b: u8, s:u8, i: u8) -> bool{
        self.base == b && self.sub == s && self.interface ==  i
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Device {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub header_type: u8,
    class_code: ClassCode,
}

// -----------------------------------------------
// IO ポート操作
// -----------------------------------------------

unsafe fn io_out32(addr: u16, data: u32) {
    asm!(
        "out dx, eax",
        in("dx") addr,
        in("eax") data,
    );
}

unsafe fn io_in32(addr: u16) -> u32 {
    let data: u32;
    asm!(
        "in eax, dx",
        out("eax") data,
        in("dx") addr,
    );
    data
}

// -----------------------------------------------
// CONFIG_ADDRESS / CONFIG_DATA 操作
// -----------------------------------------------

fn write_address(address: u32) {
    unsafe { io_out32(CONFIG_ADDRESS, address) }
}

fn write_data(value: u32) {
    unsafe { io_out32(CONFIG_DATA, value) }
}

fn read_data() -> u32 {
    unsafe { io_in32(CONFIG_DATA) }
}

// reg_addrは対象のPCIコンフィギュレーションのデータをどの部分から読むかのオフセット
// CONFIG_DATAからは常に三十二ビットずつしか読めないため
fn make_address(bus: u8, device: u8, function: u8, reg_addr: u8) -> u32 {
    (1u32 << 31)
        | ((bus as u32) << 16)
        | ((device as u32) << 11)
        | ((function as u32) << 8)
        | ((reg_addr & 0xfc) as u32)
}

// -----------------------------------------------
// PCI コンフィギュレーション空間 Read 系
// -----------------------------------------------
// rustの切り詰めは下位ビットのみを正確に残す

pub fn read_vendor_id(bus: u8, device: u8, function: u8) -> u16 {
    write_address(make_address(bus, device, function, 0x00));
    read_data() as u16
}

pub fn read_device_id(bus: u8, device: u8, function: u8) -> u16 {
    write_address(make_address(bus, device, function, 0x00));
    (read_data() >> 16) as u16
}

pub fn read_header_type(bus: u8, device: u8, function: u8) -> u8 {
    write_address(make_address(bus, device, function, 0x0c));
    ((read_data() >> 16) & 0xff) as u8
}

pub fn read_class_code(bus: u8, device: u8, function: u8) -> ClassCode {
    write_address(make_address(bus, device, function, 0x08));
    let reg = read_data();
    ClassCode{
        base: (reg >> 24) as u8,
        sub: (reg >> 16) as u8,
        interface: (reg >> 8) as u8,
    }
}

pub fn read_bus_numbers(bus: u8, device: u8, function: u8) -> u32 {
    write_address(make_address(bus, device, function, 0x18));
    read_data()
}

pub fn is_single_function_device(header_type: u8) -> bool {
    (header_type & 0x80) == 0
}

// -----------------------------------------------
// デバイス登録
// -----------------------------------------------

fn add_device(bus: u8, device: u8, function: u8, header_type: u8, class_code: ClassCode) -> Result<(), PciError> {
    let mut devices = DEVICES.lock();
    let mut num = NUM_DEVICE.lock();

    if *num == devices.len() {
        return Err(PciError::Full);
    }

    devices[*num] = Some(Device { bus, device, function, header_type, class_code});
    *num += 1;
    Ok(())
}

// -----------------------------------------------
// スキャン
// -----------------------------------------------

fn scan_function(bus: u8, device: u8, function: u8) -> Result<(), PciError> {
    let header_type = read_header_type(bus, device, function);
    let class_code = read_class_code(bus, device, function);
    add_device(bus, device, function, header_type, class_code)?;

    if class_code.match_base_sub(0x06, 0x04) {
        // PCI-PCI ブリッジ
        let bus_numbers = read_bus_numbers(bus, device, function);
        let secondary_bus = ((bus_numbers >> 8) & 0xff) as u8;
        return scan_bus(secondary_bus);
    }

    Ok(())
}

fn scan_device(bus: u8, device: u8) -> Result<(), PciError> {
    scan_function(bus, device, 0)?;

    if is_single_function_device(read_header_type(bus, device, 0)) {
        return Ok(());
    }

    for function in 1..8u8 {
        if read_vendor_id(bus, device, function) == 0xffff {
            continue;
        }
        scan_function(bus, device, function)?;
    }

    Ok(())
}

fn scan_bus(bus: u8) -> Result<(), PciError> {
    for device in 0..32u8 {
        if read_vendor_id(bus, device, 0) == 0xffff {
            continue;
        }
        scan_device(bus, device)?;
    }
    Ok(())
}

pub fn scan_all_bus() -> Result<(), PciError> {
    *NUM_DEVICE.lock() = 0;

    let header_type = read_header_type(0, 0, 0);
    if is_single_function_device(header_type) {
        return scan_bus(0);
    }

    for function in 1..8u8 {
        if read_vendor_id(0, 0, function) == 0xffff {
            continue;
        }
        scan_bus(function)?;
    }

    Ok(())
}