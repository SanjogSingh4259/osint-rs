use crate::shell;
use crate::spinner::spin;
use anyhow::Result;
use colored::*;
use std::time::Duration;

/// Runs `nmap -F -T4 <target>` (fast scan of the ~100 most common
/// ports). This is the "what's actually running/listening" piece —
/// the closest thing to "servers" in the live-recon sense, distinct
/// from the purely passive whois/dns/recon-ng data.
pub fn run(target: &str, top_ports: bool) -> Result<String> {
    let args: Vec<&str> = if top_ports {
        vec!["-F", "-T4", target]
    } else {
        vec!["-T4", target]
    };
    let timeout = if top_ports { Duration::from_secs(90) } else { Duration::from_secs(240) };

    let output = spin(&format!("scanning {target}..."), || {
        shell::run_timeout("nmap", &args, timeout)
    })?;

    let mut report = String::new();
    report.push_str(&format!("{}\n", format!("Open ports on {target}").bold().cyan()));

    let mut in_table = false;
    let mut found_any = false;
    for line in output.lines() {
        if line.starts_with("PORT") {
            in_table = true;
            report.push_str(&format!("  {}\n", line.bold()));
            continue;
        }
        if in_table {
            if line.trim().is_empty() {
                in_table = false;
                continue;
            }
            found_any = true;
            let colored_line = if line.contains("open") {
                line.green().to_string()
            } else {
                line.normal().to_string()
            };
            report.push_str(&format!("  {colored_line}\n"));
        }
    }

    if !found_any {
        report.push_str("  (no open ports found in scanned range)\n");
    }

    Ok(report)
}
