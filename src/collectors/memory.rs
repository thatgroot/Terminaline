use crate::types::*;
use proc_maps::{get_process_maps, MapRange};

pub fn classify_region(map: &MapRange) -> (RegionType, String) {
    let name = map
        .filename()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    if name.contains("[stack]") || name.to_lowercase().contains("stack") {
        return (RegionType::Stack, name);
    }
    if name.contains("[heap]") || name.to_lowercase().contains("heap") {
        return (RegionType::Heap, name);
    }
    if !name.is_empty() {
        if name.contains(".dylib") {
            return (RegionType::Dylib, name);
        }
        if name.ends_with(".so") || name.contains("__TEXT") {
            return (RegionType::Code, name);
        }
        return (RegionType::MappedFile, name);
    }
    if map.is_exec() {
        return (RegionType::Code, "[anon exec]".into());
    }
    if map.is_write() && map.size() > 1024 * 1024 {
        return (RegionType::Heap, "[anon heap]".into());
    }
    (RegionType::Anonymous, "[anon]".into())
}

pub fn collect_process_maps(pid: u32) -> Vec<ProcessRegion> {
    get_process_maps(pid as proc_maps::Pid)
        .unwrap_or_default()
        .iter()
        .map(|m| {
            let (rt, name) = classify_region(m);
            ProcessRegion {
                start: m.start(),
                end: m.start() + m.size(),
                size: m.size(),
                perms: format!(
                    "{}{}{}",
                    if m.is_read() { "R" } else { "-" },
                    if m.is_write() { "W" } else { "-" },
                    if m.is_exec() { "X" } else { "-" }
                ),
                region_type: rt,
                name,
            }
        })
        .collect()
}
