use crate::types::*;
use crate::utils::*;

pub fn collect_processes() -> Vec<ProcessInfo> {
    let text = cmd_output(
        "ps",
        &[
            "axo",
            "pid,ppid,user,%cpu,%mem,rss,vsz,state,nlwp,comm,lstart",
        ],
    );
    let mut procs = Vec::new();
    for line in text.lines().skip(1) {
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() < 10 {
            continue;
        }
        let pid: u32 = cols[0].parse().unwrap_or(0);
        let ppid: u32 = cols[1].parse().unwrap_or(0);
        let user = cols[2].to_string();
        let cpu: f64 = cols[3].parse().unwrap_or(0.0);
        let mem: f64 = cols[4].parse().unwrap_or(0.0);
        let rss_kb: u64 = cols[5].parse().unwrap_or(0);
        let vsz_kb: u64 = cols[6].parse().unwrap_or(0);
        let state = cols[7].to_string();
        let threads: u32 = cols[8].parse().unwrap_or(1);
        let command = cols[9].to_string();
        let started = if cols.len() > 10 {
            cols[10..].join(" ")
        } else {
            String::new()
        };
        procs.push(ProcessInfo {
            pid,
            ppid,
            user,
            cpu,
            mem,
            rss: hs(rss_kb * 1024),
            vsize: hs(vsz_kb * 1024),
            state,
            threads,
            command,
            started,
        });
    }
    procs.sort_by(|a, b| {
        b.cpu
            .partial_cmp(&a.cpu)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    procs
}
