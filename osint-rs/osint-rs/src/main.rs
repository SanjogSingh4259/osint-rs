mod commands;
mod shell;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use commands::{amass, dns, harvester, recon, report, subfinder, whois};

#[derive(Parser)]
#[command(
    name = "osint",
    version,
    about = "OSINT recon CLI for Kali Linux — thin Rust wrapper over whois, dig, and recon-ng",
    long_about = None
)]
struct Cli {
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
    /// Run whois + dns + recon-ng together and save a combined report
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

    let result = match cli.command {
        Commands::Whois { target, raw } => run_whois(&target, raw),
        Commands::Dns { target } => run_dns(&target),
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
    println!("{out}");
    Ok(())
}

fn run_dns(target: &str) -> Result<()> {
    let out = dns::run(target)?;
    println!("{out}");
    Ok(())
}

fn run_recon(target: &str, workspace: &str, modules: &[String]) -> Result<()> {
    let out = recon::run(target, workspace, modules)?;
    println!("{out}");
    Ok(())
}

fn run_subfinder(target: &str) -> Result<()> {
    let out = subfinder::run(target)?;
    println!("{out}");
    Ok(())
}

fn run_harvester(target: &str, sources: Option<&str>) -> Result<()> {
    let out = harvester::run(target, sources)?;
    println!("{out}");
    Ok(())
}

fn run_amass(target: &str) -> Result<()> {
    let out = amass::run(target)?;
    println!("{out}");
    Ok(())
}

fn run_full(target: &str, workspace: &str, modules: &[String], deep: bool) -> Result<()> {
    println!("{}", format!("Running full OSINT sweep on {target}...").bold().green());

    let whois_out = whois::run(target, false).unwrap_or_else(|e| format!("[skipped] {e}"));
    println!("{whois_out}\n");

    let dns_out = dns::run(target).unwrap_or_else(|e| format!("[skipped] {e}"));
    println!("{dns_out}\n");

    let recon_out = recon::run(target, workspace, modules).unwrap_or_else(|e| format!("[skipped] {e}"));
    println!("{recon_out}\n");

    let mut sections = vec![
        ("WHOIS", whois_out),
        ("DNS", dns_out),
        ("recon-ng", recon_out),
    ];

    if deep {
        let sub_out = subfinder::run(target).unwrap_or_else(|e| format!("[skipped] {e}"));
        println!("{sub_out}\n");

        let harv_out = harvester::run(target, None).unwrap_or_else(|e| format!("[skipped] {e}"));
        println!("{harv_out}\n");

        let amass_out = amass::run(target).unwrap_or_else(|e| format!("[skipped] {e}"));
        println!("{amass_out}\n");

        sections.push(("subfinder", sub_out));
        sections.push(("theHarvester", harv_out));
        sections.push(("amass", amass_out));
    }

    let path = report::save(target, &sections)?;

    println!(
        "{} {}",
        "Report saved to:".bold().cyan(),
        path.display()
    );

    Ok(())
}
