#![allow(dead_code)]
use ratatui::style::Color;

// ═══════════════════════════════════════════════════════════════════════════════
// MEMORY TYPES
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegionType {
    Stack,
    Heap,
    Code,
    Dylib,
    Anonymous,
    MappedFile,
}
impl RegionType {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Stack => "Stack",
            Self::Heap => "Heap",
            Self::Code => "Code",
            Self::Dylib => "Dylib",
            Self::Anonymous => "Anon",
            Self::MappedFile => "File",
        }
    }
    pub fn color(&self) -> Color {
        match self {
            Self::Stack => Color::Red,
            Self::Heap => Color::Green,
            Self::Code => Color::Blue,
            Self::Dylib => Color::Cyan,
            Self::Anonymous => Color::DarkGray,
            Self::MappedFile => Color::Yellow,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct VmStatData {
    pub page_size: u64,
    pub free: u64,
    pub active: u64,
    pub inactive: u64,
    pub speculative: u64,
    pub wired: u64,
    pub compressor: u64,
    pub purgeable: u64,
    pub throttled: u64,
    pub reactivated: u64,
    pub pageins: u64,
    pub pageouts: u64,
    pub faults: u64,
    pub copy_on_write: u64,
    pub zero_fill: u64,
    pub compressions: u64,
    pub decompressions: u64,
    pub swapins: u64,
    pub swapouts: u64,
}

#[derive(Debug, Clone)]
pub struct ProcessRegion {
    pub start: usize,
    pub end: usize,
    pub size: usize,
    pub perms: String,
    pub region_type: RegionType,
    pub name: String,
}

#[derive(Debug, Clone, Default)]
pub struct TopStats {
    pub processes: u32,
    pub threads: u32,
    pub running: u32,
    pub sleeping: u32,
    pub cpu_user: f64,
    pub cpu_sys: f64,
    pub cpu_idle: f64,
    pub sharedlibs_resident: String,
    pub sharedlibs_data: String,
    pub mem_regions_total: u64,
    pub mem_regions_resident: String,
    pub mem_regions_private: String,
    pub mem_regions_shared: String,
    pub phys_used: String,
    pub phys_wired: String,
    pub phys_compressor: String,
    pub phys_unused: String,
    pub vm_vsize: String,
    pub net_packets_in: String,
    pub net_bytes_in: String,
    pub net_packets_out: String,
    pub net_bytes_out: String,
    pub disk_reads: String,
    pub disk_read_bytes: String,
    pub disk_writes: String,
    pub disk_write_bytes: String,
    pub top_procs: Vec<(String, String, String)>,
}

#[derive(Debug, Clone, Default)]
pub struct SwapInfo {
    pub total: String,
    pub used: String,
    pub free: String,
    pub encrypted: bool,
}

// ═══════════════════════════════════════════════════════════════════════════════
// CPU TYPES
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Default)]
pub struct CpuDetailedInfo {
    pub brand: String,
    pub arch: String,
    pub core_count: u64,
    pub thread_count: u64,
    pub cores_per_package: u64,
    pub cache_line_size: u64,
    pub l1i_cache: u64,
    pub l1d_cache: u64,
    pub l2_cache: u64,
    pub l3_cache: u64,
    pub num_perf_levels: u64,
    pub perf_cores: u64,
    pub efficiency_cores: u64,
    pub perf_l1i: u64,
    pub perf_l1d: u64,
    pub perf_l2: u64,
    pub eff_l1i: u64,
    pub eff_l1d: u64,
    pub eff_l2: u64,
    pub page_size: u64,
    pub phys_mem: u64,
    pub features: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CpuCoreInfo {
    pub name: String,
    pub usage: f32,
    pub frequency: u64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// DISK TYPES
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub fs_type: String,
    pub total: u64,
    pub used: u64,
    pub available: u64,
    pub is_removable: bool,
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
    pub is_system: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DiskMode {
    Partitions,
    Files,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortMode {
    SizeDsc,
    SizeAsc,
    NameAsc,
    NameDsc,
}

#[derive(Debug, Clone, Default)]
pub struct DiskHwInfo {
    pub device_name: String,
    pub media_name: String,
    pub protocol: String,
    pub smart_status: String,
    pub disk_size: String,
    pub block_size: String,
    pub content: String,
}

#[derive(Debug, Clone, Default)]
pub struct IoStatInfo {
    pub kb_per_transfer: f64,
    pub transfers_per_sec: f64,
    pub mb_per_sec: f64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// NETWORK TYPES
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Default)]
pub struct NetInterface {
    pub name: String,
    pub mtu: u32,
    pub ip: String,
    pub pkts_in: u64,
    pub bytes_in: u64,
    pub errs_in: u64,
    pub pkts_out: u64,
    pub bytes_out: u64,
    pub errs_out: u64,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct NetConnection {
    pub pid: u32,
    pub process: String,
    pub fd: String,
    pub proto: String,
    pub local_addr: String,
    pub remote_addr: String,
    pub state: String,
}

// ═══════════════════════════════════════════════════════════════════════════════
// SYSTEM TYPES
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Default)]
pub struct GpuInfo {
    pub chipset: String,
    pub gpu_type: String,
    pub bus: String,
    pub cores: String,
    pub vendor: String,
    pub metal: String,
    pub display_type: String,
    pub resolution: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Default)]
pub struct BatteryInfo {
    pub level: String,
    pub state: String,
    pub remaining: String,
    pub cycle_count: String,
    pub condition: String,
    pub voltage: String,
    pub amperage: String,
    pub temperature: String,
    pub max_capacity: String,
    pub design_capacity: String,
    pub present: bool,
}

#[derive(Debug, Clone, Default)]
pub struct SysInfo {
    pub hostname: String,
    pub os_type: String,
    pub os_release: String,
    pub os_build: String,
    pub boot_time: String,
    pub uptime: String,
    pub hw_model: String,
}

#[derive(Debug, Clone)]
pub struct CameraInfo {
    pub name: String,
    pub model_id: String,
    pub unique_id: String,
}

// ═══════════════════════════════════════════════════════════════════════════════
// NEW: PROCESS TYPES
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub ppid: u32,
    pub user: String,
    pub cpu: f64,
    pub mem: f64,
    pub rss: String,
    pub vsize: String,
    pub state: String,
    pub threads: u32,
    pub command: String,
    pub started: String,
}

// ═══════════════════════════════════════════════════════════════════════════════
// NEW: BLUETOOTH TYPES
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Default)]
pub struct BluetoothInfo {
    pub address: String,
    pub chipset: String,
    pub firmware: String,
    pub transport: String,
    pub discoverable: bool,
    pub state: String,
    pub vendor: String,
    pub services: String,
    pub devices: Vec<BtDevice>,
}

#[derive(Debug, Clone)]
pub struct BtDevice {
    pub name: String,
    pub address: String,
    pub device_type: String,
    pub firmware: String,
    pub connected: bool,
}

// ═══════════════════════════════════════════════════════════════════════════════
// NEW: USB TYPES
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct UsbDevice {
    pub name: String,
    pub vendor: String,
    pub product_id: String,
    pub vendor_id: String,
    pub speed: String,
    pub bus_power: String,
    pub serial: String,
    pub location: String,
}

#[derive(Debug, Clone)]
pub struct ThunderboltInfo {
    pub device_name: String,
    pub speed: String,
    pub uuid: String,
    pub link_status: String,
}

// ═══════════════════════════════════════════════════════════════════════════════
// NEW: AUDIO TYPES
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct AudioDevice {
    pub name: String,
    pub manufacturer: String,
    pub sample_rate: u32,
    pub channels: u32,
    pub transport: String,
    pub is_input: bool,
    pub is_default: bool,
}

// ═══════════════════════════════════════════════════════════════════════════════
// NEW: SECURITY TYPES
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Default)]
pub struct SecurityInfo {
    pub sip_enabled: bool,
    pub gatekeeper_enabled: bool,
    pub filevault_enabled: bool,
    pub firewall_enabled: bool,
    pub firewall_stealth: bool,
    pub firewall_block_all: bool,
    pub sip_status: String,
    pub gatekeeper_status: String,
    pub filevault_status: String,
    pub firewall_status: String,
}

// ═══════════════════════════════════════════════════════════════════════════════
// NEW: SERVICES TYPES
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct ServiceInfo {
    pub pid: i32,
    pub label: String,
    pub last_exit: i32,
}

// ═══════════════════════════════════════════════════════════════════════════════
// NEW: WIFI TYPES
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Default)]
pub struct WifiInfo {
    pub interface: String,
    pub ssid: String,
    pub bssid: String,
    pub channel: String,
    pub rssi: String,
    pub noise: String,
    pub tx_rate: String,
    pub security_type: String,
    pub phy_mode: String,
    pub country_code: String,
    pub hardware: String,
}

// ═══════════════════════════════════════════════════════════════════════════════
// NEW: THERMAL TYPES
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct ThermalEntry {
    pub name: String,
    pub temperature: f64,
    pub category: String,
}

#[derive(Debug, Clone, Default)]
pub struct ThermalInfo {
    pub entries: Vec<ThermalEntry>,
    pub thermal_pressure: String,
}
