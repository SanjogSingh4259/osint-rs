use crate::shell;
use crate::spinner::spin;
use anyhow::Result;
use colored::*;
use serde_json::Value;
use std::collections::BTreeSet;

/// Queries crt.sh's certificate-transparency-log search for every
/// name that has ever had a public SSL cert issued for it under the
/// target domain. This is one of the highest-signal passive subdomain
/// discovery techniques — CT logs are public and mandatory for
/// browser-trusted certs, so it catches subdomains DNS brute-forcing
/// would miss.
pub fn run(target: &str) -> Result<String> {
    let url = format!("https://crt.sh/?q=%25.{target}&output=json");
    let body = spin(&format!("querying certificate transparency logs for {target}..."), || {
        shell::curl(&url)
    })?;

    let mut report = String::new();
    report.push_str(&format!(
        "{}\n",
        format!("Certificate transparency results for {target}").bold().cyan()
    ));

    match serde_json::from_str::<Value>(&body) {
        Ok(Value::Array(entries)) => {
            let mut names: BTreeSet<String> = BTreeSet::new();
            for entry in &entries {
                if let Some(name_value) = entry.get("name_value").and_then(|v| v.as_str()) {
                    for line in name_value.lines() {
                        let n = line.trim().trim_start_matches("*.").to_lowercase();
                        if !n.is_empty() {
                            names.insert(n);
                        }
                    }
                }
            }
            if names.is_empty() {
                report.push_str("  0 names found\n");
            } else {
                report.push_str(&format!("  {} unique names found:\n", names.len()));
                for n in &names {
                    report.push_str(&format!("  {n}\n"));
                }
            }
        }
        _ => {
            report.push_str(
                "  (crt.sh returned no parseable data — it may be rate-limiting; try again shortly)\n",
            );
        }
    }

    Ok(report)
}
