use crate::types::*;
use crate::utils::*;

pub fn collect_thermal_info() -> ThermalInfo {
    let mut info = ThermalInfo::default();

    // Read thermal sensors from ioreg
    // Also check for thermal-related entries
    let text2 = cmd_output("sysctl", &["-a"]);

    // Parse thermal pressure from sysctl
    for line in text2.lines() {
        if line.contains("kern.memorystatus_vm_pressure_level") {
            let level: i32 = line
                .split(':')
                .nth(1)
                .unwrap_or("0")
                .trim()
                .parse()
                .unwrap_or(0);
            info.thermal_pressure = match level {
                0 => "Normal".into(),
                1 => "Warning".into(),
                2 => "Urgent".into(),
                4 => "Critical".into(),
                _ => format!("Level {}", level),
            };
        }
    }

    // Parse temperature data from ioreg (AppleSmartBattery has Temperature)
    let bat_text = cmd_output("ioreg", &["-r", "-c", "AppleSmartBattery", "-d", "1"]);
    for line in bat_text.lines() {
        let l = line.trim().trim_start_matches('"').replace("\" = ", "=");
        if l.contains("Temperature=") {
            let raw: f64 = l
                .split('=')
                .next_back()
                .unwrap_or("0")
                .trim()
                .parse()
                .unwrap_or(0.0);
            info.entries.push(ThermalEntry {
                name: "Battery".into(),
                temperature: raw / 100.0,
                category: "Battery".into(),
            });
        }
    }

    // Try to get CPU die temp from powermetrics (needs sudo, fallback gracefully)
    // Use sysctl for thermal levels instead
    let thermal_text = cmd_output(
        "sysctl",
        &[
            "machdep.xcpm.cpu_thermal_level",
            "machdep.xcpm.gpu_thermal_level",
            "machdep.xcpm.io_thermal_level",
        ],
    );
    for line in thermal_text.lines() {
        let parts: Vec<&str> = line.splitn(2, ':').collect();
        if parts.len() != 2 {
            continue;
        }
        let key = parts[0].trim();
        let val: f64 = parts[1].trim().parse().unwrap_or(0.0);
        let (name, cat) = if key.contains("cpu_thermal") {
            ("CPU Thermal Level", "CPU")
        } else if key.contains("gpu_thermal") {
            ("GPU Thermal Level", "GPU")
        } else {
            ("I/O Thermal Level", "I/O")
        };
        info.entries.push(ThermalEntry {
            name: name.into(),
            temperature: val,
            category: cat.into(),
        });
    }

    if info.thermal_pressure.is_empty() {
        info.thermal_pressure = "Normal".into();
    }
    info
}
