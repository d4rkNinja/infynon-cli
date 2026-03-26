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
    println!();
    Logger::title("INFYNON Package Scanner", "blue");

    // 1. Detect lock files
    if let Some(f) = pkg_file {
        Logger::step(&format!("Using custom file: {}", f));
    } else {
        Logger::step("Detecting lock files in current directory...");
    }
    let packages = scanner::detect_locked_packages(pkg_file);

    if packages.is_empty() {
        Logger::error("No packages found in supported lock/manifest files.");
        Logger::info("Supported: package-lock.json · yarn.lock · pnpm-lock.yaml · requirements.txt");
        Logger::info("           poetry.lock · Cargo.lock · go.sum · Gemfile.lock · composer.lock");
        Logger::info("           mix.lock · pubspec.lock  — or pass --pkg-file <path>");
        return;
    }

    let mut sources: Vec<String> = packages.iter().map(|p| p.source.clone()).collect();
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
    pb.finish_with_message(format!("{} packages queued → sending to OSV...", packages.len()));

    let results = match osv::batch_query(&tuples) {
        Ok(r) => r,
        Err(e) => { Logger::error(&format!("OSV API error: {}", e)); return; }
    };

    // 3. Fetch full vuln details — show CVE ID + package name live
    let total_vulns: usize = results.iter().map(|r| r.len()).sum();
    if total_vulns == 0 {
        println!();
        Logger::success("No known vulnerabilities found for your dependency tree!");
        return;
    }

    println!();
    let detail_pb = ProgressBar::new(total_vulns as u64);
    detail_pb.set_style(
        ProgressStyle::with_template(
            "  {spinner:.yellow}  fetching  {msg:<45} [{bar:40.yellow/blue}] {pos}/{len} CVEs"
        )
        .unwrap()
        .tick_strings(&["⠋","⠙","⠹","⠸","⠼","⠴","⠦","⠧","⠇","⠏"])
        .progress_chars("█▉▊▋▌▍▎▏  "),
    );
    detail_pb.enable_steady_tick(Duration::from_millis(60));

    let mut findings: Vec<reporter::ScanFinding> = Vec::new();

    for (i, vuln_refs) in results.iter().enumerate() {
        for vref in vuln_refs {
            detail_pb.set_message(format!("{} ({})", vref.id, packages[i].name));
            detail_pb.inc(1);
            match osv::fetch_vuln_detail(&vref.id) {
                Ok(detail) => {
                    let severity = osv::severity_label(&detail);
                    let include  = fix_level.as_ref().map_or(true, |fl| fl.matches(severity));
                    if include {
                        let fixed_version = osv::first_fixed_version(&detail);
                        findings.push(reporter::ScanFinding {
                            package:      packages[i].clone(),
                            vuln:         detail,
                            severity,
                            fixed_version,
                        });
                    }
                }
                Err(e) => {
                    detail_pb.suspend(|| {
                        eprintln!("  {} could not fetch {}: {}", "warn:".yellow(), vref.id, e);
                    });
                }
            }
        }
    }
    detail_pb.finish_with_message(format!("{} CVE records fetched", total_vulns));
    println!();

    // 4. Summary table
    println!();
    print_summary_table(&findings);

    // 5. Inline fix report (always shown when there are findings)
    let has_fixes = findings.iter().any(|f| f.fixed_version.is_some());
    if has_fixes {
        print_fix_report(&findings);
    }

    // 5b. Auto-execute remediation commands when --fix was explicitly passed
    if fix_level.is_some() && has_fixes {
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

/// Execute all remediation upgrade commands with live progress spinners.
fn run_auto_fix(findings: &[reporter::ScanFinding]) {
    use std::process::Command;
    use std::collections::HashSet;

    let fixable: Vec<(&reporter::ScanFinding, String)> = findings.iter()
        .filter_map(|f| f.fixed_version.as_deref().map(|v| (f, upgrade_cmd(&f.package, v))))
        .collect();

    if fixable.is_empty() {
        Logger::info("No packages have a known fixed version available.");
        return;
    }

    println!();
    println!(
        "  {} {}\n",
        "⚡ Auto-Fix".bold().truecolor(255, 200, 50),
        format!("Executing {} remediation command(s)...", fixable.len())
            .truecolor(160, 160, 180)
    );

    // Deduplicate identical commands (e.g. same package mentioned in two CVEs)
    let mut seen: HashSet<String> = HashSet::new();
    let mut success_count = 0usize;
    let mut fail_count    = 0usize;

    for (finding, cmd) in &fixable {
        if !seen.insert(cmd.clone()) { continue; }

        let pkg_label = format!(
            "{} {} → {}",
            finding.package.name,
            finding.package.version,
            finding.fixed_version.as_deref().unwrap_or("?")
        );

        // Parse shell command into binary + args
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.is_empty() { continue; }

        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::with_template("  {spinner:.green}  {msg}")
                .unwrap()
                .tick_strings(&["⠋","⠙","⠹","⠸","⠼","⠴","⠦","⠧","⠇","⠏","✔"]),
        );
        spinner.enable_steady_tick(Duration::from_millis(60));
        spinner.set_message(format!(
            "{} {}",
            pkg_label.bold(),
            format!("({})", cmd).truecolor(100,100,120)
        ));

        let result = Command::new(parts[0])
            .args(&parts[1..])
            .output();

        match result {
            Ok(out) => {
                spinner.finish_and_clear();
                if out.status.success() {
                    success_count += 1;
                    println!(
                        "  {}  {} {}",
                        "✔".bright_green().bold(),
                        pkg_label.bold(),
                        "fixed".bright_green()
                    );
                } else {
                    fail_count += 1;
                    println!(
                        "  {}  {} — command exited with code {}",
                        "✘".bright_red().bold(),
                        pkg_label.bold(),
                        out.status.code().unwrap_or(-1)
                    );
                    // Print stderr if present
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    for line in stderr.lines().take(6) {
                        println!("       {} {}", "│".truecolor(80,80,100), line.truecolor(200,80,80));
                    }
                    // Print stdout tail if stderr empty
                    if stderr.trim().is_empty() {
                        let stdout = String::from_utf8_lossy(&out.stdout);
                        for line in stdout.lines().take(6) {
                            println!("       {} {}", "│".truecolor(80,80,100), line.truecolor(180,180,180));
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
                    pkg_label.bold(),
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
        else              { "0 failed".truecolor(100,100,120).to_string() }
    );
    println!();
}

// ── Tables ────────────────────────────────────────────────────────────────────

fn print_summary_table(findings: &[reporter::ScanFinding]) {
    use tabled::{Table, Tabled};
    use tabled::settings::{Style, Padding, object::Rows, Color};

    // Plain-text risk label (no ANSI) for tabled — color is added as row prefix
    #[derive(Tabled)]
    struct Row {
        #[tabled(rename = " Risk ")]      sev: String,
        #[tabled(rename = " Package ")]   pkg: String,
        #[tabled(rename = " Version ")]   ver: String,
        #[tabled(rename = " CVE / ID ")] cve: String,
    }

    let rows: Vec<Row> = findings.iter().map(|f| Row {
        sev: f.severity.to_string(),          // plain text — no ANSI
        pkg: f.package.name.chars().take(28).collect(),
        ver: f.package.version.chars().take(14).collect(),
        cve: f.vuln.id.clone(),
    }).collect();

    let mut table = Table::new(rows);
    table
        .with(Style::modern())
        .with(Padding::new(1, 1, 0, 0))
        .modify(Rows::first(), Color::BOLD | Color::FG_BRIGHT_CYAN);

    println!("  {}\n", "Vulnerability Summary:".bold().white());
    println!("{}", table);

    // Print summary text below each finding as indented block
    for f in findings {
        if let Some(ref s) = f.vuln.summary {
            let short: String = s.chars().take(90).collect();
            println!(
                "       {} {} {}  {}",
                "→".truecolor(80,80,100),
                f.vuln.id.truecolor(100,160,255),
                f.package.name.bold(),
                short.truecolor(160,160,180)
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
}

/// Fix report — 5-column narrow table: package, old ver, fix ver, risk, CVE.
/// Upgrade command + reason printed as indented lines below.
fn print_fix_report(findings: &[reporter::ScanFinding]) {
    use tabled::{Table, Tabled};
    use tabled::settings::{Style, Padding, object::Rows, Color};

    // Plain-text risk in fix table too
    #[derive(Tabled)]
    struct FixRow {
        #[tabled(rename = " Risk ")]      risk:    String,
        #[tabled(rename = " Package ")]   pkg:     String,
        #[tabled(rename = " Current ")]   old_ver: String,
        #[tabled(rename = " Fix Ver ")]   new_ver: String,
        #[tabled(rename = " CVE ID ")]    id:      String,
    }

    let fixable: Vec<&reporter::ScanFinding> = findings.iter()
        .filter(|f| f.fixed_version.is_some())
        .collect();

    if fixable.is_empty() { return; }

    let rows: Vec<FixRow> = fixable.iter().map(|f| {
        let fixed = f.fixed_version.as_deref().unwrap_or("N/A");
        FixRow {
            risk:    f.severity.to_string(),
            pkg:     f.package.name.chars().take(25).collect(),
            old_ver: f.package.version.chars().take(12).collect(),
            new_ver: fixed.chars().take(12).collect(),
            id:      f.vuln.id.clone(),
        }
    }).collect();

    let mut table = Table::new(rows);
    table
        .with(Style::modern())
        .with(Padding::new(1, 1, 0, 0))
        .modify(Rows::first(), Color::BOLD | Color::FG_BRIGHT_YELLOW);

    println!("  {}\n", "Fix Recommendations:".bold().truecolor(255, 170, 50));
    println!("{}", table);

    // Print upgrade command + reason below the table
    for f in &fixable {
        let fixed = f.fixed_version.as_deref().unwrap_or("N/A");
        let cmd   = upgrade_cmd(&f.package, fixed);
        let why   = f.vuln.summary.as_deref().unwrap_or("CVE in this version range").chars().take(80).collect::<String>();
        println!(
            "       {} {}  {} {}",
            "run:".truecolor(80,80,100),
            cmd.bright_green().bold(),
            "#".truecolor(60,60,80),
            why.truecolor(140,140,160)
        );
    }
    println!();
    println!(
        "  {}  Upgrade to the 'Fix Ver' column value to remediate each finding.\n",
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
    match pkg.ecosystem.as_str() {
        "npm"       => format!("npm install {}@{}", pkg.name, fixed),
        "crates.io" => format!("cargo add {}@{}", pkg.name, fixed),
        "PyPI"      => format!("pip install {}=={}", pkg.name, fixed),
        "Go"        => format!("go get {}@v{}", pkg.name, fixed),
        "RubyGems"  => format!("gem install {} -v {}", pkg.name, fixed),
        "Packagist" => format!("composer require {}:{}", pkg.name, fixed),
        "NuGet"     => format!("dotnet add package {} --version {}", pkg.name, fixed),
        "Hex"       => format!("mix deps.update {}", pkg.name),
        "pub.dev"   => format!("dart pub upgrade {}", pkg.name),
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
                            "  {} Could not resolve latest version for {} — skipping OSV check for this package",
                            "⚠".bright_yellow(),
                            name.bold()
                        );
                    });
                    // Return empty version → will return no OSV hits → fail-open for this pkg
                    (name, eco_osv.clone(), String::new())
                }
            }
        })
        .collect();

    sp.set_message(format!(
        "Checking {} package(s) against OSV vulnerability database...",
        names.len()
    ));

    let results = match osv::batch_query(&tuples) {
        Ok(r)  => r,
        Err(e) => {
            sp.finish_and_clear();
            Logger::raw_dim(&format!("  OSV check skipped (API error): {}", e));
            return (true, vec![]);
        }
    };
    sp.finish_and_clear();

    let total_hits: usize = results.iter().map(|r| r.len()).sum();
    if total_hits == 0 {
        Logger::success("OSV check passed — no known CVEs for requested packages.");
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
                    let fixed    = osv::first_fixed_version(&detail);
                    let summary  = detail.summary.clone().unwrap_or_else(|| "No description available".to_string());
                    let eco_norm = ecosystem.to_string();
                    let up_cmd   = fixed.as_deref().map(|fv| install_cmd_for_ecosystem(pkg_name, fv, &eco_norm));

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
    match ecosystem {
        "npm"       => format!("npm install {}@{}", pkg, fixed),
        "yarn"      => format!("yarn add {}@{}", pkg, fixed),
        "pnpm"      => format!("pnpm add {}@{}", pkg, fixed),
        "bun"       => format!("bun add {}@{}", pkg, fixed),
        "crates.io" => format!("cargo add {}@{}", pkg, fixed),
        "PyPI"      => format!("pip install {}=={}", pkg, fixed),
        "Go"        => {
            // Go versions need v-prefix; don't double-add it
            let ver = if fixed.starts_with('v') { fixed.to_string() } else { format!("v{}", fixed) };
            format!("go get {}@{}", pkg, ver)
        },
        "RubyGems"  => format!("gem install {} -v {}", pkg, fixed),
        "Packagist" => format!("composer require {}:{}", pkg, fixed),
        "NuGet"     => format!("dotnet add package {} --version {}", pkg, fixed),
        "Hex"       => format!("mix deps.update {}", pkg),
        "pub.dev"   => format!("dart pub upgrade {}", pkg),
        _           => format!("upgrade {} to {}", pkg, fixed),
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
