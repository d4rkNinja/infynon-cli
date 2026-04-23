use super::*;
use crate::cli::scan::{escalate_severity, tool_to_osv_ecosystem, upgrade_cmd};
use crate::engine::osv;

pub fn cmd_explain(package: &str, ecosystem: Option<&str>, pkg_file: Option<&str>) {
    println!();
    Logger::title("INFYNON Explain", "blue");
    Logger::step(&format!("Explaining '{}'...", package));

    let packages = load_packages(pkg_file);
    if packages.is_empty() {
        Logger::error("No packages found in lock files.");
        return;
    }

    let matches: Vec<&scanner::LockedPackage> = packages
        .iter()
        .filter(|pkg| pkg.name.eq_ignore_ascii_case(package))
        .filter(|pkg| {
            ecosystem
                .map(|value| pkg.ecosystem.eq_ignore_ascii_case(value))
                .unwrap_or(true)
        })
        .collect();

    if matches.is_empty() {
        Logger::error(&format!(
            "'{}' not found in any selected package file.",
            package
        ));
        return;
    }

    let direct = why_cmd::is_direct_dep(package);
    let chains = why_cmd::trace_why(package);

    for pkg in matches {
        print_package_header(pkg, direct);
        print_dependency_chains(&chains);
        print_vulnerability_context(pkg);
    }

    println!();
}

fn print_package_header(pkg: &scanner::LockedPackage, direct: bool) {
    println!();
    println!("  Package: {}@{}", pkg.name, pkg.version);
    println!("  Ecosystem: {}", pkg.ecosystem);
    println!("  Source: {}", pkg.source);
    println!(
        "  Dependency type: {}",
        if direct { "direct" } else { "transitive" }
    );
}

fn print_dependency_chains(chains: &[Vec<String>]) {
    if chains.is_empty() {
        return;
    }
    println!("  Dependency chain(s):");
    for chain in chains.iter().take(10) {
        println!("    - {}", chain.join(" -> "));
    }
}

fn print_vulnerability_context(pkg: &scanner::LockedPackage) {
    let osv_ecosystem = tool_to_osv_ecosystem(&pkg.ecosystem);
    let query = vec![(pkg.name.clone(), osv_ecosystem.clone(), pkg.version.clone())];
    let refs = match osv::batch_query(&query) {
        Ok(result) => result.into_iter().next().unwrap_or_default(),
        Err(err) => {
            println!("  Vulnerability lookup: failed ({})", err);
            return;
        }
    };

    if refs.is_empty() {
        println!("  Vulnerability lookup: no known OSV advisories for this version");
        if let Some(latest) = lookup_latest_version(pkg, &osv_ecosystem) {
            if latest != pkg.version {
                println!("  Latest registry version: {}", latest);
            }
        }
        return;
    }

    let ids: Vec<String> = refs.iter().map(|item| item.id.clone()).collect();
    let details = osv::fetch_vuln_details_batch(&ids);
    let mut worst = "UNKNOWN";
    let mut fix_candidates = Vec::new();
    let mut printed = 0usize;

    println!("  Advisories:");
    for (_id, detail) in &details {
        let detail = match detail {
            Ok(value) => value,
            Err(err) => {
                println!("    - advisory detail unavailable ({})", err);
                continue;
            }
        };
        let severity = osv::severity_label(detail);
        worst = escalate_severity(worst, severity);
        if let Some(fixed) = osv::best_fixed_version(detail, &pkg.name, &osv_ecosystem) {
            fix_candidates.push(fixed);
        }

        if printed < 5 {
            println!(
                "    - {} [{}] {}",
                detail.id,
                severity,
                detail.summary.as_deref().unwrap_or("No summary available")
            );
            printed += 1;
        }
    }

    if ids.len() > printed {
        println!(
            "    - ... and {} more advisory record(s)",
            ids.len() - printed
        );
    }

    println!("  Worst severity: {}", worst);

    let remediation =
        osv::find_safest_candidate_vs(&fix_candidates, &pkg.name, &osv_ecosystem, &pkg.version);
    let latest = lookup_latest_version(pkg, &osv_ecosystem);

    match remediation {
        Some((version, true)) => {
            println!("  Remediation plan: upgrade to {}", version);
            println!("  Confidence: high");
            println!("  Suggested command: {}", upgrade_cmd(pkg, &version));
        }
        Some((version, false)) => {
            println!(
                "  Remediation plan: upgrade to {} to reduce exposure",
                version
            );
            println!("  Confidence: medium");
            println!("  Suggested command: {}", upgrade_cmd(pkg, &version));
        }
        None => {
            println!("  Remediation plan: no verified fixed version published");
            println!("  Confidence: low");
            if let Some(ref latest) = latest {
                if latest != &pkg.version {
                    println!("  Latest registry version: {}", latest);
                    println!("  Best next step: review {}", upgrade_cmd(pkg, latest));
                }
            }
        }
    }

    if let Some(latest) = latest {
        if latest != pkg.version {
            println!("  Latest registry version: {}", latest);
        }
    }
}

fn lookup_latest_version(pkg: &scanner::LockedPackage, osv_ecosystem: &str) -> Option<String> {
    crate::engine::registry::fetch_latest_version(&pkg.name, &pkg.ecosystem)
        .or_else(|| crate::engine::registry::fetch_latest_version(&pkg.name, osv_ecosystem))
}
