use crate::types::*;
use crate::utils::*;

pub fn collect_wifi_info() -> WifiInfo {
    let mut info = WifiInfo::default();

    // Get current Wi-Fi info via networksetup
    let iface_text = cmd_output("networksetup", &["-listallhardwareports"]);
    let mut wifi_iface = String::new();
    let mut found_wifi = false;
    for line in iface_text.lines() {
        if line.contains("Wi-Fi") {
            found_wifi = true;
        }
        if found_wifi && line.starts_with("Device:") {
            wifi_iface = line[7..].trim().to_string();
            break;
        }
    }
    if wifi_iface.is_empty() {
        wifi_iface = "en0".into();
    }
    info.interface = wifi_iface.clone();

    let text = cmd_output("system_profiler", &["SPAirPortDataType"]);
    for line in text.lines() {
        let l = line.trim();
        if l.starts_with("Current Network Information:")
            || l.starts_with("SSID:")
            || l.contains("SSID:")
        {
            if l.starts_with("SSID:") {
                info.ssid = l[5..].trim().into();
            }
        }
        if l.starts_with("BSSID:") {
            info.bssid = l[6..].trim().into();
        }
        if l.starts_with("Channel:") {
            info.channel = l[8..].trim().into();
        }
        if l.starts_with("Signal / Noise:") {
            let sn = l[15..].trim();
            let parts: Vec<&str> = sn.split('/').collect();
            if parts.len() >= 2 {
                info.rssi = parts[0].trim().into();
                info.noise = parts[1].trim().into();
            }
        }
        if l.starts_with("Transmit Rate:")
            || l.starts_with("MCS Index:")
            || l.starts_with("PHY Rate:")
        {
            info.tx_rate = l.split(':').nth(1).unwrap_or("").trim().into();
        }
        if l.starts_with("Security:") {
            info.security_type = l[9..].trim().into();
        }
        if l.starts_with("PHY Mode:") {
            info.phy_mode = l[9..].trim().into();
        }
        if l.starts_with("Country Code:") {
            info.country_code = l[13..].trim().into();
        }
        // Get card type
        if l.starts_with("Card Type:") || l.starts_with("Supported Channels:") {
            if l.starts_with("Card Type:") {
                info.hardware = l[10..].trim().into();
            }
        }
    }

    // Fallback: get current SSID from networksetup
    if info.ssid.is_empty() {
        let ssid_text = cmd_output("networksetup", &["-getairportnetwork", &wifi_iface]);
        if let Some(ssid) = ssid_text.split(':').nth(1) {
            info.ssid = ssid.trim().into();
        }
    }

    info
}
