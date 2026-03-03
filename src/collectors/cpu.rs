use crate::types::*;
use crate::utils::*;
use sysinfo::System;

pub fn collect_cpu_cores(sys: &System) -> Vec<CpuCoreInfo> {
    sys.cpus()
        .iter()
        .map(|c| CpuCoreInfo {
            name: c.name().into(),
            usage: c.cpu_usage(),
            frequency: c.frequency(),
        })
        .collect()
}

pub fn collect_cpu_details() -> CpuDetailedInfo {
    let mut i = CpuDetailedInfo::default();
    let keys = [
        "machdep.cpu.brand_string",
        "machdep.cpu.core_count",
        "machdep.cpu.thread_count",
        "machdep.cpu.cores_per_package",
        "hw.cachelinesize",
        "hw.l1icachesize",
        "hw.l1dcachesize",
        "hw.l2cachesize",
        "hw.l3cachesize",
        "hw.nperflevels",
        "hw.perflevel0.physicalcpu",
        "hw.perflevel1.physicalcpu",
        "hw.perflevel0.l1icachesize",
        "hw.perflevel0.l1dcachesize",
        "hw.perflevel0.l2cachesize",
        "hw.perflevel1.l1icachesize",
        "hw.perflevel1.l1dcachesize",
        "hw.perflevel1.l2cachesize",
        "hw.pagesize",
        "hw.memsize",
    ];
    let text = cmd_output("sysctl", &keys);
    for line in text.lines() {
        let p: Vec<&str> = line.splitn(2, ':').collect();
        if p.len() != 2 {
            continue;
        }
        let (k, v) = (p[0].trim(), p[1].trim());
        match k {
            "machdep.cpu.brand_string" => i.brand = v.into(),
            "machdep.cpu.core_count" => i.core_count = v.parse().unwrap_or(0),
            "machdep.cpu.thread_count" => i.thread_count = v.parse().unwrap_or(0),
            "machdep.cpu.cores_per_package" => i.cores_per_package = v.parse().unwrap_or(0),
            "hw.cachelinesize" => i.cache_line_size = v.parse().unwrap_or(0),
            "hw.l1icachesize" => i.l1i_cache = v.parse().unwrap_or(0),
            "hw.l1dcachesize" => i.l1d_cache = v.parse().unwrap_or(0),
            "hw.l2cachesize" => i.l2_cache = v.parse().unwrap_or(0),
            "hw.l3cachesize" => i.l3_cache = v.parse().unwrap_or(0),
            "hw.nperflevels" => i.num_perf_levels = v.parse().unwrap_or(0),
            "hw.perflevel0.physicalcpu" => i.perf_cores = v.parse().unwrap_or(0),
            "hw.perflevel1.physicalcpu" => i.efficiency_cores = v.parse().unwrap_or(0),
            "hw.perflevel0.l1icachesize" => i.perf_l1i = v.parse().unwrap_or(0),
            "hw.perflevel0.l1dcachesize" => i.perf_l1d = v.parse().unwrap_or(0),
            "hw.perflevel0.l2cachesize" => i.perf_l2 = v.parse().unwrap_or(0),
            "hw.perflevel1.l1icachesize" => i.eff_l1i = v.parse().unwrap_or(0),
            "hw.perflevel1.l1dcachesize" => i.eff_l1d = v.parse().unwrap_or(0),
            "hw.perflevel1.l2cachesize" => i.eff_l2 = v.parse().unwrap_or(0),
            "hw.pagesize" => i.page_size = v.parse().unwrap_or(0),
            "hw.memsize" => i.phys_mem = v.parse().unwrap_or(0),
            _ => {}
        }
    }
    if i.brand.is_empty() {
        i.brand = "Unknown".into();
    }
    i.arch = System::cpu_arch();
    let feat_text = cmd_output("sysctl", &["hw.optional"]);
    for line in feat_text.lines() {
        let p: Vec<&str> = line.splitn(2, ':').collect();
        if p.len() == 2 && p[1].trim() == "1" {
            i.features
                .push(p[0].rsplit('.').next().unwrap_or("").into());
        }
    }
    i
}
