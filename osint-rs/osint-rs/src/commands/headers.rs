use crate::shell;
use crate::spinner::spin;
use anyhow::{bail, Result};
use colored::*;

/// Notable headers worth calling out — these leak tech-stack info
/// (server software/version, frameworks, CDN/WAF presence) that's
/// useful for fingerprinting without running a heavier tool.
const NOTABLE: [&str; 9] = [
    "server",
    "x-powered-by",
    "x-aspnet-version",
    "x-generator",
    "via",
    "x-cache",
    "cf-ray",
    "x-drupal-cache",
    "set-cookie",
];

/// Fetches HTTP response headers via curl (following redirects, GET
/// rather than HEAD since some servers reject HEAD requests) and
/// highlights the ones that reveal server software or infrastructure.
/// This is the one "touches the target" lookup besides `ports` — it's
/// a single lightweight request, not a scan.
pub fn run(target: &str) -> Result<String> {
    let https_url = format!("https://{target}");
    let http_url = format!("http://{target}");

    let mut used_url = https_url.clone();
    let mut output = spin(&format!("fetching headers for {target} (https)..."), || {
        shell::run(
            "curl",
            &["-s", "-D", "-", "-o", "/dev/null", "-L", "--max-time", "10", &https_url],
        )
    });

    if output.is_err() {
        used_url = http_url.clone();
        output = spin(&format!("fetching headers for {target} (http fallback)..."), || {
            shell::run(
                "curl",
                &["-s", "-D", "-", "-o", "/dev/null", "-L", "--max-time", "10", &http_url],
            )
        });
    }

    let output = match output {
        Ok(o) if !o.trim().is_empty() => o,
        _ => bail!("no response from {target} over HTTPS or HTTP"),
    };

    let mut report = String::new();
    report.push_str(&format!(
        "{}\n",
        format!("HTTP headers for {used_url}").bold().cyan()
    ));

    // curl -L prints headers for every hop; keep only the final block.
    let final_block = output
        .split("\r\n\r\n")
        .filter(|b| !b.trim().is_empty())
        .last()
        .unwrap_or(&output);

    let mut notable_found = 0;
    for line in final_block.lines() {
        let lower = line.to_lowercase();
        let is_notable = NOTABLE.iter().any(|n| lower.starts_with(n));
        if is_notable {
            notable_found += 1;
            report.push_str(&format!("  {}\n", line.yellow()));
        } else {
            report.push_str(&format!("  {line}\n"));
        }
    }

    if notable_found == 0 {
        report.push_str("  (no notable fingerprinting headers exposed)\n");
    }

    Ok(report)
}
