use crate::types::*;
use crate::utils::*;

pub fn parse_vm_stat() -> VmStatData {
    let text = cmd_output("vm_stat", &[]);
    let mut d = VmStatData::default();
    for line in text.lines() {
        if line.starts_with("Mach Virtual Memory") {
            if let Some(p) = line.find("page size of ") {
                d.page_size = line[p + 13..]
                    .split_whitespace()
                    .next()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(16384);
            }
            continue;
        }
        let parts: Vec<&str> = line.splitn(2, ':').collect();
        if parts.len() != 2 {
            continue;
        }
        let v: u64 = parts[1].trim().trim_end_matches('.').parse().unwrap_or(0);
        match parts[0].trim() {
            "Pages free" => d.free = v,
            "Pages active" => d.active = v,
            "Pages inactive" => d.inactive = v,
            "Pages speculative" => d.speculative = v,
            "Pages wired down" => d.wired = v,
            "Pages purgeable" => d.purgeable = v,
            "Pages stored in compressor" => d.compressor = v,
            "Pageins" => d.pageins = v,
            "Pageouts" => d.pageouts = v,
            "\"Translation faults\"" => d.faults = v,
            "Pages copy-on-write" => d.copy_on_write = v,
            "Pages zero filled" => d.zero_fill = v,
            "Compressions" => d.compressions = v,
            "Decompressions" => d.decompressions = v,
            "Swapins" => d.swapins = v,
            "Swapouts" => d.swapouts = v,
            "Pages throttled" => d.throttled = v,
            "Pages reactivated" => d.reactivated = v,
            _ => {}
        }
    }
    if d.page_size == 0 {
        d.page_size = 16384;
    }
    d
}

pub fn parse_top() -> TopStats {
    let text = cmd_output(
        "top",
        &["-l", "1", "-n", "10", "-stats", "pid,command,rsize,cpu"],
    );
    let mut s = TopStats::default();
    for line in text.lines() {
        let l = line.trim();
        if l.starts_with("Processes:") {
            for part in l.split(',') {
                let p = part.trim();
                if p.contains("total") {
                    s.processes = p
                        .split_whitespace()
                        .next()
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(0);
                }
                if p.contains("running") {
                    s.running = p
                        .split_whitespace()
                        .next()
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(0);
                }
                if p.contains("sleeping") {
                    s.sleeping = p
                        .split_whitespace()
                        .next()
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(0);
                }
                if p.contains("threads") {
                    s.threads = p
                        .split_whitespace()
                        .next()
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(0);
                }
            }
        } else if l.starts_with("CPU usage:") {
            for part in l[10..].split(',') {
                let p = part.trim();
                if p.contains("user") {
                    s.cpu_user = p
                        .replace('%', "")
                        .split_whitespace()
                        .next()
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(0.0);
                }
                if p.contains("sys") {
                    s.cpu_sys = p
                        .replace('%', "")
                        .split_whitespace()
                        .next()
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(0.0);
                }
                if p.contains("idle") {
                    s.cpu_idle = p
                        .replace('%', "")
                        .split_whitespace()
                        .next()
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(0.0);
                }
            }
        } else if l.starts_with("PhysMem:") {
            let inner = &l[8..];
            for part in inner.split(',') {
                let p = part.trim();
                if p.contains("used") {
                    s.phys_used = p.split_whitespace().next().unwrap_or("").into();
                }
                if p.contains("wired") {
                    s.phys_wired = p
                        .trim_start_matches('(')
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .into();
                }
                if p.contains("compressor") {
                    s.phys_compressor = p.split_whitespace().next().unwrap_or("").into();
                }
                if p.contains("unused") {
                    s.phys_unused = p.split_whitespace().next().unwrap_or("").into();
                }
            }
        } else if l.starts_with("MemRegions:") {
            for part in l[11..].split(',') {
                let p = part.trim();
                if p.contains("total") {
                    s.mem_regions_total = p
                        .split_whitespace()
                        .next()
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(0);
                }
                if p.contains("resident") {
                    s.mem_regions_resident = p.split_whitespace().next().unwrap_or("").into();
                }
                if p.contains("private") {
                    s.mem_regions_private = p.split_whitespace().next().unwrap_or("").into();
                }
                if p.contains("shared") {
                    s.mem_regions_shared = p.split_whitespace().next().unwrap_or("").into();
                }
            }
        } else if l.starts_with("SharedLibs:") {
            for part in l[11..].split(',') {
                let p = part.trim();
                if p.contains("resident") {
                    s.sharedlibs_resident = p.split_whitespace().next().unwrap_or("").into();
                }
                if p.contains("data") {
                    s.sharedlibs_data = p.split_whitespace().next().unwrap_or("").into();
                }
            }
        } else if l.starts_with("Networks:") {
            for part in l[9..].split(',') {
                let p = part.trim();
                if p.contains("in") {
                    let w: Vec<&str> = p.split('/').collect();
                    if w.len() >= 2 {
                        s.net_packets_in = w[0].split_whitespace().last().unwrap_or("").into();
                        s.net_bytes_in = w[1].split_whitespace().next().unwrap_or("").into();
                    }
                }
                if p.contains("out") {
                    let w: Vec<&str> = p.split('/').collect();
                    if w.len() >= 2 {
                        s.net_packets_out = w[0].split_whitespace().last().unwrap_or("").into();
                        s.net_bytes_out = w[1].split_whitespace().next().unwrap_or("").into();
                    }
                }
            }
        } else if l.starts_with("Disks:") {
            for part in l[6..].split(',') {
                let p = part.trim();
                if p.contains("read") {
                    let w: Vec<&str> = p.split('/').collect();
                    if w.len() >= 2 {
                        s.disk_reads = w[0].split_whitespace().last().unwrap_or("").into();
                        s.disk_read_bytes = w[1].split_whitespace().next().unwrap_or("").into();
                    }
                }
                if p.contains("written") {
                    let w: Vec<&str> = p.split('/').collect();
                    if w.len() >= 2 {
                        s.disk_writes = w[0].split_whitespace().last().unwrap_or("").into();
                        s.disk_write_bytes = w[1].split_whitespace().next().unwrap_or("").into();
                    }
                }
            }
        } else if !l.is_empty()
            && l.chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
        {
            let cols: Vec<&str> = l.split_whitespace().collect();
            if cols.len() >= 4 {
                s.top_procs
                    .push((cols[1].into(), cols[2].into(), cols[3].into()));
            }
        }
    }
    s
}

pub fn parse_swap() -> SwapInfo {
    let text = sysctl_val("vm.swapusage");
    let mut s = SwapInfo::default();
    for part in text.split("  ") {
        let p = part.trim();
        if p.starts_with("total") {
            s.total = p.split('=').nth(1).unwrap_or("").trim().into();
        }
        if p.starts_with("used") {
            s.used = p.split('=').nth(1).unwrap_or("").trim().into();
        }
        if p.starts_with("free") {
            s.free = p.split('=').nth(1).unwrap_or("").trim().into();
        }
    }
    s.encrypted = text.contains("encrypted");
    s
}
