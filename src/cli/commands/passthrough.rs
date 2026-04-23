use crate::cli::args::PkgArgs;
use crate::cli::scan::{self, check_packages_before_install};
use crate::ecosystems::detector;
use crate::error::types::InfynonError;
use crate::tui::logger::Logger;
use owo_colors::OwoColorize;
use std::collections::{HashMap, HashSet};
use std::path::Path;

const EXIT_INSTALL_CHECK_ERROR: i32 = 2;
const EXIT_STRICT_BLOCK: i32 = 3;
const EXIT_INPUT_REQUIRED: i32 = 4;

pub fn execute_pkg_passthrough(args: &PkgArgs) -> Result<(), InfynonError> {
    let (ecosystem, cmd_idx) = detect_passthrough_ecosystem(args);
    let install_action = args
        .passthrough_args
        .get(cmd_idx)
        .cloned()
        .filter(is_install_action)
        .unwrap_or_default();
    let install_packages = if install_action.is_empty() {
        Vec::new()
    } else {
        args.passthrough_args[cmd_idx + 1..].to_vec()
    };

    let binary = match ensure_installed(ecosystem) {
        Some(value) => value,
        None => return Ok(()),
    };
    if !args.machine_output() {
        Logger::subtitle("🛡️", "INFYNON Secure Proxy", "Active");
        Logger::detail("» Ecosystem:", ecosystem);
        Logger::success(&format!("'{}' binary found — proceeding", binary));
    }

    if install_packages.is_empty() {
        run_passthrough_command(ecosystem, &binary, &args.passthrough_args[cmd_idx..]);
        return Ok(());
    }
    run_install_flow(args, ecosystem, &binary, &install_action, &install_packages)
}

fn detect_passthrough_ecosystem(args: &PkgArgs) -> (&'static str, usize) {
    let first_arg = &args.passthrough_args[0];
    let known = [
        "npm", "yarn", "pnpm", "bun", "pip", "uv", "poetry", "cargo", "go", "gem", "composer",
        "nuget", "hex", "pub",
    ];
    if known.contains(&first_arg.as_str()) {
        return (Box::leak(first_arg.clone().into_boxed_str()), 1);
    }
    let exists = |file: &str| Path::new(file).exists();
    let ecosystem = if exists("package.json") && exists("bun.lockb") || exists("bun.lockb") {
        "bun"
    } else if exists("package.json") && exists("pnpm-lock.yaml") || exists("pnpm-lock.yaml") {
        "pnpm"
    } else if exists("package.json") && exists("yarn.lock") || exists("yarn.lock") {
        "yarn"
    } else if exists("package.json") || exists("package-lock.json") {
        "npm"
    } else if exists("Cargo.toml") {
        "cargo"
    } else if exists("pyproject.toml") && exists("uv.lock") || exists("uv.lock") {
        "uv"
    } else if exists("pyproject.toml") && exists("poetry.lock") || exists("poetry.lock") {
        "poetry"
    } else if exists("pyproject.toml")
        || exists("requirements.txt")
        || exists("setup.py")
        || exists("setup.cfg")
    {
        "pip"
    } else if exists("go.mod") {
        "go"
    } else if exists("composer.json") || exists("composer.lock") {
        "composer"
    } else if exists("Gemfile") || exists("Gemfile.lock") {
        "gem"
    } else if exists("pubspec.yaml") || exists("pubspec.lock") {
        "pub"
    } else if exists("mix.exs") || exists("mix.lock") {
        "hex"
    } else {
        "auto-detected"
    };
    (ecosystem, 0)
}

fn is_install_action(action: &String) -> bool {
    matches!(
        action.as_str(),
        "install" | "add" | "i" | "require" | "get" | "update" | "upgrade" | "up"
    )
}

fn ensure_installed(ecosystem: &str) -> Option<String> {
    let binary = match ecosystem {
        "poetry" => "poetry",
        "uv" => "uv",
        "hex" => "mix",
        "pub" => "dart",
        "nuget" => "dotnet",
        other => other,
    };
    if detector::is_installed(binary) {
        return Some(detector::resolve_binary(binary));
    }
    println!();
    println!(
        "  {} {}{}{}\n",
        "✘".red().bold(),
        "Package manager ".red().bold(),
        format!("'{}'", ecosystem).bright_red().bold(),
        " is not installed on this system.".red().bold()
    );
    if let Some(info) = detector::install_instructions(ecosystem) {
        println!("  {}  {}", "ℹ".bright_cyan().bold(), info.note.white());
        println!();
        println!(
            "  {} {}",
            "Install command:".bold().truecolor(255, 170, 50),
            info.install_cmd.bright_green()
        );
        println!(
            "  {} {}",
            "Official docs:  ".bold().truecolor(255, 170, 50),
            info.install_url.truecolor(100, 150, 255)
        );
    }
    println!();
    None
}

fn run_install_flow(
    args: &PkgArgs,
    ecosystem: &str,
    binary: &str,
    install_action: &str,
    install_packages: &[String],
) -> Result<(), InfynonError> {
    let machine_output = args.machine_output();
    let (safe, hits) =
        match check_packages_before_install(install_packages, ecosystem, machine_output) {
            Ok(value) => value,
            Err(err) => return handle_install_check_error(args, install_packages, err),
        };
    let packages = if safe {
        install_packages.to_vec()
    } else {
        resolve_install_packages(args, ecosystem, install_packages, &hits)?
    };
    if packages.is_empty() {
        if machine_output {
            emit_agent_result(machine_output, install_packages, &hits, false, "");
        }
        Logger::raw_dim("  Nothing to install.");
        return Ok(());
    }
    let mut cmd_parts = vec![install_action.to_string()];
    cmd_parts.extend(packages.iter().cloned());
    let (program, actual_args) = crate::cli::proxy_pkg_invocation(ecosystem, binary, &cmd_parts);
    let cmd = crate::cli::format_pkg_cmd(&program, &actual_args);
    if !machine_output {
        println!();
        Logger::step(&format!("Running: {}", cmd));
        println!();
    }
    let install_ok = crate::cli::proxy_pkg_cmd(ecosystem, binary, &cmd_parts)
        .map(|status| status.success())
        .unwrap_or(false);
    if !install_ok && !machine_output {
        Logger::error(&format!("Command failed: {}", cmd));
    }
    emit_agent_result(machine_output, install_packages, &hits, install_ok, &cmd);
    Ok(())
}

fn handle_install_check_error(
    args: &PkgArgs,
    install_packages: &[String],
    err: String,
) -> Result<(), InfynonError> {
    if args.machine_output() {
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({"schema_version":"infynon.pkg.install.v1","status":"error","error":err,"packages_checked":install_packages,"installed":false})).unwrap());
        std::process::exit(EXIT_INSTALL_CHECK_ERROR);
    }
    Logger::error(&format!("Security gate blocked install: {}", err));
    std::process::exit(EXIT_INSTALL_CHECK_ERROR);
}

fn resolve_install_packages(
    args: &PkgArgs,
    ecosystem: &str,
    install_packages: &[String],
    hits: &[scan::VulnHit],
) -> Result<Vec<String>, InfynonError> {
    if let Some(strict) = &args.strict {
        let level = scan::FixLevel::from_str(strict);
        if hits.iter().any(|hit| level.matches(hit.severity)) {
            handle_strict_block(args.machine_output(), install_packages, hits, strict);
        }
    }
    if args.yes {
        return Ok(install_packages.to_vec());
    }
    if args.skip_vulnerable {
        return Ok(skip_vulnerable_packages(
            install_packages,
            hits,
            args.machine_output(),
        ));
    }
    if args.auto_fix {
        return Ok(auto_fix_packages(
            ecosystem,
            install_packages,
            hits,
            args.machine_output(),
        ));
    }
    if args.non_interactive() {
        handle_input_required(args.machine_output(), install_packages, hits);
    }
    Ok(super::ask_vuln_decisions(install_packages, hits, ecosystem))
}

fn handle_strict_block(
    agent: bool,
    install_packages: &[String],
    hits: &[scan::VulnHit],
    strict: &str,
) {
    if agent {
        let vulns: Vec<_> = hits.iter().map(hit_to_json).collect();
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({"schema_version":"infynon.pkg.install.v1","status":"blocked","packages_checked":install_packages,"vulnerabilities":vulns,"installed":false,"blocked_by":format!("--strict {}", strict)})).unwrap());
        std::process::exit(EXIT_STRICT_BLOCK);
    }
    let label = if strict == "all" {
        "all severities".to_string()
    } else {
        format!("{}+", strict)
    };
    println!(
        "\n  {}  {} — {}  (blocking: {})\n",
        "╳".bright_red().bold(),
        "BLOCKED".bold().bright_red(),
        "--strict mode active".truecolor(200, 80, 80),
        label.truecolor(200, 120, 80)
    );
    std::process::exit(EXIT_STRICT_BLOCK);
}

fn handle_input_required(agent: bool, install_packages: &[String], hits: &[scan::VulnHit]) {
    if agent {
        let vulns: Vec<_> = hits.iter().map(hit_to_json).collect();
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({"schema_version":"infynon.pkg.install.v1","status":"input_required","error":"Vulnerable packages require an explicit non-interactive decision. Use --yes, --skip-vulnerable, --auto-fix, or --strict.","packages_checked":install_packages,"vulnerabilities":vulns,"installed":false})).unwrap());
        std::process::exit(EXIT_INPUT_REQUIRED);
    }
    Logger::error("Interactive review is disabled, but vulnerable packages require a decision.");
    Logger::info(
        "Use --yes, --skip-vulnerable, --auto-fix, or --strict to make the install deterministic.",
    );
    std::process::exit(EXIT_INPUT_REQUIRED);
}

fn skip_vulnerable_packages(
    install_packages: &[String],
    hits: &[scan::VulnHit],
    machine_output: bool,
) -> Vec<String> {
    let vuln_names: HashSet<String> = hits.iter().map(|hit| hit.package.clone()).collect();
    if machine_output {
        return install_packages
            .iter()
            .filter(|spec| !vuln_names.contains(&scan::parse_pkg_spec(spec).0))
            .cloned()
            .collect();
    }
    for name in &vuln_names {
        println!(
            "  {}  Skipping vulnerable: {}",
            "✘".bright_red(),
            name.bold()
        );
    }
    install_packages
        .iter()
        .filter(|spec| !vuln_names.contains(&scan::parse_pkg_spec(spec).0))
        .cloned()
        .collect()
}

fn auto_fix_packages(
    ecosystem: &str,
    install_packages: &[String],
    hits: &[scan::VulnHit],
    machine_output: bool,
) -> Vec<String> {
    let mut fixes: HashMap<String, Option<String>> = HashMap::new();
    let vuln_names: HashSet<String> = hits.iter().map(|hit| hit.package.clone()).collect();
    for hit in hits {
        if hit.fixed_version.is_some() {
            fixes.insert(hit.package.clone(), hit.fixed_version.clone());
        }
    }
    if machine_output {
        return install_packages
            .iter()
            .filter_map(|spec| {
                let (name, _) = scan::parse_pkg_spec(spec);
                if !vuln_names.contains(&name) {
                    return Some(spec.clone());
                }
                fixes
                    .get(&name)
                    .and_then(|value| value.clone())
                    .map(|ver| super::format_spec_for_ecosystem(&name, &ver, ecosystem))
            })
            .collect();
    }
    install_packages
        .iter()
        .filter_map(|spec| {
            let (name, _) = scan::parse_pkg_spec(spec);
            if !vuln_names.contains(&name) {
                return Some(spec.clone());
            }
            fixes
                .get(&name)
                .and_then(|value| value.clone())
                .map(|ver| {
                    let new_spec = super::format_spec_for_ecosystem(&name, &ver, ecosystem);
                    println!(
                        "  {}  Auto-fix: {} → {}",
                        "✔".bright_green(),
                        name.bold(),
                        new_spec.bright_green().bold()
                    );
                    new_spec
                })
                .or_else(|| {
                    println!(
                        "  {}  No fix available for {} — skipping",
                        "✘".bright_red(),
                        name.bold()
                    );
                    None
                })
        })
        .collect()
}

fn run_passthrough_command(ecosystem: &str, binary: &str, args: &[String]) {
    let (program, actual_args) = crate::cli::proxy_pkg_invocation(ecosystem, binary, args);
    let cmd = crate::cli::format_pkg_cmd(&program, &actual_args);
    println!();
    Logger::step(&format!("Running: {}", cmd));
    println!();
    match crate::cli::proxy_pkg_cmd(ecosystem, binary, args) {
        Ok(status) if status.success() => {}
        Ok(status) => Logger::error(&format!(
            "Command failed with exit code {}: {}",
            status
                .code()
                .map(|c| c.to_string())
                .unwrap_or_else(|| "unknown".to_string()),
            cmd
        )),
        Err(err) => Logger::error(&format!("Failed to execute '{}': {}", binary, err)),
    }
}

fn emit_agent_result(
    agent: bool,
    install_packages: &[String],
    hits: &[scan::VulnHit],
    installed: bool,
    cmd: &str,
) {
    if !agent {
        return;
    }
    let has_medium_plus = hits
        .iter()
        .any(|hit| matches!(hit.severity, "CRITICAL" | "HIGH" | "MEDIUM"));
    let status = if hits.is_empty() {
        "clean"
    } else if has_medium_plus {
        "vulnerable"
    } else {
        "warnings"
    };
    let exit_code = if hits.is_empty() {
        0
    } else if has_medium_plus {
        2
    } else {
        1
    };
    println!("{}", serde_json::to_string_pretty(&serde_json::json!({"schema_version":"infynon.pkg.install.v1","status":status,"packages_checked":install_packages,"vulnerabilities":hits.iter().map(hit_to_json).collect::<Vec<_>>(),"installed":installed,"install_cmd":cmd})).unwrap());
    std::process::exit(exit_code);
}

fn hit_to_json(hit: &scan::VulnHit) -> serde_json::Value {
    serde_json::json!({"package":hit.package,"current_version":"","cve_id":hit.cve_id,"severity":hit.severity,"summary":hit.summary,"safe_version":hit.fixed_version,"fix_cmd":hit.upgrade_cmd})
}
