use clap::Parser;
use crate::cli::args::{PkgArgs, PkgCommands, FirewallArgs, FirewallCommands};
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
                run_scan(fmt, fl, file.as_deref());
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

    Logger::subtitle("🛡️", "INFYNON Secure Proxy", "Active");
    Logger::detail("» Ecosystem:", ecosystem);
    Logger::success(&format!("'{}' binary found — proceeding", actual_binary));

    if !install_packages.is_empty() {
        let (safe, hits) = check_packages_before_install(&install_packages, ecosystem);

        let pkgs_to_install = if !safe {
            // ── --strict: block entire install if any hit matches the level ──
            if let Some(ref strict_level) = args.strict {
                let level = scan::FixLevel::from_str(strict_level);
                let blocked = hits.iter().any(|h| level.matches(h.severity));
                if blocked {
                    let level_label = if strict_level == "all" { "all severities".to_string() } else { format!("{}+", strict_level) };
                    println!(
                        "\n  {}  {} — {}  (blocking: {})\n",
                        "╳".bright_red().bold(),
                        "BLOCKED".bold().bright_red(),
                        "--strict mode active".truecolor(200,80,80),
                        level_label.truecolor(200,120,80)
                    );
                    std::process::exit(1);
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
        println!();
        Logger::step(&format!("Running: {}", cmd));
        println!();
        if let Err(e) = crate::cli::proxy_pkg_cmd(&cmd) {
            Logger::error(&format!("Failed to execute '{}': {}", actual_binary, e));
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

    // Build: package_name → best fixed_version (highest from all CVEs)
    let mut fix_map: HashMap<String, Option<String>> = HashMap::new();
    for h in hits {
        let entry = fix_map.entry(h.package.clone()).or_insert(None);
        if h.fixed_version.is_some() {
            *entry = h.fixed_version.clone();
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
        let fixed = fix_map.get(name).and_then(|v| v.clone());
        let cves: Vec<_> = hits.iter().filter(|h| &h.package == name).collect();
        let worst_sev = cves.iter().map(|h| h.severity).fold("INFORMATIONAL", scan::escalate_severity);
        let sev_colored = scan::severity_colored(worst_sev);
        let fix_hint = fixed.as_deref()
            .map(|f| format!(" → safe: {}", f.bright_green()))
            .unwrap_or_else(|| " (no fix available)".truecolor(160,100,50).to_string());
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
                    let fixed = fix_map.get(name).and_then(|v| v.clone());
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
            let fixed = fix_map.get(name).and_then(|v| v.clone());
            println!(
                "\n  Package: {}",
                name.bold().bright_white()
            );
            match &fixed {
                Some(f) => println!(
                    "  {}  Install anyway   {}  Skip   {}  Install {}",
                    "[1]".bold().bright_yellow(),
                    "[2]".bold().bright_red(),
                    "[3]".bold().bright_green(),
                    f.bright_green().bold()
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


pub fn execute_firewall_mode(start: std::time::Instant) -> Result<(), InfynonError> {
    use crate::cli::args::{FirewallCommands, RulesAction, ConfigAction};

    let args = FirewallArgs::parse();

    match args.command {
        None => {
            Logger::splash(start);
        }

        // ── Init: create default config ─────────────────────────────────
        Some(FirewallCommands::Init { port, upstream, upstream_port }) => {
            use crate::firewall::config::{init_config, save_firewall_config};
            Logger::title("INFYNON FIREWALL INIT", "blue");
            let config = init_config(port, &upstream, upstream_port);
            match save_firewall_config(&config, Some("infynon.toml")) {
                Ok(()) => {
                    Logger::success("Created infynon.toml with default configuration");
                    Logger::detail("  Listen:", &format!("{}:{}", config.server.listen_address, config.server.listen_port));
                    Logger::detail("  Upstream:", &format!("{}:{}", config.upstream.address, config.upstream.port));
                    Logger::detail("  WAF:", "enabled (SQLi, XSS, path traversal, cmd injection, header injection)");
                    Logger::detail("  Rate limit:", "100 req/min per IP, 1000 req/s global");
                    Logger::detail("  IP filtering:", "blocklist mode (auto-reputation enabled)");
                    Logger::info("Edit infynon.toml to customize. Run `infynon start` to begin.");
                }
                Err(e) => Logger::error(&format!("Failed to create config: {}", e)),
            }
        }

        // ── Start: run the reverse proxy + TUI ──────────────────────────
        Some(FirewallCommands::Start { config, port, upstream, headless }) => {
            use crate::firewall::config::load_firewall_config;

            Logger::title("INFYNON FIREWALL", "red");
            Logger::step("Loading configuration...");

            let mut cfg = load_firewall_config(config.as_deref())
                .map_err(|e| InfynonError::System(e))?;

            // Apply CLI overrides
            if let Some(p) = port { cfg.server.listen_port = p; }
            if let Some(ref u) = upstream {
                if let Some((addr, port_str)) = u.rsplit_once(':') {
                    cfg.upstream.address = addr.to_string();
                    if let Ok(p) = port_str.parse::<u16>() { cfg.upstream.port = p; }
                }
            }

            let listen_addr = format!("{}:{}", cfg.server.listen_address, cfg.server.listen_port);
            let upstream_addr = format!("{}:{}", cfg.upstream.address, cfg.upstream.port);
            Logger::detail("  Listen:", &listen_addr);
            Logger::detail("  Upstream:", &upstream_addr);
            Logger::detail("  WAF:", if cfg.waf.enabled { "enabled" } else { "disabled" });
            Logger::detail("  Rate Limit:", if cfg.rate_limit.enabled { "enabled" } else { "disabled" });
            Logger::detail("  IP Mode:", &cfg.ip.mode.to_string());
            Logger::detail("  Rules:", &format!("{} custom rules loaded", cfg.rules.len()));

            let (state, rt, shutdown_tx) = bootstrap_firewall(cfg, config.clone())?;

            // Start config file watcher (hot-reload)
            let watch_state = state.clone();
            let watch_config_path = config.clone();
            rt.spawn(async move {
                config_watch_loop(watch_state, watch_config_path).await;
            });

            Logger::success(&format!("Firewall running on {}", listen_addr));

            if headless {
                Logger::info("Running in headless mode. Press Ctrl+C to stop.");
                rt.block_on(async {
                    let _ = tokio::signal::ctrl_c().await;
                });
                let _ = shutdown_tx.send(true);
            } else {
                Logger::info("Starting TUI monitor... (press 'q' to quit, '?' for help)");
                println!();
                run_firewall_tui(state.clone());
                let _ = shutdown_tx.send(true);
            }

            // Give server time to shut down gracefully
            rt.block_on(async {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            });

            Logger::success("Firewall stopped.");
        }

        // ── Monitor: TUI only (starts its own proxy) ────────────────────
        Some(FirewallCommands::Monitor { config, view: _ }) => {
            use crate::firewall::config::load_firewall_config;

            let cfg = load_firewall_config(config.as_deref())
                .map_err(|e| InfynonError::System(e))?;

            let (state, _rt, _shutdown_tx) = bootstrap_firewall(cfg, config.clone())?;
            run_firewall_tui(state);
        }

        // ── Status ──────────────────────────────────────────────────────
        Some(FirewallCommands::Status { config }) => {
            use crate::firewall::config::load_firewall_config;
            Logger::title("INFYNON FIREWALL STATUS", "blue");
            match load_firewall_config(config.as_deref()) {
                Ok(cfg) => {
                    Logger::detail("  Config:", &crate::firewall::config::default_config_path().display().to_string());
                    Logger::detail("  Listen:", &format!("{}:{}", cfg.server.listen_address, cfg.server.listen_port));
                    Logger::detail("  Upstream:", &format!("{}:{}", cfg.upstream.address, cfg.upstream.port));
                    Logger::detail("  WAF:", if cfg.waf.enabled { "enabled" } else { "disabled" });
                    Logger::detail("  Rate Limit:", if cfg.rate_limit.enabled { "enabled" } else { "disabled" });
                    Logger::detail("  IP Mode:", &cfg.ip.mode.to_string());
                    Logger::detail("  Rules:", &format!("{} custom rules", cfg.rules.len()));
                    Logger::detail("  Maintenance:", if cfg.server.maintenance_mode { "ON" } else { "OFF" });
                    Logger::detail("  SQLi:", if cfg.waf.sqli_protection { "ON" } else { "OFF" });
                    Logger::detail("  XSS:", if cfg.waf.xss_protection { "ON" } else { "OFF" });
                    Logger::detail("  Path Traversal:", if cfg.waf.path_traversal_protection { "ON" } else { "OFF" });
                }
                Err(e) => Logger::error(&format!("Config error: {}", e)),
            }
        }

        // ── Block/Unblock IP ────────────────────────────────────────────
        Some(FirewallCommands::BlockIp { ip }) => {
            Logger::title("INFYNON FIREWALL", "red");
            // Append to blocklist file
            let path = std::path::Path::new("blocklists/ip-blocklist.txt");
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            match std::fs::OpenOptions::new().create(true).append(true).open(path) {
                Ok(mut f) => {
                    use std::io::Write;
                    let _ = writeln!(f, "{}", ip);
                    Logger::success(&format!("Blocked IP: {}", ip));
                    Logger::info("IP added to blocklist file. Active on next config reload or restart.");
                }
                Err(e) => Logger::error(&format!("Failed to write blocklist: {}", e)),
            }
        }
        Some(FirewallCommands::UnblockIp { ip }) => {
            Logger::title("INFYNON FIREWALL", "blue");
            let path = "blocklists/ip-blocklist.txt";
            match std::fs::read_to_string(path) {
                Ok(content) => {
                    let filtered: String = content.lines()
                        .filter(|l| l.trim() != ip)
                        .collect::<Vec<_>>()
                        .join("\n");
                    let _ = std::fs::write(path, filtered + "\n");
                    Logger::success(&format!("Unblocked IP: {}", ip));
                }
                Err(_) => Logger::error("No blocklist file found."),
            }
        }

        // ── Rules management ────────────────────────────────────────────
        Some(FirewallCommands::Rules { action }) => {
            use crate::firewall::config::load_firewall_config;
            Logger::title("INFYNON FIREWALL RULES", "blue");
            match action {
                RulesAction::List => {
                    match load_firewall_config(None) {
                        Ok(cfg) => {
                            if cfg.rules.is_empty() {
                                Logger::info("No custom rules defined. Add [[rules]] sections to infynon.toml.");
                                Logger::info("Built-in WAF rules are always active when WAF is enabled.");
                            } else {
                                println!("  {:<5} {:<30} {:<8} {}", "PRI", "NAME", "STATUS", "DESCRIPTION");
                                println!("  {}", "-".repeat(75));
                                for rule in &cfg.rules {
                                    let status = if rule.enabled { "ON" } else { "OFF" };
                                    println!(
                                        "  {:<5} {:<30} {:<8} {}",
                                        rule.priority,
                                        rule.name,
                                        status,
                                        rule.description,
                                    );
                                }
                            }
                        }
                        Err(e) => Logger::error(&format!("Config error: {}", e)),
                    }
                }
                RulesAction::Enable { ref name } => {
                    toggle_rule(name, true);
                }
                RulesAction::Disable { ref name } => {
                    toggle_rule(name, false);
                }
            }
        }

        // ── Logs ────────────────────────────────────────────────────────
        Some(FirewallCommands::Logs { follow: _, verdict, ip, since: _, count }) => {
            Logger::title("INFYNON FIREWALL LOGS", "blue");
            let log_path = "logs/access.jsonl";
            match std::fs::read_to_string(log_path) {
                Ok(content) => {
                    let lines: Vec<&str> = content.lines().collect();
                    let start = if lines.len() > count { lines.len() - count } else { 0 };
                    let mut shown = 0;
                    for line in &lines[start..] {
                        if let Ok(event) = serde_json::from_str::<crate::firewall::events::FirewallEvent>(line) {
                            // Apply filters
                            if let Some(ref v) = verdict {
                                if !event.verdict.to_string().eq_ignore_ascii_case(v) { continue; }
                            }
                            if let Some(ref filter_ip) = ip {
                                if &event.source_ip != filter_ip { continue; }
                            }
                            println!(
                                "  {} {:<16} {:<6} {:<30} {:<12} {}",
                                event.timestamp.format("%H:%M:%S"),
                                event.source_ip,
                                event.method,
                                truncate_str(&event.path, 30),
                                event.verdict,
                                event.blocked_reason.as_deref().unwrap_or("-"),
                            );
                            shown += 1;
                        }
                    }
                    if shown == 0 {
                        Logger::info("No matching log entries found.");
                    }
                }
                Err(_) => Logger::info("No log file found. Start the firewall first."),
            }
        }

        // ── Config ──────────────────────────────────────────────────────
        Some(FirewallCommands::ConfigCmd { action }) => {
            use crate::firewall::config::load_firewall_config;
            Logger::title("INFYNON FIREWALL CONFIG", "blue");
            match action {
                Some(ConfigAction::Check) | None => {
                    match load_firewall_config(None) {
                        Ok(cfg) => {
                            Logger::success("Configuration is valid.");
                            Logger::detail("  Server:", &format!("{}:{}", cfg.server.listen_address, cfg.server.listen_port));
                            Logger::detail("  Upstream:", &format!("{}:{}", cfg.upstream.address, cfg.upstream.port));
                            Logger::detail("  Rules:", &format!("{} custom rules", cfg.rules.len()));
                        }
                        Err(e) => Logger::error(&format!("Configuration error: {}", e)),
                    }
                }
                Some(ConfigAction::Show) => {
                    match load_firewall_config(None) {
                        Ok(cfg) => {
                            match toml::to_string_pretty(&cfg) {
                                Ok(s) => println!("{}", s),
                                Err(e) => Logger::error(&format!("Serialization error: {}", e)),
                            }
                        }
                        Err(e) => Logger::error(&format!("Config error: {}", e)),
                    }
                }
            }
        }

        // ── Legacy commands (not yet implemented) ───────────────────────
        Some(FirewallCommands::Daemon) => {
            Logger::title("INFYNON FIREWALL ENGINE", "red");
            Logger::error("Background daemon not yet implemented.");
        }
        Some(FirewallCommands::UpdateIntel) => {
            Logger::title("INFYNON FIREWALL ENGINE", "red");
            Logger::error("Intelligence update pipeline not yet implemented.");
        }
    }
    Ok(())
}

/// Build the shared firewall state, tokio runtime, and spawn proxy + cleanup tasks.
/// Returns (state, runtime, shutdown_sender) for the caller to use.
fn bootstrap_firewall(
    cfg: crate::firewall::config::FirewallConfig,
    config_path: Option<String>,
) -> Result<(
    std::sync::Arc<crate::firewall::server::SharedState>,
    tokio::runtime::Runtime,
    tokio::sync::watch::Sender<bool>,
), InfynonError> {
    use crate::firewall::server::SharedState;
    use crate::firewall::pipeline::Pipeline;
    use crate::firewall::stats::Stats;
    use crate::firewall::logger::EventLogger;
    use std::sync::Arc;

    let pipeline = Pipeline::new(&cfg);
    let stats = Stats::new();
    let logger = EventLogger::new(&cfg.logging.access_log, &cfg.logging.blocked_log);
    let maintenance = cfg.server.maintenance_mode;
    let max_events = cfg.tui.max_events_in_memory;

    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| InfynonError::System(format!("Failed to create runtime: {}", e)))?;

    let event_tx = rt.block_on(async { logger.spawn() });

    let state = Arc::new(SharedState {
        pipeline: std::sync::RwLock::new(pipeline),
        stats,
        config: std::sync::RwLock::new(cfg),
        event_tx,
        recent_events: std::sync::Mutex::new(std::collections::VecDeque::new()),
        max_recent: max_events,
        shutdown: shutdown_rx,
        config_path,
        maintenance_mode: std::sync::atomic::AtomicBool::new(maintenance),
        start_time: std::time::Instant::now(),
    });

    // Spawn proxy server
    let server_state = state.clone();
    rt.spawn(async move {
        if let Err(e) = crate::firewall::server::run_proxy(server_state).await {
            eprintln!("Proxy server error: {}", e);
        }
    });

    // Spawn periodic cleanup
    let cleanup_state = state.clone();
    rt.spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            if let Ok(pipeline) = cleanup_state.pipeline.read() {
                pipeline.cleanup();
            }
        }
    });

    // Spawn daily digest email scheduler
    let digest_state = state.clone();
    rt.spawn(async move {
        crate::firewall::mailer::daily_digest_loop(digest_state).await;
    });

    // Spawn periodic alert checker (every 30 seconds)
    let alert_state = state.clone();
    rt.spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            let snapshot = alert_state.stats.snapshot();
            crate::firewall::mailer::check_and_alert(&alert_state, &snapshot);
        }
    });

    Ok((state, rt, shutdown_tx))
}

/// Toggle a rule's enabled status in the config file
fn toggle_rule(name: &str, enabled: bool) {
    use crate::firewall::config::{load_firewall_config, save_firewall_config};
    match load_firewall_config(None) {
        Ok(mut cfg) => {
            let mut found = false;
            for rule in &mut cfg.rules {
                if rule.name == name {
                    rule.enabled = enabled;
                    found = true;
                    break;
                }
            }
            if !found {
                Logger::error(&format!("Rule '{}' not found in config.", name));
                return;
            }
            match save_firewall_config(&cfg, None) {
                Ok(()) => {
                    let action = if enabled { "Enabled" } else { "Disabled" };
                    Logger::success(&format!("{} rule '{}'", action, name));
                }
                Err(e) => Logger::error(&format!("Failed to save config: {}", e)),
            }
        }
        Err(e) => Logger::error(&format!("Config error: {}", e)),
    }
}

/// Config file watcher — polls for file changes and reloads config
async fn config_watch_loop(
    state: std::sync::Arc<crate::firewall::server::SharedState>,
    config_path: Option<String>,
) {
    let path = crate::firewall::config::config_path_for(config_path.as_deref());
    let mut last_modified = tokio::fs::metadata(&path).await
        .and_then(|m| m.modified())
        .ok();

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        let current_modified = tokio::fs::metadata(&path).await
            .and_then(|m| m.modified())
            .ok();

        if current_modified != last_modified {
            if let Ok(new_cfg) = crate::firewall::config::load_firewall_config(config_path.as_deref()) {
                let maint = new_cfg.server.maintenance_mode;
                if let Ok(mut cfg) = state.config.write() {
                    *cfg = new_cfg;
                }
                state.maintenance_mode.store(maint, std::sync::atomic::Ordering::Relaxed);
                // Rebuild pipeline so new rules/WAF/rate-limit settings take effect
                state.rebuild_pipeline();
            }
            last_modified = current_modified;
        }
    }
}

fn run_firewall_tui(state: std::sync::Arc<crate::firewall::server::SharedState>) {
    use crossterm::{
        event::{self, Event, KeyCode},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };
    use ratatui::{backend::CrosstermBackend, Terminal};
    use std::io;

    // Setup terminal
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

    let mut app = crate::tui::firewall_app::App::new(state);

    loop {
        // Render
        let _ = terminal.draw(|f| {
            crate::tui::views::render(f, &app);
        });

        // Handle input (with timeout for refresh)
        if event::poll(std::time::Duration::from_millis(100)).unwrap_or(false) {
            if let Ok(Event::Key(key)) = event::read() {
                // Ctrl+C = quit
                if key.code == KeyCode::Char('c')
                    && key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL)
                {
                    break;
                }
                app.handle_key(key);
            }
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    let _ = disable_raw_mode();
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
    let _ = terminal.show_cursor();
}

