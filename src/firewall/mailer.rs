use std::sync::Arc;
use chrono::{Utc, Datelike, Timelike};
use lettre::message::{header::ContentType, Mailbox};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

use crate::firewall::config::EmailConfig;
use crate::firewall::stats::StatsSnapshot;

/// Send an alert email when suspicious activity is detected.
pub fn send_alert_email(
    config: &EmailConfig,
    subject: &str,
    alert_type: &str,
    details: &str,
    top_ips: &[(String, u64)],
    top_rules: &[(String, u64)],
) {
    if !config.enabled || config.to.is_empty() || config.from.is_empty() {
        return;
    }

    let html = build_alert_html(subject, alert_type, details, top_ips, top_rules);
    send_email(config, subject, &html);
}

/// Send a daily digest email with full day stats.
pub fn send_daily_digest(
    config: &EmailConfig,
    snapshot: &StatsSnapshot,
    top_blocked_ips: &[(String, u64)],
    top_rules: &[(String, u64)],
    top_paths: &[(String, u64)],
) {
    if !config.enabled || !config.daily_digest || config.to.is_empty() || config.from.is_empty() {
        return;
    }

    let today = Utc::now().format("%Y-%m-%d").to_string();
    let subject = format!("INFYNON Daily Report — {}", today);
    let html = build_digest_html(snapshot, top_blocked_ips, top_rules, top_paths, &today);
    send_email(config, &subject, &html);
}

fn send_email(config: &EmailConfig, subject: &str, html_body: &str) {
    let from: Mailbox = match config.from.parse() {
        Ok(m) => m,
        Err(_) => return,
    };

    for recipient in &config.to {
        let to: Mailbox = match recipient.parse() {
            Ok(m) => m,
            Err(_) => continue,
        };

        let email = match Message::builder()
            .from(from.clone())
            .to(to)
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(html_body.to_string())
        {
            Ok(e) => e,
            Err(_) => continue,
        };

        // Send via SMTP
        if config.provider == "smtp" || config.provider == "ses" {
            let host = if config.provider == "ses" && !config.ses.region.is_empty() {
                format!("email-smtp.{}.amazonaws.com", config.ses.region)
            } else {
                config.smtp.host.clone()
            };

            let (username, password) = if config.provider == "ses" {
                (config.ses.access_key_id.clone(), config.ses.secret_access_key.clone())
            } else {
                (config.smtp.username.clone(), config.smtp.password.clone())
            };

            if host.is_empty() { return; }

            let creds = Credentials::new(username, password);

            let port = if config.provider == "ses" { 587 } else { config.smtp.port };

            let mailer = if config.smtp.tls || config.provider == "ses" {
                SmtpTransport::starttls_relay(&host)
                    .ok()
                    .map(|b| b.port(port).credentials(creds).build())
            } else {
                SmtpTransport::builder_dangerous(&host)
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
}

/// Spawn the daily digest scheduler. Runs forever, sends digest at configured hour.
pub async fn daily_digest_loop(state: Arc<crate::firewall::server::SharedState>) {
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;

        let (enabled, digest_hour) = {
            match state.config.read() {
                Ok(cfg) => (cfg.email.enabled && cfg.email.daily_digest, cfg.email.daily_digest_hour),
                Err(_) => continue,
            }
        };

        if !enabled { continue; }

        let now = Utc::now();
        if now.hour() == digest_hour && now.minute() == 0 {
            let snapshot = state.stats.snapshot();
            let email_config = match state.config.read() {
                Ok(cfg) => cfg.email.clone(),
                Err(_) => continue,
            };

            send_daily_digest(
                &email_config,
                &snapshot,
                &snapshot.top_blocked_ips,
                &snapshot.top_rules,
                &snapshot.top_paths,
            );

            // Sleep 61 seconds to avoid sending twice in the same minute
            tokio::time::sleep(std::time::Duration::from_secs(61)).await;
        }
    }
}

/// Check if alert threshold is crossed and send email.
pub fn check_and_alert(state: &crate::firewall::server::SharedState, snapshot: &StatsSnapshot) {
    let email_config = match state.config.read() {
        Ok(cfg) => {
            if !cfg.email.enabled || cfg.email.alert_on_block_threshold == 0 {
                return;
            }
            cfg.email.clone()
        }
        Err(_) => return,
    };

    // Check blocks/minute threshold
    let blocks_per_min = snapshot.blocks_per_second * 60.0;
    if blocks_per_min >= email_config.alert_on_block_threshold as f64 {
        send_alert_email(
            &email_config,
            &format!("INFYNON Alert: High block rate ({:.0}/min)", blocks_per_min),
            "High Block Rate",
            &format!(
                "Block rate has exceeded threshold: {:.0} blocks/min (threshold: {})",
                blocks_per_min, email_config.alert_on_block_threshold
            ),
            &snapshot.top_blocked_ips,
            &snapshot.top_rules,
        );
    }
}

// ── HTML Email Templates ────────────────────────────────────────────────────

fn build_alert_html(
    subject: &str,
    alert_type: &str,
    details: &str,
    top_ips: &[(String, u64)],
    top_rules: &[(String, u64)],
) -> String {
    let ip_rows: String = top_ips.iter().take(10)
        .map(|(ip, count)| format!(
            "<tr><td style='padding:6px 12px;border-bottom:1px solid #2a2a3e;color:#e0e0e0'>{}</td>\
             <td style='padding:6px 12px;border-bottom:1px solid #2a2a3e;color:#ff4444;font-weight:bold;text-align:right'>{}</td></tr>",
            ip, count
        ))
        .collect();

    let rule_rows: String = top_rules.iter().take(10)
        .map(|(name, count)| format!(
            "<tr><td style='padding:6px 12px;border-bottom:1px solid #2a2a3e;color:#e0e0e0'>{}</td>\
             <td style='padding:6px 12px;border-bottom:1px solid #2a2a3e;color:#ffc832;font-weight:bold;text-align:right'>{}</td></tr>",
            name, count
        ))
        .collect();

    format!(r#"<!DOCTYPE html>
<html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width"></head>
<body style="margin:0;padding:0;background:#0a0a14;font-family:system-ui,-apple-system,sans-serif">
<table width="100%" cellpadding="0" cellspacing="0" style="background:#0a0a14;padding:20px 0">
<tr><td align="center">
<table width="600" cellpadding="0" cellspacing="0" style="background:#12121e;border-radius:8px;overflow:hidden">
  <tr><td style="background:linear-gradient(135deg,#ff4444 0%,#cc2200 100%);padding:24px 32px">
    <h1 style="margin:0;color:#fff;font-size:20px">🛡️ INFYNON ALERT</h1>
    <p style="margin:4px 0 0;color:rgba(255,255,255,0.8);font-size:13px">{alert_type}</p>
  </td></tr>
  <tr><td style="padding:24px 32px">
    <p style="color:#e0e0e0;font-size:14px;line-height:1.6;margin:0 0 16px">{details}</p>
    <p style="color:#888;font-size:12px;margin:0 0 20px">Timestamp: {timestamp} UTC</p>

    <h3 style="color:#00d2ff;font-size:14px;margin:20px 0 8px;border-bottom:1px solid #2a2a3e;padding-bottom:6px">Top Blocked IPs</h3>
    <table width="100%" cellpadding="0" cellspacing="0">{ip_rows}</table>

    <h3 style="color:#00d2ff;font-size:14px;margin:20px 0 8px;border-bottom:1px solid #2a2a3e;padding-bottom:6px">Top Triggered Rules</h3>
    <table width="100%" cellpadding="0" cellspacing="0">{rule_rows}</table>
  </td></tr>
  <tr><td style="background:#0e0e1a;padding:16px 32px;text-align:center">
    <p style="margin:0;color:#555;font-size:11px">Protected by <span style="color:#00d2ff">INFYNON</span> Firewall</p>
  </td></tr>
</table>
</td></tr></table>
</body></html>"#,
        alert_type = alert_type,
        details = details,
        timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S"),
        ip_rows = ip_rows,
        rule_rows = rule_rows,
    )
}

fn build_digest_html(
    snapshot: &StatsSnapshot,
    top_blocked_ips: &[(String, u64)],
    top_rules: &[(String, u64)],
    top_paths: &[(String, u64)],
    date: &str,
) -> String {
    let block_rate = if snapshot.total_requests > 0 {
        snapshot.total_blocked as f64 / snapshot.total_requests as f64 * 100.0
    } else {
        0.0
    };

    let ip_rows: String = top_blocked_ips.iter().take(15)
        .map(|(ip, count)| format!(
            "<tr><td style='padding:6px 12px;border-bottom:1px solid #2a2a3e;color:#e0e0e0'>{}</td>\
             <td style='padding:6px 12px;border-bottom:1px solid #2a2a3e;color:#ff4444;font-weight:bold;text-align:right'>{}</td></tr>",
            ip, count
        ))
        .collect();

    let rule_rows: String = top_rules.iter().take(15)
        .map(|(name, count)| format!(
            "<tr><td style='padding:6px 12px;border-bottom:1px solid #2a2a3e;color:#e0e0e0'>{}</td>\
             <td style='padding:6px 12px;border-bottom:1px solid #2a2a3e;color:#ffc832;font-weight:bold;text-align:right'>{}</td></tr>",
            name, count
        ))
        .collect();

    let path_rows: String = top_paths.iter().take(15)
        .map(|(path, count)| format!(
            "<tr><td style='padding:6px 12px;border-bottom:1px solid #2a2a3e;color:#e0e0e0'>{}</td>\
             <td style='padding:6px 12px;border-bottom:1px solid #2a2a3e;color:#00d2ff;font-weight:bold;text-align:right'>{}</td></tr>",
            path, count
        ))
        .collect();

    format!(r#"<!DOCTYPE html>
<html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width"></head>
<body style="margin:0;padding:0;background:#0a0a14;font-family:system-ui,-apple-system,sans-serif">
<table width="100%" cellpadding="0" cellspacing="0" style="background:#0a0a14;padding:20px 0">
<tr><td align="center">
<table width="600" cellpadding="0" cellspacing="0" style="background:#12121e;border-radius:8px;overflow:hidden">
  <tr><td style="background:linear-gradient(135deg,#00d2ff 0%,#0080aa 100%);padding:24px 32px">
    <h1 style="margin:0;color:#fff;font-size:20px">🛡️ INFYNON Daily Report</h1>
    <p style="margin:4px 0 0;color:rgba(255,255,255,0.8);font-size:13px">{date}</p>
  </td></tr>
  <tr><td style="padding:24px 32px">

    <table width="100%" cellpadding="0" cellspacing="0" style="margin-bottom:24px">
      <tr>
        <td style="background:#1a1a2e;border-radius:6px;padding:16px;text-align:center;width:33%">
          <p style="margin:0;color:#888;font-size:11px;text-transform:uppercase">Total Requests</p>
          <p style="margin:4px 0 0;color:#00d2ff;font-size:24px;font-weight:bold">{total_requests}</p>
        </td>
        <td width="8"></td>
        <td style="background:#1a1a2e;border-radius:6px;padding:16px;text-align:center;width:33%">
          <p style="margin:0;color:#888;font-size:11px;text-transform:uppercase">Blocked</p>
          <p style="margin:4px 0 0;color:#ff4444;font-size:24px;font-weight:bold">{total_blocked}</p>
        </td>
        <td width="8"></td>
        <td style="background:#1a1a2e;border-radius:6px;padding:16px;text-align:center;width:33%">
          <p style="margin:0;color:#888;font-size:11px;text-transform:uppercase">Block Rate</p>
          <p style="margin:4px 0 0;color:#ffc832;font-size:24px;font-weight:bold">{block_rate:.1}%</p>
        </td>
      </tr>
    </table>

    <table width="100%" cellpadding="0" cellspacing="0" style="margin-bottom:24px">
      <tr>
        <td style="background:#1a1a2e;border-radius:6px;padding:16px;text-align:center;width:33%">
          <p style="margin:0;color:#888;font-size:11px;text-transform:uppercase">Rate Limited</p>
          <p style="margin:4px 0 0;color:#ff8c32;font-size:20px;font-weight:bold">{rate_limited}</p>
        </td>
        <td width="8"></td>
        <td style="background:#1a1a2e;border-radius:6px;padding:16px;text-align:center;width:33%">
          <p style="margin:0;color:#888;font-size:11px;text-transform:uppercase">Flagged</p>
          <p style="margin:4px 0 0;color:#ffc832;font-size:20px;font-weight:bold">{flagged}</p>
        </td>
        <td width="8"></td>
        <td style="background:#1a1a2e;border-radius:6px;padding:16px;text-align:center;width:33%">
          <p style="margin:0;color:#888;font-size:11px;text-transform:uppercase">Uptime</p>
          <p style="margin:4px 0 0;color:#00ffa0;font-size:20px;font-weight:bold">{uptime}</p>
        </td>
      </tr>
    </table>

    <h3 style="color:#ff4444;font-size:14px;margin:20px 0 8px;border-bottom:1px solid #2a2a3e;padding-bottom:6px">Top Blocked IPs</h3>
    <table width="100%" cellpadding="0" cellspacing="0">{ip_rows}</table>

    <h3 style="color:#ffc832;font-size:14px;margin:20px 0 8px;border-bottom:1px solid #2a2a3e;padding-bottom:6px">Top Triggered Rules</h3>
    <table width="100%" cellpadding="0" cellspacing="0">{rule_rows}</table>

    <h3 style="color:#00d2ff;font-size:14px;margin:20px 0 8px;border-bottom:1px solid #2a2a3e;padding-bottom:6px">Top Requested Paths</h3>
    <table width="100%" cellpadding="0" cellspacing="0">{path_rows}</table>

  </td></tr>
  <tr><td style="background:#0e0e1a;padding:16px 32px;text-align:center">
    <p style="margin:0;color:#555;font-size:11px">Protected by <span style="color:#00d2ff">INFYNON</span> Firewall &mdash; Daily Digest</p>
  </td></tr>
</table>
</td></tr></table>
</body></html>"#,
        date = date,
        total_requests = crate::utils::format_number(snapshot.total_requests),
        total_blocked = crate::utils::format_number(snapshot.total_blocked),
        block_rate = block_rate,
        rate_limited = crate::utils::format_number(snapshot.total_rate_limited),
        flagged = crate::utils::format_number(snapshot.total_flagged),
        uptime = snapshot.format_uptime(),
        ip_rows = ip_rows,
        rule_rows = rule_rows,
        path_rows = path_rows,
    )
}
