use anyhow::{bail, Result};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

/// Does a bare-bones HTTP/1.1 GET over plain TCP (port 80) and returns
/// the response body. Deliberately hand-rolled instead of pulling in
/// a full HTTP client crate: we only ever call one endpoint
/// (ip-api.com's free, TLS-free JSON API), so a ~30-line client keeps
/// the dependency tree small and the behavior easy to audit.
pub fn get(host: &str, path: &str) -> Result<String> {
    let addr = format!("{host}:80");
    let mut stream = TcpStream::connect(&addr)?;
    stream.set_read_timeout(Some(Duration::from_secs(8)))?;
    stream.set_write_timeout(Some(Duration::from_secs(8)))?;

    let request = format!(
        "GET {path} HTTP/1.1\r\nHost: {host}\r\nUser-Agent: osint-rs\r\nConnection: close\r\n\r\n"
    );
    stream.write_all(request.as_bytes())?;

    let mut raw = Vec::new();
    stream.read_to_end(&mut raw)?;
    let text = String::from_utf8_lossy(&raw).to_string();

    let Some(split) = text.find("\r\n\r\n") else {
        bail!("malformed HTTP response from {host}");
    };
    let (headers, body) = text.split_at(split);
    let body = &body[4..];

    if !headers.contains("200 OK") {
        let status_line = headers.lines().next().unwrap_or("unknown status");
        bail!("{host} returned: {status_line}");
    }

    if headers.to_lowercase().contains("transfer-encoding: chunked") {
        return Ok(dechunk(body));
    }

    Ok(body.to_string())
}

/// Minimal chunked-transfer-encoding decoder (each chunk is prefixed
/// with its length in hex on its own line, ending in a zero-length
/// chunk).
fn dechunk(body: &str) -> String {
    let mut out = String::new();
    let mut rest = body;
    loop {
        let Some(line_end) = rest.find("\r\n") else { break };
        let (size_line, remainder) = rest.split_at(line_end);
        let remainder = &remainder[2..];
        let Ok(size) = usize::from_str_radix(size_line.trim(), 16) else { break };
        if size == 0 {
            break;
        }
        if remainder.len() < size {
            out.push_str(remainder);
            break;
        }
        out.push_str(&remainder[..size]);
        rest = &remainder[size..];
        rest = rest.strip_prefix("\r\n").unwrap_or(rest);
    }
    out
}
