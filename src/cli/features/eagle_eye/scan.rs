use super::email::send_eagle_eye_email;
use super::types::{EagleEyeConfig, ScanFinding};
use crate::engine::{osv, scanner};
use crate::tui::logger::Logger;
use owo_colors::OwoColorize;

pub(super) fn run_scan_cycle(config: &EagleEyeConfig) {
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string();
    println!("  {} {} Starting scan cycle...", "🔎".bold(), timestamp.truecolor(120, 120, 140));

    let mut all_findings = Vec::new();
    for path in &config.scan_paths {
        println!("  {} Scanning: {}", ">>".truecolor(255, 100, 100).bold(), path.bold());
        all_findings.extend(scan_path(path, config));
    }

    println!();
    if all_findings.is_empty() {
        Logger::success(&format!("All {} projects are clean!", config.scan_paths.len()));
    } else {
        println!(
            "  {} {} vulnerabilities found across {} project(s)\n",
            "⚠".bright_yellow().bold(),
            all_findings.len(),
            config.scan_paths.len(),
        );
        for finding in &all_findings {
            let severity = match finding.severity.as_str() {
                "CRITICAL" => finding.severity.bright_red().bold().to_string(),
                "HIGH" => finding.severity.red().to_string(),
                "MEDIUM" => finding.severity.yellow().to_string(),
                "LOW" => finding.severity.green().to_string(),
                _ => finding.severity.truecolor(120, 120, 140).to_string(),
            };
            println!(
                "    [{}] {} {} @ {} — {}",
                severity,
                finding.cve_id.truecolor(180, 180, 200),
                finding.package.bold(),
                finding.version.truecolor(120, 120, 140),
                crate::utils::truncate_str(&finding.summary, 50),
            );
        }
        send_eagle_eye_email(config, &all_findings);
    }
    println!();
}

fn scan_path(path: &str, config: &EagleEyeConfig) -> Vec<ScanFinding> {
    let original_dir = std::env::current_dir().ok();
    if std::env::set_current_dir(path).is_err() {
        println!("    {} Path not found: {}", "✘".red(), path);
        return Vec::new();
    }

    let packages = scanner::detect_locked_packages(None);
    if packages.is_empty() {
        println!("    {} No lock files found", "·".truecolor(100, 100, 120));
        restore_dir(original_dir.as_ref());
        return Vec::new();
    }
    println!("    {} Found {} packages", "·".truecolor(100, 100, 120), packages.len());

    let queries: Vec<(String, String, String)> = packages
        .iter()
        .map(|pkg| (pkg.name.clone(), pkg.ecosystem.clone(), pkg.version.clone()))
        .collect();
    let results = match osv::batch_query(&queries) {
        Ok(value) => value,
        Err(err) => {
            println!("    {} OSV query failed: {}", "✘".red(), err);
            restore_dir(original_dir.as_ref());
            return Vec::new();
        }
    };

    let vuln_ids: Vec<String> = results.iter().flat_map(|refs| refs.iter().map(|item| item.id.clone())).collect();
    let details = osv::fetch_vuln_details_batch(&vuln_ids)
        .into_iter()
        .filter_map(|(id, result)| result.ok().map(|detail| (id, detail)))
        .collect::<std::collections::HashMap<_, _>>();

    let mut findings = Vec::new();
    for (index, refs) in results.iter().enumerate() {
        for vuln_ref in refs {
            let detail = details.get(&vuln_ref.id);
            let severity = detail.map(osv::severity_label).unwrap_or("INFORMATIONAL");
            if !config.risk_levels.iter().any(|level| level.eq_ignore_ascii_case(severity)) {
                continue;
            }
            let pkg = &packages[index];
            findings.push(ScanFinding {
                project_path: path.to_string(),
                package: pkg.name.clone(),
                version: pkg.version.clone(),
                ecosystem: pkg.ecosystem.clone(),
                cve_id: vuln_ref.id.clone(),
                severity: severity.to_string(),
                summary: detail
                    .and_then(|item| item.summary.clone())
                    .unwrap_or_else(|| "No description available".into()),
                fixed_version: detail.and_then(osv::first_fixed_version).unwrap_or_default(),
            });
        }
    }

    let status = if findings.is_empty() {
        "clean".bright_green().bold().to_string()
    } else {
        format!("{} vulnerabilities found", findings.len()).bright_red().bold().to_string()
    };
    println!("    {} {}", "✔".green(), status);
    restore_dir(original_dir.as_ref());
    findings
}

fn restore_dir(original_dir: Option<&std::path::PathBuf>) {
    if let Some(dir) = original_dir {
        let _ = std::env::set_current_dir(dir);
    }
}
