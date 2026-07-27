use crate::shell;
use anyhow::Result;
use colored::*;

/// Runs the system `whois` binary against a target and prints a
/// lightly cleaned-up summary (registrar, dates, name servers) plus
/// the raw record.
pub fn run(target: &str, raw: bool) -> Result<String> {
    let output = shell::run("whois", &[target])?;

    if raw {
        return Ok(output);
    }

    let mut summary = String::new();
    summary.push_str(&format!("{}\n", format!("WHOIS summary for {target}").bold().cyan()));

    let keys = [
        "Domain Name",
        "Registrar",
        "Creation Date",
        "Registry Expiry Date",
        "Updated Date",
        "Name Server",
        "Registrant Country",
        "Domain Status",
    ];

    for line in output.lines() {
        let lower = line.to_lowercase();
        if keys.iter().any(|k| lower.starts_with(&k.to_lowercase())) {
            summary.push_str(line.trim());
            summary.push('\n');
        }
    }

    if summary.lines().count() <= 1 {
        summary.push_str("(no structured fields matched — showing raw output)\n\n");
        summary.push_str(&output);
    }

    Ok(summary)
}
