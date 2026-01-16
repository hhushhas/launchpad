use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Print a header/title
pub fn header(text: &str) {
    println!();
    println!("{}", style(text).bold().cyan());
}

/// Print a step message
pub fn step(text: &str) {
    println!("{} {}", style("→").dim(), text);
}

/// Print a success message
pub fn success(text: &str) {
    println!("{} {}", style("✓").green(), text);
}

/// Print a warning message
pub fn warn(text: &str) {
    println!("{} {}", style("⚠").yellow(), text);
}

/// Print an error message
pub fn error(text: &str) {
    eprintln!("{} {}", style("✗").red(), text);
}

/// Print a check pass result
pub fn check_pass(name: &str, message: &str) {
    println!("{} {} {}", style("✓").green(), style(name).bold(), style(message).dim());
}

/// Print a check fail result
pub fn check_fail(name: &str, message: &str) {
    println!("{} {} {}", style("✗").red(), style(name).bold(), style(message).dim());
}

/// Create a spinner for long-running operations
pub fn spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(Duration::from_millis(100));
    pb
}

/// Create a progress bar
pub fn progress_bar(len: u64, message: &str) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{bar:40.cyan/dim}] {pos}/{len}")
            .unwrap()
            .progress_chars("━━─"),
    );
    pb.set_message(message.to_string());
    pb
}
