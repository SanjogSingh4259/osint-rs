mod commands;
mod http;
mod shell;
mod spinner;
mod ui;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use commands::{amass, dns, harvester, ipinfo, ports, recon, report, subfinder, whois};

#[derive(Parser)]
#[command(
    name = "osint",
    version,
    about = "OSINT recon CLI for Kali Linux — thin Rust wrapper over whois, dig, recon-ng, subfinder, theHarvester, amass, and nmap",
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
    /// Run whois + dns + ip + recon-ng + ports together and save a combined report
    Full {
        target: String,
        #[arg(long, default_value = "osint")]
        workspace: String,
        #[arg(long, value_delimiter = ',')]
        modules: Vec<String>,
        /// Also run subfinder, theHarvester, and amass (slower, noisier)
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
    print!("{}", ui::boxed("🗄️", "WHOIS", &out));
    Ok(())
}

fn run_dns(target: &str) -> Result<()> {
    let out = dns::run(target)?;
    print!("{}", ui::boxed("🌐", "DNS", &out));
    Ok(())
}

fn run_ip(target: &str) -> Result<()> {
    let out = ipinfo::run(target)?;
    print!("{}", ui::boxed("📡", "IP / GEOLOCATION", &out));
    Ok(())
}

fn run_ports(target: &str, fast: bool) -> Result<()> {
    let out = ports::run(target, fast)?;
    print!("{}", ui::boxed("🛰️", "OPEN PORTS", &out));
    Ok(())
}

fn run_recon(target: &str, workspace: &str, modules: &[String]) -> Result<()> {
    let out = recon::run(target, workspace, modules)?;
    print!("{}", ui::boxed("🔍", "RECON-NG", &out));
    Ok(())
}

fn run_subfinder(target: &str) -> Result<()> {
    let out = subfinder::run(target)?;
    print!("{}", ui::boxed("🧭", "SUBFINDER", &out));
    Ok(())
}

fn run_harvester(target: &str, sources: Option<&str>) -> Result<()> {
    let out = harvester::run(target, sources)?;
    print!("{}", ui::boxed("🎯", "THEHARVESTER", &out));
    Ok(())
}

fn run_amass(target: &str) -> Result<()> {
    let out = amass::run(target)?;
    print!("{}", ui::boxed("🕸️", "AMASS", &out));
    Ok(())
}

fn run_full(target: &str, workspace: &str, modules: &[String], deep: bool) -> Result<()> {
    let start = std::time::Instant::now();
    println!(
        "{}",
        format!("▶ running full OSINT sweep on {target}...").bold().green()
    );
    println!();

    let whois_out = whois::run(target, false).unwrap_or_else(|e| format!("[skipped] {e}"));
    print!("{}", ui::boxed("🗄️", "WHOIS", &whois_out));

    let dns_out = dns::run(target).unwrap_or_else(|e| format!("[skipped] {e}"));
    print!("{}", ui::boxed("🌐", "DNS", &dns_out));

    let ip_out = ipinfo::run(target).unwrap_or_else(|e| format!("[skipped] {e}"));
    print!("{}", ui::boxed("📡", "IP / GEOLOCATION", &ip_out));

    let recon_out =
        recon::run(target, workspace, modules).unwrap_or_else(|e| format!("[skipped] {e}"));
    print!("{}", ui::boxed("🔍", "RECON-NG", &recon_out));

    let ports_out = ports::run(target, true).unwrap_or_else(|e| format!("[skipped] {e}"));
    print!("{}", ui::boxed("🛰️", "OPEN PORTS", &ports_out));

    let mut sections = vec![
        ("WHOIS", whois_out),
        ("DNS", dns_out),
        ("IP / Geolocation", ip_out),
        ("recon-ng", recon_out),
        ("Open ports", ports_out),
    ];

    if deep {
        let sub_out = subfinder::run(target).unwrap_or_else(|e| format!("[skipped] {e}"));
        print!("{}", ui::boxed("🧭", "SUBFINDER", &sub_out));

        let harv_out = harvester::run(target, None).unwrap_or_else(|e| format!("[skipped] {e}"));
        print!("{}", ui::boxed("🎯", "THEHARVESTER", &harv_out));

        let amass_out = amass::run(target).unwrap_or_else(|e| format!("[skipped] {e}"));
        print!("{}", ui::boxed("🕸️", "AMASS", &amass_out));

        sections.push(("subfinder", sub_out));
        sections.push(("theHarvester", harv_out));
        sections.push(("amass", amass_out));
    }

    let path = report::save(target, &sections)?;
    let elapsed = start.elapsed();

    println!(
        "{} {}  {}",
        "✔ report saved to:".bold().green(),
        path.display(),
        format!("({:.1}s)", elapsed.as_secs_f32()).dimmed()
    );

    Ok(())
}
