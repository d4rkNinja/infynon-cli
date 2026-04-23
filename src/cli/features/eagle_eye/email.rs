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

    crate::utils::send_smtp_email(
        &config.smtp.host,
        config.smtp.port,
        &config.smtp.username,
        &password,
        config.smtp.tls,
        &config.from,
        &config.recipients,
        &subject,
        &html,
    );

    for recipient in &config.recipients {
        Logger::success(&format!("Alert email sent to {}", recipient));
    }
}
