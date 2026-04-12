// ================================================================
// @file usb/xhci/register.rs
//
// xHCIのMMRの定義に基づいた構造体の定義
// ================================================================

use core::ptr::{read_volatile, write_volatile};
use core::marker::PhantomData;

use modular_bitfield::prelude::*;

// アクセス権限を表すマーカー型
pub struct ReadOnly;
pub struct ReadWrite;

// 全てのregisterフィールドをラップする構造体
// publicなフィールドに対してRWの制限
// volatileなアクセスを保証するため
#[repr(C, packed)]
pub struct MemMapRegister<T: Copy, Access> {
    value: T,
    // 実際のメモリは使わないが、型としてAccessを保持するために必要
    _marker: PhantomData<Access>,
}

// readはどちらの権限でも使える
impl<T: Copy, Access> MemMapRegister<T, Access> {
    pub fn read(&self) -> T {
        unsafe { read_volatile(&raw const self.value) }
    }
}

// writeはReadWriteの時だけ使える
impl<T: Copy> MemMapRegister<T, ReadWrite> {
    pub fn write(&mut self, value: T) {
        unsafe { write_volatile(&raw mut self.value, value) }
    }
}

// 連続したレジスタ群へ配列のようにアクセスするためのラッパー
// 既にあるメモリ空間（ポインタ）を、配列のように [index] でアクセスできるようにする」ため

pub struct ArrayWrapper<T> {
    ptr: *mut T,
    len: usize,
}

impl<T> ArrayWrapper<T> {
    pub unsafe fn new(base_addr: usize, len: usize) -> Self {
        Self {
            ptr: base_addr as *mut T,
            len,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    // pub unsafe fn get(&self, index: usize) -> &T {
    //     assert!(index < self.len);
    //     &*self.ptr.add(index)
    // }

    pub unsafe fn get_mut(&self, index: usize) -> *mut T {
        assert!(index < self.len);
        self.ptr.add(index)
    }
}

// ================================================================
// CapabilityRegisters
// ================================================================

#[bitfield(bits = 32)]
#[derive(Copy, Clone)]
pub struct Hcsparams1 {
    pub max_device_slots: B8,
    pub max_interrupters: B11,
    #[skip] __: B5,
    pub max_ports: B8,
}

#[bitfield(bits = 32)]
#[derive(Copy, Clone)]
pub struct Hcsparams2 {
    pub isochronous_scheduling_threshold: B4,
    pub event_ring_segment_table_max: B4,
    #[skip] __: B13,
    pub max_scratchpad_buffers_high: B5,
    pub scratchpad_restore: B1,
    pub max_scratchpad_buffers_low: B5,
}

#[bitfield(bits = 32)]
#[derive(Copy, Clone)]
pub struct Hcsparams3 {
    pub u1_device_exit_latency: B8,
    #[skip] __: B8,
    pub u2_device_exit_latency: B16,
}

#[bitfield(bits = 32)]
#[derive(Copy, Clone)]
pub struct Hccparams1 {
    pub addressing_capability_64: B1,
    pub bw_negotiation_capability: B1,
    pub context_size: B1,
    pub port_power_control: B1,
    pub port_indicators: B1,
    pub light_hc_reset_capability: B1,
    pub latency_tolerance_messaging_capability: B1,
    pub no_secondary_sid_support: B1,
    pub parse_all_event_data: B1,
    pub stopped_short_packet_capability: B1,
    pub stopped_edtla_capability: B1,
    pub contiguous_frame_id_capability: B1,
    pub maximum_primary_stream_array_size: B4,
    pub xhci_extended_capabilities_pointer: B16,
}

#[bitfield(bits = 32)]
#[derive(Copy, Clone)]
pub struct Dboff {
    #[skip] __: B2,
    pub doorbell_array_offset: B30,
}

#[bitfield(bits = 32)]
#[derive(Copy, Clone)]
pub struct Rtsoff {
    #[skip] __: B5,
    pub runtime_register_space_offset: B27,
}

#[bitfield(bits = 32)]
#[derive(Copy, Clone)]
pub struct Hccparams2 {
    pub u3_entry_capability: B1,
    pub configure_endpoint_command_max_exit_latency_too_large_capability: B1,
    pub force_save_context_capability: B1,
    pub compliance_transition_capability: B1,
    pub large_esit_payload_capability: B1,
    pub configuration_information_capability: B1,
    #[skip] __: B26,
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct CapabilityRegisters {
    pub CAPLENGTH:  MemMapRegister<u8,       ReadOnly>,
    _reserved:      u8,  // MMIOアクセス不要なのでMemMapRegisterでラップしない
    pub HCIVERSION: MemMapRegister<u16,      ReadOnly>,
    pub HCSPARAMS1: MemMapRegister<Hcsparams1, ReadOnly>,
    pub HCSPARAMS2: MemMapRegister<Hcsparams2, ReadOnly>,
    pub HCSPARAMS3: MemMapRegister<Hcsparams3, ReadOnly>,
    pub HCCPARAMS1: MemMapRegister<Hccparams1, ReadOnly>,
    pub DBOFF:      MemMapRegister<Dboff,    ReadOnly>,
    pub RTSOFF:     MemMapRegister<Rtsoff,   ReadOnly>,
    pub HCCPARAMS2: MemMapRegister<Hccparams2, ReadOnly>,
}

// ================================================================
// OperationalRegisters
// ================================================================

#[bitfield(bits = 32)]
#[derive(Copy, Clone)]
pub struct Usbcmd {
    pub run_stop: B1,
    pub host_controller_reset: B1,
    pub interrupter_enable: B1,
    pub host_system_error_enable: B1,
    #[skip] __: B3,
    pub light_host_controller_reset: B1,
    pub controller_save_state: B1,
    pub controller_restore_state: B1,
    pub enable_wrap_event: B1,
    pub enable_u3_mfindex_stop: B1,
    pub stopped_short_packet_enable: B1,
    pub cem_enable: B1,
    #[skip] __: B18,
}

#[bitfield(bits = 32)]
#[derive(Copy, Clone)]
pub struct Usbsts {
    pub host_controller_halted: B1,
    #[skip] __: B1,
    pub host_system_error: B1,
    pub event_interrupt: B1,
    pub port_change_detect: B1,
    #[skip] __: B3,
    pub save_state_status: B1,
    pub restore_state_status: B1,
    pub save_restore_error: B1,
    pub controller_not_ready: B1,
    pub host_controller_error: B1,
    #[skip] __: B19,
}

#[bitfield(bits = 64)]
#[derive(Copy, Clone)]
pub struct Crcr {
    pub ring_cycle_state: B1,
    pub command_stop: B1,
    pub command_abort: B1,
    pub command_ring_running: B1,
    #[skip] __: B2,
    pub command_ring_pointer: B58,
}

#[bitfield(bits = 64)]
#[derive(Copy, Clone)]
pub struct Dcbaap {
    #[skip] __: B6,
    pub device_context_base_address_array_pointer: B58,
}

#[bitfield(bits = 32)]
#[derive(Copy, Clone)]
pub struct Config {
    pub max_device_slots_enabled: B8,
    pub u3_entry_enable: B1,
    pub configuration_information_enable: B1,
    #[skip] __: B22,
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct OperationalRegisters {
    pub USBCMD:   MemMapRegister<Usbcmd, ReadWrite>,
    pub USBSTS:   MemMapRegister<Usbsts, ReadWrite>,
    pub PAGESIZE: MemMapRegister<u32,    ReadOnly>,
    _reserved1:   [u8; 8],
    pub DNCTRL:   MemMapRegister<u32,    ReadWrite>,
    pub CRCR:     MemMapRegister<Crcr,   ReadWrite>,
    _reserved2:   [u8; 16],
    pub DCBAAP:   MemMapRegister<Dcbaap, ReadWrite>,
    pub CONFIG:   MemMapRegister<Config, ReadWrite>,
}

// ================================================================
// PortRegisterSet
// ================================================================

#[bitfield(bits = 32)]
#[derive(Copy, Clone)]
pub struct Portsc {
    pub current_connect_status: B1,
    pub port_enabled_disabled: B1,
    #[skip] __: B1,
    pub over_current_active: B1,
    pub port_reset: B1,
    pub port_link_state: B4,
    pub port_power: B1,
    pub port_speed: B4,
    pub port_indicator_control: B2,
    pub port_link_state_write_strobe: B1,
    pub connect_status_change: B1,
    pub port_enabled_disabled_change: B1,
    pub warm_port_reset_change: B1,
    pub over_current_change: B1,
    pub port_reset_change: B1,
    pub port_link_state_change: B1,
    pub port_config_error_change: B1,
    pub cold_attach_status: B1,
    pub wake_on_connect_enable: B1,
    pub wake_on_disconnect_enable: B1,
    pub wake_on_over_current_enable: B1,
    #[skip] __: B2,
    pub device_removable: B1,
    pub warm_port_reset: B1,
}

#[bitfield(bits = 32)]
#[derive(Copy, Clone)]
pub struct Portpmsc {
    pub u1_timeout: B8,
    pub u2_timeout: B8,
    pub force_link_pm_accept: B1,
    #[skip] __: B15,
}

#[bitfield(bits = 32)]
#[derive(Copy, Clone)]
pub struct Portli {
    pub link_error_count: B16,
    pub rx_lane_count: B4,
    pub tx_lane_count: B4,
    #[skip] __: B8,
}

#[bitfield(bits = 32)]
#[derive(Copy, Clone)]
pub struct Porthlpmc {
    pub host_initiated_resume_duration_mode: B2,
    pub l1_timeout: B8,
    pub best_effort_service_latency_deep: B4,
    #[skip] __: B18,
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct PortRegisterSet {
    pub PORTSC:    MemMapRegister<Portsc,    ReadWrite>,
    pub PORTPMSC:  MemMapRegister<Portpmsc,  ReadWrite>,
    pub PORTLI:    MemMapRegister<Portli,    ReadOnly>,
    pub PORTHLPMC: MemMapRegister<Porthlpmc, ReadWrite>,
}

// ================================================================
// InterrupterRegisterSet
// ================================================================

#[bitfield(bits = 32)]
#[derive(Copy, Clone)]
pub struct Iman {
    pub interrupt_pending: B1,
    pub interrupt_enable: B1,
    #[skip] __: B30,
}

#[bitfield(bits = 32)]
#[derive(Copy, Clone)]
pub struct Imod {
    pub interrupt_moderation_interval: B16,
    pub interrupt_moderation_counter: B16,
}

#[bitfield(bits = 32)]
#[derive(Copy, Clone)]
pub struct Erstsz {
    pub event_ring_segment_table_size: B16,
    #[skip] __: B16,
}

#[bitfield(bits = 64)]
#[derive(Copy, Clone)]
pub struct Erstba {
    #[skip] __: B6,
    pub event_ring_segment_table_base_address: B58,
}

#[bitfield(bits = 64)]
#[derive(Copy, Clone)]
pub struct Erdp {
    pub dequeue_erst_segment_index: B3,
    pub event_handler_busy: B1,
    pub event_ring_dequeue_pointer: B60,
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct InterrupterRegisterSet {
    pub IMAN:   MemMapRegister<Iman,   ReadWrite>,
    pub IMOD:   MemMapRegister<Imod,   ReadWrite>,
    pub ERSTSZ: MemMapRegister<Erstsz, ReadWrite>,
    _reserved:  u32,  // MMIOアクセス不要なのでMemMapRegisterでラップしない
    pub ERSTBA: MemMapRegister<Erstba, ReadWrite>,
    pub ERDP:   MemMapRegister<Erdp,   ReadWrite>,
}

// ================================================================
// DoorbellRegister
// ================================================================

#[bitfield(bits = 32)]
#[derive(Copy, Clone)]
pub struct DoorbellBitmap {
    pub db_target: B8,
    #[skip] __: B8,
    pub db_stream_id: B16,
}

#[repr(C)]
pub struct DoorbellRegister {
    reg: MemMapRegister<DoorbellBitmap, ReadWrite>,
}

impl DoorbellRegister {
    pub fn ring(&mut self, target: u8, stream_id: u16) {
        let mut value = DoorbellBitmap::new();
        value.set_db_target(target);
        value.set_db_stream_id(stream_id);
        self.reg.write(value);
    }
}

// ================================================================
// ExtendedRegister
// ================================================================

#[bitfield(bits = 32)]
#[derive(Copy, Clone)]
pub struct ExtendedRegisterBitmap {
    pub capability_id: B8,
    pub next_pointer: B8,
    pub value: B16,
}

// 拡張レジスタの共通ヘッダ
// capability_id, next_pointer, valueのフィールドを持つ
#[repr(C)]
pub struct ExtendedRegister {
    pub reg: MemMapRegister<ExtendedRegisterBitmap, ReadWrite>,
}

// ================================================================
// USBLEGSUP (Extended Capability)
// ================================================================

#[bitfield(bits = 32)]
#[derive(Copy, Clone)]
pub struct Usblegsup {
    pub capability_id: B8,
    pub next_pointer: B8,
    pub hc_bios_owned_semaphore: B1,
    #[skip] __: B7,
    pub hc_os_owned_semaphore: B1,
    #[skip] __: B7,
}

pub struct UsblegsupRegister {
    reg: MemMapRegister<Usblegsup, ReadWrite>,
}