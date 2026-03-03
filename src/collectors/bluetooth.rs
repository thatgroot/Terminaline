use crate::types::*;
use crate::utils::*;

pub fn collect_bluetooth() -> BluetoothInfo {
    let text = cmd_output("system_profiler", &["SPBluetoothDataType"]);
    let mut info = BluetoothInfo::default();
    let mut in_controller = true;
    let mut current_device: Option<BtDevice> = None;

    for line in text.lines() {
        let l = line.trim();
        if l.is_empty() {
            continue;
        }

        // Controller section
        if l.starts_with("Address:") {
            info.address = l[8..].trim().into();
        }
        if l.starts_with("Chipset:") {
            info.chipset = l[8..].trim().into();
        }
        if l.starts_with("Firmware Version:") {
            info.firmware = l[17..].trim().into();
        }
        if l.starts_with("Transport:") {
            info.transport = l[10..].trim().into();
        }
        if l.starts_with("Vendor ID:") {
            info.vendor = l[10..].trim().into();
        }
        if l.starts_with("Discoverable:") {
            info.discoverable = l[13..].trim().to_lowercase() == "yes";
        }
        if l.starts_with("State:") || l.starts_with("Bluetooth Power:") {
            info.state = l.split(':').nth(1).unwrap_or("").trim().into();
        }

        // Devices section
        if l.contains("Connected:") || l.contains("Paired:") || l.contains("Not Connected:") {
            in_controller = false;
        }

        if !in_controller {
            // A device starts when we see a name with a colon at high indent
            if l.ends_with(':')
                && !l.starts_with("Address")
                && !l.starts_with("Firmware")
                && !l.starts_with("Vendor")
                && !l.starts_with("Major")
                && !l.starts_with("Minor")
                && !l.starts_with("Connected")
                && !l.starts_with("Paired")
                && !l.starts_with("Not")
            {
                if let Some(dev) = current_device.take() {
                    info.devices.push(dev);
                }
                current_device = Some(BtDevice {
                    name: l.trim_end_matches(':').to_string(),
                    address: String::new(),
                    device_type: String::new(),
                    firmware: String::new(),
                    connected: false,
                });
            }
            if let Some(ref mut dev) = current_device {
                if l.starts_with("Address:") {
                    dev.address = l[8..].trim().into();
                }
                if l.starts_with("Minor Type:") {
                    dev.device_type = l[11..].trim().into();
                }
                if l.starts_with("Firmware Version:") {
                    dev.firmware = l[17..].trim().into();
                }
                if l.starts_with("Connected:") {
                    dev.connected = l[10..].trim().to_lowercase() == "yes";
                }
            }
        }
    }
    if let Some(dev) = current_device {
        info.devices.push(dev);
    }
    info
}
