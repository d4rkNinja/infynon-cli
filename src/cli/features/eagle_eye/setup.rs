use super::config::{config_path, load_config, save_config};
use super::prompt::{
    collect_scan_paths, parse_csv_list, prompt_bool, prompt_line, prompt_optional_line, prompt_u16,
    prompt_u32,
};
use super::types::risk_levels_for_choice;
use crate::tui::logger::Logger;
use owo_colors::OwoColorize;

pub fn cmd_setup() {
    Logger::title("EAGLE EYE SETUP", "blue");
    println!(
        "  {}  Configure scheduled vulnerability monitoring with email alerts.\n",
        "🦅".bold()
    );

    let mut config = load_config();

    println!("  {}  {}\n", "1".bold().bright_cyan(), "SMTP Configuration".bold().white());
    let host_default = if config.smtp.host.is_empty() {
        "smtp.gmail.com"
    } else {
        &config.smtp.host
    };
    config.smtp.host = prompt_line("SMTP Host", host_default);
    config.smtp.port = prompt_u16("SMTP Port", config.smtp.port);

    let user_default = if config.smtp.username.is_empty() {
        "your-email@gmail.com"
    } else {
        &config.smtp.username
    };
    config.smtp.username = prompt_line("SMTP Username", user_default);
    let password_env_default = if config.smtp.password_env.is_empty() {
        "INFYNON_SMTP_PASSWORD"
    } else {
        &config.smtp.password_env
    };
    config.smtp.password_env = prompt_line("SMTP Password env var", password_env_default);
    config.smtp.legacy_password.clear();
    if std::env::var(&config.smtp.password_env).ok().filter(|value| !value.trim().is_empty()).is_none() {
        Logger::info(&format!("Set {} before running Eagle Eye.", config.smtp.password_env));
    }
    config.smtp.tls = prompt_bool("Use TLS", config.smtp.tls);

    println!("\n  {}  {}\n", "2".bold().bright_cyan(), "Email Addresses".bold().white());
    let from_default = if config.from.is_empty() {
        config.smtp.username.as_str()
    } else {
        &config.from
    };
    config.from = prompt_line("From address", from_default);

    let recip_default = if config.recipients.is_empty() {
        "admin@example.com".to_string()
    } else {
        config.recipients.join(", ")
    };
    if let Some(recipients) = prompt_optional_line("Recipients (comma-separated)", &recip_default) {
        config.recipients = parse_csv_list(&recipients);
    }

    println!("\n  {}  {}\n", "3".bold().bright_cyan(), "Project Paths to Monitor".bold().white());
    collect_scan_paths(&mut config.scan_paths);

    println!("\n  {}  {}\n", "4".bold().bright_cyan(), "Scan Schedule".bold().white());
    config.interval_hours = prompt_u32("Scan interval in hours", config.interval_hours);

    println!("\n  {}  {}\n", "5".bold().bright_cyan(), "Risk Level Threshold".bold().white());
    println!("  Select which severity levels trigger an email alert:");
    println!("  {}  CRITICAL only", "[1]".bold().bright_red());
    println!("  {}  CRITICAL + HIGH", "[2]".bold().bright_red());
    println!("  {}  CRITICAL + HIGH + MEDIUM", "[3]".bold().bright_yellow());
    println!("  {}  CRITICAL + HIGH + MEDIUM + LOW", "[4]".bold().bright_green());
    println!("  {}  ALL (including INFORMATIONAL)", "[5]".bold().bright_cyan());
    println!();
    let choice = prompt_line("Choice", "2");
    config.risk_levels = risk_levels_for_choice(&choice);
    config.enabled = true;

    match save_config(&config) {
        Ok(()) => {
            println!();
            Logger::success("Eagle Eye configuration saved!");
            Logger::detail("  Config:", &config_path().display().to_string());
            Logger::detail("  Paths:", &format!("{} project(s)", config.scan_paths.len()));
            Logger::detail("  Interval:", &format!("every {} hours", config.interval_hours));
            Logger::detail("  Risk:", &config.risk_levels.join(", "));
            Logger::detail("  Email to:", &config.recipients.join(", "));
            Logger::detail("  Status:", "ENABLED");
            println!();
            Logger::info("Run `infynon pkg eagle-eye start` to begin monitoring.");
        }
        Err(err) => Logger::error(&format!("Failed to save config: {}", err)),
    }
}
