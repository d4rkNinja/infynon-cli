use crate::engine::osv;
use crate::tui::logger::Logger;
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use std::time::Duration;

pub fn tool_to_osv_ecosystem(ecosystem: &str) -> String {
    match ecosystem {
        "pip" | "pip3" | "uv" | "poetry" => "PyPI".to_string(),
        "yarn" | "pnpm" | "bun" => "npm".to_string(),
        "cargo" => "crates.io".to_string(),
        "go" => "Go".to_string(),
        "gem" => "RubyGems".to_string(),
        "composer" => "Packagist".to_string(),
        "nuget" | "dotnet" => "NuGet".to_string(),
        "hex" | "mix" => "Hex".to_string(),
        "pub" | "dart" => "pub.dev".to_string(),
        other => other.to_string(),
    }
}

pub fn check_packages_before_install(names: &[String], ecosystem: &str, agent: bool) -> Result<(bool, Vec<crate::cli::scan::VulnHit>), String> {
    let sp = ProgressBar::new_spinner();
    if !agent {
        sp.set_style(ProgressStyle::with_template("  {spinner:.cyan}  {msg}").unwrap().tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]));
        sp.enable_steady_tick(Duration::from_millis(60));
    }
    let osv_eco = tool_to_osv_ecosystem(ecosystem);
    let tuples: Vec<(String, String, String)> = names.iter().map(|spec| resolve_spec(spec, &osv_eco, agent, &sp)).collect::<Result<_, _>>()?;
    if !agent { sp.set_message(format!("Checking {} package(s) against vulnerability database...", names.len())); }
    let results = osv::batch_query(&tuples).map_err(|err| { if !agent { sp.finish_and_clear(); Logger::raw_dim(&format!("  CVE check skipped (API error): {}", err)); } format!("vulnerability check failed: {}", err) })?;
    if !agent { sp.finish_and_clear(); }
    if results.iter().all(|refs| refs.is_empty()) {
        if !agent { Logger::success("Security check passed — no known CVEs for requested packages."); }
        return Ok((true, vec![]));
    }
    Ok((false, collect_hits(&tuples, &results, ecosystem, agent)))
}

pub fn escalate_severity(current: &'static str, new: &'static str) -> &'static str {
    let rank = |value: &str| match value { "CRITICAL" => 4, "HIGH" => 3, "MEDIUM" => 2, "LOW" => 1, _ => 0 };
    if rank(new) > rank(current) { new } else { current }
}

fn resolve_spec(spec: &str, osv_eco: &str, agent: bool, sp: &ProgressBar) -> Result<(String, String, String), String> {
    let (name, version) = crate::cli::scan::parse_pkg_spec(spec);
    if !version.is_empty() { return Ok((name, osv_eco.to_string(), version)); }
    if !agent { sp.set_message(format!("Resolving latest version of {}...", name)); }
    crate::engine::registry::fetch_latest_version(&name, osv_eco)
        .map(|latest| (name.clone(), osv_eco.to_string(), latest))
        .ok_or_else(|| format!("could not resolve latest version for '{}' while verifying install safety", name))
}

fn collect_hits(tuples: &[(String, String, String)], results: &[Vec<osv::OsvVulnRef>], ecosystem: &str, agent: bool) -> Vec<crate::cli::scan::VulnHit> {
    let mut hits = Vec::new();
    for (index, refs) in results.iter().enumerate() {
        if refs.is_empty() { continue; }
        let (pkg_name, pkg_ver) = (&tuples[index].0, &tuples[index].2);
        if !agent { print_pkg_warning(pkg_name, pkg_ver, refs.len()); }
        let mut infos = Vec::new();
        let mut candidates = Vec::new();
        for vuln_ref in refs.iter().take(5) {
            match osv::fetch_vuln_detail(&vuln_ref.id) {
                Ok(detail) => { let severity = osv::severity_label(&detail); let fixed = osv::best_fixed_version(&detail, pkg_name, &tuples[index].1); let summary = detail.summary.clone().unwrap_or_else(|| "No description available".into()); if let Some(ref value) = fixed { candidates.push(value.clone()); } infos.push((vuln_ref.id.clone(), severity, summary, fixed)); }
                Err(_) => hits.push(crate::cli::scan::VulnHit { package: pkg_name.clone(), cve_id: vuln_ref.id.clone(), severity: "UNKNOWN", summary: "Could not fetch CVE details".into(), fixed_version: None, upgrade_cmd: None, fix_is_clean: false }),
            }
        }
        let validated = osv::find_safest_candidate_vs(&candidates, pkg_name, &tuples[index].1, pkg_ver);
        for (id, severity, summary, _) in infos {
            let fixed = validated.as_ref().map(|(version, _)| version.clone());
            if !agent { println!("    {}  {}  {}", crate::cli::scan::report::severity_badge(severity), id.truecolor(100, 160, 255), summary.chars().take(80).collect::<String>().truecolor(160, 160, 180)); }
            hits.push(crate::cli::scan::VulnHit { package: pkg_name.clone(), cve_id: id, severity, summary: summary.chars().take(100).collect(), fixed_version: fixed.clone(), upgrade_cmd: fixed.as_deref().map(|value| install_cmd_for_ecosystem(pkg_name, value, ecosystem)), fix_is_clean: validated.as_ref().map(|(_, clean)| *clean).unwrap_or(false) });
        }
        if !agent { print_fix_recommendation(validated.as_ref(), pkg_name, ecosystem, refs.len()); }
    }
    if !agent { println!(); }
    hits
}

fn install_cmd_for_ecosystem(pkg: &str, fixed: &str, ecosystem: &str) -> String {
    match ecosystem {
        "npm" => format!("npm install {}@{}", pkg, fixed),
        "yarn" => format!("yarn add {}@{}", pkg, fixed),
        "pnpm" => format!("pnpm add {}@{}", pkg, fixed),
        "bun" => format!("bun add {}@{}", pkg, fixed),
        "crates.io" | "cargo" => format!("cargo add {}@{}", pkg, fixed),
        "uv" => format!("uv add {}=={}", pkg, fixed),
        "poetry" => format!("poetry add {}=={}", pkg, fixed),
        "PyPI" | "pip" | "pip3" => format!("{} install {}=={}", crate::ecosystems::detector::resolve_binary("pip"), pkg, fixed),
        "Go" | "go" => format!("go get {}@{}", pkg, if fixed.starts_with('v') { fixed.to_string() } else { format!("v{}", fixed) }),
        "RubyGems" | "gem" => format!("{} install {} -v {}", crate::ecosystems::detector::resolve_binary("gem"), pkg, fixed),
        "Packagist" | "composer" => format!("composer require {}:{}", pkg, fixed),
        "NuGet" | "nuget" => format!("dotnet add package {} --version {}", pkg, fixed),
        "Hex" | "hex" => format!("mix deps.update {}", pkg),
        "pub.dev" | "pub" => format!("{} pub upgrade {}", crate::ecosystems::detector::resolve_binary("dart"), pkg),
        _ => format!("upgrade {} to {}", pkg, fixed),
    }
}

fn print_pkg_warning(pkg_name: &str, pkg_ver: &str, count: usize) {
    println!("\n  {} {}", "⚠".bright_yellow().bold(), format!("'{}@{}' has {} known vulnerability(ies):", pkg_name, pkg_ver, count).bold().bright_yellow());
}

fn print_fix_recommendation(validated: Option<&(String, bool)>, pkg_name: &str, ecosystem: &str, refs_len: usize) {
    match validated {
        Some((version, true)) => println!("       {} {} {}   {}", "→".truecolor(80, 80, 100), "safe version:".bright_green(), version.bright_green().bold(), install_cmd_for_ecosystem(pkg_name, version, ecosystem).truecolor(100, 140, 100)),
        Some((version, false)) => println!("       {} {} {} {}   {}", "→".truecolor(80, 80, 100), "best available:".bright_yellow(), version.bright_yellow().bold(), "(still has CVEs, but reduces risk)".truecolor(180, 140, 50), install_cmd_for_ecosystem(pkg_name, version, ecosystem).truecolor(100, 140, 100)),
        None => println!("       {} {}", "→".truecolor(80, 80, 100), "No fixed version published yet".truecolor(180, 120, 50)),
    }
    if refs_len > 5 { println!("       {} {} more CVEs — run 'infynon pkg scan' for full report", "…".truecolor(80, 80, 100), refs_len - 5); }
}
