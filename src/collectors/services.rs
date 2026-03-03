use crate::types::*;
use crate::utils::*;

pub fn collect_services() -> Vec<ServiceInfo> {
    let text = cmd_output("launchctl", &["list"]);
    let mut services = Vec::new();
    for line in text.lines().skip(1) {
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() < 3 {
            continue;
        }
        let pid: i32 = cols[0].trim().parse().unwrap_or(-1);
        let last_exit: i32 = cols[1].trim().parse().unwrap_or(0);
        let label = cols[2].trim().to_string();
        services.push(ServiceInfo {
            pid,
            label,
            last_exit,
        });
    }
    services.sort_by(|a, b| {
        let a_running = a.pid > 0;
        let b_running = b.pid > 0;
        b_running.cmp(&a_running).then(a.label.cmp(&b.label))
    });
    services
}
