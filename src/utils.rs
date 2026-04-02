/// Truncate a string to `max` characters, appending "..." if truncated.
pub fn truncate_str(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}...", &s[..max.saturating_sub(3)])
    } else {
        s.to_string()
    }
}

/// Format byte count in human-readable form (e.g. "1.5 GB", "10.0 MB").
pub fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_073_741_824 { format!("{:.1} GB", bytes as f64 / 1_073_741_824.0) }
    else if bytes >= 1_048_576 { format!("{:.1} MB", bytes as f64 / 1_048_576.0) }
    else if bytes >= 1024 { format!("{:.1} KB", bytes as f64 / 1024.0) }
    else { format!("{} B", bytes) }
}

/// Format byte count in compact form without spaces (e.g. "10MB", "4KB").
pub fn format_bytes_short(bytes: u64) -> String {
    if bytes >= 1_048_576 { format!("{:.0}MB", bytes as f64 / 1_048_576.0) }
    else if bytes >= 1024 { format!("{:.0}KB", bytes as f64 / 1024.0) }
    else { format!("{}B", bytes) }
}

/// Format a large number with K/M suffixes (e.g. "1.5K", "2.3M").
pub fn format_number(n: u64) -> String {
    if n >= 1_000_000 { format!("{:.1}M", n as f64 / 1_000_000.0) }
    else if n >= 1_000 { format!("{:.1}K", n as f64 / 1_000.0) }
    else { n.to_string() }
}

/// Send an HTML email via SMTP. Used by Eagle Eye email notifications.
pub fn send_smtp_email(
    host: &str,
    port: u16,
    username: &str,
    password: &str,
    tls: bool,
    from: &str,
    recipients: &[String],
    subject: &str,
    html_body: &str,
) {
    use lettre::message::{header::ContentType, Mailbox};
    use lettre::transport::smtp::authentication::Credentials;
    use lettre::{Message, SmtpTransport, Transport};

    if host.is_empty() || recipients.is_empty() || from.is_empty() {
        return;
    }

    let from_mailbox: Mailbox = match from.parse() {
        Ok(m) => m,
        Err(_) => return,
    };

    for recipient in recipients {
        let to: Mailbox = match recipient.parse() {
            Ok(m) => m,
            Err(_) => continue,
        };

        let email = match Message::builder()
            .from(from_mailbox.clone())
            .to(to)
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(html_body.to_string())
        {
            Ok(e) => e,
            Err(_) => continue,
        };

        let creds = Credentials::new(username.to_string(), password.to_string());

        let mailer = if tls {
            SmtpTransport::starttls_relay(host)
                .ok()
                .map(|b| b.port(port).credentials(creds).build())
        } else {
            SmtpTransport::builder_dangerous(host)
                .port(port)
                .credentials(creds)
                .build()
                .into()
        };

        if let Some(mailer) = mailer {
            let _ = mailer.send(&email);
        }
    }
}
