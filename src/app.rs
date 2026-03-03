use crate::collectors::*;
use crate::types::*;
use std::time::{Duration, Instant};
use sysinfo::System;

pub const TAB_COUNT: usize = 18;

pub struct App {
    pub tab: usize,
    pub sys: System,
    // Existing data
    pub vm_stat: VmStatData,
    pub top_stats: TopStats,
    pub swap_info: SwapInfo,
    pub sys_info: SysInfo,
    pub regions: Vec<ProcessRegion>,
    pub region_scroll: usize,
    pub cpu_details: CpuDetailedInfo,
    pub cpu_cores: Vec<CpuCoreInfo>,
    pub load_avg: [f64; 3],
    pub cpu_scroll: usize,
    pub disk_list: Vec<DiskInfo>,
    pub disk_hw: DiskHwInfo,
    pub iostat: IoStatInfo,
    pub disk_mode: DiskMode,
    pub disk_cursor: usize,
    pub disk_files: Vec<FileEntry>,
    pub disk_sort: SortMode,
    pub disk_path: String,
    pub disk_file_cursor: usize,
    pub disk_filter_system: bool,
    pub net_interfaces: Vec<NetInterface>,
    pub net_scroll: usize,
    pub gpu: GpuInfo,
    pub battery: BatteryInfo,
    pub cameras: Vec<CameraInfo>,
    pub activity_connections: Vec<NetConnection>,
    pub activity_scroll: usize,
    // New tab data
    pub processes: Vec<ProcessInfo>,
    pub process_scroll: usize,
    pub bluetooth: BluetoothInfo,
    pub bt_scroll: usize,
    pub usb_devices: Vec<UsbDevice>,
    pub thunderbolt: Vec<ThunderboltInfo>,
    pub usb_scroll: usize,
    pub audio_devices: Vec<AudioDevice>,
    pub audio_scroll: usize,
    pub security: SecurityInfo,
    pub services: Vec<ServiceInfo>,
    pub service_scroll: usize,
    pub wifi: WifiInfo,
    pub thermal: ThermalInfo,
    pub thermal_scroll: usize,
    // Timing
    pub last_refresh: Instant,
    pub tick_count: u64,
    pub pid: u32,
    pub ram_scroll: usize,
    // Sparkline history
    pub ram_history: Vec<f64>,
    pub cpu_history: Vec<f64>,
    pub net_in_history: Vec<u64>,
    pub net_out_history: Vec<u64>,
    pub prev_bytes_in: u64,
    pub prev_bytes_out: u64,
}

impl App {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        std::thread::sleep(Duration::from_millis(200));
        sys.refresh_cpu_usage();
        let pid = std::process::id();
        let la = System::load_average();
        Self {
            tab: 0,
            vm_stat: ram::parse_vm_stat(),
            top_stats: ram::parse_top(),
            swap_info: ram::parse_swap(),
            sys_info: system::parse_sysinfo(),
            regions: memory::collect_process_maps(pid),
            region_scroll: 0,
            cpu_details: cpu::collect_cpu_details(),
            cpu_cores: cpu::collect_cpu_cores(&sys),
            load_avg: [la.one, la.five, la.fifteen],
            cpu_scroll: 0,
            disk_list: disk::collect_disks(),
            disk_hw: disk::parse_diskutil(),
            iostat: disk::parse_iostat(),
            disk_mode: DiskMode::Partitions,
            disk_cursor: 0,
            disk_files: Vec::new(),
            disk_sort: SortMode::SizeDsc,
            disk_path: String::new(),
            disk_file_cursor: 0,
            disk_filter_system: true,
            net_interfaces: network::parse_netstat(),
            net_scroll: 0,
            gpu: system::parse_gpu(),
            battery: system::parse_battery(),
            cameras: system::collect_cameras(),
            activity_connections: network::parse_lsof(),
            activity_scroll: 0,
            // New tabs
            processes: process::collect_processes(),
            process_scroll: 0,
            bluetooth: bluetooth::collect_bluetooth(),
            bt_scroll: 0,
            usb_devices: usb::collect_usb_devices(),
            thunderbolt: usb::collect_thunderbolt(),
            usb_scroll: 0,
            audio_devices: audio::collect_audio_devices(),
            audio_scroll: 0,
            security: security::collect_security_info(),
            services: services::collect_services(),
            service_scroll: 0,
            wifi: wifi::collect_wifi_info(),
            thermal: thermal::collect_thermal_info(),
            thermal_scroll: 0,
            sys,
            last_refresh: Instant::now(),
            tick_count: 0,
            pid,
            ram_scroll: 0,
            ram_history: Vec::new(),
            cpu_history: Vec::new(),
            net_in_history: Vec::new(),
            net_out_history: Vec::new(),
            prev_bytes_in: 0,
            prev_bytes_out: 0,
        }
    }

    pub fn refresh(&mut self) {
        self.sys.refresh_memory();
        self.sys.refresh_cpu_usage();
        self.vm_stat = ram::parse_vm_stat();
        self.top_stats = ram::parse_top();
        self.swap_info = ram::parse_swap();
        self.cpu_cores = cpu::collect_cpu_cores(&self.sys);
        let la = System::load_average();
        self.load_avg = [la.one, la.five, la.fifteen];
        self.regions = memory::collect_process_maps(self.pid);
        self.iostat = disk::parse_iostat();
        self.net_interfaces = network::parse_netstat();

        // Sparkline updates
        let total = self.sys.total_memory();
        let used = self.sys.used_memory();
        let pct = if total > 0 {
            used as f64 / total as f64 * 100.0
        } else {
            0.0
        };
        self.ram_history.push(pct);
        if self.ram_history.len() > 60 {
            self.ram_history.remove(0);
        }

        let avg_cpu = if !self.cpu_cores.is_empty() {
            self.cpu_cores.iter().map(|c| c.usage).sum::<f32>() / self.cpu_cores.len() as f32
        } else {
            0.0
        };
        self.cpu_history.push(avg_cpu as f64);
        if self.cpu_history.len() > 60 {
            self.cpu_history.remove(0);
        }

        // Network bandwidth tracking
        let total_in: u64 = self.net_interfaces.iter().map(|i| i.bytes_in).sum();
        let total_out: u64 = self.net_interfaces.iter().map(|i| i.bytes_out).sum();
        if self.prev_bytes_in > 0 {
            self.net_in_history
                .push(total_in.saturating_sub(self.prev_bytes_in));
            self.net_out_history
                .push(total_out.saturating_sub(self.prev_bytes_out));
            if self.net_in_history.len() > 60 {
                self.net_in_history.remove(0);
            }
            if self.net_out_history.len() > 60 {
                self.net_out_history.remove(0);
            }
        }
        self.prev_bytes_in = total_in;
        self.prev_bytes_out = total_out;

        // Periodic updates
        if self.tick_count.is_multiple_of(3) {
            self.activity_connections = network::parse_lsof();
            self.processes = process::collect_processes();
        }
        if self.tick_count.is_multiple_of(5) {
            self.disk_list = disk::collect_disks();
            self.battery = system::parse_battery();
            self.thermal = thermal::collect_thermal_info();
        }
        if self.tick_count.is_multiple_of(10) {
            self.services = services::collect_services();
            self.wifi = wifi::collect_wifi_info();
        }
        if self.tick_count.is_multiple_of(30) {
            self.cameras = system::collect_cameras();
            self.gpu = system::parse_gpu();
            self.disk_hw = disk::parse_diskutil();
            self.bluetooth = bluetooth::collect_bluetooth();
            self.usb_devices = usb::collect_usb_devices();
            self.thunderbolt = usb::collect_thunderbolt();
            self.audio_devices = audio::collect_audio_devices();
            self.security = security::collect_security_info();
        }
        self.last_refresh = Instant::now();
        self.tick_count += 1;
    }
}
