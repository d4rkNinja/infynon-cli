use crate::tui::logger::Logger;
use crate::engine::{osv, scanner};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ── Eagle Eye Config ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EagleEyeConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub smtp: EagleEyeSmtp,
    #[serde(default)]
    pub scan_paths: Vec<String>,
    #[serde(default = "default_interval")]
    pub interval_hours: u32,
    #[serde(default = "default_risk_levels")]
    pub risk_levels: Vec<String>,
    #[serde(default)]
    pub recipients: Vec<String>,
    #[serde(default)]
    pub from: String,
}

fn default_interval() -> u32 { 24 }
fn default_risk_levels() -> Vec<String> {
    vec!["CRITICAL".into(), "HIGH".into()]
}

impl Default for EagleEyeConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            smtp: EagleEyeSmtp::default(),
            scan_paths: Vec::new(),
            interval_hours: default_interval(),
            risk_levels: default_risk_levels(),
            recipients: Vec::new(),
            from: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EagleEyeSmtp {
    #[serde(default)]
    pub host: String,
    #[serde(default = "default_smtp_port")]
    pub port: u16,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
    #[serde(default = "default_tls")]
    pub tls: bool,
}

fn default_smtp_port() -> u16 { 587 }
fn default_tls() -> bool { true }

impl Default for EagleEyeSmtp {
    fn default() -> Self {
        Self {
            host: String::new(),
            port: default_smtp_port(),
            username: String::new(),
            password: String::new(),
            tls: true,
        }
    }
}

// ── Config file path ────────────────────────────────────────────────────────

fn config_path() -> PathBuf {
    crate::firewall::config::config_dir().join("eagle-eye.toml")
}

fn load_config() -> EagleEyeConfig {
    let path = config_path();
    if !path.exists() {
        return EagleEyeConfig::default();
    }
    match std::fs::read_to_string(&path) {
        Ok(content) => toml::from_str(&content).unwrap_or_default(),
        Err(_) => EagleEyeConfig::default(),
    }
}

fn save_config(config: &EagleEyeConfig) -> Result<(), String> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config dir: {}", e))?;
    }
    let content = toml::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    std::fs::write(&path, content)
        .map_err(|e| format!("Failed to write config: {}", e))
}

// ── Commands ────────────────────────────────────────────────────────────────

pub fn cmd_setup() {
    use std::io::{self, Write};

    Logger::title("EAGLE EYE SETUP", "blue");
    println!("  {}  Configure scheduled vulnerability monitoring with email alerts.\n",
        "🦅".bold());

    let mut config = load_config();

    // SMTP setup
    println!("  {}  {}\n", "1".bold().bright_cyan(), "SMTP Configuration".bold().white());

    print!("  SMTP Host [{}]: ", if config.smtp.host.is_empty() { "smtp.gmail.com" } else { &config.smtp.host });
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    let input = input.trim();
    if !input.is_empty() { config.smtp.host = input.to_string(); }
    else if config.smtp.host.is_empty() { config.smtp.host = "smtp.gmail.com".to_string(); }

    print!("  SMTP Port [{}]: ", config.smtp.port);
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    if let Ok(p) = input.trim().parse::<u16>() { config.smtp.port = p; }

    print!("  SMTP Username [{}]: ", if config.smtp.username.is_empty() { "your-email@gmail.com" } else { &config.smtp.username });
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    let input = input.trim();
    if !input.is_empty() { config.smtp.username = input.to_string(); }

    print!("  SMTP Password [{}]: ", if config.smtp.password.is_empty() { "enter password" } else { "****" });
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    let input = input.trim();
    if !input.is_empty() { config.smtp.password = input.to_string(); }

    print!("  Use TLS [{}]: ", if config.smtp.tls { "yes" } else { "no" });
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    let input = input.trim().to_lowercase();
    if input == "no" || input == "false" || input == "n" { config.smtp.tls = false; }
    else if !input.is_empty() { config.smtp.tls = true; }

    // Email addresses
    println!("\n  {}  {}\n", "2".bold().bright_cyan(), "Email Addresses".bold().white());

    print!("  From address [{}]: ", if config.from.is_empty() { &config.smtp.username } else { &config.from });
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    let input = input.trim();
    if !input.is_empty() { config.from = input.to_string(); }
    else if config.from.is_empty() { config.from = config.smtp.username.clone(); }

    let recip_display = if config.recipients.is_empty() { "admin@example.com".to_string() } else { config.recipients.join(", ") };
    print!("  Recipients (comma-separated) [{}]: ", recip_display);
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    let input = input.trim();
    if !input.is_empty() {
        config.recipients = input.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
    }

    // Scan paths
    println!("\n  {}  {}\n", "3".bold().bright_cyan(), "Project Paths to Monitor".bold().white());
    println!("  Enter full paths to project directories (one per line).");
    println!("  Eagle Eye will scan for lock files in each path.");
    println!("  Enter an empty line when done.\n");

    if !config.scan_paths.is_empty() {
        println!("  Current paths:");
        for p in &config.scan_paths {
            println!("    {} {}", "·".truecolor(60, 60, 80), p.truecolor(180, 180, 200));
        }
        print!("\n  Keep existing paths? [Y/n]: ");
        io::stdout().flush().ok();
        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();
        if input.trim().to_lowercase() == "n" {
            config.scan_paths.clear();
        }
    }

    loop {
        print!("  Path {}: ", config.scan_paths.len() + 1);
        io::stdout().flush().ok();
        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();
        let input = input.trim();
        if input.is_empty() { break; }
        config.scan_paths.push(input.to_string());
    }

    // Scan interval
    println!("\n  {}  {}\n", "4".bold().bright_cyan(), "Scan Schedule".bold().white());
    print!("  Scan interval in hours [{}]: ", config.interval_hours);
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    if let Ok(h) = input.trim().parse::<u32>() { config.interval_hours = h.max(1); }

    // Risk level selection
    println!("\n  {}  {}\n", "5".bold().bright_cyan(), "Risk Level Threshold".bold().white());
    println!("  Select which severity levels trigger an email alert:");
    println!("  {}  CRITICAL only", "[1]".bold().bright_red());
    println!("  {}  CRITICAL + HIGH", "[2]".bold().bright_red());
    println!("  {}  CRITICAL + HIGH + MEDIUM", "[3]".bold().bright_yellow());
    println!("  {}  CRITICAL + HIGH + MEDIUM + LOW", "[4]".bold().bright_green());
    println!("  {}  ALL (including INFORMATIONAL)", "[5]".bold().bright_cyan());
    println!();
    print!("  Choice [2]: ");
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    config.risk_levels = match input.trim() {
        "1" => vec!["CRITICAL".into()],
        "3" => vec!["CRITICAL".into(), "HIGH".into(), "MEDIUM".into()],
        "4" => vec!["CRITICAL".into(), "HIGH".into(), "MEDIUM".into(), "LOW".into()],
        "5" => vec!["CRITICAL".into(), "HIGH".into(), "MEDIUM".into(), "LOW".into(), "INFORMATIONAL".into()],
        _ => vec!["CRITICAL".into(), "HIGH".into()],
    };

    // Enable
    config.enabled = true;

    // Save
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
        Err(e) => Logger::error(&format!("Failed to save config: {}", e)),
    }
}

pub fn cmd_status() {
    Logger::title("EAGLE EYE STATUS", "blue");
    let config = load_config();

    let status = if config.enabled { "ENABLED".bold().bright_green().to_string() } else { "DISABLED".bold().bright_red().to_string() };
    Logger::detail("  Status:", &status);
    Logger::detail("  Config:", &config_path().display().to_string());

    if config.scan_paths.is_empty() {
        Logger::info("  No scan paths configured. Run `infynon pkg eagle-eye setup`.");
        return;
    }

    Logger::detail("  Paths:", &format!("{}", config.scan_paths.len()));
    for p in &config.scan_paths {
        println!("    {} {}", "·".truecolor(60, 60, 80), p.truecolor(180, 180, 200));
    }
    Logger::detail("  Interval:", &format!("every {} hours", config.interval_hours));
    Logger::detail("  Risk levels:", &config.risk_levels.join(", "));
    Logger::detail("  SMTP:", &format!("{}:{}", config.smtp.host, config.smtp.port));
    Logger::detail("  From:", &config.from);
    Logger::detail("  To:", &config.recipients.join(", "));
}

pub fn cmd_enable() {
    let mut config = load_config();
    config.enabled = true;
    match save_config(&config) {
        Ok(()) => Logger::success("Eagle Eye monitoring ENABLED"),
        Err(e) => Logger::error(&format!("Failed: {}", e)),
    }
}

pub fn cmd_disable() {
    let mut config = load_config();
    config.enabled = false;
    match save_config(&config) {
        Ok(()) => Logger::success("Eagle Eye monitoring DISABLED"),
        Err(e) => Logger::error(&format!("Failed: {}", e)),
    }
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
        Logger::error("SMTP not configured. Run `infynon pkg eagle-eye setup` first.");
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

    // Run first scan immediately, then loop
    run_scan_cycle(&config);

    loop {
        let sleep_secs = config.interval_hours as u64 * 3600;
        Logger::raw_dim(&format!("  Next scan in {} hours...", config.interval_hours));
        std::thread::sleep(std::time::Duration::from_secs(sleep_secs));

        // Reload config in case it changed
        let config = load_config();
        if !config.enabled {
            Logger::info("Eagle Eye has been disabled. Stopping.");
            break;
        }
        run_scan_cycle(&config);
    }
}

// ── Core scan logic ─────────────────────────────────────────────────────────

fn run_scan_cycle(config: &EagleEyeConfig) {
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string();
    println!("  {} {} Starting scan cycle...", "🔎".bold(), timestamp.truecolor(120, 120, 140));

    let mut all_findings: Vec<ScanFinding> = Vec::new();

    for path in &config.scan_paths {
        println!("  {} Scanning: {}", ">>".truecolor(255, 100, 100).bold(), path.bold());

        let original_dir = std::env::current_dir().ok();
        if std::env::set_current_dir(path).is_err() {
            println!("    {} Path not found: {}", "✘".red(), path);
            continue;
        }

        let packages = scanner::detect_locked_packages(None);
        if packages.is_empty() {
            println!("    {} No lock files found", "·".truecolor(100, 100, 120));
            if let Some(ref dir) = original_dir {
                let _ = std::env::set_current_dir(dir);
            }
            continue;
        }

        println!("    {} Found {} packages", "·".truecolor(100, 100, 120), packages.len());

        // Query OSV for vulnerabilities
        let queries: Vec<(String, String, String)> = packages.iter().map(|p| {
            (p.name.clone(), scanner_eco_to_osv(&p.ecosystem), p.version.clone())
        }).collect();

        let results = match osv::batch_query(&queries) {
            Ok(r) => r,
            Err(e) => {
                println!("    {} OSV query failed: {}", "✘".red(), e);
                if let Some(ref dir) = original_dir {
                    let _ = std::env::set_current_dir(dir);
                }
                continue;
            }
        };
        let mut path_vulns = 0;

        for (i, vuln_refs) in results.iter().enumerate() {
            if vuln_refs.is_empty() { continue; }
            let pkg = &packages[i];

            for vuln_ref in vuln_refs {
                let detail = osv::fetch_vuln_detail(&vuln_ref.id).ok();
                let severity = detail.as_ref()
                    .map(|d| classify_severity(d))
                    .unwrap_or("INFORMATIONAL");

                if !config.risk_levels.iter().any(|r| r.eq_ignore_ascii_case(severity)) {
                    continue;
                }

                path_vulns += 1;
                all_findings.push(ScanFinding {
                    project_path: path.clone(),
                    package: pkg.name.clone(),
                    version: pkg.version.clone(),
                    ecosystem: pkg.ecosystem.clone(),
                    cve_id: vuln_ref.id.clone(),
                    severity: severity.to_string(),
                    summary: detail.as_ref()
                        .and_then(|d| d.summary.clone())
                        .unwrap_or_else(|| "No description available".into()),
                    fixed_version: detail.as_ref()
                        .map(|d| osv::first_fixed_version(d))
                        .flatten()
                        .unwrap_or_default(),
                });
            }
        }

        let status = if path_vulns > 0 {
            format!("{} vulnerabilities found", path_vulns).bright_red().bold().to_string()
        } else {
            "clean".bright_green().bold().to_string()
        };
        println!("    {} {}", "✔".green(), status);

        if let Some(ref dir) = original_dir {
            let _ = std::env::set_current_dir(dir);
        }
    }

    // Summary
    println!();
    if all_findings.is_empty() {
        Logger::success(&format!("All {} projects are clean!", config.scan_paths.len()));
    } else {
        println!("  {} {} vulnerabilities found across {} project(s)\n",
            "⚠".bright_yellow().bold(),
            all_findings.len(),
            config.scan_paths.len(),
        );

        // Print summary table
        for finding in &all_findings {
            let sev_color = match finding.severity.as_str() {
                "CRITICAL" => finding.severity.bright_red().bold().to_string(),
                "HIGH" => finding.severity.red().to_string(),
                "MEDIUM" => finding.severity.yellow().to_string(),
                "LOW" => finding.severity.green().to_string(),
                _ => finding.severity.truecolor(120, 120, 140).to_string(),
            };
            println!("    [{}] {} {} @ {} — {}",
                sev_color,
                finding.cve_id.truecolor(180, 180, 200),
                finding.package.bold(),
                finding.version.truecolor(120, 120, 140),
                crate::utils::truncate_str(&finding.summary, 50),
            );
        }

        // Send email alert
        send_eagle_eye_email(config, &all_findings);
    }
    println!();
}

#[derive(Debug, Clone)]
struct ScanFinding {
    project_path: String,
    package: String,
    version: String,
    ecosystem: String,
    cve_id: String,
    severity: String,
    summary: String,
    fixed_version: String,
}

fn scanner_eco_to_osv(eco: &str) -> String {
    match eco.to_lowercase().as_str() {
        "npm" | "yarn" | "pnpm" | "bun" | "javascript" => "npm".into(),
        "pip" | "pypi" | "python" | "uv" | "poetry" => "PyPI".into(),
        "cargo" | "rust" | "crates.io" => "crates.io".into(),
        "go" | "golang" => "Go".into(),
        "gem" | "ruby" | "rubygems" => "RubyGems".into(),
        "composer" | "php" | "packagist" => "Packagist".into(),
        "nuget" | "dotnet" => "NuGet".into(),
        "hex" | "elixir" => "Hex".into(),
        "pub" | "dart" => "Pub".into(),
        other => other.into(),
    }
}

fn classify_severity(detail: &osv::OsvVulnDetail) -> &'static str {
    for s in &detail.severity {
        if let Some(ref score) = s.score {
            if let Ok(val) = score.parse::<f64>() {
                if val >= 9.0 { return "CRITICAL"; }
                if val >= 7.0 { return "HIGH"; }
                if val >= 4.0 { return "MEDIUM"; }
                if val >= 0.1 { return "LOW"; }
            }
        }
    }
    "INFORMATIONAL"
}

// ── Email ───────────────────────────────────────────────────────────────────

fn send_eagle_eye_email(config: &EagleEyeConfig, findings: &[ScanFinding]) {
    use lettre::message::{header::ContentType, Mailbox};
    use lettre::transport::smtp::authentication::Credentials;
    use lettre::{Message, SmtpTransport, Transport};

    if config.smtp.host.is_empty() || config.recipients.is_empty() || config.from.is_empty() {
        return;
    }

    let from: Mailbox = match config.from.parse() {
        Ok(m) => m,
        Err(_) => return,
    };

    let subject = format!(
        "🦅 Eagle Eye Alert: {} vulnerabilities found across your projects",
        findings.len()
    );

    let html = build_eagle_eye_html(findings, config);

    for recipient in &config.recipients {
        let to: Mailbox = match recipient.parse() {
            Ok(m) => m,
            Err(_) => continue,
        };

        let email = match Message::builder()
            .from(from.clone())
            .to(to)
            .subject(&subject)
            .header(ContentType::TEXT_HTML)
            .body(html.clone())
        {
            Ok(e) => e,
            Err(_) => continue,
        };

        let creds = Credentials::new(config.smtp.username.clone(), config.smtp.password.clone());

        let mailer = if config.smtp.tls {
            SmtpTransport::starttls_relay(&config.smtp.host)
                .ok()
                .map(|b| b.port(config.smtp.port).credentials(creds).build())
        } else {
            SmtpTransport::builder_dangerous(&config.smtp.host)
                .port(config.smtp.port)
                .credentials(creds)
                .build()
                .into()
        };

        if let Some(mailer) = mailer {
            match mailer.send(&email) {
                Ok(_) => Logger::success(&format!("Alert email sent to {}", recipient)),
                Err(e) => Logger::error(&format!("Failed to send email to {}: {}", recipient, e)),
            }
        }
    }
}

fn build_eagle_eye_html(findings: &[ScanFinding], config: &EagleEyeConfig) -> String {
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string();

    // Group by project
    let mut by_project: std::collections::BTreeMap<String, Vec<&ScanFinding>> = std::collections::BTreeMap::new();
    for f in findings {
        by_project.entry(f.project_path.clone()).or_default().push(f);
    }

    // Count by severity
    let critical = findings.iter().filter(|f| f.severity == "CRITICAL").count();
    let high = findings.iter().filter(|f| f.severity == "HIGH").count();
    let medium = findings.iter().filter(|f| f.severity == "MEDIUM").count();
    let low = findings.iter().filter(|f| f.severity == "LOW").count();

    let project_sections: String = by_project.iter().map(|(project, vulns)| {
        let rows: String = vulns.iter().map(|v| {
            let sev_color = match v.severity.as_str() {
                "CRITICAL" => "#ff4444",
                "HIGH" => "#ff6644",
                "MEDIUM" => "#ffc832",
                "LOW" => "#44cc44",
                _ => "#888888",
            };
            let fix = if v.fixed_version.is_empty() { "No fix available".to_string() } else { format!("Fix: {}", v.fixed_version) };
            format!(
                "<tr>\
                <td style='padding:8px 12px;border-bottom:1px solid #2a2a3e'><span style='color:{sev_color};font-weight:bold'>{severity}</span></td>\
                <td style='padding:8px 12px;border-bottom:1px solid #2a2a3e;color:#00d2ff'>{cve}</td>\
                <td style='padding:8px 12px;border-bottom:1px solid #2a2a3e;color:#e0e0e0'><b>{pkg}</b> @ {ver}</td>\
                <td style='padding:8px 12px;border-bottom:1px solid #2a2a3e;color:#888'>{summary}</td>\
                <td style='padding:8px 12px;border-bottom:1px solid #2a2a3e;color:#00ffa0'>{fix}</td>\
                </tr>",
                sev_color = sev_color,
                severity = v.severity,
                cve = v.cve_id,
                pkg = v.package,
                ver = v.version,
                summary = crate::utils::truncate_str(&v.summary, 60),
                fix = fix,
            )
        }).collect();

        format!(
            "<h3 style='color:#00d2ff;font-size:14px;margin:20px 0 8px;border-bottom:1px solid #2a2a3e;padding-bottom:6px'>\
            📁 {project} ({count} issue{s})</h3>\
            <table width='100%' cellpadding='0' cellspacing='0'>\
            <tr style='color:#666;font-size:11px;text-transform:uppercase'>\
            <td style='padding:4px 12px'>Severity</td><td style='padding:4px 12px'>CVE</td>\
            <td style='padding:4px 12px'>Package</td><td style='padding:4px 12px'>Description</td>\
            <td style='padding:4px 12px'>Fix</td></tr>\
            {rows}</table>",
            project = project,
            count = vulns.len(),
            s = if vulns.len() == 1 { "" } else { "s" },
            rows = rows,
        )
    }).collect();

    format!(r#"<!DOCTYPE html>
<html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width"></head>
<body style="margin:0;padding:0;background:#0a0a14;font-family:system-ui,-apple-system,sans-serif">
<table width="100%" cellpadding="0" cellspacing="0" style="background:#0a0a14;padding:20px 0">
<tr><td align="center">
<table width="640" cellpadding="0" cellspacing="0" style="background:#12121e;border-radius:8px;overflow:hidden">
  <tr><td style="background:linear-gradient(135deg,#6c3ce0 0%,#3a1d8e 100%);padding:24px 32px">
    <h1 style="margin:0;color:#fff;font-size:22px">🦅 Eagle Eye Alert</h1>
    <p style="margin:4px 0 0;color:rgba(255,255,255,0.8);font-size:13px">{total} vulnerabilities found — {timestamp}</p>
  </td></tr>
  <tr><td style="padding:24px 32px">
    <table width="100%" cellpadding="0" cellspacing="0" style="margin-bottom:24px">
      <tr>
        <td style="background:#1a1a2e;border-radius:6px;padding:14px;text-align:center;width:25%">
          <p style="margin:0;color:#888;font-size:10px;text-transform:uppercase">Critical</p>
          <p style="margin:4px 0 0;color:#ff4444;font-size:22px;font-weight:bold">{critical}</p>
        </td><td width="6"></td>
        <td style="background:#1a1a2e;border-radius:6px;padding:14px;text-align:center;width:25%">
          <p style="margin:0;color:#888;font-size:10px;text-transform:uppercase">High</p>
          <p style="margin:4px 0 0;color:#ff6644;font-size:22px;font-weight:bold">{high}</p>
        </td><td width="6"></td>
        <td style="background:#1a1a2e;border-radius:6px;padding:14px;text-align:center;width:25%">
          <p style="margin:0;color:#888;font-size:10px;text-transform:uppercase">Medium</p>
          <p style="margin:4px 0 0;color:#ffc832;font-size:22px;font-weight:bold">{medium}</p>
        </td><td width="6"></td>
        <td style="background:#1a1a2e;border-radius:6px;padding:14px;text-align:center;width:25%">
          <p style="margin:0;color:#888;font-size:10px;text-transform:uppercase">Low</p>
          <p style="margin:4px 0 0;color:#44cc44;font-size:22px;font-weight:bold">{low}</p>
        </td>
      </tr>
    </table>
    <p style="color:#888;font-size:12px;margin:0 0 4px">Monitoring {path_count} project(s) — Risk threshold: {risk_levels}</p>
    {project_sections}
  </td></tr>
  <tr><td style="background:#0e0e1a;padding:16px 32px;text-align:center">
    <p style="margin:0;color:#555;font-size:11px">🦅 Eagle Eye by <span style="color:#00d2ff">INFYNON</span> — Scheduled Vulnerability Monitoring</p>
  </td></tr>
</table>
</td></tr></table>
</body></html>"#,
        total = findings.len(),
        timestamp = timestamp,
        critical = critical,
        high = high,
        medium = medium,
        low = low,
        path_count = config.scan_paths.len(),
        risk_levels = config.risk_levels.join(", "),
        project_sections = project_sections,
    )
}
