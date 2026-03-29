use crate::engine::{osv, scanner, reporter};
use crate::tui::logger::Logger;
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub enum OutputFormat { Markdown, Pdf, Both }

#[derive(Debug, Clone, PartialEq)]
pub enum FixLevel { Critical, High, Medium, Low, Informational, All }

impl FixLevel {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "critical"      => Self::Critical,
            "high"          => Self::High,
            "medium"        => Self::Medium,
            "low"           => Self::Low,
            "informational" => Self::Informational,
            _               => Self::All,
        }
    }

    pub fn matches(&self, severity: &str) -> bool {
        match self {
            Self::All           => true,
            Self::Critical      => severity == "CRITICAL",
            Self::High          => matches!(severity, "CRITICAL" | "HIGH"),
            Self::Medium        => matches!(severity, "CRITICAL" | "HIGH" | "MEDIUM"),
            Self::Low           => matches!(severity, "CRITICAL" | "HIGH" | "MEDIUM" | "LOW"),
            Self::Informational => true,
        }
    }
}

/// Main entry point for `infynon pkg scan`
pub fn run_scan(output: Option<OutputFormat>, fix_level: Option<FixLevel>, pkg_file: Option<&str>) {
    use std::io::{self, Write};

    println!();
    Logger::title("INFYNON Package Scanner", "blue");

    // 1. Detect lock files
    if let Some(f) = pkg_file {
        Logger::step(&format!("Using custom file: {}", f));
    } else {
        Logger::step("Detecting lock files in current directory...");
    }

    let packages = if pkg_file.is_some() {
        scanner::detect_locked_packages(pkg_file)
    } else {
        let found_files = scanner::detect_lock_files();

        if found_files.is_empty() {
            Logger::error("No packages found in supported lock/manifest files.");
            Logger::info("Supported: package-lock.json · yarn.lock · pnpm-lock.yaml · requirements.txt");
            Logger::info("           poetry.lock · Cargo.lock · go.sum · Gemfile.lock · composer.lock");
            Logger::info("           mix.lock · pubspec.lock  — or pass --pkg-file <path>");
            return;
        }

        if found_files.len() == 1 {
            // Single file — use it directly
            scanner::parse_selected_files(&[found_files[0].0])
        } else {
            // Multiple lock files detected — let user choose
            println!();
            println!(
                "  {} Found {} lock/manifest files:\n",
                "ℹ".bright_cyan().bold(),
                found_files.len()
            );
            for (idx, (file, eco)) in found_files.iter().enumerate() {
                println!(
                    "     {}  {} {}",
                    format!("[{}]", idx + 1).bold().bright_cyan(),
                    file.bold(),
                    format!("({})", eco).truecolor(120, 120, 140)
                );
            }
            println!();
            println!(
                "     {}  Scan all files",
                "[A]".bold().bright_green()
            );
            println!();
            print!("  Select files to scan (e.g. 1,3 or A for all): ");
            io::stdout().flush().ok();

            let mut choice = String::new();
            io::stdin().read_line(&mut choice).ok();
            let choice = choice.trim();

            let selected_files: Vec<&str> = if choice.eq_ignore_ascii_case("a") || choice.is_empty() {
                found_files.iter().map(|(f, _)| *f).collect()
            } else {
                choice.split(',')
                    .filter_map(|s| {
                        let idx: usize = s.trim().parse().ok()?;
                        if idx >= 1 && idx <= found_files.len() {
                            Some(found_files[idx - 1].0)
                        } else {
                            None
                        }
                    })
                    .collect()
            };

            if selected_files.is_empty() {
                Logger::error("No valid files selected.");
                return;
            }

            println!();
            for f in &selected_files {
                Logger::detail("» Scanning:", f);
            }

            scanner::parse_selected_files(&selected_files)
        }
    };

    if packages.is_empty() {
        Logger::error("No packages found in selected lock/manifest files.");
        return;
    }

    let mut sources: Vec<String> = packages.iter().map(|p| p.source.clone()).collect();
    sources.sort();
    sources.dedup();
    Logger::success(&format!("Found {} pinned packages from: {}", packages.len(), sources.join(", ")));
    println!();

    // 2. Build batch query — show each package name live in the spinner
    println!();
    let pb = ProgressBar::new(packages.len() as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "  {spinner:.cyan}  checking {msg:<45} [{bar:40.cyan/blue}] {pos}/{len}"
        )
        .unwrap()
        .tick_strings(&["⠋","⠙","⠹","⠸","⠼","⠴","⠦","⠧","⠇","⠏"])
        .progress_chars("█▉▊▋▌▍▎▏  "),
    );
    pb.enable_steady_tick(Duration::from_millis(60));

    let tuples: Vec<(String, String, String)> = packages.iter().map(|p| {
        pb.set_message(format!("{}@{}", p.name, p.version));
        pb.inc(1);
        (p.name.clone(), p.ecosystem.clone(), p.version.clone())
    }).collect();
    pb.finish_with_message(format!("{} packages queued → checking vulnerabilities...", packages.len()));

    let results = match osv::batch_query(&tuples) {
        Ok(r) => r,
        Err(e) => { Logger::error(&format!("Vulnerability DB error: {}", e)); return; }
    };

    // 3. Collect unique vuln IDs and map them back to packages
    let mut vuln_to_packages: Vec<(String, usize)> = Vec::new(); // (vuln_id, package_index)
    let mut unique_ids: Vec<String> = Vec::new();
    {
        use std::collections::HashSet;
        let mut seen: HashSet<String> = HashSet::new();
        for (i, vuln_refs) in results.iter().enumerate() {
            for vref in vuln_refs {
                if seen.insert(vref.id.clone()) {
                    unique_ids.push(vref.id.clone());
                }
                vuln_to_packages.push((vref.id.clone(), i));
            }
        }
    }

    if unique_ids.is_empty() {
        println!();
        Logger::success("No known vulnerabilities found for your dependency tree!");
        return;
    }

    // 4. Fetch full vuln details in parallel (20 concurrent threads)
    println!();
    let detail_pb = ProgressBar::new(unique_ids.len() as u64);
    detail_pb.set_style(
        ProgressStyle::with_template(
            "  {spinner:.yellow}  fetching  {msg:<45} [{bar:40.yellow/blue}] {pos}/{len} CVEs"
        )
        .unwrap()
        .tick_strings(&["⠋","⠙","⠹","⠸","⠼","⠴","⠦","⠧","⠇","⠏"])
        .progress_chars("█▉▊▋▌▍▎▏  "),
    );
    detail_pb.enable_steady_tick(Duration::from_millis(60));
    detail_pb.set_message(format!("{} unique CVEs (parallel fetch)...", unique_ids.len()));

    let detail_results = osv::fetch_vuln_details_batch(&unique_ids);

    // Build lookup: vuln_id → detail
    let mut detail_map: std::collections::HashMap<String, osv::OsvVulnDetail> = std::collections::HashMap::new();
    for (id, result) in detail_results {
        detail_pb.inc(1);
        match result {
            Ok(detail) => { detail_map.insert(id, detail); }
            Err(e) => {
                detail_pb.suspend(|| {
                    eprintln!("  {} could not fetch {}: {}", "warn:".yellow(), id, e);
                });
            }
        }
    }
    detail_pb.finish_with_message(format!("{} CVE records fetched", detail_map.len()));
    println!();

    // 5. Build findings from the detail map
    let mut findings: Vec<reporter::ScanFinding> = Vec::new();
    for (vuln_id, pkg_idx) in &vuln_to_packages {
        if let Some(detail) = detail_map.get(vuln_id) {
            let severity = osv::severity_label(detail);
            let include = fix_level.as_ref().map_or(true, |fl| fl.matches(severity));
            if include {
                let fixed_version = osv::best_fixed_version(
                    detail,
                    &packages[*pkg_idx].name,
                    &packages[*pkg_idx].ecosystem,
                );
                findings.push(reporter::ScanFinding {
                    package:           packages[*pkg_idx].clone(),
                    vuln:              detail.clone(),
                    severity,
                    fixed_version,
                    suggested_version: None,
                });
            }
        }
    }

    // 4. For findings with no fixed version, look up latest stable from registry
    {
        use std::collections::HashMap;
        let mut cache: HashMap<(String, String), Option<String>> = HashMap::new();
        for f in findings.iter_mut() {
            if f.fixed_version.is_none() {
                let key = (f.package.name.clone(), f.package.ecosystem.clone());
                let latest = cache.entry(key).or_insert_with(|| {
                    crate::engine::registry::fetch_latest_version(&f.package.name, &f.package.ecosystem)
                });
                // Only suggest if the latest version differs from the current one
                if let Some(ref lv) = latest {
                    if lv != &f.package.version {
                        f.suggested_version = Some(lv.clone());
                    }
                }
            }
        }
    }

    // 5. Unified report table (vulnerability + remediation in one table)
    println!();
    print_report_table(&findings);

    // 6. Auto-execute remediation commands when --fix was explicitly passed
    let has_remediation = findings.iter().any(|f| f.fixed_version.is_some() || f.suggested_version.is_some());
    if fix_level.is_some() && has_remediation {
        run_auto_fix(&findings);
    }

    // 6. Write files only if --output was explicitly passed
    if let Some(ref fmt) = output {
        match fmt {
            OutputFormat::Markdown | OutputFormat::Both => {
                let path = "infynon-scan-report.md";
                match reporter::write_markdown(&findings, path) {
                    Ok(_)  => Logger::success(&format!("Markdown report written → {}", path)),
                    Err(e) => Logger::error(&format!("Failed to write markdown: {}", e)),
                }
            }
            _ => {}
        }
        match fmt {
            OutputFormat::Pdf | OutputFormat::Both => {
                let path = "infynon-scan-report.pdf";
                match reporter::write_pdf(&findings, path) {
                    Ok(_)  => Logger::success(&format!("PDF report written → {}", path)),
                    Err(e) => Logger::error(&format!("Failed to write PDF: {}", e)),
                }
            }
            _ => {}
        }
    } else {
        Logger::raw_dim("  Tip: pass --output markdown|pdf|both to save a report file.");
    }
    println!();
}

/// Execute one upgrade command per unique vulnerable package.
/// Groups all CVE findings for the same package and picks the single best fix version,
/// so a package affected by N CVEs results in exactly one upgrade command, not N.
fn run_auto_fix(findings: &[reporter::ScanFinding]) {
    use std::collections::HashMap;

    // Key: (name, ecosystem, source)  →  (pkg reference, confirmed_fixes, suggested_fixes)
    type Key = (String, String, String);
    let mut pkg_map: HashMap<Key, (&crate::engine::scanner::LockedPackage, Vec<String>, Vec<String>)> =
        HashMap::new();

    for f in findings {
        let key = (
            f.package.name.clone(),
            f.package.ecosystem.clone(),
            f.package.source.clone(),
        );
        let entry = pkg_map.entry(key).or_insert_with(|| (&f.package, Vec::new(), Vec::new()));
        if let Some(ref fv) = f.fixed_version {
            entry.1.push(fv.clone()); // confirmed fix from CVE database
        } else if let Some(ref sv) = f.suggested_version {
            entry.2.push(sv.clone()); // fallback: registry latest
        }
    }

    // Build one command per package using the best available version
    struct FixItem { label: String, cmd: String }

    let items: Vec<FixItem> = pkg_map.values()
        .filter_map(|(pkg, confirmed, suggested)| {
            let best = if !confirmed.is_empty() {
                osv::max_version(confirmed)
            } else {
                osv::max_version(suggested)
            }?;
            Some(FixItem {
                label: format!("{} {} → {}", pkg.name, pkg.version, best),
                cmd:   upgrade_cmd(pkg, &best),
            })
        })
        .collect();

    if items.is_empty() {
        Logger::info("No packages have a known fixed or suggested version available.");
        return;
    }

    println!();
    println!(
        "  {} {}\n",
        "⚡ Auto-Fix".bold().truecolor(255, 200, 50),
        format!("Upgrading {} package(s)...", items.len()).truecolor(160, 160, 180)
    );

    let mut success_count = 0usize;
    let mut fail_count    = 0usize;

    for item in &items {
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::with_template("  {spinner:.green}  {msg}")
                .unwrap()
                .tick_strings(&["⠋","⠙","⠹","⠸","⠼","⠴","⠦","⠧","⠇","⠏","✔"]),
        );
        spinner.enable_steady_tick(Duration::from_millis(60));
        spinner.set_message(format!(
            "{} {}",
            item.label.bold(),
            format!("({})", item.cmd).truecolor(100, 100, 120)
        ));

        match crate::cli::run_pkg_cmd(&item.cmd) {
            Ok(out) => {
                spinner.finish_and_clear();
                if out.status.success() {
                    success_count += 1;
                    println!(
                        "  {}  {} {}",
                        "✔".bright_green().bold(),
                        item.label.bold(),
                        "fixed".bright_green()
                    );
                } else {
                    fail_count += 1;
                    println!(
                        "  {}  {} — command exited with code {}",
                        "✘".bright_red().bold(),
                        item.label.bold(),
                        out.status.code().unwrap_or(-1)
                    );
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    for line in stderr.lines().take(6) {
                        println!("       {} {}", "│".truecolor(80, 80, 100), line.truecolor(200, 80, 80));
                    }
                    if stderr.trim().is_empty() {
                        let stdout = String::from_utf8_lossy(&out.stdout);
                        for line in stdout.lines().take(6) {
                            println!("       {} {}", "│".truecolor(80, 80, 100), line.truecolor(180, 180, 180));
                        }
                    }
                }
            }
            Err(e) => {
                spinner.finish_and_clear();
                fail_count += 1;
                println!(
                    "  {}  {} — could not run: {}",
                    "✘".bright_red().bold(),
                    item.label.bold(),
                    e.to_string().truecolor(200, 80, 80)
                );
            }
        }
    }

    println!();
    println!(
        "  Auto-fix complete  {}  {}",
        format!("{} succeeded", success_count).bold().bright_green(),
        if fail_count > 0 { format!("{} failed", fail_count).bold().bright_red().to_string() }
        else              { "0 failed".truecolor(100, 100, 120).to_string() }
    );
    println!();
}

// ── Tables ────────────────────────────────────────────────────────────────────

/// Unified report table: vulnerability info + remediation in a single table.
fn print_report_table(findings: &[reporter::ScanFinding]) {
    use tabled::{Table, Tabled};
    use tabled::settings::{Style, Padding, object::Rows, Color};

    #[derive(Tabled)]
    struct Row {
        #[tabled(rename = " Risk ")]          sev: String,
        #[tabled(rename = " Package ")]       pkg: String,
        #[tabled(rename = " Version ")]       ver: String,
        #[tabled(rename = " CVE / ID ")]      cve: String,
        #[tabled(rename = " Remediation ")]   fix: String,
    }

    let rows: Vec<Row> = findings.iter().map(|f| {
        let fix = if let Some(ref fv) = f.fixed_version {
            fv.clone()
        } else if let Some(ref sv) = f.suggested_version {
            format!("~{} (latest)", sv)
        } else {
            "No fix".to_string()
        };
        Row {
            sev: f.severity.to_string(),
            pkg: f.package.name.chars().take(25).collect(),
            ver: f.package.version.chars().take(12).collect(),
            cve: f.vuln.id.clone(),
            fix: fix.chars().take(18).collect(),
        }
    }).collect();

    let mut table = Table::new(rows);
    table
        .with(Style::modern())
        .with(Padding::new(1, 1, 0, 0))
        .modify(Rows::first(), Color::BOLD | Color::FG_BRIGHT_CYAN);

    println!("  {}\n", "Vulnerability Report:".bold().white());
    println!("{}", table);

    // Print detailed remediation info below the table
    for f in findings {
        let summary = f.vuln.summary.as_deref().unwrap_or("CVE in this version range");
        let short: String = summary.chars().take(80).collect();

        if let Some(ref fv) = f.fixed_version {
            let cmd = upgrade_cmd(&f.package, fv);
            println!(
                "       {} {} {}  {} → {}  {}",
                "→".truecolor(80,80,100),
                f.vuln.id.truecolor(100,160,255),
                f.package.name.bold(),
                "fix:".bright_green(),
                cmd.bright_green().bold(),
                short.truecolor(140,140,160)
            );
        } else if let Some(ref sv) = f.suggested_version {
            let cmd = upgrade_cmd(&f.package, sv);
            println!(
                "       {} {} {}  {} {} → {}  {}",
                "→".truecolor(80,80,100),
                f.vuln.id.truecolor(100,160,255),
                f.package.name.bold(),
                "no DB fix".truecolor(180,120,50),
                "try latest:".truecolor(200,180,80),
                cmd.truecolor(200,200,100).bold(),
                short.truecolor(140,140,160)
            );
        } else {
            println!(
                "       {} {} {}  {}  {}",
                "→".truecolor(80,80,100),
                f.vuln.id.truecolor(100,160,255),
                f.package.name.bold(),
                "no fix available".truecolor(180,80,50),
                short.truecolor(140,140,160)
            );
        }
    }
    println!();

    let (crit, high, med, low, info) = reporter::severity_counts(findings);
    println!(
        "  {}  {}  {}  {}  {}\n",
        format!("CRITICAL: {}", crit).bold().bright_red(),
        format!("HIGH: {}",     high).bold().red(),
        format!("MEDIUM: {}",   med).bold().yellow(),
        format!("LOW: {}",      low).bold().bright_green(),
        format!("INFO: {}",     info).truecolor(140, 140, 160),
    );

    println!(
        "  {}  Upgrade to the 'Remediation' column value to fix each finding.\n     {}  ~ prefix means: no known fix in vulnerability DB — latest stable version suggested.\n",
        "ℹ".bright_cyan(),
        "ℹ".bright_cyan()
    );
}

// ── Helpers ───────────────────────────────────────────────────────────────────

pub fn severity_colored(sev: &str) -> String {
    match sev {
        "CRITICAL"      => sev.bright_red().bold().to_string(),
        "HIGH"          => sev.red().bold().to_string(),
        "MEDIUM"        => sev.yellow().bold().to_string(),
        "LOW"           => sev.bright_green().to_string(),
        _               => sev.truecolor(140, 140, 160).to_string(),
    }
}

pub fn upgrade_cmd(pkg: &scanner::LockedPackage, fixed: &str) -> String {
    // Determine the actual CLI tool from the source lock file, not just the ecosystem
    let source = pkg.source.as_str();
    let source_file = std::path::Path::new(source)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(source);

    match pkg.ecosystem.as_str() {
        "npm" => {
            // Detect yarn/pnpm/bun from lock file source
            match source_file {
                "yarn.lock"      => format!("yarn add {}@{}", pkg.name, fixed),
                "pnpm-lock.yaml" => format!("pnpm add {}@{}", pkg.name, fixed),
                "bun.lockb" | "bun.lock" => format!("bun add {}@{}", pkg.name, fixed),
                _ => {
                    // package.json / package-lock.json — check if bun.lockb exists nearby
                    if std::path::Path::new("bun.lockb").exists() {
                        format!("bun add {}@{}", pkg.name, fixed)
                    } else {
                        format!("npm install {}@{}", pkg.name, fixed)
                    }
                }
            }
        }
        "crates.io" => format!("cargo add {}@{}", pkg.name, fixed),
        "PyPI" => {
            use crate::ecosystems::detector::resolve_binary;
            // Detect uv/poetry from lock file source
            match source_file {
                "uv.lock"      => format!("uv add {}=={}", pkg.name, fixed),
                "poetry.lock"  => format!("poetry add {}=={}", pkg.name, fixed),
                _ => {
                    if std::path::Path::new("uv.lock").exists() {
                        format!("uv add {}=={}", pkg.name, fixed)
                    } else if std::path::Path::new("poetry.lock").exists() {
                        format!("poetry add {}=={}", pkg.name, fixed)
                    } else {
                        // Resolve at runtime: pip → pip3 on systems that ship only pip3
                        format!("{} install {}=={}", resolve_binary("pip"), pkg.name, fixed)
                    }
                }
            }
        }
        "Go" => {
            let ver = if fixed.starts_with('v') { fixed.to_string() } else { format!("v{}", fixed) };
            format!("go get {}@{}", pkg.name, ver)
        }
        "RubyGems"  => {
            use crate::ecosystems::detector::resolve_binary;
            format!("{} install {} -v {}", resolve_binary("gem"), pkg.name, fixed)
        }
        "Packagist" => format!("composer require {}:{}", pkg.name, fixed),
        "NuGet"     => format!("dotnet add package {} --version {}", pkg.name, fixed),
        "Hex"       => format!("mix deps.update {}", pkg.name),
        "pub.dev"   => {
            use crate::ecosystems::detector::resolve_binary;
            // dart or flutter — both expose `<binary> pub upgrade`
            format!("{} pub upgrade {}", resolve_binary("dart"), pkg.name)
        }
        _           => format!("upgrade {} to {}", pkg.name, fixed),
    }
}

// ── Install-time security gate ────────────────────────────────────────────────

/// A single vulnerability hit found during install-time check.
pub struct VulnHit {
    pub package:       String,
    pub cve_id:        String,
    pub severity:      &'static str,
    pub summary:       String,
    pub fixed_version: Option<String>,
    pub upgrade_cmd:   Option<String>,
}

/// Check packages before install. Returns (all_clear, hits).
/// Prints a full rich warning block per vulnerable package.
pub fn check_packages_before_install(names: &[String], ecosystem: &str) -> (bool, Vec<VulnHit>) {
    use indicatif::{ProgressBar, ProgressStyle};

    // Parse each CLI arg like `picomatch@4.0.3`, `requests==2.28.0`, `serde:1.0`
    // into (name, ecosystem, version).
    // If NO version is given (bare name like `express`), resolve latest from the registry —
    // OSV requires a real version; sending empty version always returns zero results (false-negative).
    let sp = ProgressBar::new_spinner();
    sp.set_style(
        ProgressStyle::with_template("  {spinner:.cyan}  {msg}")
            .unwrap()
            .tick_strings(&["⠋","⠙","⠹","⠸","⠼","⠴","⠦","⠧","⠇","⠏"]),
    );
    sp.enable_steady_tick(Duration::from_millis(60));

    let eco_osv = ecosystem.to_string();

    let tuples: Vec<(String, String, String)> = names.iter()
        .map(|spec| {
            let (name, ver) = parse_pkg_spec(spec);
            if !ver.is_empty() {
                return (name, eco_osv.clone(), ver);
            }
            // No version specified — resolve latest from registry
            sp.set_message(format!("Resolving latest version of {}...", name));
            match crate::engine::registry::fetch_latest_version(&name, &eco_osv) {
                Some(latest) => {
                    sp.suspend(|| {
                        println!(
                            "  {} {} {} {}",
                            "→".truecolor(80,80,100),
                            name.bold(),
                            "latest:".truecolor(100,100,120),
                            latest.bright_cyan().bold()
                        );
                    });
                    (name, eco_osv.clone(), latest)
                }
                None => {
                    sp.suspend(|| {
                        println!(
                            "  {} Could not resolve latest version for {} — skipping vulnerability check",
                            "⚠".bright_yellow(),
                            name.bold()
                        );
                    });
                    // Return empty version → will return no hits → fail-open for this pkg
                    (name, eco_osv.clone(), String::new())
                }
            }
        })
        .collect();

    sp.set_message(format!(
        "Checking {} package(s) against vulnerability database...",
        names.len()
    ));

    let results = match osv::batch_query(&tuples) {
        Ok(r)  => r,
        Err(e) => {
            sp.finish_and_clear();
            Logger::raw_dim(&format!("  CVE check skipped (API error): {}", e));
            return (true, vec![]);
        }
    };
    sp.finish_and_clear();

    let total_hits: usize = results.iter().map(|r| r.len()).sum();
    if total_hits == 0 {
        Logger::success("Security check passed — no known CVEs for requested packages.");
        return (true, vec![]);
    }

    // Fetch full detail for each hit
    let mut hits: Vec<VulnHit> = Vec::new();
    let mut highest_sev = "INFORMATIONAL";

    for (i, vuln_refs) in results.iter().enumerate() {
        if vuln_refs.is_empty() { continue; }
        let pkg_name = &tuples[i].0;
        let pkg_ver  = &tuples[i].2;

        println!();
        println!(
            "  {} {}",
            "⚠".bright_yellow().bold(),
            format!("'{}@{}' has {} known vulnerability(ies):", pkg_name, pkg_ver, vuln_refs.len())
                .bold().bright_yellow()
        );

        for vref in vuln_refs.iter().take(5) {
            match osv::fetch_vuln_detail(&vref.id) {
                Ok(detail) => {
                    let sev      = osv::severity_label(&detail);
                    let fixed    = osv::best_fixed_version(&detail, pkg_name, &eco_osv);
                    let summary  = detail.summary.clone().unwrap_or_else(|| "No description available".to_string());
                    let up_cmd   = fixed.as_deref().map(|fv| install_cmd_for_ecosystem(pkg_name, fv, ecosystem));

                    // Track highest severity across all findings
                    highest_sev = escalate_severity(highest_sev, sev);

                    // Per-CVE block
                    let sev_label = match sev {
                        "CRITICAL" => format!(" {} ", sev).bold().on_bright_red().white().to_string(),
                        "HIGH"     => format!(" {} ", sev).bold().on_red().white().to_string(),
                        "MEDIUM"   => format!(" {} ", sev).bold().on_yellow().black().to_string(),
                        "LOW"      => format!(" {} ", sev).bold().on_bright_green().black().to_string(),
                        _          => format!(" {} ", sev).truecolor(120,120,140).to_string(),
                    };

                    println!(
                        "    {}  {}  {}",
                        sev_label,
                        vref.id.truecolor(100, 160, 255),
                        summary.chars().take(80).collect::<String>().truecolor(160, 160, 180)
                    );

                    if let Some(ref fv) = fixed {
                        println!(
                            "       {} safe version: {}   {}",
                            "→".truecolor(80,80,100),
                            fv.bright_green().bold(),
                            up_cmd.as_deref().unwrap_or("").truecolor(100,140,100)
                        );
                    } else {
                        println!(
                            "       {} {}",
                            "→".truecolor(80,80,100),
                            "No fixed version published yet".truecolor(180,120,50)
                        );
                    }

                    hits.push(VulnHit {
                        package:       pkg_name.clone(),
                        cve_id:        vref.id.clone(),
                        severity:      sev,
                        summary:       summary.chars().take(100).collect(),
                        fixed_version: fixed,
                        upgrade_cmd:   up_cmd,
                    });
                }
                Err(_) => {
                    println!(
                        "    {}  {}  {}",
                        " UNKNOWN ".truecolor(120,120,140),
                        vref.id.truecolor(100,160,255),
                        "(could not fetch CVE details)".truecolor(120,120,140)
                    );
                    hits.push(VulnHit {
                        package:       names[i].clone(),
                        cve_id:        vref.id.clone(),
                        severity:      "UNKNOWN",
                        summary:       "Could not fetch CVE details".to_string(),
                        fixed_version: None,
                        upgrade_cmd:   None,
                    });
                }
            }
        }
        if vuln_refs.len() > 5 {
            println!(
                "       {} {} more CVEs — run 'infynon pkg scan' for full report",
                "…".truecolor(80,80,100),
                vuln_refs.len() - 5
            );
        }
    }

    println!();
    (false, hits)
}

pub fn escalate_severity(current: &'static str, new: &'static str) -> &'static str {
    let rank = |s: &str| match s {
        "CRITICAL" => 4, "HIGH" => 3, "MEDIUM" => 2, "LOW" => 1, _ => 0,
    };
    if rank(new) > rank(current) { new } else { current }
}

fn install_cmd_for_ecosystem(pkg: &str, fixed: &str, ecosystem: &str) -> String {
    use crate::ecosystems::detector::resolve_binary;
    match ecosystem {
        "npm"       => format!("npm install {}@{}", pkg, fixed),
        "yarn"      => format!("yarn add {}@{}", pkg, fixed),
        "pnpm"      => format!("pnpm add {}@{}", pkg, fixed),
        "bun"       => format!("bun add {}@{}", pkg, fixed),
        "crates.io" | "cargo" => format!("cargo add {}@{}", pkg, fixed),
        "uv"        => format!("uv add {}=={}", pkg, fixed),
        "poetry"    => format!("poetry add {}=={}", pkg, fixed),
        "PyPI" | "pip" | "pip3" => {
            // Resolve at runtime so Linux systems with only pip3 get the right binary
            format!("{} install {}=={}", resolve_binary("pip"), pkg, fixed)
        }
        "Go" | "go" => {
            let ver = if fixed.starts_with('v') { fixed.to_string() } else { format!("v{}", fixed) };
            format!("go get {}@{}", pkg, ver)
        }
        "RubyGems" | "gem" => {
            format!("{} install {} -v {}", resolve_binary("gem"), pkg, fixed)
        }
        "Packagist" | "composer" => format!("composer require {}:{}", pkg, fixed),
        "NuGet" | "nuget" => format!("dotnet add package {} --version {}", pkg, fixed),
        "Hex" | "hex"     => format!("mix deps.update {}", pkg),
        "pub.dev" | "pub" => {
            format!("{} pub upgrade {}", resolve_binary("dart"), pkg)
        }
        _ => format!("upgrade {} to {}", pkg, fixed),
    }
}

/// Parse a package spec string (CLI argument) into (name, version) for ALL ecosystems:
///
///   npm/yarn/pnpm/bun/cargo/go:   `name@version`  `@scope/name@version`
///   pip/uv/poetry:                `name==version`  `name>=version`  `name~=version`
///   gem (CLI):                    `name:version`   or just `name`
///   composer:                     `vendor/pkg:version`
///   nuget:                        `Package.Name`   (version is space-separated arg)
///   hex/pub:                      `name`           (no version in CLI format)
/// Public alias used by commands.rs.
pub fn parse_pkg_spec(spec: &str) -> (String, String) {
    let spec = spec.trim();

    // ── Scoped npm package: @scope/name@version ──────────────────────────
    // e.g. @types/node@20.0.0  →  name=@types/node  ver=20.0.0
    if spec.starts_with('@') {
        // The version separator is the LAST '@' at position > 0
        if let Some(pos) = spec[1..].rfind('@') {
            let pos = pos + 1; // offset back
            let name = spec[..pos].to_string();
            let ver  = spec[pos + 1..].to_string();
            if !ver.is_empty() {
                return (name, ver);
            }
        }
        return (spec.to_string(), String::new());
    }

    // ── pip/uv/poetry: name==version  name>=version  name~=version ───────
    // Order matters: check multi-char operators first
    for sep in &["==", "~=", "<=", ">=", "!="] {
        if let Some(pos) = spec.find(sep) {
            // Strip extras: requests[security]==... → requests
            let raw_name = spec[..pos].trim();
            let name = raw_name.split('[').next().unwrap_or(raw_name).trim().to_string();
            // Only take the first version constraint if multiple (e.g. >=1.0,<2.0)
            let ver = spec[pos + sep.len()..]
                .split(',')
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            if !name.is_empty() && !ver.is_empty() {
                return (name, ver);
            }
        }
    }
    // Single-char pip constraints (> / <) — less precise, use as fallback
    for sep in &[">", "<"] {
        if let Some(pos) = spec.find(sep) {
            let name = spec[..pos].split('[').next().unwrap_or(&spec[..pos]).trim().to_string();
            let ver  = spec[pos + 1..].split(',').next().unwrap_or("").trim().to_string();
            if !name.is_empty() && !ver.is_empty() {
                return (name, ver);
            }
        }
    }

    // ── gem CLI / composer: name:version ─────────────────────────────────
    // Disambiguate from URLs (http://...) and Go modules with slashes
    if spec.contains(':') && !spec.starts_with("http") && !spec.starts_with("git") {
        if let Some(pos) = spec.find(':') {
            let name = spec[..pos].trim().to_string();
            let ver  = spec[pos + 1..].trim().to_string();
            if !name.is_empty() && !ver.is_empty() {
                return (name, ver);
            }
        }
    }

    // ── npm / cargo / go / bun / pnpm: name@version ──────────────────────
    if let Some(pos) = spec.find('@') {
        let name = spec[..pos].trim().to_string();
        let ver  = spec[pos + 1..].trim().to_string();
        if !name.is_empty() && !ver.is_empty() {
            return (name, ver);
        }
    }

    // ── nuget / hex / pub: bare name (version comes from lock file) ───────
    (spec.to_string(), String::new())
}
