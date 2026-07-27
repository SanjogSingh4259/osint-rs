use crate::http;
use crate::shell;
use crate::spinner::spin;
use anyhow::Result;
use colored::*;
use serde_json::Value;
use std::net::IpAddr;

/// Resolves a target to an IP (if it's a hostname), then gathers:
/// reverse DNS, geolocation, ISP/org, and ASN — the "where does this
/// actually live" picture that plain whois/dns don't give you in one
/// place.
pub fn run(target: &str) -> Result<String> {
    let ip = if target.parse::<IpAddr>().is_ok() {
        target.to_string()
    } else {
        let out = spin(&format!("resolving {target}..."), || {
            shell::run("dig", &["+short", "A", target])
        })?;
        let first = out.lines().find(|l| !l.trim().is_empty());
        match first {
            Some(ip) => ip.trim().to_string(),
            None => anyhow::bail!("could not resolve {target} to an IPv4 address"),
        }
    };

    let rdns = spin("reverse DNS lookup...", || {
        shell::run("dig", &["+short", "-x", &ip])
    })
    .unwrap_or_default();
    let rdns = rdns.lines().next().unwrap_or("(none)").trim().to_string();

    let geo_body = spin("querying geolocation...", || {
        http::get(
            "ip-api.com",
            &format!("/json/{ip}?fields=status,message,country,regionName,city,isp,org,as,query"),
        )
    });

    let mut report = String::new();
    report.push_str(&format!("{}\n", format!("IP info for {target}").bold().cyan()));
    report.push_str(&format!("  {:<14} {ip}\n", "IP address:".dimmed()));
    report.push_str(&format!("  {:<14} {}\n", "Reverse DNS:".dimmed(), if rdns.is_empty() { "(none)" } else { &rdns }));

    match geo_body {
        Ok(body) => match serde_json::from_str::<Value>(&body) {
            Ok(json) if json.get("status").and_then(|s| s.as_str()) == Some("success") => {
                let field = |k: &str| json.get(k).and_then(|v| v.as_str()).unwrap_or("—").to_string();
                report.push_str(&format!("  {:<14} {}\n", "Country:".dimmed(), field("country")));
                report.push_str(&format!("  {:<14} {}\n", "Region/City:".dimmed(), format!("{}, {}", field("regionName"), field("city"))));
                report.push_str(&format!("  {:<14} {}\n", "ISP:".dimmed(), field("isp")));
                report.push_str(&format!("  {:<14} {}\n", "Org:".dimmed(), field("org")));
                report.push_str(&format!("  {:<14} {}\n", "ASN:".dimmed(), field("as")));
            }
            Ok(json) => {
                let msg = json.get("message").and_then(|v| v.as_str()).unwrap_or("lookup failed");
                report.push_str(&format!("  {} {msg}\n", "Geolocation:".dimmed()));
            }
            Err(_) => {
                report.push_str("  Geolocation: (could not parse response)\n");
            }
        },
        Err(e) => {
            report.push_str(&format!("  Geolocation: [skipped] {e}\n"));
        }
    }

    Ok(report)
}
