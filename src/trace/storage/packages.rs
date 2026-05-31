pub fn package_risks() -> Result<Vec<PackageRisk>, String> {
    let packages = engine::scanner::detect_locked_packages(None);
    if packages.is_empty() {
        return Ok(Vec::new());
    }
    let queries: Vec<(String, String, String)> = packages
        .iter()
        .map(|pkg| {
            (
                pkg.name.clone(),
                crate::ecosystems::catalog::canonical_osv_ecosystem(&pkg.ecosystem)
                    .unwrap_or(&pkg.ecosystem)
                    .to_string(),
                pkg.version.clone(),
            )
        })
        .collect();

    let results = engine::osv::batch_query(&queries)?;
    let notes =
        retrieve_notes(None, Some(TraceScope::Package), None, None, None, None).unwrap_or_default();

    let mut out = Vec::new();
    for (pkg, refs) in packages.iter().zip(results.iter()) {
        for vuln in refs {
            let severity = engine::osv::fetch_vuln_detail(&vuln.id)
                .ok()
                .map(|d| engine::osv::severity_label(&d))
                .unwrap_or("UNKNOWN")
                .to_string();
            let installed_by = notes
                .iter()
                .find(|n| n.target.eq_ignore_ascii_case(&pkg.name))
                .map(|n| n.author.clone());
            out.push(PackageRisk {
                package: pkg.name.clone(),
                version: pkg.version.clone(),
                ecosystem: pkg.ecosystem.clone(),
                severity,
                vulnerability_id: vuln.id.clone(),
                source_file: pkg.source.clone(),
                installed_by,
            });
        }
    }
    out.sort_by(|a, b| b.severity.cmp(&a.severity).then(a.package.cmp(&b.package)));
    Ok(out)
}
