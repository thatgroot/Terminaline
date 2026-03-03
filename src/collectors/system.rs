use crate::types::*;
use crate::utils::*;

pub fn parse_sysinfo() -> SysInfo {
    let boot_raw = sysctl_val("kern.boottime");
    let boot_time = boot_raw.split('}').last().unwrap_or("").trim().to_string();
    let sec_str = boot_raw
        .split("sec = ")
        .nth(1)
        .unwrap_or("0")
        .split(',')
        .next()
        .unwrap_or("0");
    let boot_sec: u64 = sec_str.trim().parse().unwrap_or(0);
    let now_sec = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let up = now_sec.saturating_sub(boot_sec);
    let days = up / 86400;
    let hrs = (up % 86400) / 3600;
    let mins = (up % 3600) / 60;
    let uptime = if days > 0 {
        format!("{}d {}h {}m", days, hrs, mins)
    } else {
        format!("{}h {}m", hrs, mins)
    };
    SysInfo {
        hostname: sysctl_val("kern.hostname"),
        os_type: sysctl_val("kern.ostype"),
        os_release: sysctl_val("kern.osrelease"),
        os_build: sysctl_val("kern.osversion"),
        boot_time,
        uptime,
        hw_model: sysctl_val("hw.targettype"),
    }
}

pub fn parse_gpu() -> GpuInfo {
    let text = cmd_output("system_profiler", &["SPDisplaysDataType"]);
    let mut g = GpuInfo::default();
    for line in text.lines() {
        let l = line.trim();
        if l.starts_with("Chipset Model:") {
            g.chipset = l[14..].trim().into();
        }
        if l.starts_with("Type:") {
            g.gpu_type = l[5..].trim().into();
        }
        if l.starts_with("Bus:") {
            g.bus = l[4..].trim().into();
        }
        if l.starts_with("Total Number of Cores:") {
            g.cores = l[22..].trim().into();
        }
        if l.starts_with("Vendor:") {
            g.vendor = l[7..].trim().into();
        }
        if l.starts_with("Metal Support:") {
            g.metal = l[14..].trim().into();
        }
        if l.starts_with("Display Type:") {
            g.display_type = l[13..].trim().into();
        }
        if l.starts_with("Resolution:") {
            g.resolution = l[11..].trim().into();
        }
        if l.starts_with("Color LCD:") || l.starts_with("LG") || l.contains("Display:") {
            g.display_name = l.trim_end_matches(':').into();
        }
    }
    g
}

pub fn parse_battery() -> BatteryInfo {
    let text = cmd_output("pmset", &["-g", "batt"]);
    let mut b = BatteryInfo::default();
    for line in text.lines() {
        if line.contains("InternalBattery") {
            b.present = true;
            if let Some(pct) = line.split('\t').nth(1) {
                b.level = pct.split(';').next().unwrap_or("").trim().into();
                let parts: Vec<&str> = pct.split(';').collect();
                if parts.len() > 1 {
                    b.state = parts[1].trim().into();
                }
                if parts.len() > 2 {
                    b.remaining = parts[2].trim().into();
                }
            }
        }
    }
    let ioreg = cmd_output("ioreg", &["-r", "-c", "AppleSmartBattery", "-d", "1"]);
    for line in ioreg.lines() {
        let l = line.trim().trim_start_matches('"').replace("\" = ", "=");
        if l.contains("CycleCount=") && !l.contains("Designed") {
            b.cycle_count = l.split('=').last().unwrap_or("").trim().into();
        }
        if l.contains("MaxCapacity=") && !l.contains("Design") {
            b.max_capacity = l.split('=').last().unwrap_or("").trim().into();
        }
        if l.contains("DesignCapacity=") {
            b.design_capacity = l.split('=').last().unwrap_or("").trim().into();
        }
        if l.contains("Temperature=") {
            b.temperature = l.split('=').last().unwrap_or("").trim().into();
        }
        if l.contains("Voltage=") {
            b.voltage = l.split('=').last().unwrap_or("").trim().into();
        }
        if l.contains("InstantAmperage=") {
            b.amperage = l.split('=').last().unwrap_or("").trim().into();
        }
        if l.contains("BatteryHealth=") {
            b.condition = l.split('=').last().unwrap_or("").trim().into();
        }
    }
    b
}

pub fn collect_cameras() -> Vec<CameraInfo> {
    let text = cmd_output("system_profiler", &["SPCameraDataType", "-json"]);
    let mut cams = Vec::new();
    let (mut name, mut model, mut uid) = (String::new(), String::new(), String::new());
    for line in text.lines() {
        let t = line.trim();
        if let Some(v) = extract_json(t, "\"_name\"") {
            if !name.is_empty() {
                cams.push(CameraInfo {
                    name: name.clone(),
                    model_id: model.clone(),
                    unique_id: uid.clone(),
                });
            }
            name = v;
            model.clear();
            uid.clear();
        } else if let Some(v) = extract_json(t, "\"spcamera_model-id\"") {
            model = v;
        } else if let Some(v) = extract_json(t, "\"spcamera_unique-id\"") {
            uid = v;
        }
    }
    if !name.is_empty() {
        cams.push(CameraInfo {
            name,
            model_id: model,
            unique_id: uid,
        });
    }
    cams
}
