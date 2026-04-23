use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

pub(crate) fn format_bytes(bytes: u64) -> String {
    crate::utils::format_bytes(bytes)
}

pub(crate) fn spinner() -> ProgressBar {
    let sp = ProgressBar::new_spinner();
    sp.set_style(
        ProgressStyle::with_template("  {spinner:.cyan}  {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    sp.enable_steady_tick(Duration::from_millis(60));
    sp
}

pub(crate) fn bar(len: u64) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_style(
        ProgressStyle::with_template(
            "  {spinner:.cyan}  {msg:<40} [{bar:40.cyan/blue}] {pos}/{len}",
        )
        .unwrap()
        .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
        .progress_chars("█▉▊▋▌▍▎▏  "),
    );
    pb.enable_steady_tick(Duration::from_millis(60));
    pb
}
