use crate::types::*;
use std::fs;
use std::process::Command;

/// Run a command and return stdout as a String.
pub fn cmd_output(cmd: &str, args: &[&str]) -> String {
    Command::new(cmd)
        .args(args)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default()
}

/// Read a single sysctl key.
pub fn sysctl_val(key: &str) -> String {
    let out = cmd_output("sysctl", &["-n", key]);
    out.trim().to_string()
}

/// Human-readable size from bytes (u64).
pub fn hs(bytes: u64) -> String {
    if bytes >= 1 << 30 {
        format!("{:.2} GB", bytes as f64 / (1 << 30) as f64)
    } else if bytes >= 1 << 20 {
        format!("{:.1} MB", bytes as f64 / (1 << 20) as f64)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

/// Human-readable size from bytes (usize).
pub fn hsu(b: usize) -> String {
    hs(b as u64)
}

/// Truncate a string, prepending '…' if too long.
pub fn trunc(s: &str, m: usize) -> String {
    if s.len() <= m {
        s.into()
    } else {
        format!("…{}", &s[s.len() - m + 1..])
    }
}

/// Extract a JSON value from a raw JSON line by key.
pub fn extract_json(line: &str, key: &str) -> Option<String> {
    if !line.contains(key) {
        return None;
    }
    let v = line.split(':').skip(1).collect::<Vec<&str>>().join(":");
    let t = v
        .trim()
        .trim_matches('"')
        .trim_end_matches(',')
        .trim_matches('"');
    if t.is_empty() {
        None
    } else {
        Some(t.into())
    }
}

/// Check if a path is a macOS system path.
pub fn is_system_path(p: &str) -> bool {
    let sys = [
        "/System",
        "/Library",
        "/usr",
        "/bin",
        "/sbin",
        "/var",
        "/private",
        "/etc",
        "/dev",
        "/tmp",
        "/cores",
        "/.fseventsd",
        "/.Spotlight",
    ];
    sys.iter().any(|s| p.starts_with(s)) || p.starts_with("/.")
}

/// Scan a directory listing with sorting and optional system path filtering.
pub fn scan_directory(path: &str, sort: SortMode, filter_system: bool) -> Vec<FileEntry> {
    let Ok(entries) = fs::read_dir(path) else {
        return Vec::new();
    };
    let mut files: Vec<FileEntry> = entries
        .filter_map(|e| {
            let e = e.ok()?;
            let meta = e.metadata().ok()?;
            let name = e.file_name().to_string_lossy().to_string();
            let full = e.path().to_string_lossy().to_string();
            let is_dir = meta.is_dir();
            let size = if is_dir {
                dir_size_shallow(&full)
            } else {
                meta.len()
            };
            let is_system = is_system_path(&full);
            Some(FileEntry {
                name,
                path: full,
                size,
                is_dir,
                is_system,
            })
        })
        .collect();
    if filter_system {
        files.retain(|f| !f.is_system);
    }
    match sort {
        SortMode::SizeDsc => files.sort_by(|a, b| b.size.cmp(&a.size)),
        SortMode::SizeAsc => files.sort_by(|a, b| a.size.cmp(&b.size)),
        SortMode::NameAsc => {
            files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        }
        SortMode::NameDsc => {
            files.sort_by(|a, b| b.name.to_lowercase().cmp(&a.name.to_lowercase()))
        }
    }
    files
}

/// Calculate total file sizes in a directory (shallow — not recursive).
pub fn dir_size_shallow(path: &str) -> u64 {
    fs::read_dir(path)
        .ok()
        .map(|entries| {
            entries
                .filter_map(|e| e.ok()?.metadata().ok().map(|m| m.len()))
                .sum()
        })
        .unwrap_or(0)
}
