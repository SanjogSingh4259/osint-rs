use anyhow::{bail, Result};
use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

/// Default timeout for quick lookups (whois, dig, curl). Longer-running
/// tools (nmap, amass, theHarvester, recon-ng) pass their own timeout
/// via `run_timeout`.
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(20);

/// Manual PATH search, avoiding the `which` crate (its newer versions
/// require a Rust edition this project's MSRV doesn't support).
fn on_path(bin: &str) -> bool {
    std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).any(|dir| dir.join(bin).is_file()))
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

/// Runs a system binary with args and returns captured stdout as a
/// String, using the default timeout. Used to shell out to whois /
/// dig / recon-ng / curl — the "simple bash cmds" this tool wraps
/// instead of reimplementing.
pub fn run(bin: &str, args: &[&str]) -> Result<String> {
    run_timeout(bin, args, DEFAULT_TIMEOUT)
}

/// Same as `run`, but with an explicit timeout. A firewalled or dead
/// target can otherwise make whois/nmap/etc. hang indefinitely with no
/// RST ever coming back — without this, one bad target would freeze
/// the whole CLI instead of failing visibly.
pub fn run_timeout(bin: &str, args: &[&str], timeout: Duration) -> Result<String> {
    if !on_path(bin) {
        let pkg = package_for(bin);
        bail!("'{bin}' is not installed or not on PATH. Install it with: sudo apt install {pkg}");
    }

    let mut child = Command::new(bin)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // Drain stdout/stderr on background threads so a chatty child
    // process can't deadlock on a full pipe buffer while we poll.
    let mut stdout_pipe = child.stdout.take().expect("stdout was piped");
    let mut stderr_pipe = child.stderr.take().expect("stderr was piped");

    let (tx_out, rx_out) = mpsc::channel();
    thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = stdout_pipe.read_to_end(&mut buf);
        let _ = tx_out.send(buf);
    });

    let (tx_err, rx_err) = mpsc::channel();
    thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = stderr_pipe.read_to_end(&mut buf);
        let _ = tx_err.send(buf);
    });

    let start = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if start.elapsed() > timeout {
            let _ = child.kill();
            let _ = child.wait();
            bail!(
                "'{bin}' timed out after {}s (target may be unreachable, or silently dropping/filtering the scan)",
                timeout.as_secs()
            );
        }
        thread::sleep(Duration::from_millis(100));
    };

    let stdout = rx_out.recv().unwrap_or_default();
    let stderr = rx_err.recv().unwrap_or_default();

    if !status.success() {
        bail!("'{bin}' exited with an error: {}", String::from_utf8_lossy(&stderr));
    }

    Ok(String::from_utf8_lossy(&stdout).to_string())
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

/// Fetches a URL via the system `curl` binary. Used for HTTPS OSINT
/// endpoints (crt.sh, web.archive.org) instead of hand-rolling TLS in
/// Rust — curl is on every Kali box already and handles redirects,
/// gzip, and certs correctly out of the box. curl's own --max-time is
/// kept as a fast first line of defense; run_timeout is the backstop.
pub fn curl(url: &str) -> Result<String> {
    run_timeout("curl", &["-s", "--max-time", "12", url], Duration::from_secs(18))
}
