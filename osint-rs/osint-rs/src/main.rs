mod commands;
mod http;
mod shell;
mod spinner;
mod ui;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use commands::{amass, certs, dns, harvester, headers, ipinfo, ports, recon, report, subfinder, wayback, whois};

#[derive(Parser)]
#[command(
    name = "osint",
    version,
    about = "OSINT recon CLI for Kali Linux — wraps whois, dig, recon-ng, subfinder, theHarvester, amass, nmap, crt.sh, and the Wayback Machine",
    long_about = None
)]
struct Cli {
    /// Skip the ASCII banner (useful when piping output)
    #[arg(long, short = 'q', global = true)]
    quiet: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// WHOIS lookup for a domain or IP
    Whois {
        target: String,
        /// Show the full raw whois record instead of the cleaned summary
        #[arg(long)]
        raw: bool,
    },
    /// DNS record lookup (A, AAAA, MX, NS, TXT) via dig
    Dns { target: String },
    /// Resolve, reverse-DNS, and geolocate a target (real-time IP/ASN/ISP info)
    Ip { target: String },
    /// Live open-port / service scan via nmap
    Ports {
        target: String,
        /// Scan all 1000 default nmap ports instead of the fast top-100
        #[arg(long)]
        full: bool,
    },
    /// Subdomain discovery via certificate transparency logs (crt.sh)
    Certs { target: String },
    /// HTTP response headers / tech fingerprint via curl
    Headers { target: String },
    /// Historical URLs archived by the Wayback Machine
    Wayback {
        target: String,
        #[arg(long, default_value_t = 50)]
        limit: u32,
    },
    /// Drive recon-ng non-interactively against a target
    Recon {
        target: String,
        /// recon-ng workspace name (created if it doesn't exist)
        #[arg(long, default_value = "osint")]
        workspace: String,
        /// Comma-separated recon-ng module paths, e.g.
        /// recon/domains-hosts/hackertarget,recon/domains-hosts/bing_domain_web
        #[arg(long, value_delimiter = ',')]
        modules: Vec<String>,
    },
    /// Passive subdomain enumeration via subfinder
    Subfinder { target: String },
    /// Email/host/IP harvesting across public sources via theHarvester
    Harvester {
        target: String,
        /// Comma-separated source list (default: key-free sources only)
        #[arg(long)]
        sources: Option<String>,
    },
    /// Passive asset enumeration via amass
    Amass { target: String },
    /// Run the full recon suite and save a combined report
    Full {
        target: String,
        #[arg(long, default_value = "osint")]
        workspace: String,
        #[arg(long, value_delimiter = ',')]
        modules: Vec<String>,
        /// Also run subfinder, theHarvester, amass, and Wayback Machine history (slower, noisier)
        #[arg(long)]
        deep: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    if !cli.quiet {
        ui::print_banner();
    }

    let result = match cli.command {
        Commands::Whois { target, raw } => run_whois(&target, raw),
        Commands::Dns { target } => run_dns(&target),
        Commands::Ip { target } => run_ip(&target),
        Commands::Ports { target, full } => run_ports(&target, !full),
        Commands::Certs { target } => run_certs(&target),
        Commands::Headers { target } => run_headers(&target),
        Commands::Wayback { target, limit } => run_wayback(&target, limit),
        Commands::Recon {
            target,
            workspace,
            modules,
        } => run_recon(&target, &workspace, &modules),
        Commands::Subfinder { target } => run_subfinder(&target),
        Commands::Harvester { target, sources } => run_harvester(&target, sources.as_deref()),
        Commands::Amass { target } => run_amass(&target),
        Commands::Full {
            target,
            workspace,
            modules,
            deep,
        } => run_full(&target, &workspace, &modules, deep),
    };

    if let Err(e) = result {
        eprintln!("{} {e}", "error:".red().bold());
        std::process::exit(1);
    }
}

fn run_whois(target: &str, raw: bool) -> Result<()> {
    let out = whois::run(target, raw)?;
    print!("{}", ui::boxed("🗄️", "WHOIS", Color::Blue, &out));
    Ok(())
}

fn run_dns(target: &str) -> Result<()> {
    let out = dns::run(target)?;
    print!("{}", ui::boxed("🌐", "DNS", Color::Green, &out));
    Ok(())
}

fn run_ip(target: &str) -> Result<()> {
    let out = ipinfo::run(target)?;
    print!("{}", ui::boxed("📡", "IP / GEOLOCATION", Color::Magenta, &out));
    Ok(())
}

fn run_ports(target: &str, fast: bool) -> Result<()> {
    let out = ports::run(target, fast)?;
    print!("{}", ui::boxed("🛰️", "OPEN PORTS", Color::Red, &out));
    Ok(())
}

fn run_certs(target: &str) -> Result<()> {
    let out = certs::run(target)?;
    print!("{}", ui::boxed("📜", "CERTIFICATE TRANSPARENCY", Color::BrightGreen, &out));
    Ok(())
}

fn run_headers(target: &str) -> Result<()> {
    let out = headers::run(target)?;
    print!("{}", ui::boxed("🧬", "HTTP HEADERS / FINGERPRINT", Color::BrightYellow, &out));
    Ok(())
}

fn run_wayback(target: &str, limit: u32) -> Result<()> {
    let out = wayback::run(target, limit)?;
    print!("{}", ui::boxed("🕰️", "WAYBACK MACHINE", Color::BrightCyan, &out));
    Ok(())
}

fn run_recon(target: &str, workspace: &str, modules: &[String]) -> Result<()> {
    let out = recon::run(target, workspace, modules)?;
    print!("{}", ui::boxed("🔍", "RECON-NG", Color::Cyan, &out));
    Ok(())
}

fn run_subfinder(target: &str) -> Result<()> {
    let out = subfinder::run(target)?;
    print!("{}", ui::boxed("🧭", "SUBFINDER", Color::Yellow, &out));
    Ok(())
}

fn run_harvester(target: &str, sources: Option<&str>) -> Result<()> {
    let out = harvester::run(target, sources)?;
    print!("{}", ui::boxed("🎯", "THEHARVESTER", Color::BrightMagenta, &out));
    Ok(())
}

fn run_amass(target: &str) -> Result<()> {
    let out = amass::run(target)?;
    print!("{}", ui::boxed("🕸️", "AMASS", Color::BrightBlue, &out));
    Ok(())
}

/// Runs a lookup, times it, prints its boxed section immediately, and
/// returns the raw text (for the end-of-run summary dashboard).
fn step(icon: &str, title: &str, color: Color, f: impl FnOnce() -> Result<String>) -> String {
    let start = std::time::Instant::now();
    let out = f().unwrap_or_else(|e| format!("[skipped] {e}"));
    let elapsed = start.elapsed().as_secs_f32();
    print!("{}", ui::boxed_timed(icon, title, color, &out, Some(elapsed)));
    out
}

fn run_full(target: &str, workspace: &str, modules: &[String], deep: bool) -> Result<()> {
    let start = std::time::Instant::now();
    print!("{}", ui::target_banner(target));
    println!();

    let whois_out = step("🗄️", "WHOIS", Color::Blue, || whois::run(target, false));
    let dns_out = step("🌐", "DNS", Color::Green, || dns::run(target));
    let ip_out = step("📡", "IP / GEOLOCATION", Color::Magenta, || ipinfo::run(target));
    let certs_out = step("📜", "CERTIFICATE TRANSPARENCY", Color::BrightGreen, || certs::run(target));
    let headers_out = step("🧬", "HTTP HEADERS / FINGERPRINT", Color::BrightYellow, || headers::run(target));
    let recon_out = step("🔍", "RECON-NG", Color::Cyan, || recon::run(target, workspace, modules));
    let ports_out = step("🛰️", "OPEN PORTS", Color::Red, || ports::run(target, true));

    let mut sections = vec![
        ("WHOIS", whois_out),
        ("DNS", dns_out),
        ("IP / Geolocation", ip_out),
        ("Certificate transparency", certs_out.clone()),
        ("HTTP headers", headers_out),
        ("recon-ng", recon_out),
        ("Open ports", ports_out.clone()),
    ];

    let mut summary_items = vec![
        ("open ports", ui::count_open_ports(&ports_out)),
        ("cert-discovered names", ui::extract_count(&certs_out, "unique names found")),
    ];

    if deep {
        let sub_out = step("🧭", "SUBFINDER", Color::Yellow, || subfinder::run(target));
        let harv_out = step("🎯", "THEHARVESTER", Color::BrightMagenta, || harvester::run(target, None));
        let amass_out = step("🕸️", "AMASS", Color::BrightBlue, || amass::run(target));
        let wayback_out = step("🕰️", "WAYBACK MACHINE", Color::BrightCyan, || wayback::run(target, 50));

        summary_items.push(("subdomains", ui::extract_count(&sub_out, "subdomains found")));
        summary_items.push(("passive assets", ui::extract_count(&amass_out, "assets found")));
        summary_items.push(("archived URLs", ui::extract_count(&wayback_out, "archived URLs found")));

        sections.push(("subfinder", sub_out));
        sections.push(("theHarvester", harv_out));
        sections.push(("amass", amass_out));
        sections.push(("Wayback Machine", wayback_out));
    }

    let path = report::save(target, &sections)?;
    let elapsed = start.elapsed();

    println!("{}", ui::summary(&summary_items));
    println!(
        "{} {}  {}",
        "✔ report saved to:".bold().green(),
        path.display(),
        format!("(total {:.1}s)", elapsed.as_secs_f32()).dimmed()
    );

    Ok(())
}
