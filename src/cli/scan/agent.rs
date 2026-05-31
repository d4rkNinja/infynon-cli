use crate::engine::{osv, scanner};

pub(super) fn run_agent_scan(
    packages: &[scanner::LockedPackage],
    fix_level: Option<&crate::cli::scan::FixLevel>,
) -> ! {
    let tuples: Vec<(String, String, String)> = packages
        .iter()
        .map(|pkg| (pkg.name.clone(), pkg.ecosystem.clone(), pkg.version.clone()))
        .collect();
    let results = match osv::batch_query(&tuples) {
        Ok(value) => value,
        Err(err) => exit_json(
            2,
            serde_json::json!({"schema_version":"infynon.pkg.scan.v1","status":"error","error":format!("Vulnerability DB error: {}", err),"packages_scanned":packages.len()}),
        ),
    };
    let mut unique_ids = Vec::new();
    let mut vuln_to_packages = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for (index, refs) in results.iter().enumerate() {
        for vuln_ref in refs {
            if seen.insert(vuln_ref.id.clone()) {
                unique_ids.push(vuln_ref.id.clone());
            }
            vuln_to_packages.push((vuln_ref.id.clone(), index));
        }
    }
    let detail_map: std::collections::HashMap<String, osv::OsvVulnDetail> =
        osv::fetch_vuln_details_batch(&unique_ids)
            .into_iter()
            .filter_map(|(id, result)| result.ok().map(|detail| (id, detail)))
            .collect();
    let mut vulns = Vec::new();
    let mut counts = [0u32; 5];
    let mut seen_pairs = std::collections::HashSet::new();
    for (vuln_id, pkg_idx) in vuln_to_packages {
        let key = (packages[pkg_idx].name.clone(), vuln_id.clone());
        if !seen_pairs.insert(key) {
            continue;
        }
        if let Some(detail) = detail_map.get(&vuln_id) {
            let severity = osv::severity_label(detail);
            if !fix_level.is_none_or(|level| level.matches(severity)) {
                continue;
            }
            match severity {
                "CRITICAL" => counts[0] += 1,
                "HIGH" => counts[1] += 1,
                "MEDIUM" => counts[2] += 1,
                "LOW" => counts[3] += 1,
                _ => counts[4] += 1,
            }
            let raw_fixed = osv::best_fixed_version(
                detail,
                &packages[pkg_idx].name,
                &packages[pkg_idx].ecosystem,
            );
            let (fixed, verified) = match raw_fixed {
                Some(ref value) => osv::find_safest_candidate_vs(
                    std::slice::from_ref(value),
                    &packages[pkg_idx].name,
                    &packages[pkg_idx].ecosystem,
                    &packages[pkg_idx].version,
                )
                .map(|(version, clean)| (Some(version), clean))
                .unwrap_or((raw_fixed.clone(), false)),
                None => (None, false),
            };
            vulns.push(serde_json::json!({"package":packages[pkg_idx].name,"ecosystem":packages[pkg_idx].ecosystem,"current_version":packages[pkg_idx].version,"cve_id":detail.id,"severity":severity,"summary":detail.summary.as_deref().unwrap_or(""),"safe_version":fixed,"fix_verified":verified,"fix_cmd":fixed.as_deref().map(|value| crate::cli::scan::upgrade_cmd(&packages[pkg_idx], value))}));
        }
    }
    let total: u32 = counts.iter().sum();
    let status = if total == 0 {
        "clean"
    } else if counts[0] > 0 || counts[1] > 0 || counts[2] > 0 {
        "vulnerable"
    } else {
        "warnings"
    };
    let exit_code = if total == 0 {
        0
    } else if status == "vulnerable" {
        2
    } else {
        1
    };
    exit_json(
        exit_code,
        serde_json::json!({"schema_version":"infynon.pkg.scan.v1","status":status,"packages_scanned":packages.len(),"vulnerabilities":vulns,"summary":{"critical":counts[0],"high":counts[1],"medium":counts[2],"low":counts[3],"informational":counts[4],"total":total}}),
    )
}

pub(super) fn exit_error(message: &str) -> ! {
    exit_json(
        2,
        serde_json::json!({"schema_version":"infynon.pkg.scan.v1","status":"error","error":message}),
    )
}

fn exit_json(code: i32, payload: serde_json::Value) -> ! {
    crate::utils::print_json_pretty(&payload);
    std::process::exit(code);
}
