use crate::shell;
use crate::spinner::spin;
use anyhow::Result;
use colored::*;

/// Runs `subfinder -d <target> -silent` for passive subdomain
/// enumeration and returns a formatted list.
pub fn run(target: &str) -> Result<String> {
    let output = spin(&format!("enumerating subdomains for {target}..."), || {
        shell::run("subfinder", &["-d", target, "-silent"])
    })?;

    let mut report = String::new();
    report.push_str(&format!(
        "{}\n",
        format!("subfinder results for {target}").bold().cyan()
    ));

    let subs: Vec<&str> = output.lines().filter(|l| !l.trim().is_empty()).collect();
    if subs.is_empty() {
        report.push_str("  (no subdomains found)\n");
    } else {
        report.push_str(&format!("  {} subdomains found:\n", subs.len()));
        for s in subs {
            report.push_str(&format!("  {s}\n"));
        }
    }

    Ok(report)
}
