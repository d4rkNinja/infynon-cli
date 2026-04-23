use crate::engine::{osv, reporter, scanner};
use crate::tui::logger::Logger;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::{HashMap, HashSet};
use std::time::Duration;

pub(super) fn run_human_scan(packages: &[scanner::LockedPackage], output: Option<crate::cli::scan::OutputFormat>, fix_level: Option<&crate::cli::scan::FixLevel>) {
    let tuples = queue_packages(packages);
    let results = match osv::batch_query(&tuples) { Ok(value) => value, Err(err) => return Logger::error(&format!("Vulnerability DB error: {}", err)) };
    let (unique_ids, vuln_to_packages) = collect_vuln_ids(&results);
    if unique_ids.is_empty() { println!(); return Logger::success("No known vulnerabilities found for your dependency tree!"); }
    let detail_map = fetch_detail_map(&unique_ids);
    let mut findings = build_findings(packages, &vuln_to_packages, &detail_map, fix_level);
    apply_safe_fix_versions(&mut findings);
    apply_latest_suggestions(&mut findings);
    println!();
    crate::cli::scan::report::print_report_table(&findings);
    if fix_level.is_some() && findings.iter().any(|finding| finding.fixed_version.is_some() || finding.suggested_version.is_some()) {
        crate::cli::scan::autofix::run_auto_fix(&findings);
    }
    write_reports(&findings, output);
}

fn queue_packages(packages: &[scanner::LockedPackage]) -> Vec<(String, String, String)> {
    println!();
    let pb = ProgressBar::new(packages.len() as u64);
    pb.set_style(ProgressStyle::with_template("  {spinner:.cyan}  checking {msg:<45} [{bar:40.cyan/blue}] {pos}/{len}").unwrap().tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]).progress_chars("█▉▊▋▌▍▎▏  "));
    pb.enable_steady_tick(Duration::from_millis(60));
    let tuples: Vec<_> = packages.iter().map(|pkg| { pb.set_message(format!("{}@{}", pkg.name, pkg.version)); pb.inc(1); (pkg.name.clone(), pkg.ecosystem.clone(), pkg.version.clone()) }).collect();
    pb.finish_with_message(format!("{} packages queued → checking vulnerabilities...", packages.len()));
    tuples
}

fn collect_vuln_ids(results: &[Vec<osv::OsvVulnRef>]) -> (Vec<String>, Vec<(String, usize)>) {
    let (mut ids, mut refs, mut seen) = (Vec::new(), Vec::new(), HashSet::new());
    for (index, vuln_refs) in results.iter().enumerate() { for vuln_ref in vuln_refs { if seen.insert(vuln_ref.id.clone()) { ids.push(vuln_ref.id.clone()); } refs.push((vuln_ref.id.clone(), index)); } }
    (ids, refs)
}

fn fetch_detail_map(unique_ids: &[String]) -> HashMap<String, osv::OsvVulnDetail> {
    println!();
    let pb = ProgressBar::new(unique_ids.len() as u64);
    pb.set_style(ProgressStyle::with_template("  {spinner:.yellow}  fetching  {msg:<45} [{bar:40.yellow/blue}] {pos}/{len} CVEs").unwrap().tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]).progress_chars("█▉▊▋▌▍▎▏  "));
    pb.enable_steady_tick(Duration::from_millis(60));
    pb.set_message(format!("{} unique CVEs (parallel fetch)...", unique_ids.len()));
    let mut map = HashMap::new();
    for (id, result) in osv::fetch_vuln_details_batch(unique_ids) { pb.inc(1); if let Ok(detail) = result { map.insert(id, detail); } }
    pb.finish_with_message(format!("{} CVE records fetched", map.len()));
    println!();
    map
}

fn build_findings(packages: &[scanner::LockedPackage], refs: &[(String, usize)], details: &HashMap<String, osv::OsvVulnDetail>, fix_level: Option<&crate::cli::scan::FixLevel>) -> Vec<reporter::ScanFinding> {
    refs.iter().filter_map(|(id, pkg_idx)| {
        let detail = details.get(id)?;
        let severity = osv::severity_label(detail);
        if !fix_level.map_or(true, |level| level.matches(severity)) { return None; }
        Some(reporter::ScanFinding { package: packages[*pkg_idx].clone(), vuln: detail.clone(), severity, fixed_version: osv::best_fixed_version(detail, &packages[*pkg_idx].name, &packages[*pkg_idx].ecosystem), suggested_version: None })
    }).collect()
}

fn apply_safe_fix_versions(findings: &mut [reporter::ScanFinding]) {
    let mut candidates: HashMap<(String, String), Vec<String>> = HashMap::new();
    let mut current: HashMap<(String, String), String> = HashMap::new();
    for finding in findings.iter() {
        let key = (finding.package.name.clone(), finding.package.ecosystem.clone());
        current.entry(key.clone()).or_insert_with(|| finding.package.version.clone());
        if let Some(ref fixed) = finding.fixed_version { candidates.entry(key).or_default().push(fixed.clone()); }
    }
    let mut safe_versions = HashMap::new();
    for ((name, eco), versions) in &candidates {
        safe_versions.insert((name.clone(), eco.clone()), osv::find_safest_candidate_vs(versions, name, eco, current.get(&(name.clone(), eco.clone())).map(|v| v.as_str()).unwrap_or("")));
    }
    for finding in findings.iter_mut() { if let Some(safe) = safe_versions.get(&(finding.package.name.clone(), finding.package.ecosystem.clone())) { finding.fixed_version = safe.as_ref().map(|(version, _)| version.clone()); } }
}

fn apply_latest_suggestions(findings: &mut [reporter::ScanFinding]) {
    let mut cache: HashMap<(String, String), Option<String>> = HashMap::new();
    for finding in findings.iter_mut() {
        if finding.fixed_version.is_some() { continue; }
        let key = (finding.package.name.clone(), finding.package.ecosystem.clone());
        let latest = cache.entry(key).or_insert_with(|| {
            let latest = crate::engine::registry::fetch_latest_version(&finding.package.name, &finding.package.ecosystem)?;
            if osv::version_vuln_count(&finding.package.name, &finding.package.ecosystem, &latest) > 0 { None } else { Some(latest) }
        });
        if let Some(ref latest) = latest { if latest != &finding.package.version { finding.suggested_version = Some(latest.clone()); } }
    }
}

fn write_reports(findings: &[reporter::ScanFinding], output: Option<crate::cli::scan::OutputFormat>) {
    match output {
        Some(crate::cli::scan::OutputFormat::Markdown) | Some(crate::cli::scan::OutputFormat::Both) => match reporter::write_markdown(findings, "infynon-scan-report.md") { Ok(_) => Logger::success("Markdown report written → infynon-scan-report.md"), Err(err) => Logger::error(&format!("Failed to write markdown: {}", err)) },
        _ => {}
    }
    match output {
        Some(crate::cli::scan::OutputFormat::Pdf) | Some(crate::cli::scan::OutputFormat::Both) => match reporter::write_pdf(findings, "infynon-scan-report.pdf") { Ok(_) => Logger::success("PDF report written → infynon-scan-report.pdf"), Err(err) => Logger::error(&format!("Failed to write PDF: {}", err)) },
        None => Logger::raw_dim("  Tip: pass --output markdown|pdf|both to save a report file."),
        _ => {}
    }
    println!();
}
