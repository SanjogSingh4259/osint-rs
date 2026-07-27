use crate::shell;
use anyhow::Result;
use colored::*;
use std::fs;
use std::io::Write;

const DEFAULT_MODULES: [&str; 1] = ["recon/domains-hosts/hackertarget"];

/// Drives recon-ng without ever dropping the user into its interactive
/// shell: writes a one-shot resource script (.rc) with the commands
/// recon-ng would normally take at its prompt, then runs
/// `recon-ng -w <workspace> -r <script>`.
pub fn run(target: &str, workspace: &str, modules: &[String]) -> Result<String> {
    let modules: Vec<String> = if modules.is_empty() {
        DEFAULT_MODULES.iter().map(|s| s.to_string()).collect()
    } else {
        modules.to_vec()
    };

    let mut script = String::new();
    script.push_str(&format!("workspaces select {workspace}\n"));

    for module in &modules {
        script.push_str(&format!("modules load {module}\n"));
        // SOURCE takes the target directly, skipping the need to
        // pre-populate the db with a separate insert step.
        script.push_str(&format!("options set SOURCE {target}\n"));
        script.push_str("run\n");
        script.push_str("back\n");
    }

    script.push_str("show hosts\n");
    script.push_str("show domains\n");
    script.push_str("exit\n");

    let script_path = std::env::temp_dir().join(format!("osint-recon-{target}.rc"));
    {
        let mut f = fs::File::create(&script_path)?;
        f.write_all(script.as_bytes())?;
    }

    let path_str = script_path.to_string_lossy().to_string();
    let output = shell::run(
        "recon-ng",
        &["-w", workspace, "-r", &path_str, "--no-check"],
    );

    let _ = fs::remove_file(&script_path);

    let mut report = String::new();
    report.push_str(&format!(
        "{}\n",
        format!("recon-ng results for {target} (workspace: {workspace})")
            .bold()
            .cyan()
    ));

    match output {
        Ok(out) => report.push_str(&out),
        Err(e) => {
            report.push_str(&format!(
                "{}\n",
                "recon-ng could not be run automatically.".red()
            ));
            report.push_str(&format!("{e}\n\n"));
            report.push_str("You can still drive it manually with the generated script:\n");
            report.push_str(&format!("  recon-ng -w {workspace} -r {path_str} --no-check\n"));
        }
    }

    Ok(report)
}
