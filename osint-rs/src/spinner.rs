use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Runs a closure behind a live spinner with the given message.
/// Used to wrap slow lookups (whois, recon-ng, amass, HTTP calls) so
/// the terminal shows visible progress instead of sitting silent.
pub fn spin<F, T>(message: &str, f: F) -> T
where
    F: FnOnce() -> T,
{
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.cyan} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(Duration::from_millis(80));

    let result = f();

    pb.finish_and_clear();
    result
}
