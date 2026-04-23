use super::types::{EagleEyeConfig, ScanFinding};
use std::collections::BTreeMap;

pub(super) fn build_eagle_eye_html(findings: &[ScanFinding], config: &EagleEyeConfig) -> String {
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string();
    let by_project = group_findings_by_project(findings);
    let (critical, high, medium, low) = severity_counts(findings);

    format!(
        r#"<!DOCTYPE html>
<html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width"></head>
<body style="margin:0;padding:0;background:#0a0a14;font-family:system-ui,-apple-system,sans-serif">
<table width="100%" cellpadding="0" cellspacing="0" style="background:#0a0a14;padding:20px 0"><tr><td align="center">
<table width="640" cellpadding="0" cellspacing="0" style="background:#12121e;border-radius:8px;overflow:hidden">
<tr><td style="background:linear-gradient(135deg,#6c3ce0 0%,#3a1d8e 100%);padding:24px 32px">
<h1 style="margin:0;color:#fff;font-size:22px">Eagle Eye Alert</h1>
<p style="margin:4px 0 0;color:rgba(255,255,255,0.8);font-size:13px">{total} vulnerabilities found - {timestamp}</p>
</td></tr>
<tr><td style="padding:24px 32px">
{summary}
<p style="color:#888;font-size:12px;margin:0 0 4px">Monitoring {path_count} project(s) - Risk threshold: {risk_levels}</p>
{projects}
</td></tr>
<tr><td style="background:#0e0e1a;padding:16px 32px;text-align:center">
<p style="margin:0;color:#555;font-size:11px">Eagle Eye by <span style="color:#00d2ff">INFYNON</span> - Scheduled Vulnerability Monitoring</p>
</td></tr></table></td></tr></table></body></html>"#,
        total = findings.len(),
        timestamp = timestamp,
        summary = render_summary(critical, high, medium, low),
        path_count = config.scan_paths.len(),
        risk_levels = config.risk_levels.join(", "),
        projects = render_project_sections(&by_project),
    )
}

fn group_findings_by_project<'a>(
    findings: &'a [ScanFinding],
) -> BTreeMap<String, Vec<&'a ScanFinding>> {
    let mut grouped = BTreeMap::new();
    for finding in findings {
        grouped
            .entry(finding.project_path.clone())
            .or_insert_with(Vec::new)
            .push(finding);
    }
    grouped
}

fn severity_counts(findings: &[ScanFinding]) -> (usize, usize, usize, usize) {
    (
        findings
            .iter()
            .filter(|item| item.severity == "CRITICAL")
            .count(),
        findings
            .iter()
            .filter(|item| item.severity == "HIGH")
            .count(),
        findings
            .iter()
            .filter(|item| item.severity == "MEDIUM")
            .count(),
        findings
            .iter()
            .filter(|item| item.severity == "LOW")
            .count(),
    )
}

fn render_summary(critical: usize, high: usize, medium: usize, low: usize) -> String {
    let cards = [
        ("Critical", "#ff4444", critical),
        ("High", "#ff6644", high),
        ("Medium", "#ffc832", medium),
        ("Low", "#44cc44", low),
    ];
    let mut parts = Vec::new();
    for (label, color, count) in cards {
        parts.push(format!(
            "<td style='background:#1a1a2e;border-radius:6px;padding:14px;text-align:center;width:25%'><p style='margin:0;color:#888;font-size:10px;text-transform:uppercase'>{label}</p><p style='margin:4px 0 0;color:{color};font-size:22px;font-weight:bold'>{count}</p></td>"
        ));
    }
    format!("<table width='100%' cellpadding='0' cellspacing='0' style='margin-bottom:24px'><tr>{}</tr></table>", parts.join("<td width='6'></td>"))
}

fn render_project_sections(grouped: &BTreeMap<String, Vec<&ScanFinding>>) -> String {
    grouped
        .iter()
        .map(|(project, vulns)| {
            format!(
                "<h3 style='color:#00d2ff;font-size:14px;margin:20px 0 8px;border-bottom:1px solid #2a2a3e;padding-bottom:6px'>📁 {project} ({count} issue{s})</h3><table width='100%' cellpadding='0' cellspacing='0'><tr style='color:#666;font-size:11px;text-transform:uppercase'><td style='padding:4px 12px'>Severity</td><td style='padding:4px 12px'>CVE</td><td style='padding:4px 12px'>Package</td><td style='padding:4px 12px'>Description</td><td style='padding:4px 12px'>Fix</td></tr>{rows}</table>",
                count = vulns.len(),
                s = if vulns.len() == 1 { "" } else { "s" },
                rows = vulns.iter().map(|finding| render_row(finding)).collect::<String>(),
            )
        })
        .collect()
}

fn render_row(finding: &ScanFinding) -> String {
    let fix = if finding.fixed_version.is_empty() {
        "No fix available".to_string()
    } else {
        format!("Fix: {}", finding.fixed_version)
    };
    format!(
        "<tr><td style='padding:8px 12px;border-bottom:1px solid #2a2a3e'><span style='color:{color};font-weight:bold'>{severity}</span></td><td style='padding:8px 12px;border-bottom:1px solid #2a2a3e;color:#00d2ff'>{cve}</td><td style='padding:8px 12px;border-bottom:1px solid #2a2a3e;color:#e0e0e0'><b>{pkg}</b> @ {ver} <span style='color:#777'>[{eco}]</span></td><td style='padding:8px 12px;border-bottom:1px solid #2a2a3e;color:#888'>{summary}</td><td style='padding:8px 12px;border-bottom:1px solid #2a2a3e;color:#00ffa0'>{fix}</td></tr>",
        color = severity_color(&finding.severity),
        severity = finding.severity,
        cve = finding.cve_id,
        pkg = finding.package,
        ver = finding.version,
        eco = finding.ecosystem,
        summary = crate::utils::truncate_str(&finding.summary, 60),
        fix = fix,
    )
}

pub(super) fn severity_color(severity: &str) -> &'static str {
    match severity {
        "CRITICAL" => "#ff4444",
        "HIGH" => "#ff6644",
        "MEDIUM" => "#ffc832",
        "LOW" => "#44cc44",
        _ => "#888888",
    }
}
