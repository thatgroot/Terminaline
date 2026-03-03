use crate::types::*;
use crate::utils::*;

pub fn collect_audio_devices() -> Vec<AudioDevice> {
    let text = cmd_output("system_profiler", &["SPAudioDataType"]);
    let mut devices = Vec::new();
    let mut current: Option<AudioDevice> = None;
    let mut is_input_section = false;

    for line in text.lines() {
        let l = line.trim();
        if l.is_empty() {
            continue;
        }

        if l.contains("Input:") || l.contains("Input Source:") {
            is_input_section = true;
        }
        if l.contains("Output:") || l.contains("Output Source:") {
            is_input_section = false;
        }

        if l.ends_with(':')
            && !l.starts_with("Manufacturer")
            && !l.starts_with("Sample Rate")
            && !l.starts_with("Source")
            && !l.starts_with("Transport")
            && !l.starts_with("Input")
            && !l.starts_with("Output")
            && !l.starts_with("Channel")
            && !l.starts_with("Default")
        {
            if let Some(dev) = current.take() {
                devices.push(dev);
            }
            current = Some(AudioDevice {
                name: l.trim_end_matches(':').to_string(),
                manufacturer: String::new(),
                sample_rate: 0,
                channels: 0,
                transport: String::new(),
                is_input: is_input_section,
                is_default: false,
            });
        }
        if let Some(ref mut dev) = current {
            if l.starts_with("Manufacturer:") {
                dev.manufacturer = l[13..].trim().into();
            }
            if l.starts_with("Current SampleRate:") || l.starts_with("Sample Rate:") {
                dev.sample_rate = l
                    .split(':')
                    .nth(1)
                    .unwrap_or("0")
                    .trim()
                    .split_whitespace()
                    .next()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
            }
            if l.starts_with("Transport:") {
                dev.transport = l[10..].trim().into();
            }
            if l.starts_with("Default ") && l.contains("Device: Yes") {
                dev.is_default = true;
            }
        }
    }
    if let Some(dev) = current {
        devices.push(dev);
    }
    devices
}
