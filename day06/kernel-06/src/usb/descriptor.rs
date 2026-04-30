use modular_bitfield::prelude::*;

/// 各ディスクリプタが持つタイプ番号を表すトレイト
/// C++のkType静的定数に相当する
pub trait Descriptor {
    const TYPE: u8;
}

/// desc_data[1]のディスクリプタタイプ番号を見て、対象の型にキャストする
/// C++のDescriptorDynamicCastに相当
pub fn descriptor_dynamic_cast<T: Descriptor>(desc_data: &[u8]) -> Option<&T> {
    if desc_data[1] == T::TYPE {
        Some(unsafe { &*(desc_data.as_ptr() as *const T) })
    } else {
        None
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct DeviceDescriptor {
    pub length: u8,             // offset 0
    pub descriptor_type: u8,    // offset 1
    pub usb_release: u16,       // offset 2
    pub device_class: u8,       // offset 4
    pub device_sub_class: u8,   // offset 5
    pub device_protocol: u8,    // offset 6
    pub max_packet_size: u8,    // offset 7
    pub vendor_id: u16,         // offset 8
    pub product_id: u16,        // offset 10
    pub device_release: u16,    // offset 12
    pub manufacturer: u8,       // offset 14
    pub product: u8,            // offset 15
    pub serial_number: u8,      // offset 16
    pub num_configurations: u8, // offset 17
}

impl Descriptor for DeviceDescriptor {
    const TYPE: u8 = 1;
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct ConfigurationDescriptor {
    pub length: u8,              // offset 0
    pub descriptor_type: u8,     // offset 1
    pub total_length: u16,       // offset 2
    pub num_interfaces: u8,      // offset 4
    pub configuration_value: u8, // offset 5
    pub configuration_id: u8,    // offset 6
    pub attributes: u8,          // offset 7
    pub max_power: u8,           // offset 8
}

impl Descriptor for ConfigurationDescriptor {
    const TYPE: u8 = 2;
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct InterfaceDescriptor {
    pub length: u8,              // offset 0
    pub descriptor_type: u8,     // offset 1
    pub interface_number: u8,    // offset 2
    pub alternate_setting: u8,   // offset 3
    pub num_endpoints: u8,       // offset 4
    pub interface_class: u8,     // offset 5
    pub interface_sub_class: u8, // offset 6
    pub interface_protocol: u8,  // offset 7
    pub interface_id: u8,        // offset 8
}

impl Descriptor for InterfaceDescriptor {
    const TYPE: u8 = 4;
}

// C++のunion { uint8_t data; struct { bits... } } をbitfieldで表現
#[bitfield(bits = 8)]
#[derive(Debug, Clone, Copy)]
pub struct EndpointAddress {
    pub number: B4,
    #[skip] __: B3,
    pub dir_in: B1,
}

// C++のunion { uint8_t data; struct { bits... } } をbitfieldで表現
#[bitfield(bits = 8)]
#[derive(Debug, Clone, Copy)]
pub struct EndpointAttributes {
    pub transfer_type: B2,
    pub sync_type: B2,
    pub usage_type: B2,
    #[skip] __: B2,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct EndpointDescriptor {
    pub length: u8,                      // offset 0
    pub descriptor_type: u8,             // offset 1
    pub endpoint_address: EndpointAddress, // offset 2
    pub attributes: EndpointAttributes,  // offset 3
    pub max_packet_size: u16,            // offset 4
    pub interval: u8,                    // offset 6
}

impl Descriptor for EndpointDescriptor {
    const TYPE: u8 = 5;
}

/// HIDDescriptorの末尾に続くクラス特有ディスクリプタの情報
/// HIDは1つ以上のクラス特有ディスクリプタを持ち、その数はnum_descriptorsに記載される
/// Reportディスクリプタ（type = 34）は必ず存在するためnum_descriptorsは必ず1以上
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct ClassDescriptor {
    /// クラス特有ディスクリプタのタイプ値
    pub descriptor_type: u8,
    /// クラス特有ディスクリプタのバイト数
    pub descriptor_length: u16,
}

/// HIDDescriptor本体の固定部分のみを表す
/// 末尾にClassDescriptorが可変長で続くレイアウトになっているため、
/// get_class_descriptor()でポインタ算術を使って取得する
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct HIDDescriptor {
    pub length: u8,           // offset 0
    pub descriptor_type: u8,  // offset 1
    pub hid_release: u16,     // offset 2
    pub country_code: u8,     // offset 4
    pub num_descriptors: u8,  // offset 5
}

impl Descriptor for HIDDescriptor {
    const TYPE: u8 = 33;
}

impl HIDDescriptor {
    /// HID特有のディスクリプタ情報を取得する
    /// 構造体末尾のアドレスからポインタ算術でClassDescriptorを引く
    /// indexが範囲外の場合はNoneを返す
    pub fn get_class_descriptor(&self, index: usize) -> Option<&ClassDescriptor> {
        if index >= self.num_descriptors as usize {
            return None;
        }
        let end_of_struct = unsafe {
            (self as *const HIDDescriptor).add(1) as *const ClassDescriptor
        };
        Some(unsafe { &*end_of_struct.add(index) })
    }
}