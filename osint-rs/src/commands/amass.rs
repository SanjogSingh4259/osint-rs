use crate::shell;
use crate::spinner::spin;
use anyhow::Result;
use colored::*;

/// Runs `amass enum -passive -d <target>`. Passive mode only queries
/// third-party sources (no direct probing of the target), matching the
/// OSINT-only scope of this tool.
pub fn run(target: &str) -> Result<String> {
    let output = spin(&format!("running amass (passive) on {target}..."), || {
        shell::run("amass", &["enum", "-passive", "-d", target])
    })?;

    let mut report = String::new();
    report.push_str(&format!(
        "{}\n",
        format!("amass (passive) results for {target}").bold().cyan()
    ));

    let lines: Vec<&str> = output.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.is_empty() {
        report.push_str("  (no assets found)\n");
    } else {
        report.push_str(&format!("  {} assets found:\n", lines.len()));
        for l in lines {
            report.push_str(&format!("  {l}\n"));
        }
    }

    Ok(report)
}
