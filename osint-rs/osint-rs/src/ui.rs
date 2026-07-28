use colored::*;

/// Printed once at the start of a run (skipped with --quiet).
pub fn print_banner() {
    let lines = [
        r"  ___  ____ ___ _   _ _____ ",
        r" / _ \/ ___|_ _| \ | |_   _|",
        r"| | | \___ \| ||  \| | | |  ",
        r"| |_| |___) | || |\  | | |  ",
        r" \___/|____/___|_| \_| |_|  ",
    ];
    // Cyan -> blue -> magenta gradient, line by line, for a bit of flair.
    let colors = [Color::BrightCyan, Color::Cyan, Color::BrightBlue, Color::Blue, Color::BrightMagenta];
    println!();
    for (line, color) in lines.iter().zip(colors.iter()) {
        println!("{}", line.color(*color).bold());
    }
    println!(
        "{}",
        "  passive + active OSINT recon for Kali — one binary, whois → nmap"
            .dimmed()
            .italic()
    );
    println!();
}

/// A prominent single-line banner naming the current target, printed
/// once at the top of a `full`/`deep` run.
pub fn target_banner(target: &str) -> String {
    let label = format!(" 🎯 TARGET: {target} ");
    let bar = "═".repeat(label.chars().count().max(20) + 4);
    format!(
        "{}\n{}\n{}\n",
        bar.bright_black(),
        label.bold().black().on_bright_cyan(),
        bar.bright_black()
    )
}

/// A titled, color-accented box-drawn section, e.g.:
/// ╭─ 🗄️  WHOIS ─────────────────────── 1.2s
pub fn boxed(icon: &str, title: &str, color: Color, content: &str) -> String {
    boxed_timed(icon, title, color, content, None)
}

/// Same as `boxed`, but with an optional elapsed-time suffix on the
/// header line (used by `full`/`deep` so each section shows how long
/// it actually took — reinforces that this is live data, not canned).
pub fn boxed_timed(icon: &str, title: &str, color: Color, content: &str, elapsed: Option<f32>) -> String {
    let label = format!(" {icon}  {title} ");
    let suffix = match elapsed {
        Some(s) => format!(" {} ", format!("{s:.1}s").dimmed()),
        None => String::new(),
    };
    let visible_len = label.chars().count() + elapsed.map(|_| 7).unwrap_or(0);
    let bar_len = 52usize.saturating_sub(visible_len);
    let bar = "─".repeat(bar_len);

    let mut out = String::new();
    out.push_str(&format!(
        "{}{}{}{}\n",
        "╭─".color(color),
        label.bold().color(color),
        bar.color(color),
        suffix
    ));
    for line in content.lines() {
        out.push_str(&format!("{} {line}\n", "│".color(color)));
    }
    out.push_str(&format!("{}\n", "╰──────────────────────────────────────────────────".color(color)));
    out
}

/// End-of-run dashboard summarizing counts across sections, e.g.:
/// 📊 SUMMARY   12 subdomains · 3 open ports · 47 archived URLs
pub fn summary(items: &[(&str, usize)]) -> String {
    let mut out = String::new();
    let bar = "─".repeat(52);
    out.push_str(&format!("{}\n", bar.bright_black()));
    out.push_str(&format!("{} ", "📊 SUMMARY".bold().bright_white()));
    let parts: Vec<String> = items
        .iter()
        .map(|(label, count)| format!("{} {}", count.to_string().bold().bright_green(), label.dimmed()))
        .collect();
    out.push_str(&parts.join(&format!(" {} ", "·".bright_black())));
    out.push('\n');
    out.push_str(&format!("{}\n", bar.bright_black()));
    out
}

/// Scans a block of text for a line matching "<number> <marker>" (the
/// pattern every count-producing module prints, e.g. "12 unique names
/// found") and returns that number. Used to build the summary
/// dashboard without changing every module's return type just for
/// counting.
pub fn extract_count(text: &str, marker: &str) -> usize {
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.contains(marker) {
            if let Some(tok) = trimmed.split_whitespace().next() {
                if let Ok(n) = tok.parse::<usize>() {
                    return n;
                }
            }
        }
    }
    0
}

/// Counts lines containing "open" in an nmap-style ports report.
pub fn count_open_ports(text: &str) -> usize {
    text.lines().filter(|l| l.contains("open") && l.contains('/')).count()
}
