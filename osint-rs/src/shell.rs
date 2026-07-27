use anyhow::{bail, Result};
use std::process::Command;

/// Runs a system binary with args and returns captured stdout as a String.
/// Used to shell out to whois / dig / recon-ng — the "simple bash cmds"
/// this tool wraps instead of reimplementing.
/// Manual PATH search, avoiding the `which` crate (its newer versions
/// require a Rust edition this project's MSRV doesn't support).
fn on_path(bin: &str) -> bool {
    std::env::var_os("PATH")
        .map(|paths| {
            std::env::split_paths(&paths).any(|dir| dir.join(bin).is_file())
        })
        .unwrap_or(false)
}

/// Maps a binary name to the apt package that provides it, for cases
/// where they differ (e.g. `dig` ships in the `dnsutils` package).
fn package_for(bin: &str) -> &str {
    match bin {
        "dig" => "dnsutils",
        "theHarvester" => "theharvester",
        other => other,
    }
}

pub fn run(bin: &str, args: &[&str]) -> Result<String> {
    if !on_path(bin) {
        let pkg = package_for(bin);
        bail!("'{bin}' is not installed or not on PATH. Install it with: sudo apt install {pkg}");
    }

    let output = Command::new(bin).args(args).output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        bail!("'{bin}' exited with an error: {stderr}");
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Same as run(), but does not fail the whole process if the binary is
/// missing — instead returns a friendly inline message. Reserved for a
/// future `full` scan variant that shouldn't abort on one missing tool.
#[allow(dead_code)]
pub fn run_soft(bin: &str, args: &[&str]) -> String {
    match run(bin, args) {
        Ok(out) => out,
        Err(e) => format!("[skipped] {e}"),
    }
}
