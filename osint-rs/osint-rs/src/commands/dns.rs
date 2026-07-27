use crate::shell;
use anyhow::Result;
use colored::*;

const RECORD_TYPES: [&str; 5] = ["A", "AAAA", "MX", "NS", "TXT"];

/// Runs `dig +short <TYPE> <target>` for a standard set of record
/// types and assembles a combined report.
pub fn run(target: &str) -> Result<String> {
    let mut report = String::new();
    report.push_str(&format!("{}\n", format!("DNS records for {target}").bold().cyan()));

    for rtype in RECORD_TYPES {
        let out = shell::run("dig", &["+short", rtype, target])
            .unwrap_or_else(|e| format!("(error: {e})"));
        let out = out.trim();
        report.push_str(&format!("{}\n", rtype.bold().yellow()));
        if out.is_empty() {
            report.push_str("  (none)\n");
        } else {
            for line in out.lines() {
                report.push_str(&format!("  {line}\n"));
            }
        }
    }

    Ok(report)
}
