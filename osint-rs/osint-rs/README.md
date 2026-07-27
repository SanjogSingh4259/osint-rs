# osint

A small Rust CLI that wraps three tools already on Kali (`whois`, `dig`,
`recon-ng`) behind one simple binary, so a full recon pass is one command
instead of three separate tool invocations and syntaxes to remember.

It does not reimplement whois or recon-ng — it shells out to them and
formats the output. That's intentional: those databases/engines are
already correct and maintained; this tool just makes them easy to drive.

## Requirements (all preinstalled on Kali, or one apt install away)

- `whois`
- `dig` (package `dnsutils`)
- `recon-ng`
- `subfinder`
- `theHarvester` (package `theharvester`)
- `amass`

If any is missing, the tool tells you exactly which `apt` package to
install rather than failing silently.

## Build

```bash
cd osint-rs
cargo build --release
sudo cp target/release/osint /usr/local/bin/osint   # optional: put it on PATH
```

## Usage

```bash
osint whois example.com                 # cleaned-up WHOIS summary
osint whois example.com --raw           # full raw WHOIS record

osint dns example.com                   # A/AAAA/MX/NS/TXT via dig

osint recon example.com                 # drives recon-ng non-interactively
osint recon example.com \
  --workspace mytarget \
  --modules recon/domains-hosts/hackertarget,recon/domains-hosts/bing_domain_web

osint subfinder example.com             # passive subdomain enumeration

osint harvester example.com             # emails/hosts via key-free sources
osint harvester example.com --sources bing,crtsh,otx

osint amass example.com                 # passive asset enumeration

osint full example.com                  # whois + dns + recon-ng, saved to disk
osint full example.com --deep           # + subfinder, theHarvester, amass
```

`osint full` writes a timestamped report to `./osint-reports/`. Use
`--deep` when you want the slower, noisier passive-recon tools included
in the same report — otherwise `full` stays fast (whois/dns/recon-ng
only), which is what you'd want for a quick check.

## How the recon-ng integration works

`recon-ng` is normally interactive. This tool generates a one-shot
resource script (a `.rc` file with the exact commands you'd type at the
`recon-ng` prompt: `workspaces select`, `modules load`, `options set
SOURCE`, `run`) and calls:

```bash
recon-ng -w <workspace> -r <script.rc> --no-check
```

The script is written to a temp file and deleted after the run. You can
see the exact command it would have run (useful for debugging or for
your own scripting) in the error output if recon-ng isn't installed.

## Adding more modules / tools later

- New recon-ng modules: just pass `--modules a,b,c`, no code changes needed.
- New data sources beyond subfinder/theHarvester/amass (e.g. `httpx`,
  `naabu`, `shodan` via API): add a new file under `src/commands/`,
  following the pattern in `subfinder.rs` (shell out via `shell::run`,
  format the output, return a `String`), then wire it into `main.rs` as
  a new subcommand and optionally into `run_full`'s `--deep` branch.

## Notes

- This was built and compiled in a sandboxed Linux container without
  live network access to whois/DNS servers, so live output wasn't
  verified end-to-end here — verify the first run on your Kali box.
- Dependency versions in `Cargo.toml` are pinned to versions that build
  with Rust 1.75 (apt's Kali/Ubuntu default). If your Kali has a newer
  `rustc` (check with `rustc --version`), you can loosen the pins with
  `cargo update` for the latest features/fixes.
