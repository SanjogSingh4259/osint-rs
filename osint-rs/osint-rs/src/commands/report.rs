use anyhow::Result;
use chrono::Local;
use std::fs;
use std::path::PathBuf;

/// Writes a combined report to ./osint-reports/<target>-<timestamp>.txt
/// and returns the path written to.
pub fn save(target: &str, sections: &[(&str, String)]) -> Result<PathBuf> {
    let dir = PathBuf::from("osint-reports");
    fs::create_dir_all(&dir)?;

    let timestamp = Local::now().format("%Y%m%d-%H%M%S");
    let path = dir.join(format!("{target}-{timestamp}.txt"));

    let mut body = format!(
        "OSINT report for {target}\nGenerated: {}\n{}\n\n",
        Local::now().to_rfc2822(),
        "=".repeat(60)
    );

    for (title, content) in sections {
        body.push_str(&format!("\n## {title}\n{}\n\n", "-".repeat(40)));
        body.push_str(content);
        body.push('\n');
    }

    fs::write(&path, body)?;
    Ok(path)
}
