use crate::types::*;
use crate::utils::*;
use sysinfo::Disks;

pub fn collect_disks() -> Vec<DiskInfo> {
    Disks::new_with_refreshed_list()
        .iter()
        .map(|d| {
            let t = d.total_space();
            let a = d.available_space();
            DiskInfo {
                name: d.name().to_string_lossy().into(),
                mount_point: d.mount_point().to_string_lossy().into(),
                fs_type: d.file_system().to_string_lossy().into(),
                total: t,
                used: t.saturating_sub(a),
                available: a,
                is_removable: d.is_removable(),
            }
        })
        .collect()
}

pub fn parse_diskutil() -> DiskHwInfo {
    let text = cmd_output("diskutil", &["info", "disk0"]);
    let mut d = DiskHwInfo::default();
    for line in text.lines() {
        let p: Vec<&str> = line.splitn(2, ':').collect();
        if p.len() != 2 {
            continue;
        }
        let (k, v) = (p[0].trim(), p[1].trim());
        match k {
            "Device / Media Name" => d.media_name = v.into(),
            "Protocol" => d.protocol = v.into(),
            "SMART Status" => d.smart_status = v.into(),
            "Disk Size" => d.disk_size = v.into(),
            "Device Block Size" => d.block_size = v.into(),
            "Content (IOContent)" => d.content = v.into(),
            "Device Identifier" => d.device_name = v.into(),
            _ => {}
        }
    }
    d
}

pub fn parse_iostat() -> IoStatInfo {
    let text = cmd_output("iostat", &["-d", "-c", "1"]);
    let mut io = IoStatInfo::default();
    for line in text.lines().rev() {
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() >= 3 {
            if let (Ok(a), Ok(b), Ok(c)) = (
                cols[0].parse::<f64>(),
                cols[1].parse::<f64>(),
                cols[2].parse::<f64>(),
            ) {
                io.kb_per_transfer = a;
                io.transfers_per_sec = b;
                io.mb_per_sec = c;
                break;
            }
        }
    }
    io
}
