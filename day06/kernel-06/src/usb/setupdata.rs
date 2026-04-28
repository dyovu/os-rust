use modular_bitfield::prelude::*;

pub mod request_type {
    // --- bmRequestType recipient ---
    pub const RECIPIENT_DEVICE: u8 = 0;
    pub const RECIPIENT_INTERFACE: u8 = 1;
    pub const RECIPIENT_ENDPOINT: u8 = 2;
    pub const RECIPIENT_OTHER: u8 = 3;

    // --- bmRequestType type ---
    pub const TYPE_STANDARD: u8 = 0;
    pub const TYPE_CLASS: u8 = 1;
    pub const TYPE_VENDOR: u8 = 2;

    // --- bmRequestType direction ---
    pub const DIR_OUT: u8 = 0;
    pub const DIR_IN: u8 = 1;
}

pub mod request {
    pub const GET_STATUS: u8 = 0;
    pub const CLEAR_FEATURE: u8 = 1;
    pub const SET_FEATURE: u8 = 3;
    pub const SET_ADDRESS: u8 = 5;
    pub const GET_DESCRIPTOR: u8 = 6;
    pub const SET_DESCRIPTOR: u8 = 7;
    pub const GET_CONFIGURATION: u8 = 8;
    pub const SET_CONFIGURATION: u8 = 9;
    pub const GET_INTERFACE: u8 = 10;
    pub const SET_INTERFACE: u8 = 11;
    pub const SYNCH_FRAME: u8 = 12;
    pub const SET_ENCRYPTION: u8 = 13;
    pub const GET_ENCRYPTION: u8 = 14;
    pub const SET_HANDSHAKE: u8 = 15;
    pub const GET_HANDSHAKE: u8 = 16;
    pub const SET_CONNECTION: u8 = 17;
    pub const SET_SECURITY_DATA: u8 = 18;
    pub const GET_SECURITY_DATA: u8 = 19;
    pub const SET_WUSB_DATA: u8 = 20;
    pub const LOOPBACK_DATA_WRITE: u8 = 21;
    pub const LOOPBACK_DATA_READ: u8 = 22;
    pub const SET_INTERFACE_DS: u8 = 23;
    pub const SET_SEL: u8 = 48;
    pub const SET_ISOCH_DELAY: u8 = 49;

    // HID class specific report values
    pub const HID_GET_REPORT: u8 = 1;
    pub const HID_SET_PROTOCOL: u8 = 11;
}

pub mod descriptor_type {
    pub const DEVICE: u8 = 1;
    pub const CONFIGURATION: u8 = 2;
    pub const STRING: u8 = 3;
    pub const INTERFACE: u8 = 4;
    pub const ENDPOINT: u8 = 5;
    pub const INTERFACE_POWER: u8 = 8;
    pub const OTG: u8 = 9;
    pub const DEBUG: u8 = 10;
    pub const INTERFACE_ASSOCIATION: u8 = 11;
    pub const BOS: u8 = 15;
    pub const DEVICE_CAPABILITY: u8 = 16;
    pub const HID: u8 = 33;
    pub const SUPERSPEED_USB_ENDPOINT_COMPANION: u8 = 48;
    pub const SUPERSPEED_PLUS_ISOCHRONOUS_ENDPOINT_COMPANION: u8 = 49;
}

// C++のunionのbits部分をbitfieldで表現
#[bitfield(bits = 8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RequestType {
    pub recipient: B5,
    pub ty: B2, // Rustではtypeが予約語であるためtyを使用
    pub direction: B1,
}

// 構造体の中に構造体（RequestType）を含める形
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(C, packed)]
pub struct SetupData {
    pub request_type: RequestType,
    pub request: u8,
    pub value: u16,
    pub index: u16,
    pub length: u16,
}

impl SetupData {
    pub fn request_type_as_u8(&self) -> u8 {
        self.request_type.into_bytes()[0]
    }
}