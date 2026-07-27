use colored::*;

/// Printed once at the start of a run (skipped with --quiet).
pub fn print_banner() {
    let art = r#"
  ___  ____ ___ _   _ _____
 / _ \/ ___|_ _| \ | |_   _|
| | | \___ \| ||  \| | | |
| |_| |___) | || |\  | | |
 \___/|____/___|_| \_| |_|
"#;
    println!("{}", art.bright_cyan().bold());
    println!(
        "{}",
        "  passive OSINT recon — whois · dns · recon-ng · subfinder · theHarvester · amass"
            .dimmed()
    );
    println!();
}

/// A titled box-drawn section header, e.g.:
/// ╭─ 🌐  WHOIS ────────────────────────────
pub fn section_header(icon: &str, title: &str) -> String {
    let label = format!(" {icon}  {title} ");
    let bar = "─".repeat(50usize.saturating_sub(label.chars().count()));
    format!("{}{}{}", "╭─".bright_black(), label.bold().white(), bar.bright_black())
}

/// Closing rule for a section.
pub fn section_footer() -> String {
    format!("{}", "╰────────────────────────────────────────────────".bright_black())
}

/// Wraps a block of content between a header/footer pair.
pub fn boxed(icon: &str, title: &str, content: &str) -> String {
    let mut out = String::new();
    out.push_str(&section_header(icon, title));
    out.push('\n');
    for line in content.lines() {
        out.push_str(&format!("{} {line}\n", "│".bright_black()));
    }
    out.push_str(&section_footer());
    out.push('\n');
    out
}
