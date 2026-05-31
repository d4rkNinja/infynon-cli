use super::html::build_eagle_eye_html;
use super::secret::resolve_smtp_password;
use super::types::{EagleEyeConfig, ScanFinding};
use crate::tui::logger::Logger;

pub(super) fn send_eagle_eye_email(config: &EagleEyeConfig, findings: &[ScanFinding]) {
    let Some(password) = resolve_smtp_password(&config.smtp) else {
        return Logger::error("SMTP password is not available. Skipping Eagle Eye email delivery.");
    };
    let subject = format!(
        "Eagle Eye Alert: {} vulnerabilities found across your projects",
        findings.len()
    );
    let html = build_eagle_eye_html(findings, config);

    crate::utils::send_smtp_email(crate::utils::SmtpEmail {
        host: &config.smtp.host,
        port: config.smtp.port,
        username: &config.smtp.username,
        password: &password,
        tls: config.smtp.tls,
        from: &config.from,
        recipients: &config.recipients,
        subject: &subject,
        html_body: &html,
    });

    for recipient in &config.recipients {
        Logger::success(&format!("Alert email sent to {}", recipient));
    }
}
