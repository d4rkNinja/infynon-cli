use clap::Parser;
use crate::cli::args::{
    AiAction, ApiCommands, AssertionAction, EnvAction, FlowAction, NodeAction, PromptAction,
    PkgArgs, PkgCommands, RootArgs, RootCommands,
};
use crate::cli::scan::{self, run_scan, check_packages_before_install, OutputFormat, FixLevel};
use crate::cli::features;
use crate::error::types::InfynonError;
use crate::tui::logger::Logger;
use crate::ecosystems::detector;
use std::path::Path;
use owo_colors::OwoColorize;
use crate::utils::truncate_str;

pub fn execute_pkg_mode() -> Result<(), InfynonError> {
    // When invoked as `infynon pkg ...`, strip the "pkg" arg before clap parses
    let raw_args: Vec<String> = std::env::args().collect();
    let args = if raw_args.len() > 1 && raw_args[1] == "pkg" {
        let filtered: Vec<String> = std::iter::once(raw_args[0].clone())
            .chain(raw_args[2..].iter().cloned())
            .collect();
        PkgArgs::parse_from(filtered)
    } else {
        PkgArgs::parse()
    };

    if let Err(message) = crate::cli::validate::validate_pkg_args(&args) {
        Logger::error(&message);
        return Ok(());
    }

    // ── Route subcommands first ────────────────────────────────────────────
    if let Some(cmd) = args.command {
        match cmd {
            PkgCommands::Scan { output, fix, pkg_file } => {
                let fmt = output.as_deref().map(|o| match o.to_lowercase().as_str() {
                    "pdf"  => OutputFormat::Pdf,
                    "both" => OutputFormat::Both,
                    _      => OutputFormat::Markdown,
                });
                let fl   = fix.map(|f| FixLevel::from_str(&f));
                let file = pkg_file.or(args.pkg_file);
                run_scan(fmt, fl, file.as_deref(), args.agent);
                return Ok(());
            }
            PkgCommands::Audit { pkg_file } => {
                let file = pkg_file.or(args.pkg_file);
                features::cmd_audit_deep(file.as_deref());
                return Ok(());
            }
            PkgCommands::Why { package, pkg_file } => {
                let file = pkg_file.or(args.pkg_file);
                features::cmd_why(&package, file.as_deref());
                return Ok(());
            }
            PkgCommands::Outdated { pkg_file } => {
                let file = pkg_file.or(args.pkg_file);
                features::cmd_outdated(file.as_deref());
                return Ok(());
            }
            PkgCommands::Diff { package, v1, v2, ecosystem } => {
                features::cmd_diff(&package, &v1, &v2, ecosystem.as_deref());
                return Ok(());
            }
            PkgCommands::Doctor { pkg_file } => {
                let file = pkg_file.or(args.pkg_file);
                features::cmd_doctor(file.as_deref());
                return Ok(());
            }
            PkgCommands::Size { packages, ecosystem } => {
                if packages.is_empty() {
                    Logger::error("Please specify at least one package name.");
                    return Ok(());
                }
                features::cmd_size(&packages, ecosystem.as_deref());
                return Ok(());
            }
            PkgCommands::Search { query, ecosystem } => {
                features::cmd_search(&query, ecosystem.as_deref());
                return Ok(());
            }
            PkgCommands::Fix { auto: _, pkg_file } => {
                let file = pkg_file.or(args.pkg_file);
                features::cmd_fix_auto(file.as_deref());
                return Ok(());
            }
            PkgCommands::Clean { pkg_file } => {
                let file = pkg_file.or(args.pkg_file);
                features::cmd_clean(file.as_deref());
                return Ok(());
            }
            PkgCommands::Migrate { from, to } => {
                features::cmd_migrate(&from, &to);
                return Ok(());
            }
            PkgCommands::EagleEye { action } => {
                use crate::cli::args::EagleEyeAction;
                match action {
                    EagleEyeAction::Setup => features::eagle_eye::cmd_setup(),
                    EagleEyeAction::Start => features::eagle_eye::cmd_start(),
                    EagleEyeAction::Status => features::eagle_eye::cmd_status(),
                    EagleEyeAction::Enable => features::eagle_eye::cmd_enable(),
                    EagleEyeAction::Disable => features::eagle_eye::cmd_disable(),
                }
                return Ok(());
            }
        }
    }

    if args.passthrough_args.is_empty() {
        Logger::splash_pkg();
        return Ok(());
    }


    let first_arg = &args.passthrough_args[0];
    let known_ecosystems = vec!["npm", "yarn", "pnpm", "bun", "pip", "uv", "poetry", "cargo", "go", "gem", "composer", "nuget", "hex", "pub"];

    let mut ecosystem = "auto-detected";
    let mut install_packages = vec![];
    let mut install_action = String::new();
    let mut cmd_idx = 0;

    if known_ecosystems.contains(&first_arg.as_str()) {
        ecosystem = first_arg;
        cmd_idx = 1;
    } else {
        let p = |f: &str| Path::new(f).exists();

        // ── JavaScript / Node.js ─────────────────────────────────────────────
        // Combined (manifest + lock) takes priority; lock alone next; manifest last
        if      p("package.json") && p("bun.lockb")          { ecosystem = "bun"; }
        else if p("package.json") && p("pnpm-lock.yaml")     { ecosystem = "pnpm"; }
        else if p("package.json") && p("yarn.lock")          { ecosystem = "yarn"; }
        else if p("package.json") && p("package-lock.json")  { ecosystem = "npm"; }
        else if p("bun.lockb")                               { ecosystem = "bun"; }
        else if p("pnpm-lock.yaml")                          { ecosystem = "pnpm"; }
        else if p("yarn.lock")                               { ecosystem = "yarn"; }
        else if p("package.json") || p("package-lock.json")  { ecosystem = "npm"; }
        // ── Rust ─────────────────────────────────────────────────────────────
        else if p("Cargo.toml") && p("Cargo.lock")           { ecosystem = "cargo"; }
        else if p("Cargo.toml")                              { ecosystem = "cargo"; }
        // ── Python ───────────────────────────────────────────────────────────
        // pyproject.toml + lock is most specific; lock alone next; manifest fallback
        else if p("pyproject.toml") && p("uv.lock")          { ecosystem = "uv"; }
        else if p("pyproject.toml") && p("poetry.lock")      { ecosystem = "poetry"; }
        else if p("uv.lock")                                 { ecosystem = "uv"; }
        else if p("poetry.lock")                             { ecosystem = "poetry"; }
        else if p("pyproject.toml") || p("requirements.txt") || p("setup.py") || p("setup.cfg") { ecosystem = "pip"; }
        // ── Go ───────────────────────────────────────────────────────────────
        else if p("go.mod") && p("go.sum")                   { ecosystem = "go"; }
        else if p("go.mod")                                  { ecosystem = "go"; }
        // ── PHP / Composer ───────────────────────────────────────────────────
        else if p("composer.json") && p("composer.lock")     { ecosystem = "composer"; }
        else if p("composer.json") || p("composer.lock")     { ecosystem = "composer"; }
        // ── Ruby ─────────────────────────────────────────────────────────────
        else if p("Gemfile") && p("Gemfile.lock")            { ecosystem = "gem"; }
        else if p("Gemfile") || p("Gemfile.lock")            { ecosystem = "gem"; }
        // ── Dart / Flutter ───────────────────────────────────────────────────
        else if p("pubspec.yaml") && p("pubspec.lock")       { ecosystem = "pub"; }
        else if p("pubspec.yaml") || p("pubspec.lock")       { ecosystem = "pub"; }
        // ── Elixir / Hex ─────────────────────────────────────────────────────
        else if p("mix.exs") && p("mix.lock")                { ecosystem = "hex"; }
        else if p("mix.exs") || p("mix.lock")                { ecosystem = "hex"; }
    }

    if args.passthrough_args.len() > cmd_idx {
        let action = &args.passthrough_args[cmd_idx];
        // Recognize all common "add a package" actions across ecosystems:
        // install/add/i/require/get = initial install
        // update/upgrade/up         = upgrade specific packages (also needs CVE check)
        if matches!(action.as_str(),
            "install" | "add" | "i" | "require" | "get" |
            "update"  | "upgrade" | "up"
        ) {
            install_action = action.clone();
            install_packages = args.passthrough_args[cmd_idx + 1..].to_vec();
        }
    }

    // ── Binary availability check ────────────────────────────────────────
    // Maps ecosystem identifier → the binary name to check for on PATH.
    // nuget → dotnet  (modern .NET uses the dotnet CLI, not the legacy nuget.exe)
    // hex   → mix     (Elixir's build tool that manages hex packages)
    // pub   → dart    (Dart SDK ships dart; Flutter also exposes it as flutter)
    let binary_to_check = match ecosystem {
        "poetry" => "poetry",
        "uv"     => "uv",
        "hex"    => "mix",
        "pub"    => "dart",
        "nuget"  => "dotnet",
        other    => other,
    };

    if !detector::is_installed(binary_to_check) {
        println!();
        println!(
            "  {} {}{}{}\n",
            "✘".red().bold(),
            "Package manager ".red().bold(),
            format!("'{}'" , ecosystem).bright_red().bold(),
            " is not installed on this system.".red().bold()
        );
        if let Some(info) = detector::install_instructions(ecosystem) {
            println!("  {}  {}", "ℹ".bright_cyan().bold(), info.note.white());
            println!();
            println!("  {} {}", "Install command:".bold().truecolor(255,170,50), info.install_cmd.bright_green());
            println!("  {} {}", "Official docs:  ".bold().truecolor(255,170,50), info.install_url.truecolor(100,150,255));
        }
        println!();
        return Ok(());
    }

    // Resolve the actual binary name that is present on this system.
    // e.g. "pip" may resolve to "pip3" on Linux; "dart" may resolve to "flutter".
    let actual_binary = detector::resolve_binary(binary_to_check);

    if !args.agent {
        Logger::subtitle("🛡️", "INFYNON Secure Proxy", "Active");
        Logger::detail("» Ecosystem:", ecosystem);
        Logger::success(&format!("'{}' binary found — proceeding", actual_binary));
    }

    if !install_packages.is_empty() {
        let (safe, hits) = check_packages_before_install(&install_packages, ecosystem, args.agent);

        let pkgs_to_install = if !safe {
            // ── --strict: block entire install if any hit matches the level ──
            if let Some(ref strict_level) = args.strict {
                let level = scan::FixLevel::from_str(strict_level);
                let blocked = hits.iter().any(|h| level.matches(h.severity));
                if blocked {
                    if args.agent {
                        let vulns: Vec<serde_json::Value> = hits.iter().map(|h| serde_json::json!({
                            "package":         h.package,
                            "current_version": "",
                            "cve_id":          h.cve_id,
                            "severity":        h.severity,
                            "summary":         h.summary,
                            "safe_version":    h.fixed_version,
                            "fix_cmd":         h.upgrade_cmd
                        })).collect();
                        let json = serde_json::json!({
                            "status":           "blocked",
                            "packages_checked": install_packages,
                            "vulnerabilities":  vulns,
                            "installed":        false,
                            "blocked_by":       format!("--strict {}", strict_level)
                        });
                        println!("{}", serde_json::to_string_pretty(&json).unwrap());
                        std::process::exit(3);
                    }
                    let level_label = if strict_level == "all" { "all severities".to_string() } else { format!("{}+", strict_level) };
                    println!(
                        "\n  {}  {} — {}  (blocking: {})\n",
                        "╳".bright_red().bold(),
                        "BLOCKED".bold().bright_red(),
                        "--strict mode active".truecolor(200,80,80),
                        level_label.truecolor(200,120,80)
                    );
                    std::process::exit(3);
                }
            }

            // ── CI non-interactive flags ──────────────────────────────────────
            if args.yes {
                // Install everything regardless of vulnerabilities
                println!(
                    "\n  {}  {} — installing all packages (including vulnerable)\n",
                    "⚠".bright_yellow().bold(),
                    "--yes mode".bold().bright_yellow(),
                );
                install_packages.clone()
            } else if args.skip_vulnerable {
                // Skip every vulnerable package, install only clean ones
                let vuln_names: std::collections::HashSet<String> =
                    hits.iter().map(|h| h.package.clone()).collect();
                let safe_specs: Vec<String> = install_packages.iter()
                    .filter(|spec| {
                        let (name, _) = scan::parse_pkg_spec(spec);
                        !vuln_names.contains(&name)
                    })
                    .cloned()
                    .collect();
                for name in &vuln_names {
                    println!("  {}  Skipping vulnerable: {}", "✘".bright_red(), name.bold());
                }
                if safe_specs.is_empty() {
                    Logger::raw_dim("  All packages were vulnerable and skipped. Nothing to install.");
                    return Ok(());
                }
                safe_specs
            } else if args.auto_fix {
                // Auto-upgrade to safe version; skip if no fix available
                let mut fix_map: std::collections::HashMap<String, Option<String>> =
                    std::collections::HashMap::new();
                for h in &hits {
                    let entry = fix_map.entry(h.package.clone()).or_insert(None);
                    if h.fixed_version.is_some() {
                        *entry = h.fixed_version.clone();
                    }
                }
                let vuln_names: std::collections::HashSet<String> =
                    hits.iter().map(|h| h.package.clone()).collect();
                let mut auto_specs: Vec<String> = Vec::new();
                for spec in &install_packages {
                    let (name, _) = scan::parse_pkg_spec(spec);
                    if vuln_names.contains(&name) {
                        match fix_map.get(&name).and_then(|v| v.clone()) {
                            Some(ver) => {
                                let new_spec = format_spec_for_ecosystem(&name, &ver, ecosystem);
                                println!("  {}  Auto-fix: {} → {}", "✔".bright_green(), name.bold(), new_spec.bright_green().bold());
                                auto_specs.push(new_spec);
                            }
                            None => {
                                println!("  {}  No fix available for {} — skipping", "✘".bright_red(), name.bold());
                            }
                        }
                    } else {
                        auto_specs.push(spec.clone());
                    }
                }
                if auto_specs.is_empty() {
                    Logger::raw_dim("  Nothing to install after auto-fix resolution.");
                    return Ok(());
                }
                auto_specs
            } else {
                // Interactive mode (default — not suitable for CI)
                let final_specs = ask_vuln_decisions(&install_packages, &hits, ecosystem);
                if final_specs.is_empty() {
                    Logger::raw_dim("  All packages skipped. Nothing to install.");
                    return Ok(());
                }
                final_specs
            }
        } else {
            install_packages.clone()
        };

        // Build and run the real install command in the user's working directory
        let mut cmd_parts = vec![install_action.clone()];
        cmd_parts.extend(pkgs_to_install.iter().cloned());
        let cmd = build_proxy_cmd(ecosystem, &actual_binary, &cmd_parts);
        if !args.agent {
            println!();
            Logger::step(&format!("Running: {}", cmd));
            println!();
        }
        let install_ok = crate::cli::proxy_pkg_cmd(&cmd).is_ok();
        if !install_ok && !args.agent {
            Logger::error(&format!("Failed to execute '{}'", actual_binary));
        }

        if args.agent {
            let vulns: Vec<serde_json::Value> = hits.iter().map(|h| serde_json::json!({
                "package":         h.package,
                "current_version": "",
                "cve_id":          h.cve_id,
                "severity":        h.severity,
                "summary":         h.summary,
                "safe_version":    h.fixed_version,
                "fix_cmd":         h.upgrade_cmd
            })).collect();
            let has_medium_plus = hits.iter().any(|h| matches!(h.severity, "CRITICAL"|"HIGH"|"MEDIUM"));
            let status = if hits.is_empty() { "clean" } else if has_medium_plus { "vulnerable" } else { "warnings" };
            let exit_code: i32 = if hits.is_empty() { 0 } else if has_medium_plus { 2 } else { 1 };
            let json = serde_json::json!({
                "status":           status,
                "packages_checked": install_packages,
                "vulnerabilities":  vulns,
                "installed":        install_ok,
                "install_cmd":      cmd
            });
            println!("{}", serde_json::to_string_pretty(&json).unwrap());
            std::process::exit(exit_code);
        }
    } else {
        // Pass-through: forward all args directly to the real package manager binary.
        // This covers non-install commands: npm run, cargo build, pip list, etc.
        let cmd = build_proxy_cmd(ecosystem, &actual_binary, &args.passthrough_args[cmd_idx..].to_vec());
        println!();
        Logger::step(&format!("Running: {}", cmd));
        println!();
        if let Err(e) = crate::cli::proxy_pkg_cmd(&cmd) {
            Logger::error(&format!("Failed to execute '{}': {}", actual_binary, e));
        }
    }

    Ok(())
}

// ── Proxy command builder ─────────────────────────────────────────────────────

/// Build the full shell command string for the real package manager.
///
/// `ecosystem`      — the logical identifier ("npm", "pip", "nuget", "pub", …)
/// `actual_binary`  — the binary that was actually found on PATH
///                    (e.g. "pip3" instead of "pip", "flutter" instead of "dart")
///
/// Ecosystem-specific command shapes:
/// - `pub` → `<dart|flutter> pub <args>`  (pub is a sub-tool of the Dart SDK)
/// - all others → `<actual_binary> <args>`
fn build_proxy_cmd(ecosystem: &str, actual_binary: &str, args: &[String]) -> String {
    let suffix = args.join(" ");
    match ecosystem {
        "pub" => format!("{} pub {}", actual_binary, suffix),
        _     => format!("{} {}", actual_binary, suffix),
    }
}

// ── Interactive vulnerability decision prompt ─────────────────────────────────

#[derive(Debug, Clone)]
enum PkgAction {
    /// Install the original (vulnerable) version anyway
    InstallVulnerable,
    /// Skip — don't install this package
    Skip,
    /// Install the recommended safe version
    InstallFixed(String),
}

/// Show a per-package decision prompt for all vulnerable packages in the install list.
/// Returns the final list of package specs to actually install.
fn ask_vuln_decisions(
    original_specs: &[String],
    hits: &[scan::VulnHit],
    ecosystem: &str,
) -> Vec<String> {
    use std::collections::HashMap;
    use std::io::{self, Write};

    // Build: package_name → (best fixed_version, is_clean)
    let mut fix_map: HashMap<String, (Option<String>, bool)> = HashMap::new();
    for h in hits {
        let entry = fix_map.entry(h.package.clone()).or_insert((None, false));
        if h.fixed_version.is_some() {
            entry.0 = h.fixed_version.clone();
            entry.1 = h.fix_is_clean;
        }
    }

    // Packages that hit vulnerabilities (by parsed name)
    let vuln_names: std::collections::HashSet<String> = hits.iter()
        .map(|h| h.package.clone())
        .collect();

    // ── Summary header ────────────────────────────────────────────────────────
    println!();
    println!(
        "  {} {} vulnerable package(s) in your install list:\n",
        "⚠".bold().bright_yellow(),
        vuln_names.len()
    );

    for (idx, name) in vuln_names.iter().enumerate() {
        let (fixed, is_clean) = fix_map.get(name)
            .map(|(v, c)| (v.clone(), *c))
            .unwrap_or((None, true));
        let cves: Vec<_> = hits.iter().filter(|h| &h.package == name).collect();
        let worst_sev = cves.iter().map(|h| h.severity).fold("INFORMATIONAL", scan::escalate_severity);
        let sev_colored = scan::severity_colored(worst_sev);
        let fix_hint = match fixed.as_deref() {
            Some(f) if is_clean => format!(" → safe: {}", f.bright_green()),
            Some(f)             => format!(" → reduced risk: {} {}", f.bright_yellow(), "(still has CVEs)".truecolor(160,120,50)),
            None                => " (no fix available)".truecolor(160,100,50).to_string(),
        };
        println!(
            "  {}  {}  [{}]  {} CVE(s){}",
            format!("{:>2}.", idx+1).truecolor(80,80,100),
            name.bold(),
            sev_colored,
            cves.len(),
            fix_hint
        );
    }
    println!();

    // ── Apply-to-all shortcut ─────────────────────────────────────────────────
    println!("  {}  Apply same action to ALL infected packages?", "→".truecolor(100,100,140));
    println!(
        "     {}  Install anyway (vulnerable)   {}  Skip all   {}  Install recommended   {}  Decide per package\n",
        "[1]".bold().bright_yellow(),
        "[2]".bold().bright_red(),
        "[3]".bold().bright_green(),
        "[4]".bold().bright_cyan(),
    );
    print!("  Choice (1/2/3/4): ");
    io::stdout().flush().ok();

    let mut global_choice = String::new();
    io::stdin().read_line(&mut global_choice).ok();
    let global_action: Option<PkgAction> = match global_choice.trim() {
        "1" => Some(PkgAction::InstallVulnerable),
        "2" => Some(PkgAction::Skip),
        "3" => {
            // Best fix per package
            Some(PkgAction::InstallFixed("__per_pkg__".to_string()))
        }
        _ => None, // Per-package
    };

    // ── Build per-package decisions ───────────────────────────────────────────
    let mut decisions: HashMap<String, PkgAction> = HashMap::new();

    if let Some(ref ga) = global_action {
        for name in &vuln_names {
            let action = match ga {
                PkgAction::InstallFixed(_) => {
                    let fixed = fix_map.get(name).and_then(|(v, _)| v.clone());
                    match fixed {
                        Some(f) => PkgAction::InstallFixed(f),
                        None    => {
                            println!(
                                "  {} No fix for {} — falling back to: install vulnerable",
                                "⚠".bright_yellow(), name.bold()
                            );
                            PkgAction::InstallVulnerable
                        }
                    }
                }
                other => other.clone(),
            };
            decisions.insert(name.clone(), action);
        }
    } else {
        // Per-package prompts
        println!();
        for name in &vuln_names {
            let (fixed, is_clean) = fix_map.get(name)
                .map(|(v, c)| (v.clone(), *c))
                .unwrap_or((None, true));
            println!(
                "\n  Package: {}",
                name.bold().bright_white()
            );
            match &fixed {
                Some(f) if is_clean => println!(
                    "  {}  Install anyway   {}  Skip   {}  Install {} {}",
                    "[1]".bold().bright_yellow(),
                    "[2]".bold().bright_red(),
                    "[3]".bold().bright_green(),
                    f.bright_green().bold(),
                    "(verified clean)".bright_green()
                ),
                Some(f) => println!(
                    "  {}  Install anyway   {}  Skip   {}  Install {} {}",
                    "[1]".bold().bright_yellow(),
                    "[2]".bold().bright_red(),
                    "[3]".bold().bright_yellow(),
                    f.bright_yellow().bold(),
                    "(reduces risk, still has CVEs)".truecolor(180,140,50)
                ),
                None => println!(
                    "  {}  Install anyway (no fix available)   {}  Skip",
                    "[1]".bold().bright_yellow(),
                    "[2]".bold().bright_red(),
                ),
            }
            print!("  Choice ({}): ", if fixed.is_some() { "1/2/3" } else { "1/2" });
            io::stdout().flush().ok();

            let mut line = String::new();
            io::stdin().read_line(&mut line).ok();
            let action = match (line.trim(), &fixed) {
                ("2", _)          => PkgAction::Skip,
                ("3", Some(f))    => PkgAction::InstallFixed(f.clone()),
                _                 => PkgAction::InstallVulnerable,
            };
            decisions.insert(name.clone(), action);
        }
    }

    // ── Print decision summary ────────────────────────────────────────────────
    println!();
    println!("  {}  Decision summary:\n", "✦".truecolor(100,160,255));
    for (name, action) in &decisions {
        let label = match action {
            PkgAction::InstallVulnerable   => "install vulnerable".bright_yellow().to_string(),
            PkgAction::Skip                => "skip".bright_red().to_string(),
            PkgAction::InstallFixed(v)     => format!("install {}", v.bright_green()),
        };
        println!("     {}  {} → {}", "·".truecolor(60,60,80), name.bold(), label);
    }
    println!();

    // ── Build final spec list ─────────────────────────────────────────────────
    let mut final_specs: Vec<String> = Vec::new();

    for spec in original_specs {
        let (pkg_name, _) = scan::parse_pkg_spec(spec);

        if let Some(action) = decisions.get(&pkg_name) {
            match action {
                PkgAction::Skip => {
                    println!(
                        "  {} Skipping {}",
                        "✘".bright_red(), pkg_name.bold()
                    );
                }
                PkgAction::InstallFixed(ver) => {
                    let new_spec = format_spec_for_ecosystem(&pkg_name, ver, ecosystem);
                    println!(
                        "  {} {} → {}",
                        "✔".bright_green(), pkg_name.bold(), new_spec.bright_green().bold()
                    );
                    final_specs.push(new_spec);
                }
                PkgAction::InstallVulnerable => {
                    final_specs.push(spec.clone());
                }
            }
        } else {
            // Clean package — always include
            final_specs.push(spec.clone());
        }
    }
    final_specs
}

/// Format a package@version spec for the given ecosystem CLI install syntax.
fn format_spec_for_ecosystem(name: &str, ver: &str, ecosystem: &str) -> String {
    match ecosystem {
        "pip" | "pip3" | "uv" | "poetry" => format!("{}=={}", name, ver),
        "gem"                             => format!("{}:{}", name, ver),
        "composer"                        => format!("{}:{}", name, ver),
        "nuget"                           => format!("{} --version {}", name, ver),
        _                                 => format!("{}@{}", name, ver), // npm/cargo/go/bun/pnpm/yarn/hex/pub
    }
}


pub fn execute_root_mode() -> Result<(), InfynonError> {
    let args = RootArgs::parse();

    match args.command {
        None => Logger::splash_root(),
        Some(RootCommands::Pkg { .. }) => return execute_pkg_mode(),
        Some(RootCommands::Weave { action }) => execute_api_command(action),
        Some(RootCommands::Trace { action }) => crate::trace::commands::execute(action),
    }

    Ok(())
}

// -- API command router --------------------------------------------------------

fn execute_api_command(action: ApiCommands) {
    use crate::api::commands::{ai_cmd, attach, flow, import, node, validate};

    if let Err(message) = crate::cli::validate::validate_api_command(&action) {
        Logger::error(&message);
        return;
    }

    match action {
        ApiCommands::Tui { flow_id } => {
            run_api_tui(flow_id.as_deref());
        }

        ApiCommands::Node { action } => match action {
            NodeAction::Create { ai } => node::cmd_node_create(ai.as_deref()),
            NodeAction::Get { id } => node::cmd_node_get(&id),
            NodeAction::List => node::cmd_node_list(),
            NodeAction::Remove { id } => node::cmd_node_remove(&id),
            NodeAction::Clone { id, new_id } => node::cmd_node_clone(&id, &new_id),
            NodeAction::Run { id, base_url, set, prompt } => {
                let url = match base_url
                    .or_else(|| crate::api::commands::env::env_base_url())
                {
                    Some(u) => u,
                    None => {
                        crate::tui::logger::Logger::error("BASE_URL is not set. Add it to .infynon/.env or pass --base-url <url>");
                        return;
                    }
                };
                node::cmd_node_run(&id, &url, &set, prompt);
            }
            NodeAction::Export { id, format, base_url } => {
                node::cmd_node_export(&id, &format, base_url.as_deref())
            }
            NodeAction::Assertion { node_id, action } => match action {
                AssertionAction::List => node::cmd_node_assertion_list(&node_id),
                AssertionAction::Enable { index } => node::cmd_node_assertion_enable(&node_id, index),
                AssertionAction::Disable { index } => node::cmd_node_assertion_disable(&node_id, index),
                AssertionAction::Toggle { index } => node::cmd_node_assertion_toggle(&node_id, index),
                AssertionAction::Add { check, on_fail } => node::cmd_node_assertion_add(&node_id, &check, &on_fail),
                AssertionAction::Remove { index } => node::cmd_node_assertion_remove(&node_id, index),
            },
            NodeAction::Prompt { node_id, action } => match action {
                PromptAction::List => node::cmd_node_prompt_list(&node_id),
                PromptAction::Add { var, label, secret, default, prompt_type, options } => {
                    node::cmd_node_prompt_add(&node_id, &var, &label, secret, default, &prompt_type, options)
                }
                PromptAction::Remove { index } => node::cmd_node_prompt_remove(&node_id, index),
            },
        },

        ApiCommands::Flow { action } => match action {
            FlowAction::Create { name, ai } => flow::cmd_flow_create(&name, ai.as_deref()),
            FlowAction::List => flow::cmd_flow_list(),
            FlowAction::Show { id } => flow::cmd_flow_show(&id),
            FlowAction::Run { id, base_url, set, output } => flow::cmd_flow_run(&id, base_url.as_deref(), &set, output.as_deref()),
            FlowAction::RunAll { base_url, set, output } => flow::cmd_flow_run_all(base_url.as_deref(), &set, output.as_deref()),
            FlowAction::Remove { id } => flow::cmd_flow_remove(&id),
            FlowAction::Merge { flow1, flow2, join_at, name } => {
                flow::cmd_flow_merge(&flow1, &flow2, &join_at, &name)
            }
        },

        ApiCommands::Validate => validate::cmd_validate(),

        ApiCommands::Attach { from, to, carry, condition, ai } => {
            attach::cmd_attach(&from, &to, &carry, condition.as_deref(), ai)
        }

        ApiCommands::Detach { from, to } => attach::cmd_detach(&from, &to),

        ApiCommands::Ai { action } => match action {
            AiAction::Suggest { after } => ai_cmd::cmd_ai_suggest(&after),
            AiAction::Attach { after, flow } => ai_cmd::cmd_ai_attach(&after, flow.as_deref()),
            AiAction::Complete { flow_id } => ai_cmd::cmd_ai_complete(&flow_id),
            AiAction::Probe { flow_id, base_url } => {
                ai_cmd::cmd_ai_probe(&flow_id, base_url.as_deref())
            }
            AiAction::BuildFlow { nodes, name } => ai_cmd::cmd_ai_build_flow(&nodes, &name),
            AiAction::Explain { flow_id, run } => ai_cmd::cmd_ai_explain(&flow_id, run),
            AiAction::Assert { node_id } => ai_cmd::cmd_ai_assert(&node_id),
            AiAction::Branch { node_id } => ai_cmd::cmd_ai_branch(&node_id),
        },

        ApiCommands::Import { spec, flow, base_url, prefix, dry_run } => {
            import::cmd_import(&spec, flow.as_deref(), base_url.as_deref(), prefix.as_deref(), dry_run);
        }

        ApiCommands::Env { action } => {
            use crate::api::commands::env;
            match action {
                EnvAction::List => env::cmd_env_list(),
                EnvAction::Set { key, value } => env::cmd_env_set(&key, &value),
                EnvAction::Delete { key } => env::cmd_env_delete(&key),
                EnvAction::Get { key, reveal } => env::cmd_env_get(&key, reveal),
            }
        }
    }
}

fn run_api_tui(flow_id: Option<&str>) {
    use crossterm::{
        event::{self, Event, KeyCode},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };
    use ratatui::{backend::CrosstermBackend, Terminal};
    use std::io;

    let _ = enable_raw_mode();
    let mut stdout = io::stdout();
    let _ = execute!(stdout, EnterAlternateScreen);
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = match Terminal::new(backend) {
        Ok(t) => t,
        Err(e) => {
            let _ = disable_raw_mode();
            eprintln!("Failed to initialize terminal: {}", e);
            return;
        }
    };

    let mut app = crate::tui::api_app::ApiApp::new(flow_id);

    loop {
        app.poll_run_events();

        let _ = terminal.draw(|f| {
            crate::tui::api_views::render(f, &mut app);
        });

        if event::poll(std::time::Duration::from_millis(50)).unwrap_or(false) {
            if let Ok(Event::Key(key)) = event::read() {
                // Only handle Press events — ignore Repeat and Release (Windows key-repeat fix)
                if key.kind == crossterm::event::KeyEventKind::Press {
                    if key.code == KeyCode::Char('c')
                        && key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL)
                    {
                        break;
                    }
                    app.handle_key(key);
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    let _ = disable_raw_mode();
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
    let _ = terminal.show_cursor();
}
