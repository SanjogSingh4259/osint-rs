use crate::shell;
use crate::spinner::spin;
use anyhow::Result;
use colored::*;
use std::time::Duration;

/// Sources that work without an API key, used as the default when the
/// user doesn't pass --sources explicitly.
const DEFAULT_SOURCES: &str = "bing,duckduckgo,crtsh,otx,hackertarget,rapiddns";

/// Runs `theHarvester -d <target> -b <sources>` and returns its
/// stdout output (emails, hosts, IPs found across the given sources).
pub fn run(target: &str, sources: Option<&str>) -> Result<String> {
    let sources = sources.unwrap_or(DEFAULT_SOURCES);
    let output = spin(&format!("harvesting OSINT data for {target}..."), || {
        shell::run_timeout("theHarvester", &["-d", target, "-b", sources], Duration::from_secs(90))
    })?;

    let mut report = String::new();
    report.push_str(&format!(
        "{}\n",
        format!("theHarvester results for {target} (sources: {sources})")
            .bold()
            .cyan()
    ));
    report.push_str(&output);
    Ok(report)
}
