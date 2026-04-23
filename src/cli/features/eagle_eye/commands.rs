use super::config::{config_path, load_config, save_config};
use super::scan::run_scan_cycle;
use super::secret::{password_status, resolve_smtp_password};
use crate::tui::logger::Logger;
use owo_colors::OwoColorize;

pub fn cmd_status() {
    Logger::title("EAGLE EYE STATUS", "blue");
    let config = load_config();

    let status = if config.enabled {
        "ENABLED".bold().bright_green().to_string()
    } else {
        "DISABLED".bold().bright_red().to_string()
    };
    Logger::detail("  Status:", &status);
    Logger::detail("  Config:", &config_path().display().to_string());

    if config.scan_paths.is_empty() {
        Logger::info("  No scan paths configured. Run `infynon pkg eagle-eye setup`.");
        return;
    }

    Logger::detail("  Paths:", &format!("{}", config.scan_paths.len()));
    for path in &config.scan_paths {
        println!("    {} {}", "·".truecolor(60, 60, 80), path.truecolor(180, 180, 200));
    }
    Logger::detail("  Interval:", &format!("every {} hours", config.interval_hours));
    Logger::detail("  Risk levels:", &config.risk_levels.join(", "));
    Logger::detail("  SMTP:", &format!("{}:{}", config.smtp.host, config.smtp.port));
    Logger::detail("  SMTP secret:", &password_status(&config.smtp));
    Logger::detail("  From:", &config.from);
    Logger::detail("  To:", &config.recipients.join(", "));
}

pub fn cmd_enable() {
    update_enabled_state(true, "ENABLED");
}

pub fn cmd_disable() {
    update_enabled_state(false, "DISABLED");
}

pub fn cmd_start() {
    let config = load_config();
    if !config.enabled {
        Logger::error("Eagle Eye is disabled. Run `infynon pkg eagle-eye enable` first.");
        return;
    }
    if config.scan_paths.is_empty() {
        Logger::error("No scan paths configured. Run `infynon pkg eagle-eye setup` first.");
        return;
    }
    if config.smtp.host.is_empty() || config.recipients.is_empty() {
        Logger::error("SMTP host or recipients are not configured. Run `infynon pkg eagle-eye setup` first.");
        return;
    }
    if resolve_smtp_password(&config.smtp).is_none() {
        Logger::error("SMTP password is not available. Set the configured env var or re-run `infynon pkg eagle-eye setup`.");
        return;
    }

    Logger::title("EAGLE EYE MONITORING", "blue");
    println!("  {}  Starting scheduled vulnerability monitoring...\n", "🦅".bold());
    Logger::detail("  Paths:", &format!("{} project(s)", config.scan_paths.len()));
    Logger::detail("  Interval:", &format!("every {} hours", config.interval_hours));
    Logger::detail("  Risk:", &config.risk_levels.join(", "));
    Logger::detail("  Email to:", &config.recipients.join(", "));
    println!();
    Logger::info("Press Ctrl+C to stop monitoring.\n");

    run_scan_cycle(&config);
    loop {
        Logger::raw_dim(&format!("  Next scan in {} hours...", config.interval_hours));
        std::thread::sleep(std::time::Duration::from_secs(config.interval_hours as u64 * 3600));
        let updated = load_config();
        if !updated.enabled {
            Logger::info("Eagle Eye has been disabled. Stopping.");
            break;
        }
        run_scan_cycle(&updated);
    }
}

fn update_enabled_state(enabled: bool, label: &str) {
    let mut config = load_config();
    config.enabled = enabled;
    match save_config(&config) {
        Ok(()) => Logger::success(&format!("Eagle Eye monitoring {}", label)),
        Err(err) => Logger::error(&format!("Failed: {}", err)),
    }
}
