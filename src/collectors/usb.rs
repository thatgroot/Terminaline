use crate::types::*;
use crate::utils::*;

pub fn collect_usb_devices() -> Vec<UsbDevice> {
    let text = cmd_output("system_profiler", &["SPUSBDataType"]);
    let mut devices = Vec::new();
    let mut current: Option<UsbDevice> = None;

    for line in text.lines() {
        let l = line.trim();
        if l.is_empty() {
            continue;
        }

        if l.ends_with(':')
            && !l.starts_with("Product ID")
            && !l.starts_with("Vendor ID")
            && !l.starts_with("Speed")
            && !l.starts_with("Location")
            && !l.starts_with("Serial")
            && !l.starts_with("Bus Power")
            && !l.starts_with("USB")
        {
            if let Some(dev) = current.take() {
                if !dev.name.is_empty() {
                    devices.push(dev);
                }
            }
            current = Some(UsbDevice {
                name: l.trim_end_matches(':').to_string(),
                vendor: String::new(),
                product_id: String::new(),
                vendor_id: String::new(),
                speed: String::new(),
                bus_power: String::new(),
                serial: String::new(),
                location: String::new(),
            });
        }
        if let Some(ref mut dev) = current {
            if l.starts_with("Product ID:") {
                dev.product_id = l[11..].trim().into();
            }
            if l.starts_with("Vendor ID:") {
                dev.vendor_id = l[10..].trim().into();
            } else if l.starts_with("Vendor:") {
                dev.vendor = l[7..].trim().into();
            }
            if l.starts_with("Speed:") {
                dev.speed = l[6..].trim().into();
            }
            if l.starts_with("Bus Power") {
                dev.bus_power = l.split(':').nth(1).unwrap_or("").trim().into();
            }
            if l.starts_with("Serial Number:") {
                dev.serial = l[14..].trim().into();
            }
            if l.starts_with("Location ID:") {
                dev.location = l[12..].trim().into();
            }
        }
    }
    if let Some(dev) = current {
        if !dev.name.is_empty() {
            devices.push(dev);
        }
    }
    devices
}

pub fn collect_thunderbolt() -> Vec<ThunderboltInfo> {
    let text = cmd_output("system_profiler", &["SPThunderboltDataType"]);
    let mut devices = Vec::new();
    let (mut name, mut speed, mut uuid, mut link) =
        (String::new(), String::new(), String::new(), String::new());

    for line in text.lines() {
        let l = line.trim();
        if l.is_empty() {
            continue;
        }
        if l.ends_with(':')
            && !l.starts_with("Speed")
            && !l.starts_with("UUID")
            && !l.starts_with("Link")
        {
            if !name.is_empty() {
                devices.push(ThunderboltInfo {
                    device_name: name.clone(),
                    speed: speed.clone(),
                    uuid: uuid.clone(),
                    link_status: link.clone(),
                });
            }
            name = l.trim_end_matches(':').to_string();
            speed.clear();
            uuid.clear();
            link.clear();
        }
        if l.starts_with("Speed:") {
            speed = l[6..].trim().into();
        }
        if l.starts_with("UUID:") {
            uuid = l[5..].trim().into();
        }
        if l.starts_with("Link Status:") {
            link = l[12..].trim().into();
        }
    }
    if !name.is_empty() {
        devices.push(ThunderboltInfo {
            device_name: name,
            speed,
            uuid,
            link_status: link,
        });
    }
    devices
}
