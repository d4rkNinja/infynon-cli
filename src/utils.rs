use reqwest::blocking::Client;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Duration;

/// Shared HTTP client for all blocking network calls.
pub fn http_client() -> &'static Client {
    static CLIENT: OnceLock<Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        let ua = format!(
            "infynon/{} (https://github.com/d4rkNinja/infynon-cli)",
            env!("CARGO_PKG_VERSION")
        );
        Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent(ua)
            .build()
            .unwrap_or_default()
    })
}

/// Project-local `.infynon` directory.
pub fn project_infynon_dir() -> PathBuf {
    PathBuf::from(".infynon")
}

/// Project-local path under `.infynon`.
pub fn project_infynon_path(parts: &[&str]) -> PathBuf {
    parts
        .iter()
        .fold(project_infynon_dir(), |path, part| path.join(part))
}

pub fn ensure_dir(path: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(path)
}

pub fn ensure_parent_dir(path: &Path) -> std::io::Result<()> {
    match path.parent() {
        Some(parent) => ensure_dir(parent),
        None => Ok(()),
    }
}

pub fn user_home_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        std::env::var_os("USERPROFILE")
            .or_else(|| std::env::var_os("HOME"))
            .map(PathBuf::from)
    }

    #[cfg(not(windows))]
    {
        std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .map(PathBuf::from)
    }
}

/// User-global `.infynon` directory.
pub fn home_infynon_dir() -> PathBuf {
    user_home_dir()
        .map(|home| home.join(".infynon"))
        .unwrap_or_else(project_infynon_dir)
}

pub fn storage_key(input: &str) -> String {
    let mut out = String::with_capacity(input.len() * 2 + 3);
    out.push_str("id-");
    for byte in input.as_bytes() {
        out.push_str(&format!("{:02x}", byte));
    }
    out
}

pub fn is_portable_file_stem(input: &str) -> bool {
    !input.is_empty()
        && input
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Return a filesystem-safe, cross-platform file stem for user-controlled IDs.
pub fn safe_file_stem(input: &str) -> String {
    if is_portable_file_stem(input) {
        return input.to_string();
    }

    storage_key(input)
}

pub fn json_pretty<T: Serialize>(value: &T) -> String {
    match serde_json::to_string_pretty(value) {
        Ok(json) => json,
        Err(err) => {
            let details = serde_json::to_string(&err.to_string())
                .unwrap_or_else(|_| "\"serialization failed\"".to_string());
            format!(
                "{{\"status\":\"error\",\"error\":\"failed to serialize JSON output\",\"details\":{}}}",
                details
            )
        }
    }
}

pub fn print_json_pretty<T: Serialize>(value: &T) {
    println!("{}", json_pretty(value));
}

/// Truncate a string to `max` characters, appending "..." if truncated.
pub fn truncate_str(s: &str, max: usize) -> String {
    let len = s.chars().count();
    if len > max {
        let take = max.saturating_sub(3);
        format!("{}...", s.chars().take(take).collect::<String>())
    } else {
        s.to_string()
    }
}

/// Format byte count in human-readable form (e.g. "1.5 GB", "10.0 MB").
pub fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1} GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

/// Format byte count in compact form without spaces (e.g. "10MB", "4KB").
pub fn format_bytes_short(bytes: u64) -> String {
    if bytes >= 1_048_576 {
        format!("{:.0}MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.0}KB", bytes as f64 / 1024.0)
    } else {
        format!("{}B", bytes)
    }
}

/// Format a large number with K/M suffixes (e.g. "1.5K", "2.3M").
pub fn format_number(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

/// Send an HTML email via SMTP. Used by Eagle Eye email notifications.
pub struct SmtpEmail<'a> {
    pub host: &'a str,
    pub port: u16,
    pub username: &'a str,
    pub password: &'a str,
    pub tls: bool,
    pub from: &'a str,
    pub recipients: &'a [String],
    pub subject: &'a str,
    pub html_body: &'a str,
}

/// Send an HTML email via SMTP. Used by Eagle Eye email notifications.
pub fn send_smtp_email(message: SmtpEmail<'_>) {
    use lettre::message::{header::ContentType, Mailbox};
    use lettre::transport::smtp::authentication::Credentials;
    use lettre::{Message, SmtpTransport, Transport};

    if message.host.is_empty() || message.recipients.is_empty() || message.from.is_empty() {
        return;
    }

    let from_mailbox: Mailbox = match message.from.parse() {
        Ok(m) => m,
        Err(_) => return,
    };

    for recipient in message.recipients {
        let to: Mailbox = match recipient.parse() {
            Ok(m) => m,
            Err(_) => continue,
        };

        let email = match Message::builder()
            .from(from_mailbox.clone())
            .to(to)
            .subject(message.subject)
            .header(ContentType::TEXT_HTML)
            .body(message.html_body.to_string())
        {
            Ok(e) => e,
            Err(_) => continue,
        };

        let creds = Credentials::new(message.username.to_string(), message.password.to_string());

        let mailer = if message.tls {
            SmtpTransport::starttls_relay(message.host)
                .ok()
                .map(|b| b.port(message.port).credentials(creds).build())
        } else {
            SmtpTransport::builder_dangerous(message.host)
                .port(message.port)
                .credentials(creds)
                .build()
                .into()
        };

        if let Some(mailer) = mailer {
            let _ = mailer.send(&email);
        }
    }
}
