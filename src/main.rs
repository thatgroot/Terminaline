use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use proc_maps::{get_process_maps, MapRange};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, Borders, Cell, Gauge, Paragraph, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Table, Tabs, Wrap,
    },
    Frame, Terminal,
};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::process::Command;
use std::time::{Duration, Instant};
use sysinfo::{Disks, System};

// ═══════════════════════════════════════════════════════════════════════════════
// DATA STRUCTURES
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum RegionType { Stack, Heap, Code, Dylib, Anonymous, MappedFile }
impl RegionType {
    fn label(&self) -> &'static str {
        match self { Self::Stack=>"Stack", Self::Heap=>"Heap", Self::Code=>"Code",
            Self::Dylib=>"Dylib", Self::Anonymous=>"Anon", Self::MappedFile=>"File" }
    }
    fn color(&self) -> Color {
        match self { Self::Stack=>Color::Red, Self::Heap=>Color::Green, Self::Code=>Color::Blue,
            Self::Dylib=>Color::Cyan, Self::Anonymous=>Color::DarkGray, Self::MappedFile=>Color::Yellow }
    }
}

#[derive(Debug, Default, Clone)]
struct VmStatData {
    page_size: u64, free: u64, active: u64, inactive: u64, speculative: u64,
    wired: u64, compressor: u64, purgeable: u64, throttled: u64, reactivated: u64,
    pageins: u64, pageouts: u64, faults: u64, copy_on_write: u64, zero_fill: u64,
    compressions: u64, decompressions: u64, swapins: u64, swapouts: u64,
}

#[derive(Debug, Clone)]
struct ProcessRegion { start: usize, end: usize, size: usize, perms: String, region_type: RegionType, name: String }

#[derive(Debug, Clone, Default)]
struct TopStats {
    processes: u32, threads: u32, running: u32, sleeping: u32,
    cpu_user: f64, cpu_sys: f64, cpu_idle: f64,
    sharedlibs_resident: String, sharedlibs_data: String,
    mem_regions_total: u64, mem_regions_resident: String, mem_regions_private: String, mem_regions_shared: String,
    phys_used: String, phys_wired: String, phys_compressor: String, phys_unused: String,
    vm_vsize: String, net_packets_in: String, net_bytes_in: String, net_packets_out: String, net_bytes_out: String,
    disk_reads: String, disk_read_bytes: String, disk_writes: String, disk_write_bytes: String,
    top_procs: Vec<(String, String, String)>, // name, mem, cpu%
}

#[derive(Debug, Clone, Default)]
struct SwapInfo { total: String, used: String, free: String, encrypted: bool }

#[derive(Debug, Clone, Default)]
struct CpuDetailedInfo {
    brand: String, arch: String, core_count: u64, thread_count: u64, cores_per_package: u64,
    cache_line_size: u64, l1i_cache: u64, l1d_cache: u64, l2_cache: u64, l3_cache: u64,
    num_perf_levels: u64, perf_cores: u64, efficiency_cores: u64,
    perf_l1i: u64, perf_l1d: u64, perf_l2: u64, eff_l1i: u64, eff_l1d: u64, eff_l2: u64,
    page_size: u64, phys_mem: u64, features: Vec<String>,
}

#[derive(Debug, Clone)] struct CpuCoreInfo { name: String, usage: f32, frequency: u64 }
#[derive(Debug, Clone)] struct CameraInfo { name: String, model_id: String, unique_id: String }
#[derive(Debug, Clone)] struct DiskInfo { name: String, mount_point: String, fs_type: String, total: u64, used: u64, available: u64, is_removable: bool }

#[derive(Debug, Clone)]
struct FileEntry {
    name: String,
    path: String,
    size: u64,
    is_dir: bool,
    is_system: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum DiskMode { Partitions, Files }

#[derive(Debug, Clone, Copy, PartialEq)]
enum SortMode { SizeDsc, SizeAsc, NameAsc, NameDsc }

#[derive(Debug, Clone, Default)]
struct DiskHwInfo { device_name: String, media_name: String, protocol: String, smart_status: String, disk_size: String, block_size: String, content: String }

#[derive(Debug, Clone, Default)]
struct IoStatInfo { kb_per_transfer: f64, transfers_per_sec: f64, mb_per_sec: f64 }

#[derive(Debug, Clone, Default)]
struct NetInterface { name: String, mtu: u32, ip: String, pkts_in: u64, bytes_in: u64, errs_in: u64, pkts_out: u64, bytes_out: u64, errs_out: u64, status: String }

#[derive(Debug, Clone, Default)]
struct GpuInfo { chipset: String, gpu_type: String, bus: String, cores: String, vendor: String, metal: String, display_type: String, resolution: String, display_name: String }

#[derive(Debug, Clone, Default)]
struct BatteryInfo { level: String, state: String, remaining: String, cycle_count: String, condition: String, voltage: String, amperage: String, temperature: String, max_capacity: String, design_capacity: String, present: bool }

#[derive(Debug, Clone, Default)]
struct SysInfo { hostname: String, os_type: String, os_release: String, os_build: String, boot_time: String, uptime: String, hw_model: String }

#[derive(Debug, Clone)]
struct NetConnection {
    pid: u32,
    process: String,
    fd: String,
    proto: String,
    local_addr: String,
    remote_addr: String,
    state: String,
}

fn parse_lsof() -> Vec<NetConnection> {
    let out = Command::new("lsof").args(["-i","-P","-n"]).output().ok();
    let out = match out { Some(o) => String::from_utf8_lossy(&o.stdout).to_string(), None => return Vec::new() };
    let mut conns: Vec<NetConnection> = Vec::new();
    for line in out.lines().skip(1) {
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() < 9 { continue; }
        let pid: u32 = cols[1].parse().unwrap_or(0);
        let process = cols[0].to_string();
        let fd = cols[3].to_string();
        let proto_type = cols[7].to_string(); // TCP or UDP
        let name = cols[8..].join(" ");
        let (local, remote, state) = if name.contains("->") {
            let parts: Vec<&str> = name.splitn(2, "->").collect();
            let rem_state: Vec<&str> = parts.get(1).unwrap_or(&"").splitn(2, " ").collect();
            (parts[0].to_string(), rem_state[0].to_string(),
             rem_state.get(1).unwrap_or(&"").trim_matches(|c| c=='(' || c==')').to_string())
        } else {
            let parts: Vec<&str> = name.splitn(2, " ").collect();
            (parts[0].to_string(), String::new(),
             parts.get(1).unwrap_or(&"").trim_matches(|c| c=='(' || c==')').to_string())
        };
        conns.push(NetConnection { pid, process, fd, proto: proto_type, local_addr: local, remote_addr: remote, state });
    }
    conns
}

// ═══════════════════════════════════════════════════════════════════════════════
// FILE SCANNING
// ═══════════════════════════════════════════════════════════════════════════════
fn is_system_path(p: &str) -> bool {
    let sys = ["/System","/Library","/usr","/bin","/sbin","/var","/private","/etc","/dev","/tmp","/cores","/.fseventsd","/.Spotlight"];
    sys.iter().any(|s| p.starts_with(s)) || p.starts_with("/.")
}

fn scan_directory(path: &str, sort: SortMode, filter_system: bool) -> Vec<FileEntry> {
    let Ok(entries) = fs::read_dir(path) else { return Vec::new() };
    let mut files: Vec<FileEntry> = entries.filter_map(|e| {
        let e = e.ok()?;
        let meta = e.metadata().ok()?;
        let name = e.file_name().to_string_lossy().to_string();
        let full = e.path().to_string_lossy().to_string();
        let is_dir = meta.is_dir();
        let size = if is_dir { dir_size_shallow(&full) } else { meta.len() };
        let is_system = is_system_path(&full);
        Some(FileEntry { name, path: full, size, is_dir, is_system })
    }).collect();
    if filter_system { files.retain(|f| !f.is_system); }
    match sort {
        SortMode::SizeDsc => files.sort_by(|a,b| b.size.cmp(&a.size)),
        SortMode::SizeAsc => files.sort_by(|a,b| a.size.cmp(&b.size)),
        SortMode::NameAsc => files.sort_by(|a,b| a.name.to_lowercase().cmp(&b.name.to_lowercase())),
        SortMode::NameDsc => files.sort_by(|a,b| b.name.to_lowercase().cmp(&a.name.to_lowercase())),
    }
    files
}

fn dir_size_shallow(path: &str) -> u64 {
    fs::read_dir(path).ok().map(|entries| {
        entries.filter_map(|e| e.ok()?.metadata().ok().map(|m| m.len())).sum()
    }).unwrap_or(0)
}

// ═══════════════════════════════════════════════════════════════════════════════
// APP STATE
// ═══════════════════════════════════════════════════════════════════════════════
const TAB_COUNT: usize = 10;

struct App {
    tab: usize,
    sys: System,
    vm_stat: VmStatData, top_stats: TopStats, swap_info: SwapInfo, sys_info: SysInfo,
    regions: Vec<ProcessRegion>, region_scroll: usize,
    cpu_details: CpuDetailedInfo, cpu_cores: Vec<CpuCoreInfo>, load_avg: [f64; 3], cpu_scroll: usize,
    disk_list: Vec<DiskInfo>, disk_hw: DiskHwInfo, iostat: IoStatInfo, disk_scroll: usize,
    disk_mode: DiskMode, disk_cursor: usize, disk_files: Vec<FileEntry>, disk_sort: SortMode,
    disk_path: String, disk_file_cursor: usize, disk_filter_system: bool,
    net_interfaces: Vec<NetInterface>, net_scroll: usize,
    gpu: GpuInfo,
    battery: BatteryInfo,
    cameras: Vec<CameraInfo>,
    activity_connections: Vec<NetConnection>, activity_scroll: usize,
    last_refresh: Instant, tick_count: u64, pid: u32,
    ram_scroll: usize,
}

impl App {
    fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        std::thread::sleep(Duration::from_millis(200));
        sys.refresh_cpu_usage();
        let pid = std::process::id();
        let la = System::load_average();
        Self {
            tab: 0, vm_stat: parse_vm_stat(), top_stats: parse_top(), swap_info: parse_swap(),
            sys_info: parse_sysinfo(), regions: collect_process_maps(pid), region_scroll: 0,
            cpu_details: collect_cpu_details(), cpu_cores: collect_cpu_cores(&sys),
            load_avg: [la.one, la.five, la.fifteen], cpu_scroll: 0,
            disk_list: collect_disks(), disk_hw: parse_diskutil(), iostat: parse_iostat(), disk_scroll: 0,
            disk_mode: DiskMode::Partitions, disk_cursor: 0, disk_files: Vec::new(), disk_sort: SortMode::SizeDsc,
            disk_path: String::new(), disk_file_cursor: 0, disk_filter_system: true,
            net_interfaces: parse_netstat(), net_scroll: 0,
            gpu: parse_gpu(), battery: parse_battery(), cameras: collect_cameras(),
            activity_connections: parse_lsof(), activity_scroll: 0,
            sys, last_refresh: Instant::now(), tick_count: 0, pid, ram_scroll: 0,
        }
    }
    fn refresh(&mut self) {
        self.sys.refresh_memory();
        self.sys.refresh_cpu_usage();
        self.vm_stat = parse_vm_stat();
        self.top_stats = parse_top();
        self.swap_info = parse_swap();
        self.cpu_cores = collect_cpu_cores(&self.sys);
        let la = System::load_average();
        self.load_avg = [la.one, la.five, la.fifteen];
        self.regions = collect_process_maps(self.pid);
        self.iostat = parse_iostat();
        self.net_interfaces = parse_netstat();
        if self.tick_count % 5 == 0 { self.disk_list = collect_disks(); self.battery = parse_battery(); }
        if self.tick_count % 3 == 0 { self.activity_connections = parse_lsof(); }
        if self.tick_count % 30 == 0 { self.cameras = collect_cameras(); self.gpu = parse_gpu(); self.disk_hw = parse_diskutil(); }
        self.last_refresh = Instant::now();
        self.tick_count += 1;
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// DATA COLLECTION
// ═══════════════════════════════════════════════════════════════════════════════

fn cmd_output(cmd: &str, args: &[&str]) -> String {
    Command::new(cmd).args(args).output().map(|o| String::from_utf8_lossy(&o.stdout).to_string()).unwrap_or_default()
}

fn sysctl_val(key: &str) -> String {
    let out = cmd_output("sysctl", &["-n", key]);
    out.trim().to_string()
}

fn parse_vm_stat() -> VmStatData {
    let text = cmd_output("vm_stat", &[]);
    let mut d = VmStatData::default();
    for line in text.lines() {
        if line.starts_with("Mach Virtual Memory") {
            if let Some(p) = line.find("page size of ") { d.page_size = line[p+13..].split_whitespace().next().and_then(|s| s.parse().ok()).unwrap_or(16384); }
            continue;
        }
        let parts: Vec<&str> = line.splitn(2, ':').collect();
        if parts.len() != 2 { continue; }
        let v: u64 = parts[1].trim().trim_end_matches('.').parse().unwrap_or(0);
        match parts[0].trim() {
            "Pages free"=>d.free=v, "Pages active"=>d.active=v, "Pages inactive"=>d.inactive=v,
            "Pages speculative"=>d.speculative=v, "Pages wired down"=>d.wired=v,
            "Pages purgeable"=>d.purgeable=v, "Pages stored in compressor"=>d.compressor=v,
            "Pageins"=>d.pageins=v, "Pageouts"=>d.pageouts=v, "\"Translation faults\""=>d.faults=v,
            "Pages copy-on-write"=>d.copy_on_write=v, "Pages zero filled"=>d.zero_fill=v,
            "Compressions"=>d.compressions=v, "Decompressions"=>d.decompressions=v,
            "Swapins"=>d.swapins=v, "Swapouts"=>d.swapouts=v,
            "Pages throttled"=>d.throttled=v, "Pages reactivated"=>d.reactivated=v, _=>{}
        }
    }
    if d.page_size == 0 { d.page_size = 16384; }
    d
}

fn parse_top() -> TopStats {
    let text = cmd_output("top", &["-l", "1", "-n", "10", "-stats", "pid,command,rsize,cpu"]);
    let mut s = TopStats::default();
    for line in text.lines() {
        let l = line.trim();
        if l.starts_with("Processes:") {
            for part in l.split(',') {
                let p = part.trim();
                if p.contains("total") { s.processes = p.split_whitespace().next().and_then(|v| v.parse().ok()).unwrap_or(0); }
                if p.contains("running") { s.running = p.split_whitespace().next().and_then(|v| v.parse().ok()).unwrap_or(0); }
                if p.contains("sleeping") { s.sleeping = p.split_whitespace().next().and_then(|v| v.parse().ok()).unwrap_or(0); }
                if p.contains("threads") { s.threads = p.split_whitespace().next().and_then(|v| v.parse().ok()).unwrap_or(0); }
            }
        } else if l.starts_with("CPU usage:") {
            for part in l[10..].split(',') {
                let p = part.trim();
                if p.contains("user") { s.cpu_user = p.replace('%', "").split_whitespace().next().and_then(|v| v.parse().ok()).unwrap_or(0.0); }
                if p.contains("sys") { s.cpu_sys = p.replace('%', "").split_whitespace().next().and_then(|v| v.parse().ok()).unwrap_or(0.0); }
                if p.contains("idle") { s.cpu_idle = p.replace('%', "").split_whitespace().next().and_then(|v| v.parse().ok()).unwrap_or(0.0); }
            }
        } else if l.starts_with("PhysMem:") {
            let inner = &l[8..];
            for part in inner.split(',') {
                let p = part.trim();
                if p.contains("used") { s.phys_used = p.split_whitespace().next().unwrap_or("").into(); }
                if p.contains("wired") { s.phys_wired = p.trim_start_matches('(').split_whitespace().next().unwrap_or("").into(); }
                if p.contains("compressor") { s.phys_compressor = p.split_whitespace().next().unwrap_or("").into(); }
                if p.contains("unused") { s.phys_unused = p.split_whitespace().next().unwrap_or("").into(); }
            }
        } else if l.starts_with("MemRegions:") {
            for part in l[11..].split(',') {
                let p = part.trim();
                if p.contains("total") { s.mem_regions_total = p.split_whitespace().next().and_then(|v| v.parse().ok()).unwrap_or(0); }
                if p.contains("resident") { s.mem_regions_resident = p.split_whitespace().next().unwrap_or("").into(); }
                if p.contains("private") { s.mem_regions_private = p.split_whitespace().next().unwrap_or("").into(); }
                if p.contains("shared") { s.mem_regions_shared = p.split_whitespace().next().unwrap_or("").into(); }
            }
        } else if l.starts_with("SharedLibs:") {
            for part in l[11..].split(',') {
                let p = part.trim();
                if p.contains("resident") { s.sharedlibs_resident = p.split_whitespace().next().unwrap_or("").into(); }
                if p.contains("data") { s.sharedlibs_data = p.split_whitespace().next().unwrap_or("").into(); }
            }
        } else if l.starts_with("Networks:") {
            for part in l[9..].split(',') {
                let p = part.trim();
                if p.contains("in") { let w: Vec<&str> = p.split('/').collect(); if w.len()>=2 { s.net_packets_in=w[0].split_whitespace().last().unwrap_or("").into(); s.net_bytes_in=w[1].split_whitespace().next().unwrap_or("").into(); } }
                if p.contains("out") { let w: Vec<&str> = p.split('/').collect(); if w.len()>=2 { s.net_packets_out=w[0].split_whitespace().last().unwrap_or("").into(); s.net_bytes_out=w[1].split_whitespace().next().unwrap_or("").into(); } }
            }
        } else if l.starts_with("Disks:") {
            for part in l[6..].split(',') {
                let p = part.trim();
                if p.contains("read") { let w: Vec<&str> = p.split('/').collect(); if w.len()>=2 { s.disk_reads=w[0].split_whitespace().last().unwrap_or("").into(); s.disk_read_bytes=w[1].split_whitespace().next().unwrap_or("").into(); } }
                if p.contains("written") { let w: Vec<&str> = p.split('/').collect(); if w.len()>=2 { s.disk_writes=w[0].split_whitespace().last().unwrap_or("").into(); s.disk_write_bytes=w[1].split_whitespace().next().unwrap_or("").into(); } }
            }
        } else if !l.is_empty() && l.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
            let cols: Vec<&str> = l.split_whitespace().collect();
            if cols.len() >= 4 { s.top_procs.push((cols[1].into(), cols[2].into(), cols[3].into())); }
        }
    }
    s
}

fn parse_swap() -> SwapInfo {
    let text = sysctl_val("vm.swapusage");
    let mut s = SwapInfo::default();
    for part in text.split("  ") {
        let p = part.trim();
        if p.starts_with("total") { s.total = p.split('=').nth(1).unwrap_or("").trim().into(); }
        if p.starts_with("used") { s.used = p.split('=').nth(1).unwrap_or("").trim().into(); }
        if p.starts_with("free") { s.free = p.split('=').nth(1).unwrap_or("").trim().into(); }
    }
    s.encrypted = text.contains("encrypted");
    s
}

fn parse_sysinfo() -> SysInfo {
    let boot_raw = sysctl_val("kern.boottime");
    let boot_time = boot_raw.split('}').last().unwrap_or("").trim().to_string();
    // Calculate uptime
    let sec_str = boot_raw.split("sec = ").nth(1).unwrap_or("0").split(',').next().unwrap_or("0");
    let boot_sec: u64 = sec_str.trim().parse().unwrap_or(0);
    let now_sec = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
    let up = now_sec.saturating_sub(boot_sec);
    let days = up / 86400; let hrs = (up % 86400) / 3600; let mins = (up % 3600) / 60;
    let uptime = if days > 0 { format!("{}d {}h {}m", days, hrs, mins) } else { format!("{}h {}m", hrs, mins) };
    SysInfo {
        hostname: sysctl_val("kern.hostname"), os_type: sysctl_val("kern.ostype"),
        os_release: sysctl_val("kern.osrelease"), os_build: sysctl_val("kern.osversion"),
        boot_time, uptime, hw_model: sysctl_val("hw.targettype"),
    }
}

fn classify_region(map: &MapRange) -> (RegionType, String) {
    let name = map.filename().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
    if name.contains("[stack]") || name.to_lowercase().contains("stack") { return (RegionType::Stack, name); }
    if name.contains("[heap]") || name.to_lowercase().contains("heap") { return (RegionType::Heap, name); }
    if !name.is_empty() {
        if name.contains(".dylib") { return (RegionType::Dylib, name); }
        if name.ends_with(".so") || name.contains("__TEXT") { return (RegionType::Code, name); }
        return (RegionType::MappedFile, name);
    }
    if map.is_exec() { return (RegionType::Code, "[anon exec]".into()); }
    if map.is_write() && map.size() > 1024*1024 { return (RegionType::Heap, "[anon heap]".into()); }
    (RegionType::Anonymous, "[anon]".into())
}

fn collect_process_maps(pid: u32) -> Vec<ProcessRegion> {
    get_process_maps(pid as proc_maps::Pid).unwrap_or_default().iter().map(|m| {
        let (rt, name) = classify_region(m);
        ProcessRegion { start: m.start(), end: m.start()+m.size(), size: m.size(),
            perms: format!("{}{}{}", if m.is_read(){"R"}else{"-"}, if m.is_write(){"W"}else{"-"}, if m.is_exec(){"X"}else{"-"}),
            region_type: rt, name }
    }).collect()
}

fn collect_cpu_cores(sys: &System) -> Vec<CpuCoreInfo> {
    sys.cpus().iter().map(|c| CpuCoreInfo { name: c.name().into(), usage: c.cpu_usage(), frequency: c.frequency() }).collect()
}

fn collect_cpu_details() -> CpuDetailedInfo {
    let mut i = CpuDetailedInfo::default();
    let keys = ["machdep.cpu.brand_string","machdep.cpu.core_count","machdep.cpu.thread_count","machdep.cpu.cores_per_package",
        "hw.cachelinesize","hw.l1icachesize","hw.l1dcachesize","hw.l2cachesize","hw.l3cachesize",
        "hw.nperflevels","hw.perflevel0.physicalcpu","hw.perflevel1.physicalcpu",
        "hw.perflevel0.l1icachesize","hw.perflevel0.l1dcachesize","hw.perflevel0.l2cachesize",
        "hw.perflevel1.l1icachesize","hw.perflevel1.l1dcachesize","hw.perflevel1.l2cachesize","hw.pagesize","hw.memsize"];
    let text = cmd_output("sysctl", &keys);
    for line in text.lines() {
        let p: Vec<&str> = line.splitn(2, ':').collect();
        if p.len()!=2 { continue; }
        let (k,v) = (p[0].trim(), p[1].trim());
        match k {
            "machdep.cpu.brand_string"=>i.brand=v.into(), "machdep.cpu.core_count"=>i.core_count=v.parse().unwrap_or(0),
            "machdep.cpu.thread_count"=>i.thread_count=v.parse().unwrap_or(0), "machdep.cpu.cores_per_package"=>i.cores_per_package=v.parse().unwrap_or(0),
            "hw.cachelinesize"=>i.cache_line_size=v.parse().unwrap_or(0), "hw.l1icachesize"=>i.l1i_cache=v.parse().unwrap_or(0),
            "hw.l1dcachesize"=>i.l1d_cache=v.parse().unwrap_or(0), "hw.l2cachesize"=>i.l2_cache=v.parse().unwrap_or(0),
            "hw.l3cachesize"=>i.l3_cache=v.parse().unwrap_or(0), "hw.nperflevels"=>i.num_perf_levels=v.parse().unwrap_or(0),
            "hw.perflevel0.physicalcpu"=>i.perf_cores=v.parse().unwrap_or(0), "hw.perflevel1.physicalcpu"=>i.efficiency_cores=v.parse().unwrap_or(0),
            "hw.perflevel0.l1icachesize"=>i.perf_l1i=v.parse().unwrap_or(0), "hw.perflevel0.l1dcachesize"=>i.perf_l1d=v.parse().unwrap_or(0),
            "hw.perflevel0.l2cachesize"=>i.perf_l2=v.parse().unwrap_or(0), "hw.perflevel1.l1icachesize"=>i.eff_l1i=v.parse().unwrap_or(0),
            "hw.perflevel1.l1dcachesize"=>i.eff_l1d=v.parse().unwrap_or(0), "hw.perflevel1.l2cachesize"=>i.eff_l2=v.parse().unwrap_or(0),
            "hw.pagesize"=>i.page_size=v.parse().unwrap_or(0), "hw.memsize"=>i.phys_mem=v.parse().unwrap_or(0), _=>{}
        }
    }
    if i.brand.is_empty() { i.brand = "Unknown".into(); }
    i.arch = System::cpu_arch();
    let feat_text = cmd_output("sysctl", &["hw.optional"]);
    for line in feat_text.lines() {
        let p: Vec<&str> = line.splitn(2, ':').collect();
        if p.len()==2 && p[1].trim()=="1" { i.features.push(p[0].rsplit('.').next().unwrap_or("").into()); }
    }
    i
}

fn collect_disks() -> Vec<DiskInfo> {
    Disks::new_with_refreshed_list().iter().map(|d| {
        let t=d.total_space(); let a=d.available_space();
        DiskInfo { name: d.name().to_string_lossy().into(), mount_point: d.mount_point().to_string_lossy().into(),
            fs_type: d.file_system().to_string_lossy().into(), total:t, used:t.saturating_sub(a), available:a, is_removable: d.is_removable() }
    }).collect()
}

fn parse_diskutil() -> DiskHwInfo {
    let text = cmd_output("diskutil", &["info", "disk0"]);
    let mut d = DiskHwInfo::default();
    for line in text.lines() {
        let p: Vec<&str> = line.splitn(2, ':').collect();
        if p.len()!=2 { continue; }
        let (k,v) = (p[0].trim(), p[1].trim());
        match k {
            "Device / Media Name"=>d.media_name=v.into(), "Protocol"=>d.protocol=v.into(),
            "SMART Status"=>d.smart_status=v.into(), "Disk Size"=>d.disk_size=v.into(),
            "Device Block Size"=>d.block_size=v.into(), "Content (IOContent)"=>d.content=v.into(),
            "Device Identifier"=>d.device_name=v.into(), _=>{}
        }
    }
    d
}

fn parse_iostat() -> IoStatInfo {
    let text = cmd_output("iostat", &["-d", "-c", "1"]);
    let mut io = IoStatInfo::default();
    for line in text.lines().rev() {
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() >= 3 {
            if let (Ok(a), Ok(b), Ok(c)) = (cols[0].parse::<f64>(), cols[1].parse::<f64>(), cols[2].parse::<f64>()) {
                io.kb_per_transfer = a; io.transfers_per_sec = b; io.mb_per_sec = c; break;
            }
        }
    }
    io
}

fn parse_netstat() -> Vec<NetInterface> {
    let text = cmd_output("netstat", &["-ib"]);
    let ifconfig = cmd_output("ifconfig", &[]);
    let mut ifaces = Vec::new();
    for line in text.lines().skip(1) {
        let c: Vec<&str> = line.split_whitespace().collect();
        if c.len() < 11 { continue; }
        let name = c[0].to_string();
        if name.ends_with('*') { continue; } // skip inactive link entries
        let mtu: u32 = c[1].parse().unwrap_or(0);
        if mtu == 0 { continue; }
        // Only take link-level entries (have MAC or <Link#>)
        if !c[2].contains("Link") { continue; }
        let pkts_in: u64 = c[4].parse().unwrap_or(0);
        let errs_in: u64 = c[5].parse().unwrap_or(0);
        let bytes_in: u64 = c[6].parse().unwrap_or(0);
        let pkts_out: u64 = c[7].parse().unwrap_or(0);
        let errs_out: u64 = c[8].parse().unwrap_or(0);
        let bytes_out: u64 = c[9].parse().unwrap_or(0);
        // Get IP and status from ifconfig
        let mut ip = String::new();
        let mut status = String::new();
        let mut in_iface = false;
        for ifl in ifconfig.lines() {
            if ifl.starts_with(&name) && ifl.contains(": flags=") { in_iface = true; }
            else if !ifl.starts_with(' ') && !ifl.starts_with('\t') { in_iface = false; }
            if in_iface {
                if ifl.contains("inet ") && !ifl.contains("inet6") {
                    ip = ifl.split("inet ").nth(1).unwrap_or("").split_whitespace().next().unwrap_or("").into();
                }
                if ifl.contains("status:") { status = ifl.split("status:").nth(1).unwrap_or("").trim().into(); }
            }
        }
        if status.is_empty() { status = "unknown".into(); }
        ifaces.push(NetInterface { name, mtu, ip, pkts_in, bytes_in, errs_in, pkts_out, bytes_out, errs_out, status });
    }
    ifaces
}

fn parse_gpu() -> GpuInfo {
    let text = cmd_output("system_profiler", &["SPDisplaysDataType"]);
    let mut g = GpuInfo::default();
    for line in text.lines() {
        let l = line.trim();
        if l.starts_with("Chipset Model:") { g.chipset = l[14..].trim().into(); }
        if l.starts_with("Type:") { g.gpu_type = l[5..].trim().into(); }
        if l.starts_with("Bus:") { g.bus = l[4..].trim().into(); }
        if l.starts_with("Total Number of Cores:") { g.cores = l[22..].trim().into(); }
        if l.starts_with("Vendor:") { g.vendor = l[7..].trim().into(); }
        if l.starts_with("Metal Support:") { g.metal = l[14..].trim().into(); }
        if l.starts_with("Display Type:") { g.display_type = l[13..].trim().into(); }
        if l.starts_with("Resolution:") { g.resolution = l[11..].trim().into(); }
        if l.starts_with("Color LCD:") || l.starts_with("LG") || l.contains("Display:") { g.display_name = l.trim_end_matches(':').into(); }
    }
    g
}

fn parse_battery() -> BatteryInfo {
    let text = cmd_output("pmset", &["-g", "batt"]);
    let mut b = BatteryInfo::default();
    for line in text.lines() {
        if line.contains("InternalBattery") {
            b.present = true;
            if let Some(pct) = line.split('\t').nth(1) {
                b.level = pct.split(';').next().unwrap_or("").trim().into();
                let parts: Vec<&str> = pct.split(';').collect();
                if parts.len() > 1 { b.state = parts[1].trim().into(); }
                if parts.len() > 2 { b.remaining = parts[2].trim().into(); }
            }
        }
    }
    let ioreg = cmd_output("ioreg", &["-r", "-c", "AppleSmartBattery", "-d", "1"]);
    for line in ioreg.lines() {
        let l = line.trim().trim_start_matches('"').replace("\" = ", "=");
        if l.contains("CycleCount=") && !l.contains("Designed") { b.cycle_count = l.split('=').last().unwrap_or("").trim().into(); }
        if l.contains("MaxCapacity=") && !l.contains("Design") { b.max_capacity = l.split('=').last().unwrap_or("").trim().into(); }
        if l.contains("DesignCapacity=") { b.design_capacity = l.split('=').last().unwrap_or("").trim().into(); }
        if l.contains("Temperature=") { b.temperature = l.split('=').last().unwrap_or("").trim().into(); }
        if l.contains("Voltage=") { b.voltage = l.split('=').last().unwrap_or("").trim().into(); }
        if l.contains("InstantAmperage=") { b.amperage = l.split('=').last().unwrap_or("").trim().into(); }
        if l.contains("BatteryHealth=") { b.condition = l.split('=').last().unwrap_or("").trim().into(); }
    }
    b
}

fn collect_cameras() -> Vec<CameraInfo> {
    let text = cmd_output("system_profiler", &["SPCameraDataType", "-json"]);
    let mut cams = Vec::new();
    let (mut name, mut model, mut uid) = (String::new(), String::new(), String::new());
    for line in text.lines() {
        let t = line.trim();
        if let Some(v) = extract_json(t, "\"_name\"") {
            if !name.is_empty() { cams.push(CameraInfo { name: name.clone(), model_id: model.clone(), unique_id: uid.clone() }); }
            name = v; model.clear(); uid.clear();
        } else if let Some(v) = extract_json(t, "\"spcamera_model-id\"") { model = v; }
        else if let Some(v) = extract_json(t, "\"spcamera_unique-id\"") { uid = v; }
    }
    if !name.is_empty() { cams.push(CameraInfo { name, model_id: model, unique_id: uid }); }
    cams
}

fn extract_json(line: &str, key: &str) -> Option<String> {
    if !line.contains(key) { return None; }
    let v = line.split(':').skip(1).collect::<Vec<&str>>().join(":");
    let t = v.trim().trim_matches('"').trim_end_matches(',').trim_matches('"');
    if t.is_empty() { None } else { Some(t.into()) }
}

fn hs(bytes: u64) -> String {
    if bytes >= 1<<30 { format!("{:.2} GB", bytes as f64 / (1<<30) as f64) }
    else if bytes >= 1<<20 { format!("{:.1} MB", bytes as f64 / (1<<20) as f64) }
    else if bytes >= 1024 { format!("{:.1} KB", bytes as f64 / 1024.0) }
    else { format!("{} B", bytes) }
}
fn hsu(b: usize) -> String { hs(b as u64) }
fn trunc(s: &str, m: usize) -> String { if s.len()<=m { s.into() } else { format!("…{}", &s[s.len()-m+1..]) } }

// ═══════════════════════════════════════════════════════════════════════════════
// UI RENDERING
// ═══════════════════════════════════════════════════════════════════════════════

fn ui(f: &mut Frame, app: &App) {
    let ch = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(10), Constraint::Length(3)]).split(f.area());
    let titles: Vec<Line> = ["1:RAM","2:Map","3:Vis","4:CPU","5:Disk","6:Net","7:GPU","8:Bat","9:Cam","0:Act"]
        .iter().map(|t| Line::from(format!(" {} ", t))).collect();
    f.render_widget(Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(format!(" ⚡ {} │ {} │ up {} ", app.sys_info.hostname, app.sys_info.os_type, app.sys_info.uptime)))
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)).select(app.tab), ch[0]);
    match app.tab {
        0=>render_ram(f,app,ch[1]), 1=>render_regions(f,app,ch[1]), 2=>render_visual(f,app,ch[1]),
        3=>render_cpu(f,app,ch[1]), 4=>render_disk(f,app,ch[1]), 5=>render_net(f,app,ch[1]),
        6=>render_gpu(f,app,ch[1]), 7=>render_battery(f,app,ch[1]), 8=>render_camera(f,app,ch[1]),
        9=>render_activity(f,app,ch[1]), _=>{}
    }
    f.render_widget(Paragraph::new(Line::from(vec![
        Span::styled(" q",Style::default().fg(Color::Red).bold()), Span::raw(":Quit "),
        Span::styled("1-9",Style::default().fg(Color::Cyan).bold()), Span::raw(":Tab "),
        Span::styled("↑↓/jk",Style::default().fg(Color::Yellow).bold()), Span::raw(":Scroll "),
        Span::styled("Tab",Style::default().fg(Color::Green).bold()), Span::raw(":Next "),
        Span::raw(format!("│ PID:{} │ Tick:{} │ {}/{}/{}",app.pid,app.tick_count,app.sys_info.os_type,app.sys_info.os_release,app.sys_info.os_build)),
    ])).block(Block::default().borders(Borders::ALL)).alignment(Alignment::Center), ch[2]);
}

// ── Tab 1: RAM ──
fn render_ram(f: &mut Frame, app: &App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();
    let total = app.sys.total_memory(); let used = app.sys.used_memory();
    let pct = if total>0 { used as f64/total as f64*100.0 } else { 0.0 };
    let pc = if pct<50.0{Color::Green} else if pct<80.0{Color::Yellow} else {Color::Red};
    // Header
    lines.push(Line::from(vec![
        Span::styled("━━━ Physical Memory ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━", Style::default().fg(Color::Cyan).bold())]));
    lines.push(Line::from(vec![
        Span::styled("  Total: ", Style::default().fg(Color::White).bold()), Span::raw(hs(total)),
        Span::raw("  │  "), Span::styled("Used: ", Style::default().fg(pc).bold()), Span::raw(format!("{} ({:.1}%)", hs(used), pct)),
        Span::raw("  │  "), Span::styled("Free: ", Style::default().fg(Color::Green)), Span::raw(hs(total.saturating_sub(used))),
    ]));
    // Top overview from top command
    lines.push(Line::from(vec![
        Span::styled("  PhysMem: ", Style::default().fg(Color::Yellow)), Span::raw(&app.top_stats.phys_used), Span::raw(" used"),
        Span::raw("  ("), Span::styled("wired ", Style::default().fg(Color::Red)), Span::raw(&app.top_stats.phys_wired),
        Span::raw(", "), Span::styled("compressor ", Style::default().fg(Color::Magenta)), Span::raw(&app.top_stats.phys_compressor),
        Span::raw(")  "), Span::styled("unused: ", Style::default().fg(Color::Green)), Span::raw(&app.top_stats.phys_unused),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Regions: ", Style::default().fg(Color::Cyan)), Span::raw(format!("{} total", app.top_stats.mem_regions_total)),
        Span::raw(format!("  (resident {}, private {}, shared {})", app.top_stats.mem_regions_resident, app.top_stats.mem_regions_private, app.top_stats.mem_regions_shared)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  SharedLibs: ", Style::default().fg(Color::Blue)), Span::raw(format!("{} resident, {} data", app.top_stats.sharedlibs_resident, app.top_stats.sharedlibs_data)),
    ]));
    lines.push(Line::from(""));
    // Swap
    lines.push(Line::from(Span::styled("━━━ Swap ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━", Style::default().fg(Color::Magenta).bold())));
    lines.push(Line::from(vec![
        Span::styled("  Total: ", Style::default().fg(Color::White)), Span::raw(&app.swap_info.total),
        Span::raw("  │  "), Span::styled("Used: ", Style::default().fg(Color::Yellow)), Span::raw(&app.swap_info.used),
        Span::raw("  │  "), Span::styled("Free: ", Style::default().fg(Color::Green)), Span::raw(&app.swap_info.free),
        Span::raw(if app.swap_info.encrypted {"  │  🔒 Encrypted"} else {""}),
    ]));
    lines.push(Line::from(""));
    // VM Pages
    let ps = app.vm_stat.page_size;
    let vm = &app.vm_stat;
    lines.push(Line::from(Span::styled(format!("━━━ VM Page Categories (page size: {} bytes) ━━━━━━━━━━━━━━", ps), Style::default().fg(Color::Green).bold())));
    let cats = [("Active",vm.active,Color::Green),("Inactive",vm.inactive,Color::Yellow),("Speculative",vm.speculative,Color::Blue),
        ("Wired",vm.wired,Color::Red),("Compressed",vm.compressor,Color::Magenta),("Purgeable",vm.purgeable,Color::Cyan),
        ("Free",vm.free,Color::White),("Throttled",vm.throttled,Color::DarkGray),("Reactivated",vm.reactivated,Color::LightYellow)];
    for (label,pages,color) in &cats {
        let size = pages * ps;
        let bar_w = 20usize;
        let total_pages = vm.active+vm.inactive+vm.speculative+vm.wired+vm.compressor+vm.free;
        let frac = if total_pages>0 { (*pages as f64 / total_pages as f64 * bar_w as f64) as usize } else { 0 };
        lines.push(Line::from(vec![
            Span::styled(format!("  {:>12} ", label), Style::default().fg(*color)),
            Span::styled("█".repeat(frac), Style::default().fg(*color)),
            Span::styled("░".repeat(bar_w.saturating_sub(frac)), Style::default().fg(Color::DarkGray)),
            Span::raw(format!(" {:>8} pages  {:>10}", pages, hs(size))),
        ]));
    }
    lines.push(Line::from(""));
    // Page fault stats
    lines.push(Line::from(Span::styled("━━━ Page Fault & Compression Stats ━━━━━━━━━━━━━━━━━━━━━━━━━", Style::default().fg(Color::Red).bold())));
    lines.push(Line::from(vec![
        Span::styled("  Faults: ", Style::default().fg(Color::Cyan)), Span::raw(format!("{}", vm.faults)),
        Span::raw("  │  "), Span::styled("Pageins: ", Style::default().fg(Color::Green)), Span::raw(format!("{}", vm.pageins)),
        Span::raw("  │  "), Span::styled("Pageouts: ", Style::default().fg(Color::Red)), Span::raw(format!("{}", vm.pageouts)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  CoW: ", Style::default().fg(Color::Yellow)), Span::raw(format!("{}", vm.copy_on_write)),
        Span::raw("  │  "), Span::styled("Zero Fill: ", Style::default().fg(Color::Magenta)), Span::raw(format!("{}", vm.zero_fill)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Compressions: ", Style::default().fg(Color::Cyan)), Span::raw(format!("{}", vm.compressions)),
        Span::raw("  │  "), Span::styled("Decompressions: ", Style::default().fg(Color::Green)), Span::raw(format!("{}", vm.decompressions)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Swap In: ", Style::default().fg(Color::Yellow)), Span::raw(format!("{}", vm.swapins)),
        Span::raw("  │  "), Span::styled("Swap Out: ", Style::default().fg(Color::Red)), Span::raw(format!("{}", vm.swapouts)),
    ]));
    lines.push(Line::from(""));
    // Process stats
    lines.push(Line::from(Span::styled("━━━ Process Stats ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━", Style::default().fg(Color::Yellow).bold())));
    lines.push(Line::from(vec![
        Span::styled("  Processes: ", Style::default().fg(Color::Cyan)), Span::raw(format!("{}", app.top_stats.processes)),
        Span::raw("  │  "), Span::styled("Running: ", Style::default().fg(Color::Green)), Span::raw(format!("{}", app.top_stats.running)),
        Span::raw("  │  "), Span::styled("Sleeping: ", Style::default().fg(Color::DarkGray)), Span::raw(format!("{}", app.top_stats.sleeping)),
        Span::raw("  │  "), Span::styled("Threads: ", Style::default().fg(Color::Magenta)), Span::raw(format!("{}", app.top_stats.threads)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  CPU: ", Style::default().fg(Color::Cyan)),
        Span::styled(format!("{:.1}% user", app.top_stats.cpu_user), Style::default().fg(Color::Green)),
        Span::raw("  "), Span::styled(format!("{:.1}% sys", app.top_stats.cpu_sys), Style::default().fg(Color::Red)),
        Span::raw("  "), Span::styled(format!("{:.1}% idle", app.top_stats.cpu_idle), Style::default().fg(Color::DarkGray)),
    ]));
    lines.push(Line::from(""));
    // Top memory consumers
    lines.push(Line::from(Span::styled("━━━ Top Memory Consumers ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━", Style::default().fg(Color::LightBlue).bold())));
    for (i, (name, mem, cpu)) in app.top_stats.top_procs.iter().enumerate() {
        lines.push(Line::from(vec![
            Span::styled(format!("  #{:<2} ", i+1), Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{:<25}", name), Style::default().fg(Color::White).bold()),
            Span::styled(format!("{:>8}", mem), Style::default().fg(Color::Yellow)),
            Span::raw("  "), Span::styled(format!("{:>6}", cpu), Style::default().fg(Color::Cyan)),
        ]));
    }
    // Scrollable render
    let ih = area.height.saturating_sub(2) as usize;
    let total = lines.len();
    let scroll = app.ram_scroll.min(total.saturating_sub(ih));
    let vis: Vec<Line> = lines.into_iter().skip(scroll).take(ih).collect();
    f.render_widget(Paragraph::new(vis).block(Block::default().borders(Borders::ALL).title(format!(" 🧠 RAM [{}-{}/{}] ", scroll+1,(scroll+ih).min(total),total))), area);
    let mut sb = ScrollbarState::new(total).position(scroll);
    f.render_stateful_widget(Scrollbar::new(ScrollbarOrientation::VerticalRight), area, &mut sb);
}

// ── Tab 2: Process Regions ──
fn render_regions(f: &mut Frame, app: &App, area: Rect) {
    let ch = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(8), Constraint::Length(7)]).split(area);

    // ── Summary ──
    let tv: usize = app.regions.iter().map(|r| r.size).sum();
    let mut tc: HashMap<RegionType,(usize,usize)> = HashMap::new();
    for r in &app.regions { let e=tc.entry(r.region_type).or_insert((0,0)); e.0+=1; e.1+=r.size; }
    let sp: Vec<Span> = [RegionType::Stack,RegionType::Heap,RegionType::Code,RegionType::Dylib,RegionType::Anonymous,RegionType::MappedFile]
        .iter().flat_map(|t| { let(c,s)=tc.get(t).unwrap_or(&(0,0)); vec![
            Span::styled(format!(" {} ",t.label()),Style::default().fg(Color::Black).bg(t.color())),
            Span::raw(format!(":{} ({})  ",c,hs(*s as u64))),
    ]}).collect();
    f.render_widget(Paragraph::new(vec![
        Line::from(vec![Span::styled("Regions: ",Style::default().fg(Color::Cyan).bold()),Span::raw(format!("{}  ",app.regions.len())),
            Span::styled("Virtual: ",Style::default().fg(Color::Yellow).bold()),Span::raw(hsu(tv)),
            Span::raw("  │  "),Span::styled("PID: ",Style::default().fg(Color::Magenta).bold()),Span::raw(format!("{}",app.pid))]),
        Line::from(sp),
    ]).block(Block::default().borders(Borders::ALL).title(" Process Memory Map ")), ch[0]);

    // ── Table with highlighted cursor ──
    let hdr = Row::new(["Start","End","Size","Perm","Type","Name"].map(|h| Cell::from(h).style(Style::default().bold().fg(Color::Cyan)))).height(1).bottom_margin(0);
    let vh = ch[1].height.saturating_sub(3) as usize; // visible height
    let cursor = app.region_scroll.min(app.regions.len().saturating_sub(1));
    // Auto-scroll to keep cursor visible
    let scroll_top = if cursor >= vh { cursor - vh + 1 } else { 0 };

    let rows: Vec<Row> = app.regions.iter().enumerate().skip(scroll_top).take(vh).map(|(i, r)| {
        let c = r.region_type.color();
        let row = Row::new(vec![
            Cell::from(format!("0x{:012x}",r.start)),
            Cell::from(format!("0x{:012x}",r.end)),
            Cell::from(hsu(r.size)),
            Cell::from(r.perms.clone()),
            Cell::from(r.region_type.label()).style(Style::default().fg(Color::Black).bg(c)),
            Cell::from(trunc(&r.name,40)),
        ]);
        if i == cursor {
            row.style(Style::default().fg(Color::White).bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        } else {
            row.style(Style::default().fg(c))
        }
    }).collect();

    f.render_widget(Table::new(rows,[Constraint::Min(16),Constraint::Min(16),Constraint::Length(10),Constraint::Length(5),Constraint::Length(6),Constraint::Min(30)])
        .header(hdr).block(Block::default().borders(Borders::ALL)
        .title(format!(" Regions [{}/{}] ↑↓ navigate, Enter to inspect ", cursor+1, app.regions.len()))), ch[1]);
    let mut sb = ScrollbarState::new(app.regions.len()).position(cursor);
    f.render_stateful_widget(Scrollbar::new(ScrollbarOrientation::VerticalRight), ch[1], &mut sb);

    // ── Detail panel for selected region ──
    if let Some(r) = app.regions.get(cursor) {
        let size_pct = if tv > 0 { r.size as f64 / tv as f64 * 100.0 } else { 0.0 };
        let detail = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("▸ ", Style::default().fg(Color::Cyan)),
                Span::styled(format!("{} ", r.region_type.label()), Style::default().fg(Color::Black).bg(r.region_type.color()).bold()),
                Span::raw("  "),
                Span::styled(&r.name, Style::default().fg(Color::White).bold()),
            ]),
            Line::from(vec![
                Span::styled("  Address: ", Style::default().fg(Color::Yellow)),
                Span::raw(format!("0x{:016x} → 0x{:016x}", r.start, r.end)),
                Span::raw("  │  "),
                Span::styled("Size: ", Style::default().fg(Color::Green)),
                Span::raw(format!("{} ({:.2}% of virtual)", hsu(r.size), size_pct)),
            ]),
            Line::from(vec![
                Span::styled("  Permissions: ", Style::default().fg(Color::Magenta)),
                Span::styled(if r.perms.contains('R') {"R"} else {"-"}, Style::default().fg(if r.perms.contains('R') {Color::Green} else {Color::DarkGray})),
                Span::styled(if r.perms.contains('W') {"W"} else {"-"}, Style::default().fg(if r.perms.contains('W') {Color::Yellow} else {Color::DarkGray})),
                Span::styled(if r.perms.contains('X') {"X"} else {"-"}, Style::default().fg(if r.perms.contains('X') {Color::Red} else {Color::DarkGray})),
                Span::raw(format!("  │  Pages: ~{}", r.size / app.vm_stat.page_size.max(1) as usize)),
            ]),
        ]).block(Block::default().borders(Borders::ALL).title(" ▸ Selected Region Detail "));
        f.render_widget(detail, ch[2]);
    }
}

// ── Tab 3: Visual ──
fn render_visual(f: &mut Frame, app: &App, area: Rect) {
    let ch = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(8), Constraint::Length(12)]).split(area);

    // ── Legend ──
    let ls: Vec<Span> = [RegionType::Stack,RegionType::Heap,RegionType::Code,RegionType::Dylib,RegionType::Anonymous,RegionType::MappedFile]
        .iter().flat_map(|t| vec![
            Span::styled(format!(" █ {} ",t.label()),Style::default().fg(t.color())),Span::raw(" ")
        ]).collect();
    f.render_widget(Paragraph::new(Line::from(ls)).block(Block::default().borders(Borders::ALL).title(" Legend ")).alignment(Alignment::Center), ch[0]);

    // ── Block map with selected region highlighted ──
    if app.regions.is_empty() {
        f.render_widget(Paragraph::new("No regions.").block(Block::default().borders(Borders::ALL)).alignment(Alignment::Center), ch[1]);
    } else {
        let iw = ch[1].width.saturating_sub(2) as usize;
        let ih = ch[1].height.saturating_sub(2) as usize;
        if iw > 0 && ih > 0 {
            let mn = app.regions.iter().map(|r|r.start).min().unwrap_or(0);
            let mx = app.regions.iter().map(|r|r.end).max().unwrap_or(1);
            let ar = if mx>mn{mx-mn}else{1};
            let total_cells = iw*ih;
            let bpc = (ar/total_cells.max(1)).max(1);
            let mut cells = vec![('·',Color::DarkGray);total_cells];
            let cursor = app.region_scroll.min(app.regions.len().saturating_sub(1));

            for (idx, r) in app.regions.iter().enumerate() {
                let so = r.start.saturating_sub(mn);
                let eo = r.end.saturating_sub(mn).min(ar);
                let (cs,ce) = (so/bpc, (eo/bpc).min(total_cells));
                let glyph = match r.region_type {
                    RegionType::Stack=>'█', RegionType::Heap=>'▓', RegionType::Code=>'▒',
                    RegionType::Dylib=>'░', RegionType::Anonymous=>'·', RegionType::MappedFile=>'▪',
                };
                let color = if idx == cursor { Color::White } else { r.region_type.color() };
                for i in cs..ce { if i<total_cells { cells[i]=(glyph,color); } }
            }

            let lines: Vec<Line> = (0..ih).map(|row| {
                let s = row*iw; let e = (s+iw).min(total_cells);
                Line::from(cells[s..e].iter().map(|(c,col)|
                    Span::styled(c.to_string(),Style::default().fg(*col))
                ).collect::<Vec<_>>())
            }).collect();

            f.render_widget(Paragraph::new(lines).block(Block::default().borders(Borders::ALL)
                .title(format!(" Address Space (0x{:x}..0x{:x}, {}/cell) — Selected region in white ",mn,mx,hsu(bpc)))), ch[1]);
        }
    }

    // ── Memory Summary Panel ──
    let tv: usize = app.regions.iter().map(|r| r.size).sum();
    let mut tc: HashMap<RegionType,(usize,usize)> = HashMap::new();
    for r in &app.regions { let e = tc.entry(r.region_type).or_insert((0,0)); e.0+=1; e.1+=r.size; }

    let mut summary: Vec<Line> = Vec::new();
    summary.push(Line::from(vec![
        Span::styled("Total Virtual: ", Style::default().fg(Color::Cyan).bold()),
        Span::raw(hsu(tv)),
        Span::raw(format!("  ({} regions, ~{} pages)", app.regions.len(), tv / app.vm_stat.page_size.max(1) as usize)),
        Span::raw("  │  "),
        Span::styled("PhysMem: ", Style::default().fg(Color::Yellow)),
        Span::raw(format!("{} used, {} free", app.top_stats.phys_used, app.top_stats.phys_unused)),
    ]));
    summary.push(Line::from(""));

    let types = [RegionType::Stack,RegionType::Heap,RegionType::Code,RegionType::Dylib,RegionType::MappedFile,RegionType::Anonymous];
    for t in &types {
        let (count, size) = tc.get(t).copied().unwrap_or((0,0));
        let pct = if tv > 0 { size as f64 / tv as f64 * 100.0 } else { 0.0 };
        let bw = 20usize;
        let filled = (pct as usize * bw / 100).min(bw);
        summary.push(Line::from(vec![
            Span::styled(format!("  {:>8} ", t.label()), Style::default().fg(t.color()).bold()),
            Span::styled("█".repeat(filled), Style::default().fg(t.color())),
            Span::styled("░".repeat(bw.saturating_sub(filled)), Style::default().fg(Color::DarkGray)),
            Span::raw(format!(" {:5.1}%  {:>3} regions  {:>10}", pct, count, hsu(size))),
        ]));
    }

    // Selected region info at bottom
    if let Some(r) = app.regions.get(app.region_scroll.min(app.regions.len().saturating_sub(1))) {
        summary.push(Line::from(""));
        summary.push(Line::from(vec![
            Span::styled("  ▸ Selected: ", Style::default().fg(Color::White).bold()),
            Span::styled(format!("{} ", r.region_type.label()), Style::default().fg(Color::Black).bg(r.region_type.color())),
            Span::raw(format!(" {} — {} ({})", trunc(&r.name, 30), hsu(r.size), r.perms)),
        ]));
    }

    f.render_widget(Paragraph::new(summary).block(Block::default().borders(Borders::ALL).title(" Memory Breakdown ")), ch[2]);
}

// ── Tab 4: CPU ──
fn render_cpu(f: &mut Frame, app: &App, area: Rect) {
    let d = &app.cpu_details;
    let ch = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(7),Constraint::Length(3),Constraint::Min(10)]).split(area);
    // Identity
    let mut id_lines = vec![
        Line::from(vec![Span::styled("Brand: ",Style::default().fg(Color::Cyan).bold()),
            Span::styled(&d.brand,Style::default().fg(Color::White).bold())]),
        Line::from(vec![Span::styled("Arch: ",Style::default().fg(Color::Yellow).bold()),Span::raw(&d.arch),
            Span::raw("  │  "),Span::styled("Cores: ",Style::default().fg(Color::Green).bold()),Span::raw(format!("{}",d.core_count)),
            Span::raw("  │  "),Span::styled("Threads: ",Style::default().fg(Color::Magenta).bold()),Span::raw(format!("{}",d.thread_count)),
            Span::raw("  │  "),Span::styled("Pkg: ",Style::default().fg(Color::LightBlue).bold()),Span::raw(format!("{}",d.cores_per_package))]),
        Line::from(vec![Span::styled("Load: ",Style::default().fg(Color::Red).bold()),
            Span::raw(format!("{:.2}/{:.2}/{:.2} (1/5/15m)",app.load_avg[0],app.load_avg[1],app.load_avg[2])),
            Span::raw("  │  "),Span::styled("CPU: ",Style::default().fg(Color::Cyan)),
            Span::styled(format!("{:.1}%u ",app.top_stats.cpu_user),Style::default().fg(Color::Green)),
            Span::styled(format!("{:.1}%s ",app.top_stats.cpu_sys),Style::default().fg(Color::Red)),
            Span::styled(format!("{:.1}%i",app.top_stats.cpu_idle),Style::default().fg(Color::DarkGray))]),
    ];
    if d.num_perf_levels>=2 {
        id_lines.push(Line::from(vec![Span::styled("Topology: ",Style::default().fg(Color::Cyan).bold()),
            Span::styled(format!("{} P-cores",d.perf_cores),Style::default().fg(Color::Green).bold()),Span::raw(" + "),
            Span::styled(format!("{} E-cores",d.efficiency_cores),Style::default().fg(Color::Yellow).bold())]));
    }
    f.render_widget(Paragraph::new(id_lines).block(Block::default().borders(Borders::ALL).title(" 🔬 Processor ")), ch[0]);
    // Overall gauge
    let avg = if !app.cpu_cores.is_empty() { app.cpu_cores.iter().map(|c|c.usage).sum::<f32>()/app.cpu_cores.len() as f32 } else { 0.0 };
    let ap = avg.min(100.0) as u16;
    let ac = if ap<50{Color::Green} else if ap<80{Color::Yellow} else {Color::Red};
    f.render_widget(Gauge::default().block(Block::default().borders(Borders::ALL).title(" Overall "))
        .gauge_style(Style::default().fg(ac).bg(Color::DarkGray)).percent(ap).label(format!("{:.1}%",avg)), ch[1]);
    // Scrollable details
    let mut cl: Vec<Line> = Vec::new();
    // Cache
    cl.push(Line::from(Span::styled("━━━ Cache Hierarchy ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",Style::default().fg(Color::Cyan).bold())));
    cl.push(Line::from(vec![Span::styled("  Line Size: ",Style::default().fg(Color::Yellow)),Span::raw(format!("{} B",d.cache_line_size))]));
    if d.num_perf_levels>=2 {
        cl.push(Line::from(Span::styled("  ┌─ P-Cores ─────────────────────",Style::default().fg(Color::Green))));
        cl.push(Line::from(vec![Span::styled("  │ L1i: ",Style::default().fg(Color::Green)),Span::raw(hs(d.perf_l1i)),
            Span::styled("  L1d: ",Style::default().fg(Color::Green)),Span::raw(hs(d.perf_l1d))]));
        cl.push(Line::from(vec![Span::styled("  │ L2:  ",Style::default().fg(Color::Green)),Span::raw(hs(d.perf_l2))]));
        cl.push(Line::from(Span::styled("  ├─ E-Cores ─────────────────────",Style::default().fg(Color::Yellow))));
        cl.push(Line::from(vec![Span::styled("  │ L1i: ",Style::default().fg(Color::Yellow)),Span::raw(hs(d.eff_l1i)),
            Span::styled("  L1d: ",Style::default().fg(Color::Yellow)),Span::raw(hs(d.eff_l1d))]));
        cl.push(Line::from(vec![Span::styled("  │ L2:  ",Style::default().fg(Color::Yellow)),Span::raw(hs(d.eff_l2))]));
        cl.push(Line::from(Span::styled("  └───────────────────────────────",Style::default().fg(Color::DarkGray))));
    } else {
        cl.push(Line::from(vec![Span::styled("  L1i: ",Style::default().fg(Color::Green)),Span::raw(hs(d.l1i_cache)),
            Span::styled("  L1d: ",Style::default().fg(Color::Green)),Span::raw(hs(d.l1d_cache))]));
        cl.push(Line::from(vec![Span::styled("  L2: ",Style::default().fg(Color::Yellow)),Span::raw(hs(d.l2_cache))]));
        if d.l3_cache>0 { cl.push(Line::from(vec![Span::styled("  L3: ",Style::default().fg(Color::Red)),Span::raw(hs(d.l3_cache))])); }
    }
    cl.push(Line::from(""));
    // Features
    cl.push(Line::from(Span::styled("━━━ ISA Extensions ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",Style::default().fg(Color::Magenta).bold())));
    let fpl = 6; let mut fi = 0;
    while fi < d.features.len() {
        let end = (fi+fpl).min(d.features.len());
        let spans: Vec<Span> = d.features[fi..end].iter().flat_map(|feat| {
            let c = if feat.contains("AES")||feat.contains("SHA")||feat.contains("PMULL"){Color::Red}
                else if feat.contains("SIMD")||feat.contains("FP16")||feat.contains("BF16")||feat.contains("I8MM")||feat.contains("DotProd"){Color::Cyan}
                else if feat.contains("LSE")||feat.contains("CRC")||feat.contains("PAuth")||feat.contains("BTI"){Color::Yellow}
                else{Color::White};
            vec![Span::styled(format!(" {} ",feat),Style::default().fg(Color::Black).bg(c)),Span::raw(" ")]
        }).collect();
        cl.push(Line::from([vec![Span::raw("  ")],spans].concat()));
        fi = end;
    }
    cl.push(Line::from(""));
    cl.push(Line::from(vec![Span::raw("  "),
        Span::styled(" Crypto ",Style::default().fg(Color::Black).bg(Color::Red)),Span::raw(" "),
        Span::styled(" SIMD/ML ",Style::default().fg(Color::Black).bg(Color::Cyan)),Span::raw(" "),
        Span::styled(" Atomic/Sec ",Style::default().fg(Color::Black).bg(Color::Yellow)),Span::raw(" "),
        Span::styled(" Other ",Style::default().fg(Color::Black).bg(Color::White))]));
    cl.push(Line::from(""));
    // Registers
    cl.push(Line::from(Span::styled("━━━ Register Architecture ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",Style::default().fg(Color::LightRed).bold())));
    let is_arm = d.arch.contains("arm64")||d.arch.contains("aarch64");
    cl.push(Line::from(vec![Span::styled("  Width: ",Style::default().fg(Color::Cyan)),Span::raw(if is_arm{"64-bit AArch64"}else{"64-bit x86_64"})]));
    cl.push(Line::from(vec![Span::styled("  GP Regs: ",Style::default().fg(Color::Yellow)),Span::raw(if is_arm{"31×64-bit (X0-X30) + SP,PC,PSTATE"}else{"16×64-bit (RAX-R15)"})]));
    cl.push(Line::from(vec![Span::styled("  FP/SIMD: ",Style::default().fg(Color::Green)),Span::raw(if is_arm{"32×128-bit (V0-V31, NEON)"}else{"16×256-bit YMM / 32×512-bit ZMM"})]));
    if d.features.iter().any(|f|f.contains("PAuth")) { cl.push(Line::from(vec![Span::styled("  PAC: ",Style::default().fg(Color::Magenta)),Span::raw("Enabled (IA,IB,DA,DB,GA)")])); }
    if d.features.iter().any(|f|f.contains("BTI")) { cl.push(Line::from(vec![Span::styled("  BTI: ",Style::default().fg(Color::LightRed)),Span::raw("Enabled (HW CFI)")])); }
    cl.push(Line::from(""));
    // Per-core
    cl.push(Line::from(Span::styled(format!("━━━ Per-Core ({} CPUs) ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",app.cpu_cores.len()),Style::default().fg(Color::Green).bold())));
    let bw = area.width.saturating_sub(32) as usize;
    for core in &app.cpu_cores {
        let p = core.usage.min(100.0); let fl = (p as usize*bw)/100; let em = bw.saturating_sub(fl);
        let c = if p<50.0{Color::Green}else if p<80.0{Color::Yellow}else{Color::Red};
        cl.push(Line::from(vec![Span::styled(format!("  {:>5} ",core.name),Style::default().fg(Color::Cyan)),
            Span::styled(format!("{:5.1}% ",p),Style::default().fg(c).bold()),
            Span::styled("█".repeat(fl),Style::default().fg(c)),Span::styled("░".repeat(em),Style::default().fg(Color::DarkGray)),
            Span::raw(format!(" {}MHz",core.frequency))]));
    }
    let ih = ch[2].height.saturating_sub(2) as usize;
    let total = cl.len(); let scroll = app.cpu_scroll.min(total.saturating_sub(ih));
    let vis: Vec<Line> = cl.into_iter().skip(scroll).take(ih).collect();
    f.render_widget(Paragraph::new(vis).block(Block::default().borders(Borders::ALL).title(format!(" 🔍 CPU [{}-{}/{}] ",scroll+1,(scroll+ih).min(total),total))), ch[2]);
    let mut sb = ScrollbarState::new(total).position(scroll);
    f.render_stateful_widget(Scrollbar::new(ScrollbarOrientation::VerticalRight), ch[2], &mut sb);
}

// ── Tab 5: Disk ──
fn render_disk(f: &mut Frame, app: &App, area: Rect) {
    match app.disk_mode {
        DiskMode::Partitions => render_disk_partitions(f, app, area),
        DiskMode::Files => render_disk_files(f, app, area),
    }
}

fn render_disk_partitions(f: &mut Frame, app: &App, area: Rect) {
    let ch = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(6), Constraint::Length(5)]).split(area);

    // ── HW + I/O info ──
    let hw = &app.disk_hw;
    let hw_lines = vec![
        Line::from(Span::styled("━━━ Primary Disk ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",Style::default().fg(Color::Cyan).bold())),
        Line::from(vec![Span::styled("  Device: ",Style::default().fg(Color::Yellow)),Span::raw(&hw.media_name),
            Span::raw("  │  "),Span::styled("Protocol: ",Style::default().fg(Color::Green)),Span::raw(&hw.protocol),
            Span::raw("  │  "),Span::styled("Size: ",Style::default().fg(Color::Magenta)),Span::raw(&hw.disk_size)]),
        Line::from(vec![Span::styled("  Block: ",Style::default().fg(Color::Cyan)),Span::raw(&hw.block_size),
            Span::raw("  │  "),Span::styled("Scheme: ",Style::default().fg(Color::Blue)),Span::raw(&hw.content),
            Span::raw("  │  "),Span::styled("SMART: ",Style::default().fg(if hw.smart_status.contains("Verified"){Color::Green}else{Color::Red})),
            Span::raw(&hw.smart_status)]),
        Line::from(vec![Span::styled("  I/O: ",Style::default().fg(Color::Yellow)),
            Span::raw(format!("{:.1} KB/t, {:.0} tps, {:.2} MB/s",app.iostat.kb_per_transfer,app.iostat.transfers_per_sec,app.iostat.mb_per_sec)),
            Span::raw("  │  "),Span::styled("Lifetime: ",Style::default().fg(Color::Magenta)),
            Span::raw(format!("{} R ({}), {} W ({})",app.top_stats.disk_reads,app.top_stats.disk_read_bytes,app.top_stats.disk_writes,app.top_stats.disk_write_bytes))]),
    ];
    f.render_widget(Paragraph::new(hw_lines).block(Block::default().borders(Borders::ALL).title(" 💾 Disk Hardware & I/O ")), ch[0]);

    // ── Partition list with cursor ──
    let hdr = Row::new(["Mount","Used","Total","Type","Usage"].map(|h|
        Cell::from(h).style(Style::default().bold().fg(Color::Cyan)))).height(1);
    let vh = ch[1].height.saturating_sub(3) as usize;
    let cursor = app.disk_cursor.min(app.disk_list.len().saturating_sub(1));
    let scroll_top = if cursor >= vh { cursor - vh + 1 } else { 0 };

    let rows: Vec<Row> = app.disk_list.iter().enumerate().skip(scroll_top).take(vh).map(|(i, d)| {
        let p = if d.total>0{d.used as f64/d.total as f64*100.0}else{0.0};
        let c = if p<70.0{Color::Green}else if p<90.0{Color::Yellow}else{Color::Red};
        let bw = 15usize; let fl = (p as usize*bw)/100;
        let bar = format!("{}{}",  "█".repeat(fl), "░".repeat(bw.saturating_sub(fl)));
        let row = Row::new(vec![
            Cell::from(format!("{}{}", d.mount_point, if d.is_removable{" ⏏"}else{""})),
            Cell::from(hs(d.used)),
            Cell::from(hs(d.total)),
            Cell::from(d.fs_type.clone()),
            Cell::from(format!("{} {:.1}%", bar, p)).style(Style::default().fg(c)),
        ]);
        if i == cursor {
            row.style(Style::default().fg(Color::White).bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        } else {
            row.style(Style::default().fg(Color::White))
        }
    }).collect();

    f.render_widget(Table::new(rows, [Constraint::Min(15),Constraint::Length(10),Constraint::Length(10),Constraint::Length(6),Constraint::Min(22)])
        .header(hdr).block(Block::default().borders(Borders::ALL)
        .title(format!(" Partitions [{}/{}] ↑↓ navigate, Enter → browse files ",cursor+1,app.disk_list.len()))), ch[1]);
    let mut sb = ScrollbarState::new(app.disk_list.len()).position(cursor);
    f.render_stateful_widget(Scrollbar::new(ScrollbarOrientation::VerticalRight), ch[1], &mut sb);

    // ── Help bar ──
    f.render_widget(Paragraph::new(vec![
        Line::from(vec![
            Span::styled(" ↑↓ ", Style::default().fg(Color::Black).bg(Color::Cyan)),Span::raw(" Navigate  "),
            Span::styled(" Enter ", Style::default().fg(Color::Black).bg(Color::Green)),Span::raw(" Browse Files  "),
            Span::styled(" Tab ", Style::default().fg(Color::Black).bg(Color::Yellow)),Span::raw(" Next Tab  "),
        ]),
    ]).block(Block::default().borders(Borders::ALL).title(" Keys ")), ch[2]);
}

fn render_disk_files(f: &mut Frame, app: &App, area: Rect) {
    let ch = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(6), Constraint::Length(5)]).split(area);

    // ── Path bar ──
    let sort_label = match app.disk_sort {
        SortMode::SizeDsc => "Size ↓", SortMode::SizeAsc => "Size ↑",
        SortMode::NameAsc => "Name A→Z", SortMode::NameDsc => "Name Z→A",
    };
    f.render_widget(Paragraph::new(Line::from(vec![
        Span::styled(" 📂 ", Style::default().fg(Color::Yellow)),
        Span::styled(&app.disk_path, Style::default().fg(Color::White).bold()),
        Span::raw("  │  "),
        Span::styled("Sort: ", Style::default().fg(Color::Cyan)),Span::raw(sort_label),
        Span::raw("  │  "),
        Span::styled("Filter: ", Style::default().fg(Color::Green)),
        Span::raw(if app.disk_filter_system { "Hide System ✓" } else { "Show All" }),
        Span::raw(format!("  │  {} items", app.disk_files.len())),
    ])).block(Block::default().borders(Borders::ALL).title(" File Browser ")), ch[0]);

    // ── File table with cursor ──
    let hdr = Row::new(["Name","Size","Type"].map(|h|
        Cell::from(h).style(Style::default().bold().fg(Color::Cyan)))).height(1);
    let vh = ch[1].height.saturating_sub(3) as usize;
    let cursor = app.disk_file_cursor.min(app.disk_files.len().saturating_sub(1));
    let scroll_top = if cursor >= vh { cursor - vh + 1 } else { 0 };

    let rows: Vec<Row> = app.disk_files.iter().enumerate().skip(scroll_top).take(vh).map(|(i, fe)| {
        let icon = if fe.is_dir { "📁 " } else { "📄 " };
        let sc = if fe.is_system { Color::DarkGray } else if fe.is_dir { Color::Cyan } else { Color::White };
        let type_label = if fe.is_dir { "DIR" } else {
            fe.name.rsplit('.').next().unwrap_or("???")
        };
        let sys_badge = if fe.is_system {
            Cell::from(format!("{} [SYS]", type_label)).style(Style::default().fg(Color::DarkGray))
        } else {
            Cell::from(type_label.to_string()).style(Style::default().fg(Color::Green))
        };
        let row = Row::new(vec![
            Cell::from(format!("{}{}", icon, fe.name)).style(Style::default().fg(sc)),
            Cell::from(hs(fe.size)),
            sys_badge,
        ]);
        if i == cursor {
            row.style(Style::default().fg(Color::White).bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        } else {
            row
        }
    }).collect();

    let title = if app.disk_files.is_empty() {
        " No files (or access denied) ".to_string()
    } else {
        format!(" [{}/{}] ↑↓ navigate, Enter → open folder, Esc → back ", cursor+1, app.disk_files.len())
    };
    f.render_widget(Table::new(rows, [Constraint::Min(40),Constraint::Length(12),Constraint::Length(12)])
        .header(hdr).block(Block::default().borders(Borders::ALL).title(title)), ch[1]);
    if !app.disk_files.is_empty() {
        let mut sb = ScrollbarState::new(app.disk_files.len()).position(cursor);
        f.render_stateful_widget(Scrollbar::new(ScrollbarOrientation::VerticalRight), ch[1], &mut sb);
    }

    // ── Help bar ──
    f.render_widget(Paragraph::new(vec![
        Line::from(vec![
            Span::styled(" ↑↓ ", Style::default().fg(Color::Black).bg(Color::Cyan)),Span::raw(" Navigate  "),
            Span::styled(" Enter ", Style::default().fg(Color::Black).bg(Color::Green)),Span::raw(" Open Dir  "),
            Span::styled(" Esc/← ", Style::default().fg(Color::Black).bg(Color::Yellow)),Span::raw(" Back  "),
            Span::styled(" s ", Style::default().fg(Color::Black).bg(Color::Magenta)),Span::raw(" Sort  "),
            Span::styled(" f ", Style::default().fg(Color::Black).bg(Color::Red)),Span::raw(" Filter  "),
        ]),
    ]).block(Block::default().borders(Borders::ALL).title(" Keys ")), ch[2]);
}

// ── Tab 6: Network ──
fn render_net(f: &mut Frame, app: &App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();
    // Global stats from top
    lines.push(Line::from(Span::styled("━━━ Network Totals (lifetime) ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",Style::default().fg(Color::Cyan).bold())));
    lines.push(Line::from(vec![
        Span::styled("  Packets In: ",Style::default().fg(Color::Green)),Span::raw(&app.top_stats.net_packets_in),
        Span::raw(" ("),Span::raw(&app.top_stats.net_bytes_in),Span::raw(")"),
        Span::raw("  │  "),Span::styled("Packets Out: ",Style::default().fg(Color::Yellow)),Span::raw(&app.top_stats.net_packets_out),
        Span::raw(" ("),Span::raw(&app.top_stats.net_bytes_out),Span::raw(")")]));
    lines.push(Line::from(""));
    // Interfaces
    lines.push(Line::from(Span::styled("━━━ Network Interfaces ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",Style::default().fg(Color::Green).bold())));
    for iface in &app.net_interfaces {
        let sc = if iface.status=="active"{Color::Green} else {Color::DarkGray};
        let active = iface.status == "active" || iface.pkts_in > 0 || iface.pkts_out > 0;
        lines.push(Line::from(vec![
            Span::styled(format!("  {:>10} ",iface.name),Style::default().fg(Color::White).bold()),
            Span::styled(format!("{:>10}",iface.status),Style::default().fg(sc)),
            Span::raw(format!("  MTU:{}",iface.mtu)),
            if !iface.ip.is_empty() { Span::styled(format!("  IP:{}",iface.ip),Style::default().fg(Color::Cyan)) } else { Span::raw("") },
        ]));
        if active {
            lines.push(Line::from(vec![
                Span::raw("             "),
                Span::styled("↓ ",Style::default().fg(Color::Green)),Span::raw(format!("{} pkts ({})  ",iface.pkts_in,hs(iface.bytes_in))),
                Span::styled("↑ ",Style::default().fg(Color::Yellow)),Span::raw(format!("{} pkts ({})  ",iface.pkts_out,hs(iface.bytes_out))),
                if iface.errs_in>0||iface.errs_out>0 { Span::styled(format!("Err:{}/{}",iface.errs_in,iface.errs_out),Style::default().fg(Color::Red)) } else { Span::raw("") },
            ]));
        }
    }
    let ih = area.height.saturating_sub(2) as usize;
    let total = lines.len(); let scroll = app.net_scroll.min(total.saturating_sub(ih));
    let vis: Vec<Line> = lines.into_iter().skip(scroll).take(ih).collect();
    f.render_widget(Paragraph::new(vis).block(Block::default().borders(Borders::ALL).title(format!(" 🌐 Network [{}-{}/{}] ",scroll+1,(scroll+ih).min(total),total))), area);
    let mut sb = ScrollbarState::new(total).position(scroll);
    f.render_stateful_widget(Scrollbar::new(ScrollbarOrientation::VerticalRight), area, &mut sb);
}

// ── Tab 7: GPU ──
fn render_gpu(f: &mut Frame, app: &App, area: Rect) {
    let g = &app.gpu;
    let lines = vec![
        Line::from(Span::styled("━━━ GPU Information ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",Style::default().fg(Color::Cyan).bold())),
        Line::from(vec![Span::styled("  Chipset: ",Style::default().fg(Color::Yellow).bold()),Span::styled(&g.chipset,Style::default().fg(Color::White).bold())]),
        Line::from(vec![Span::styled("  Type: ",Style::default().fg(Color::Green)),Span::raw(&g.gpu_type),
            Span::raw("  │  "),Span::styled("Bus: ",Style::default().fg(Color::Cyan)),Span::raw(&g.bus)]),
        Line::from(vec![Span::styled("  GPU Cores: ",Style::default().fg(Color::Magenta)),Span::raw(&g.cores),
            Span::raw("  │  "),Span::styled("Vendor: ",Style::default().fg(Color::Blue)),Span::raw(&g.vendor)]),
        Line::from(vec![Span::styled("  Metal: ",Style::default().fg(Color::LightRed).bold()),
            Span::styled(&g.metal,Style::default().fg(Color::Green).bold())]),
        Line::from(""),
        Line::from(Span::styled("━━━ Display ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",Style::default().fg(Color::Yellow).bold())),
        Line::from(vec![Span::styled("  Display: ",Style::default().fg(Color::Cyan)),Span::raw(&g.display_name)]),
        Line::from(vec![Span::styled("  Type: ",Style::default().fg(Color::Green)),Span::raw(&g.display_type)]),
        Line::from(vec![Span::styled("  Resolution: ",Style::default().fg(Color::Magenta).bold()),Span::styled(&g.resolution,Style::default().fg(Color::White).bold())]),
        Line::from(""),
        Line::from(Span::styled("━━━ Unified Memory Architecture ━━━━━━━━━━━━━━━━━━━━━━━━━━━━",Style::default().fg(Color::Green).bold())),
        Line::from(vec![Span::styled("  Physical Memory: ",Style::default().fg(Color::Cyan)),Span::raw(hs(app.cpu_details.phys_mem))]),
        Line::from(vec![Span::styled("  Note: ",Style::default().fg(Color::DarkGray)),Span::raw("Apple Silicon uses a unified memory architecture (UMA).")]),
        Line::from(vec![Span::raw("  GPU and CPU share the same physical memory pool.")]),
        Line::from(vec![Span::raw("  No dedicated VRAM — memory is dynamically allocated.")]),
    ];
    f.render_widget(Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(" 🎮 GPU / Display ")).wrap(Wrap{trim:false}), area);
}

// ── Tab 8: Battery ──
fn render_battery(f: &mut Frame, app: &App, area: Rect) {
    let b = &app.battery;
    if !b.present {
        f.render_widget(Paragraph::new("No battery detected (desktop Mac?)").block(Block::default().borders(Borders::ALL).title(" 🔋 Battery ")).alignment(Alignment::Center), area);
        return;
    }
    let pct_num: u16 = b.level.trim_end_matches('%').parse().unwrap_or(0);
    let bc = if pct_num>50{Color::Green}else if pct_num>20{Color::Yellow}else{Color::Red};
    let ch = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(3),Constraint::Min(10)]).split(area);
    f.render_widget(Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(format!(" 🔋 Battery — {} ",b.state)))
        .gauge_style(Style::default().fg(bc).bg(Color::DarkGray)).percent(pct_num)
        .label(format!("{} — {} — {}",b.level,b.state,b.remaining)), ch[0]);
    // Parse battery health
    let health_pct = if !b.max_capacity.is_empty() && !b.design_capacity.is_empty() {
        let max: f64 = b.max_capacity.parse().unwrap_or(0.0);
        let design: f64 = b.design_capacity.parse().unwrap_or(1.0);
        if design > 0.0 { format!("{:.1}%", max/design*100.0) } else { "N/A".into() }
    } else { "N/A".into() };
    let temp_c = if !b.temperature.is_empty() {
        let raw: f64 = b.temperature.parse().unwrap_or(0.0);
        format!("{:.1}°C", raw / 100.0)
    } else { "N/A".into() };
    let voltage_v = if !b.voltage.is_empty() {
        let mv: f64 = b.voltage.parse().unwrap_or(0.0);
        format!("{:.3} V", mv / 1000.0)
    } else { "N/A".into() };
    let amp_s = if !b.amperage.is_empty() {
        let ma: f64 = b.amperage.parse().unwrap_or(0.0);
        format!("{:.0} mA", ma)
    } else { "N/A".into() };
    let lines = vec![
        Line::from(Span::styled("━━━ Battery Details ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",Style::default().fg(Color::Cyan).bold())),
        Line::from(vec![Span::styled("  Charge: ",Style::default().fg(Color::Green).bold()),Span::raw(&b.level),
            Span::raw("  │  "),Span::styled("State: ",Style::default().fg(Color::Yellow)),Span::raw(&b.state)]),
        Line::from(vec![Span::styled("  Remaining: ",Style::default().fg(Color::Magenta)),Span::raw(&b.remaining)]),
        Line::from(""),
        Line::from(Span::styled("━━━ Health & Cycles ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",Style::default().fg(Color::Yellow).bold())),
        Line::from(vec![Span::styled("  Cycle Count: ",Style::default().fg(Color::Cyan)),Span::raw(&b.cycle_count)]),
        Line::from(vec![Span::styled("  Condition: ",Style::default().fg(Color::Green)),Span::raw(&b.condition)]),
        Line::from(vec![Span::styled("  Health: ",Style::default().fg(Color::Magenta)),Span::raw(&health_pct),
            Span::raw(format!("  (max: {} / design: {})",b.max_capacity,b.design_capacity))]),
        Line::from(""),
        Line::from(Span::styled("━━━ Electrical ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",Style::default().fg(Color::Green).bold())),
        Line::from(vec![Span::styled("  Voltage: ",Style::default().fg(Color::Cyan)),Span::raw(&voltage_v),
            Span::raw("  │  "),Span::styled("Current: ",Style::default().fg(Color::Yellow)),Span::raw(&amp_s)]),
        Line::from(vec![Span::styled("  Temperature: ",Style::default().fg(Color::Red)),Span::raw(&temp_c)]),
    ];
    f.render_widget(Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(" Battery Details ")), ch[1]);
}

// ── Tab 9: Camera ──
fn render_camera(f: &mut Frame, app: &App, area: Rect) {
    if app.cameras.is_empty() {
        f.render_widget(Paragraph::new(vec![Line::from(""),
            Line::from(Span::styled("No cameras detected",Style::default().fg(Color::Yellow).bold())),Line::from(""),
            Line::from("Reads from: system_profiler SPCameraDataType"),
        ]).block(Block::default().borders(Borders::ALL).title(" 📷 Cameras ")).alignment(Alignment::Center), area);
        return;
    }
    let mut lines: Vec<Line> = Vec::new();
    for (i,cam) in app.cameras.iter().enumerate() {
        lines.push(Line::from(Span::styled(format!("━━━ Camera {} ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",i+1),Style::default().fg(Color::Cyan).bold())));
        lines.push(Line::from(vec![Span::styled("  Name: ",Style::default().fg(Color::Yellow)),
            Span::styled(&cam.name,Style::default().fg(Color::White).bold()),
            Span::raw("  "),Span::styled("● Connected",Style::default().fg(Color::Green))]));
        lines.push(Line::from(vec![Span::styled("  Model: ",Style::default().fg(Color::Magenta)),Span::raw(if cam.model_id.is_empty(){"N/A"}else{&cam.model_id})]));
        lines.push(Line::from(vec![Span::styled("  UID: ",Style::default().fg(Color::Blue)),Span::raw(if cam.unique_id.is_empty(){"N/A"}else{&cam.unique_id})]));
        lines.push(Line::from(""));
    }
    f.render_widget(Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(format!(" 📷 Cameras ({}) ",app.cameras.len()))), area);
}

// ── Tab 10: System Activity ──
fn render_activity(f: &mut Frame, app: &App, area: Rect) {
    let ch = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(6), Constraint::Length(7)]).split(area);

    // ── Summary panel ──
    let total = app.activity_connections.len();
    let tcp_count = app.activity_connections.iter().filter(|c| c.proto == "TCP").count();
    let udp_count = app.activity_connections.iter().filter(|c| c.proto == "UDP").count();
    let established = app.activity_connections.iter().filter(|c| c.state == "ESTABLISHED").count();
    let listening = app.activity_connections.iter().filter(|c| c.state == "LISTEN").count();
    // Unique processes
    let mut procs: HashMap<String, usize> = HashMap::new();
    for c in &app.activity_connections { *procs.entry(c.process.clone()).or_insert(0) += 1; }
    let mut proc_counts: Vec<(String,usize)> = procs.into_iter().collect();
    proc_counts.sort_by(|a,b| b.1.cmp(&a.1));
    let top3: String = proc_counts.iter().take(3).map(|(n,c)| format!("{} ({})", n, c)).collect::<Vec<_>>().join(", ");

    f.render_widget(Paragraph::new(vec![
        Line::from(vec![
            Span::styled(" Connections: ", Style::default().fg(Color::Cyan).bold()),
            Span::raw(format!("{}", total)),
            Span::raw("  │  "),
            Span::styled("TCP: ", Style::default().fg(Color::Green)), Span::raw(format!("{}", tcp_count)),
            Span::raw("  │  "),
            Span::styled("UDP: ", Style::default().fg(Color::Yellow)), Span::raw(format!("{}", udp_count)),
            Span::raw("  │  "),
            Span::styled("ESTABLISHED: ", Style::default().fg(Color::Magenta)), Span::raw(format!("{}", established)),
            Span::raw("  │  "),
            Span::styled("LISTEN: ", Style::default().fg(Color::Blue)), Span::raw(format!("{}", listening)),
        ]),
        Line::from(vec![
            Span::styled(" Top: ", Style::default().fg(Color::Yellow).bold()),
            Span::raw(top3),
        ]),
    ]).block(Block::default().borders(Borders::ALL).title(" 🔍 System Activity — Live Network Connections ")), ch[0]);

    // ── Connection table with cursor ──
    let hdr = Row::new(["PID","Process","Proto","Local","Remote","State"].map(|h|
        Cell::from(h).style(Style::default().bold().fg(Color::Cyan)))).height(1);
    let vh = ch[1].height.saturating_sub(3) as usize;
    let cursor = app.activity_scroll.min(total.saturating_sub(1));
    let scroll_top = if cursor >= vh { cursor - vh + 1 } else { 0 };

    let rows: Vec<Row> = app.activity_connections.iter().enumerate().skip(scroll_top).take(vh).map(|(i, c)| {
        let state_color = match c.state.as_str() {
            "ESTABLISHED" => Color::Green,
            "LISTEN" => Color::Blue,
            "CLOSE_WAIT" | "TIME_WAIT" => Color::Yellow,
            "SYN_SENT" | "SYN_RECV" => Color::Magenta,
            _ => Color::White,
        };
        let row = Row::new(vec![
            Cell::from(format!("{}", c.pid)),
            Cell::from(c.process.clone()),
            Cell::from(c.proto.clone()).style(Style::default().fg(if c.proto == "TCP" { Color::Green } else { Color::Yellow })),
            Cell::from(c.local_addr.clone()),
            Cell::from(if c.remote_addr.is_empty() { "—".to_string() } else { c.remote_addr.clone() }),
            Cell::from(c.state.clone()).style(Style::default().fg(state_color)),
        ]);
        if i == cursor {
            row.style(Style::default().fg(Color::White).bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        } else {
            row
        }
    }).collect();

    f.render_widget(Table::new(rows, [Constraint::Length(7),Constraint::Length(16),Constraint::Length(5),
        Constraint::Min(22),Constraint::Min(26),Constraint::Length(14)])
        .header(hdr).block(Block::default().borders(Borders::ALL)
        .title(format!(" [{}/{}] ↑↓ select ", cursor+1, total))), ch[1]);
    if total > 0 {
        let mut sb = ScrollbarState::new(total).position(cursor);
        f.render_stateful_widget(Scrollbar::new(ScrollbarOrientation::VerticalRight), ch[1], &mut sb);
    }

    // ── Detail panel for selected connection ──
    let detail = if let Some(c) = app.activity_connections.get(cursor) {
        vec![
            Line::from(vec![
                Span::styled(" Process: ", Style::default().fg(Color::Yellow).bold()),
                Span::raw(format!("{} (PID {})", c.process, c.pid)),
                Span::raw("  │  "),
                Span::styled("FD: ", Style::default().fg(Color::Cyan)),
                Span::raw(&c.fd),
                Span::raw("  │  "),
                Span::styled("Protocol: ", Style::default().fg(Color::Green)),
                Span::raw(&c.proto),
            ]),
            Line::from(vec![
                Span::styled(" Local:  ", Style::default().fg(Color::Magenta).bold()),
                Span::raw(&c.local_addr),
            ]),
            Line::from(vec![
                Span::styled(" Remote: ", Style::default().fg(Color::Red).bold()),
                Span::raw(if c.remote_addr.is_empty() { "— (listening/bound)" } else { &c.remote_addr }),
                Span::raw("  │  "),
                Span::styled("State: ", Style::default().fg(Color::Blue)),
                Span::raw(&c.state),
            ]),
        ]
    } else {
        vec![Line::from("No connections")]
    };
    f.render_widget(Paragraph::new(detail)
        .block(Block::default().borders(Borders::ALL).title(" Connection Details ")), ch[2]);
}

// ═══════════════════════════════════════════════════════════════════════════════
// MAIN
// ═══════════════════════════════════════════════════════════════════════════════

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;
    terminal.clear()?;

    let mut app = App::new();
    let tick_rate = Duration::from_secs(1);

    loop {
        terminal.draw(|f| ui(f, &app))?;
        let timeout = tick_rate.checked_sub(app.last_refresh.elapsed()).unwrap_or(Duration::from_millis(50));
        if crossterm::event::poll(timeout.min(Duration::from_millis(100)))? {
            if let Event::Key(key) = crossterm::event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Esc => {
                            if app.tab == 4 && app.disk_mode == DiskMode::Files {
                                // Go back in file browser
                                let parent = std::path::Path::new(&app.disk_path).parent()
                                    .map(|p| p.to_string_lossy().to_string())
                                    .unwrap_or_default();
                                if !parent.is_empty() && parent.len() >= app.disk_list.get(app.disk_cursor).map(|d| d.mount_point.len()).unwrap_or(1) {
                                    app.disk_path = parent;
                                    app.disk_files = scan_directory(&app.disk_path, app.disk_sort, app.disk_filter_system);
                                    app.disk_file_cursor = 0;
                                } else {
                                    app.disk_mode = DiskMode::Partitions;
                                }
                            } else {
                                break;
                            }
                        },
                        KeyCode::Char('1') => app.tab = 0, KeyCode::Char('2') => app.tab = 1,
                        KeyCode::Char('3') => app.tab = 2, KeyCode::Char('4') => app.tab = 3,
                        KeyCode::Char('5') => app.tab = 4, KeyCode::Char('6') => app.tab = 5,
                        KeyCode::Char('7') => app.tab = 6, KeyCode::Char('8') => app.tab = 7,
                        KeyCode::Char('9') => app.tab = 8, KeyCode::Char('0') => app.tab = 9,
                        KeyCode::Tab => app.tab = (app.tab + 1) % TAB_COUNT,
                        KeyCode::BackTab => app.tab = if app.tab == 0 { TAB_COUNT - 1 } else { app.tab - 1 },
                        KeyCode::Enter => {
                            if app.tab == 4 {
                                if app.disk_mode == DiskMode::Partitions {
                                    // Open selected partition in file browser
                                    if let Some(d) = app.disk_list.get(app.disk_cursor) {
                                        app.disk_path = d.mount_point.clone();
                                        app.disk_files = scan_directory(&app.disk_path, app.disk_sort, app.disk_filter_system);
                                        app.disk_file_cursor = 0;
                                        app.disk_mode = DiskMode::Files;
                                    }
                                } else {
                                    // Navigate into directory
                                    if let Some(fe) = app.disk_files.get(app.disk_file_cursor) {
                                        if fe.is_dir {
                                            let new_path = fe.path.clone();
                                            app.disk_path = new_path;
                                            app.disk_files = scan_directory(&app.disk_path, app.disk_sort, app.disk_filter_system);
                                            app.disk_file_cursor = 0;
                                        }
                                    }
                                }
                            }
                        },
                        KeyCode::Char('s') => {
                            if app.tab == 4 && app.disk_mode == DiskMode::Files {
                                app.disk_sort = match app.disk_sort {
                                    SortMode::SizeDsc => SortMode::SizeAsc,
                                    SortMode::SizeAsc => SortMode::NameAsc,
                                    SortMode::NameAsc => SortMode::NameDsc,
                                    SortMode::NameDsc => SortMode::SizeDsc,
                                };
                                app.disk_files = scan_directory(&app.disk_path, app.disk_sort, app.disk_filter_system);
                                app.disk_file_cursor = 0;
                            }
                        },
                        KeyCode::Char('f') => {
                            if app.tab == 4 && app.disk_mode == DiskMode::Files {
                                app.disk_filter_system = !app.disk_filter_system;
                                app.disk_files = scan_directory(&app.disk_path, app.disk_sort, app.disk_filter_system);
                                app.disk_file_cursor = 0;
                            }
                        },
                        KeyCode::Right => {
                            if app.tab == 4 && app.disk_mode == DiskMode::Files {
                                // Navigate into directory (same as Enter)
                                if let Some(fe) = app.disk_files.get(app.disk_file_cursor) {
                                    if fe.is_dir {
                                        let new_path = fe.path.clone();
                                        app.disk_path = new_path;
                                        app.disk_files = scan_directory(&app.disk_path, app.disk_sort, app.disk_filter_system);
                                        app.disk_file_cursor = 0;
                                    }
                                }
                            } else {
                                app.tab = (app.tab + 1) % TAB_COUNT;
                            }
                        },
                        KeyCode::Left => {
                            if app.tab == 4 && app.disk_mode == DiskMode::Files {
                                // Go back (same as Esc)
                                let parent = std::path::Path::new(&app.disk_path).parent()
                                    .map(|p| p.to_string_lossy().to_string())
                                    .unwrap_or_default();
                                let min_len = app.disk_list.get(app.disk_cursor).map(|d| d.mount_point.len()).unwrap_or(1);
                                if !parent.is_empty() && parent.len() >= min_len {
                                    app.disk_path = parent;
                                    app.disk_files = scan_directory(&app.disk_path, app.disk_sort, app.disk_filter_system);
                                    app.disk_file_cursor = 0;
                                } else {
                                    app.disk_mode = DiskMode::Partitions;
                                }
                            } else {
                                app.tab = if app.tab == 0 { TAB_COUNT - 1 } else { app.tab - 1 };
                            }
                        },
                        KeyCode::Down | KeyCode::Char('j') => match app.tab {
                            0 => app.ram_scroll = app.ram_scroll.saturating_add(1),
                            1 | 2 => app.region_scroll = app.region_scroll.saturating_add(1).min(app.regions.len().saturating_sub(1)),
                            3 => app.cpu_scroll = app.cpu_scroll.saturating_add(1),
                            4 => {
                                if app.disk_mode == DiskMode::Partitions {
                                    app.disk_cursor = app.disk_cursor.saturating_add(1).min(app.disk_list.len().saturating_sub(1));
                                } else {
                                    app.disk_file_cursor = app.disk_file_cursor.saturating_add(1).min(app.disk_files.len().saturating_sub(1));
                                }
                            },
                            5 => app.net_scroll = app.net_scroll.saturating_add(1),
                            9 => app.activity_scroll = app.activity_scroll.saturating_add(1).min(app.activity_connections.len().saturating_sub(1)),
                            _ => {}
                        },
                        KeyCode::Up | KeyCode::Char('k') => match app.tab {
                            0 => app.ram_scroll = app.ram_scroll.saturating_sub(1),
                            1 | 2 => app.region_scroll = app.region_scroll.saturating_sub(1),
                            3 => app.cpu_scroll = app.cpu_scroll.saturating_sub(1),
                            4 => {
                                if app.disk_mode == DiskMode::Partitions {
                                    app.disk_cursor = app.disk_cursor.saturating_sub(1);
                                } else {
                                    app.disk_file_cursor = app.disk_file_cursor.saturating_sub(1);
                                }
                            },
                            5 => app.net_scroll = app.net_scroll.saturating_sub(1),
                            9 => app.activity_scroll = app.activity_scroll.saturating_sub(1),
                            _ => {}
                        },
                        KeyCode::PageDown => match app.tab {
                            0 => app.ram_scroll = app.ram_scroll.saturating_add(10),
                            1 | 2 => app.region_scroll = app.region_scroll.saturating_add(20).min(app.regions.len().saturating_sub(1)),
                            3 => app.cpu_scroll = app.cpu_scroll.saturating_add(10),
                            4 => {
                                if app.disk_mode == DiskMode::Partitions {
                                    app.disk_cursor = app.disk_cursor.saturating_add(10).min(app.disk_list.len().saturating_sub(1));
                                } else {
                                    app.disk_file_cursor = app.disk_file_cursor.saturating_add(20).min(app.disk_files.len().saturating_sub(1));
                                }
                            },
                            5 => app.net_scroll = app.net_scroll.saturating_add(10),
                            9 => app.activity_scroll = app.activity_scroll.saturating_add(20).min(app.activity_connections.len().saturating_sub(1)),
                            _ => {}
                        },
                        KeyCode::PageUp => match app.tab {
                            0 => app.ram_scroll = app.ram_scroll.saturating_sub(10),
                            1 | 2 => app.region_scroll = app.region_scroll.saturating_sub(20),
                            3 => app.cpu_scroll = app.cpu_scroll.saturating_sub(10),
                            4 => {
                                if app.disk_mode == DiskMode::Partitions {
                                    app.disk_cursor = app.disk_cursor.saturating_sub(10);
                                } else {
                                    app.disk_file_cursor = app.disk_file_cursor.saturating_sub(20);
                                }
                            },
                            5 => app.net_scroll = app.net_scroll.saturating_sub(10),
                            9 => app.activity_scroll = app.activity_scroll.saturating_sub(20),
                            _ => {}
                        },
                        _ => {}
                    }
                }
            }
        }
        if app.last_refresh.elapsed() >= tick_rate { app.refresh(); }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}
