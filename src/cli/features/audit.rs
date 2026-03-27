use super::*;
use crate::engine::osv;
use crate::cli::scan;

pub fn cmd_audit_deep(pkg_file: Option<&str>) {
    println!();
    Logger::title("INFYNON Deep Audit", "blue");

    let packages = load_packages(pkg_file);
    if packages.is_empty() {
        Logger::error("No packages found. Run from a project with lock files.");
        return;
    }

    let mut sources: Vec<String> = packages.iter().map(|p| p.source.clone()).collect();
    sources.sort();
    sources.dedup();
    Logger::step("Scanning all dependencies (direct + transitive)...");
    Logger::success(&format!("Found {} total packages from: {}", packages.len(), sources.join(", ")));

    // Build tree structure
    Logger::step("Building dependency tree...");
    let tree = build_dep_tree(&packages);

    // Query OSV for all packages
    let pb = bar(packages.len() as u64);
    let tuples: Vec<(String, String, String)> = packages.iter().map(|p| {
        pb.set_message(format!("{}@{}", p.name, p.version));
        pb.inc(1);
        (p.name.clone(), p.ecosystem.clone(), p.version.clone())
    }).collect();
    pb.finish_and_clear();

    Logger::step("Querying vulnerability database...");
    let results = match osv::batch_query(&tuples) {
        Ok(r) => r,
        Err(e) => { Logger::error(&format!("Vulnerability DB error: {}", e)); return; }
    };

    // Map vulnerabilities to packages
    let mut vuln_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut vuln_sev: HashMap<String, String> = HashMap::new();
    let mut detail_ids: Vec<String> = Vec::new();

    for (i, vuln_refs) in results.iter().enumerate() {
        if !vuln_refs.is_empty() {
            let pkg = &packages[i];
            let key = format!("{}@{}", pkg.name, pkg.version);
            let ids: Vec<String> = vuln_refs.iter().map(|v| v.id.clone()).collect();
            detail_ids.extend(ids.iter().cloned());
            vuln_map.insert(key, ids);
        }
    }

    // Fetch severity for vulnerable packages (batch) using reverse index
    if !detail_ids.is_empty() {
        detail_ids.sort();
        detail_ids.dedup();

        // Build reverse index: vuln_id → list of package keys
        let mut vuln_to_pkgs: HashMap<String, Vec<String>> = HashMap::new();
        for (key, ids) in &vuln_map {
            for id in ids {
                vuln_to_pkgs.entry(id.clone()).or_default().push(key.clone());
            }
        }

        let details = osv::fetch_vuln_details_batch(&detail_ids);
        let rank = |s: &str| match s { "CRITICAL" => 4, "HIGH" => 3, "MEDIUM" => 2, "LOW" => 1, _ => 0 };
        for (id, result) in &details {
            if let Ok(detail) = result {
                let sev = osv::severity_label(detail).to_string();
                if let Some(pkg_keys) = vuln_to_pkgs.get(id) {
                    for key in pkg_keys {
                        let entry = vuln_sev.entry(key.clone()).or_insert_with(|| "INFORMATIONAL".to_string());
                        if rank(&sev) > rank(entry) { *entry = sev.clone(); }
                    }
                }
            }
        }
    }

    let total_vulns: usize = vuln_map.values().map(|v| v.len()).sum();

    // Print tree
    println!();
    println!(
        "  {}  {} {}",
        "Dependency Tree".bold().truecolor(0, 210, 255),
        "─".repeat(30).truecolor(40, 40, 60),
        if total_vulns > 0 {
            format!("{} CVEs found", total_vulns).bold().bright_red().to_string()
        } else {
            "all clear".bold().bright_green().to_string()
        }
    );
    println!();

    for (i, node) in tree.iter().enumerate() {
        print_tree_node(node, "", i == tree.len() - 1, &vuln_map, &vuln_sev);
    }

    // ── Risk breakdown ──────────────────────────────────────────────────────
    let critical     = vuln_sev.values().filter(|s| s.as_str() == "CRITICAL").count();
    let high         = vuln_sev.values().filter(|s| s.as_str() == "HIGH").count();
    let medium       = vuln_sev.values().filter(|s| s.as_str() == "MEDIUM").count();
    let low          = vuln_sev.values().filter(|s| s.as_str() == "LOW").count();
    let informational = vuln_sev.values().filter(|s| s.as_str() == "INFORMATIONAL").count();
    let clean        = packages.len().saturating_sub(vuln_map.len());

    // Weighted risk score 0–100
    let weighted = critical * 40 + high * 20 + medium * 8 + low * 2 + informational;
    let max_weighted = packages.len() * 40;
    let risk_score: usize = if max_weighted == 0 { 0 } else { (weighted * 100 / max_weighted).min(100) };

    let (overall_label, overall_color) = if critical > 0      { ("CRITICAL RISK", (255u8,  60u8,  60u8)) }
        else if high > 0                                       { ("HIGH RISK",     (255u8, 140u8,  40u8)) }
        else if medium > 0                                     { ("MEDIUM RISK",   (255u8, 200u8,  40u8)) }
        else if low > 0 || informational > 0                   { ("LOW RISK",      (200u8, 255u8, 100u8)) }
        else                                                   { ("CLEAN",         ( 50u8, 255u8, 160u8)) };

    fn sev_bar(count: usize, total: usize) -> String {
        if total == 0 || count == 0 { return String::new(); }
        let filled = ((count * 20) / total).max(1);
        "█".repeat(filled)
    }

    println!();
    println!("  {}", "─".repeat(66).truecolor(40, 40, 60));
    println!();
    println!("  {}  {}", "◆ Risk Breakdown".bold().truecolor(0, 210, 255), format!("— {}", overall_label).bold().truecolor(overall_color.0, overall_color.1, overall_color.2));
    println!();

    let total = packages.len();
    let rows = [
        ("CRITICAL",     critical,      (255u8,  60u8,  60u8)),
        ("HIGH",         high,          (255u8, 140u8,  40u8)),
        ("MEDIUM",       medium,        (255u8, 200u8,  40u8)),
        ("LOW",          low,           (200u8, 220u8, 100u8)),
        ("INFORMATIONAL",informational, (160u8, 160u8, 200u8)),
        ("CLEAN",        clean,         ( 50u8, 220u8, 130u8)),
    ];
    for (label, count, (r, g, b)) in rows {
        let bar_str = sev_bar(count, total);
        println!(
            "     {:<14}  {:>4} packages  {}",
            label.bold().truecolor(r, g, b),
            count.to_string().bold().truecolor(r, g, b),
            bar_str.truecolor(r, g, b),
        );
    }

    println!();
    println!(
        "  {}  Risk Score: {}  ·  {} total pkgs  ·  {} CVEs  ·  {} unique pkgs affected",
        "◆".truecolor(0, 210, 255),
        format!("{}/100", risk_score).bold().truecolor(overall_color.0, overall_color.1, overall_color.2),
        total.to_string().bold(),
        total_vulns.to_string().bold().bright_yellow(),
        vuln_map.len().to_string().bold().bright_red(),
    );
    println!();
}

#[derive(Debug, Clone)]
struct DepNode {
    name: String,
    version: String,
    children: Vec<DepNode>,
}

fn build_dep_tree(packages: &[scanner::LockedPackage]) -> Vec<DepNode> {
    let mut roots: Vec<DepNode> = Vec::new();

    // npm tree from package-lock.json
    if Path::new("package-lock.json").exists() {
        if let Some(nodes) = build_npm_tree() {
            roots.extend(nodes);
        }
    }

    // Cargo tree from Cargo.lock
    if Path::new("Cargo.lock").exists() {
        if let Some(nodes) = build_cargo_tree(packages) {
            roots.extend(nodes);
        }
    }

    // Other ecosystems: flat list grouped
    let handled: HashSet<&str> = ["npm", "crates.io"].iter().cloned().collect();
    let mut other: HashMap<String, Vec<&scanner::LockedPackage>> = HashMap::new();
    for p in packages {
        if !handled.contains(p.ecosystem.as_str()) {
            other.entry(p.ecosystem.clone()).or_default().push(p);
        }
    }
    for (eco, pkgs) in &other {
        let children = pkgs.iter().map(|p| DepNode {
            name: p.name.clone(), version: p.version.clone(), children: vec![],
        }).collect();
        roots.push(DepNode {
            name: format!("[{}]", eco), version: String::new(), children,
        });
    }

    roots
}

fn build_npm_tree() -> Option<Vec<DepNode>> {
    let content = fs::read_to_string("package-lock.json").ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    let packages = json.get("packages").and_then(|p| p.as_object())?;

    // Get direct dependencies from root entry
    let root = packages.get("")?;
    let mut dep_names: Vec<String> = Vec::new();
    for key in &["dependencies", "devDependencies"] {
        if let Some(deps) = root.get(key).and_then(|d| d.as_object()) {
            dep_names.extend(deps.keys().cloned());
        }
    }

    let mut nodes = Vec::new();
    for name in &dep_names {
        let key = format!("node_modules/{}", name);
        if let Some(pkg_data) = packages.get(&key) {
            let version = pkg_data.get("version").and_then(|v| v.as_str()).unwrap_or("?").to_string();
            let mut children = Vec::new();
            if let Some(sub_deps) = pkg_data.get("dependencies").and_then(|d| d.as_object()) {
                for (sub_name, _) in sub_deps.iter().take(8) {
                    let sub_key = format!("node_modules/{}", sub_name);
                    if let Some(sub_data) = packages.get(&sub_key) {
                        let sub_ver = sub_data.get("version").and_then(|v| v.as_str()).unwrap_or("?");
                        children.push(DepNode { name: sub_name.clone(), version: sub_ver.to_string(), children: vec![] });
                    }
                }
            }
            nodes.push(DepNode { name: name.clone(), version, children });
        }
    }
    if nodes.is_empty() { None } else { Some(nodes) }
}

fn build_cargo_tree(packages: &[scanner::LockedPackage]) -> Option<Vec<DepNode>> {
    let pkg_deps = cargo_lock_deps();
    if pkg_deps.is_empty() { return None; }

    let root_name = cargo_root_name()?;

    let pkg_map: HashMap<&str, &scanner::LockedPackage> = packages.iter()
        .filter(|p| p.ecosystem == "crates.io")
        .map(|p| (p.name.as_str(), p))
        .collect();

    let root_deps = pkg_deps.get(&root_name).cloned().unwrap_or_default();
    let nodes: Vec<DepNode> = root_deps.iter().filter_map(|dep_name| {
        let pkg = pkg_map.get(dep_name.as_str())?;
        let children: Vec<DepNode> = pkg_deps.get(dep_name.as_str())
            .map(|deps| deps.iter().take(6).filter_map(|d| {
                let p = pkg_map.get(d.as_str())?;
                Some(DepNode { name: p.name.clone(), version: p.version.clone(), children: vec![] })
            }).collect())
            .unwrap_or_default();
        Some(DepNode { name: pkg.name.clone(), version: pkg.version.clone(), children })
    }).collect();

    if nodes.is_empty() { None } else { Some(nodes) }
}

fn print_tree_node(
    node: &DepNode, prefix: &str, is_last: bool,
    vuln_map: &HashMap<String, Vec<String>>,
    vuln_sev: &HashMap<String, String>,
) {
    let connector = if is_last { "└── " } else { "├── " };
    let key = format!("{}@{}", node.name, node.version);
    let is_vuln = vuln_map.contains_key(&key);

    let ver = if node.version.is_empty() { String::new() } else { format!("@{}", node.version) };

    if is_vuln {
        let ids = &vuln_map[&key];
        let sev = vuln_sev.get(&key).map(|s| s.as_str()).unwrap_or("UNKNOWN");
        println!(
            "  {}{}{}{}  {} [{}] {}",
            prefix.truecolor(60, 60, 80), connector.truecolor(60, 60, 80),
            node.name.bold().bright_red(), ver.truecolor(180, 80, 80),
            "⚠".bright_yellow(),
            scan::severity_colored(sev),
            ids.join(", ").truecolor(100, 160, 255)
        );
    } else {
        println!(
            "  {}{}{}{}",
            prefix.truecolor(60, 60, 80), connector.truecolor(60, 60, 80),
            node.name.bold().truecolor(200, 200, 220), ver.truecolor(120, 120, 140)
        );
    }

    let child_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
    for (i, child) in node.children.iter().enumerate() {
        print_tree_node(child, &child_prefix, i == node.children.len() - 1, vuln_map, vuln_sev);
    }
}
