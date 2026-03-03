use crate::types::*;
use crate::utils::*;

pub fn collect_security_info() -> SecurityInfo {
    let mut info = SecurityInfo::default();

    // SIP
    let sip = cmd_output("csrutil", &["status"]);
    info.sip_status = sip.trim().to_string();
    info.sip_enabled = sip.contains("enabled");

    // Gatekeeper
    let gk = cmd_output("spctl", &["--status"]);
    info.gatekeeper_status = gk.trim().to_string();
    info.gatekeeper_enabled = gk.contains("enabled") || gk.contains("assessments enabled");

    // FileVault
    let fv = cmd_output("fdesetup", &["status"]);
    info.filevault_status = fv.trim().to_string();
    info.filevault_enabled = fv.contains("On");

    // Firewall
    let fw = cmd_output(
        "defaults",
        &["read", "/Library/Preferences/com.apple.alf", "globalstate"],
    );
    let fw_val: i32 = fw.trim().parse().unwrap_or(0);
    info.firewall_enabled = fw_val > 0;
    info.firewall_status = match fw_val {
        0 => "Off".into(),
        1 => "On (specific services)".into(),
        2 => "On (essential services only)".into(),
        _ => format!("Unknown ({})", fw_val),
    };

    // Firewall stealth & block-all
    let stealth = cmd_output(
        "defaults",
        &[
            "read",
            "/Library/Preferences/com.apple.alf",
            "stealthenabled",
        ],
    );
    info.firewall_stealth = stealth.trim() == "1";
    let block = cmd_output(
        "defaults",
        &[
            "read",
            "/Library/Preferences/com.apple.alf",
            "allowsignedenabled",
        ],
    );
    info.firewall_block_all = block.trim() == "0";

    info
}
