use crate::types::*;
use crate::utils::*;

pub fn parse_netstat() -> Vec<NetInterface> {
    let text = cmd_output("netstat", &["-ib"]);
    let ifconfig = cmd_output("ifconfig", &[]);
    let mut ifaces = Vec::new();
    for line in text.lines().skip(1) {
        let c: Vec<&str> = line.split_whitespace().collect();
        if c.len() < 11 {
            continue;
        }
        let name = c[0].to_string();
        if name.ends_with('*') {
            continue;
        }
        let mtu: u32 = c[1].parse().unwrap_or(0);
        if mtu == 0 {
            continue;
        }
        if !c[2].contains("Link") {
            continue;
        }
        let pkts_in: u64 = c[4].parse().unwrap_or(0);
        let errs_in: u64 = c[5].parse().unwrap_or(0);
        let bytes_in: u64 = c[6].parse().unwrap_or(0);
        let pkts_out: u64 = c[7].parse().unwrap_or(0);
        let errs_out: u64 = c[8].parse().unwrap_or(0);
        let bytes_out: u64 = c[9].parse().unwrap_or(0);
        let mut ip = String::new();
        let mut status = String::new();
        let mut in_iface = false;
        for ifl in ifconfig.lines() {
            if ifl.starts_with(&name) && ifl.contains(": flags=") {
                in_iface = true;
            } else if !ifl.starts_with(' ') && !ifl.starts_with('\t') {
                in_iface = false;
            }
            if in_iface {
                if ifl.contains("inet ") && !ifl.contains("inet6") {
                    ip = ifl
                        .split("inet ")
                        .nth(1)
                        .unwrap_or("")
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .into();
                }
                if ifl.contains("status:") {
                    status = ifl.split("status:").nth(1).unwrap_or("").trim().into();
                }
            }
        }
        if status.is_empty() {
            status = "unknown".into();
        }
        ifaces.push(NetInterface {
            name,
            mtu,
            ip,
            pkts_in,
            bytes_in,
            errs_in,
            pkts_out,
            bytes_out,
            errs_out,
            status,
        });
    }
    ifaces
}

pub fn parse_lsof() -> Vec<NetConnection> {
    let out = std::process::Command::new("lsof")
        .args(["-i", "-P", "-n"])
        .output()
        .ok();
    let out = match out {
        Some(o) => String::from_utf8_lossy(&o.stdout).to_string(),
        None => return Vec::new(),
    };
    let mut conns: Vec<NetConnection> = Vec::new();
    for line in out.lines().skip(1) {
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() < 9 {
            continue;
        }
        let pid: u32 = cols[1].parse().unwrap_or(0);
        let process = cols[0].to_string();
        let fd = cols[3].to_string();
        let proto_type = cols[7].to_string();
        let name = cols[8..].join(" ");
        let (local, remote, state) = if name.contains("->") {
            let parts: Vec<&str> = name.splitn(2, "->").collect();
            let rem_state: Vec<&str> = parts.get(1).unwrap_or(&"").splitn(2, " ").collect();
            (
                parts[0].to_string(),
                rem_state[0].to_string(),
                rem_state
                    .get(1)
                    .unwrap_or(&"")
                    .trim_matches(|c| c == '(' || c == ')')
                    .to_string(),
            )
        } else {
            let parts: Vec<&str> = name.splitn(2, " ").collect();
            (
                parts[0].to_string(),
                String::new(),
                parts
                    .get(1)
                    .unwrap_or(&"")
                    .trim_matches(|c| c == '(' || c == ')')
                    .to_string(),
            )
        };
        conns.push(NetConnection {
            pid,
            process,
            fd,
            proto: proto_type,
            local_addr: local,
            remote_addr: remote,
            state,
        });
    }
    conns
}
