use crate::shell;
use crate::spinner::spin;
use anyhow::Result;
use colored::*;
use serde_json::Value;

/// Queries the Wayback Machine's CDX API for URLs it has archived
/// under the target domain — useful for finding old endpoints,
/// forgotten admin paths, or parameters that were never meant to stay
/// public. `limit` caps how many rows come back (the API can return
/// tens of thousands for popular domains).
pub fn run(target: &str, limit: u32) -> Result<String> {
    let url = format!(
        "https://web.archive.org/cdx/search/cdx?url={target}/*&output=json&fl=original&collapse=urlkey&limit={limit}"
    );
    let body = spin(&format!("querying Wayback Machine for {target}..."), || {
        shell::curl(&url)
    })?;

    let mut report = String::new();
    report.push_str(&format!(
        "{}\n",
        format!("Wayback Machine URLs for {target} (limit {limit})").bold().cyan()
    ));

    match serde_json::from_str::<Value>(&body) {
        Ok(Value::Array(rows)) if rows.len() > 1 => {
            let urls: Vec<&str> = rows[1..]
                .iter()
                .filter_map(|row| row.get(0).and_then(|v| v.as_str()))
                .collect();
            report.push_str(&format!("  {} archived URLs found:\n", urls.len()));
            for u in &urls {
                report.push_str(&format!("  {u}\n"));
            }
        }
        _ => {
            report.push_str("  0 archived URLs found\n");
        }
    }

    Ok(report)
}
