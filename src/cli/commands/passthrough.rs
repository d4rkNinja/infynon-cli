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
const EXIT_COMMAND_ERROR: i32 = 1;

pub fn execute_pkg_passthrough(args: &PkgArgs) -> Result<(), InfynonError> {
    let (ecosystem, cmd_idx) = detect_passthrough_ecosystem(args);
    let forwarded_args = &args.passthrough_args[cmd_idx..];
    let install_request = detect_install_request(&ecosystem, forwarded_args);

    let binary = ensure_installed(&ecosystem)?;
    if !args.machine_output() {
        Logger::subtitle("🛡️", "INFYNON Secure Proxy", "Active");
        Logger::detail("» Ecosystem:", &ecosystem);
        Logger::success(&format!("'{}' binary found — proceeding", binary));
    }

    let Some(install_request) = install_request else {
        return run_passthrough_command(&ecosystem, &binary, forwarded_args);
    };
    let install_packages = install_request.install_packages(forwarded_args);
    if install_packages.is_empty() {
        return run_passthrough_command(&ecosystem, &binary, forwarded_args);
    }
    run_install_flow(
        args,
        &ecosystem,
        &binary,
        forwarded_args,
        &install_request,
        &install_packages,
    )
}

fn detect_passthrough_ecosystem(args: &PkgArgs) -> (String, usize) {
    let first_arg = &args.passthrough_args[0];
    let known = [
        "npm", "yarn", "pnpm", "bun", "pip", "uv", "poetry", "cargo", "go", "gem", "composer",
        "nuget", "hex", "pub",
    ];
    if known.contains(&first_arg.as_str()) {
        return (first_arg.clone(), 1);
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
    (ecosystem.to_string(), 0)
}

fn is_install_action(action: &str) -> bool {
    matches!(
        action,
        "install" | "add" | "i" | "require" | "get" | "update" | "upgrade" | "up"
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct InstallRequest {
    action_index: usize,
    package_indices: Vec<usize>,
}

impl InstallRequest {
    fn install_packages(&self, forwarded_args: &[String]) -> Vec<String> {
        self.package_indices
            .iter()
            .filter_map(|index| forwarded_args.get(*index).cloned())
            .collect()
    }

    fn rewrite_forwarded_args(
        &self,
        forwarded_args: &[String],
        selected_packages: &[String],
    ) -> Vec<String> {
        if self.install_packages(forwarded_args) == selected_packages {
            return forwarded_args.to_vec();
        }

        let package_indices: HashSet<usize> = self.package_indices.iter().copied().collect();
        let first_package_index = self.package_indices.first().copied();
        let mut rewritten = Vec::with_capacity(forwarded_args.len() + selected_packages.len());
        for (index, arg) in forwarded_args.iter().enumerate() {
            if Some(index) == first_package_index {
                rewritten.extend(selected_packages.iter().cloned());
            }
            if package_indices.contains(&index) {
                continue;
            }
            rewritten.push(arg.clone());
        }
        rewritten
    }
}

fn detect_install_request(ecosystem: &str, forwarded_args: &[String]) -> Option<InstallRequest> {
    let action_index = find_install_action_index(ecosystem, forwarded_args)?;
    let action = forwarded_args.get(action_index)?;
    let package_indices =
        collect_install_package_indices(ecosystem, action, forwarded_args, action_index);
    if package_indices.is_empty() {
        None
    } else {
        Some(InstallRequest {
            action_index,
            package_indices,
        })
    }
}

fn find_install_action_index(ecosystem: &str, forwarded_args: &[String]) -> Option<usize> {
    let mut index = 0;
    while index < forwarded_args.len() {
        let arg = forwarded_args[index].as_str();
        if arg == "--" {
            return None;
        }
        if is_install_action(arg) {
            return Some(index);
        }
        if is_option_token(arg) {
            index += option_token_width(ecosystem, arg);
            continue;
        }
        return None;
    }
    None
}

fn collect_install_package_indices(
    ecosystem: &str,
    action: &str,
    forwarded_args: &[String],
    action_index: usize,
) -> Vec<usize> {
    let mut indices = Vec::new();
    let mut index = action_index + 1;
    let mut first_operand = true;
    while index < forwarded_args.len() {
        let arg = forwarded_args[index].as_str();
        if arg == "--" {
            break;
        }
        if is_option_token(arg) {
            index += option_token_width(ecosystem, arg);
            continue;
        }
        if first_operand && is_command_noun_operand(ecosystem, action, arg) {
            first_operand = false;
            index += 1;
            continue;
        }
        first_operand = false;
        if is_checkable_install_spec(arg) {
            indices.push(index);
        }
        index += 1;
    }
    indices
}

fn is_option_token(arg: &str) -> bool {
    arg.starts_with('-') && arg != "-"
}

fn option_token_width(ecosystem: &str, option: &str) -> usize {
    if option_takes_value(ecosystem, option) && !option_has_inline_value(option) {
        2
    } else {
        1
    }
}

fn option_has_inline_value(option: &str) -> bool {
    option.contains('=')
        || (option.starts_with('-') && !option.starts_with("--") && option.len() > 2)
}

fn option_takes_value(ecosystem: &str, option: &str) -> bool {
    let key = option.split_once('=').map(|(key, _)| key).unwrap_or(option);
    if matches!(
        key,
        "--prefix"
            | "--cwd"
            | "--dir"
            | "--directory"
            | "--workspace"
            | "--filter"
            | "--registry"
            | "--cache"
            | "--tag"
            | "--scope"
            | "--group"
            | "--source"
            | "--index-url"
            | "--extra-index-url"
            | "--find-links"
            | "--trusted-host"
            | "--target"
            | "--root"
            | "--src"
            | "--upgrade-strategy"
            | "--config-settings"
            | "--platform"
            | "--python-version"
            | "--implementation"
            | "--abi"
            | "--requirement"
            | "--constraint"
            | "--editable"
            | "--git"
            | "--path"
            | "--package"
            | "--features"
            | "--vers"
            | "-C"
            | "-r"
            | "-c"
            | "-e"
            | "-t"
            | "-f"
            | "-i"
            | "-w"
            | "-p"
            | "-F"
    ) {
        return true;
    }

    matches!(
        (ecosystem, key),
        ("cargo", "--bin")
            | ("cargo", "--example")
            | ("cargo", "--target")
            | ("cargo", "--version")
            | ("go", "-C")
            | ("gem", "--version")
            | ("gem", "-v")
            | ("nuget", "--version")
    )
}

fn is_command_noun_operand(ecosystem: &str, action: &str, arg: &str) -> bool {
    matches!(
        (ecosystem, action, arg),
        ("nuget", "add", "package") | ("nuget", "update", "package")
    )
}

fn is_checkable_install_spec(spec: &str) -> bool {
    let spec = spec.trim();
    if spec.is_empty()
        || matches!(spec, "." | "..")
        || spec.starts_with("./")
        || spec.starts_with("../")
        || spec.starts_with("~/")
        || spec.starts_with("file:")
        || spec.starts_with("git:")
        || spec.starts_with("git+")
        || spec.starts_with("http:")
        || spec.starts_with("https:")
        || spec.starts_with("ssh:")
        || spec.contains("://")
        || Path::new(spec).is_absolute()
    {
        return false;
    }
    true
}

fn ensure_installed(ecosystem: &str) -> Result<String, InfynonError> {
    if ecosystem == "auto-detected" {
        return Err(InfynonError::System(
            "Could not auto-detect a package manager for this directory. Run the command with an explicit manager, for example `infynon pkg npm install <package>`.".to_string(),
        ));
    }
    let binary = match ecosystem {
        "poetry" => "poetry",
        "uv" => "uv",
        "hex" => "mix",
        "pub" => "dart",
        "nuget" => "dotnet",
        other => other,
    };
    if detector::is_installed(binary) {
        return Ok(detector::resolve_binary(binary));
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
    Err(InfynonError::System(format!(
        "Package manager '{}' is not installed on this system.",
        ecosystem
    )))
}

fn run_install_flow(
    args: &PkgArgs,
    ecosystem: &str,
    binary: &str,
    forwarded_args: &[String],
    install_request: &InstallRequest,
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
            emit_agent_result(machine_output, install_packages, &hits, false, "", None);
        }
        Logger::raw_dim("  Nothing to install.");
        return Ok(());
    }
    let cmd_parts = install_request.rewrite_forwarded_args(forwarded_args, &packages);
    let (program, actual_args) = crate::cli::proxy_pkg_invocation(ecosystem, binary, &cmd_parts);
    let cmd = crate::cli::format_pkg_cmd(&program, &actual_args);
    if !machine_output {
        println!();
        Logger::step(&format!("Running: {}", cmd));
        println!();
    }
    let status = crate::cli::proxy_pkg_cmd(ecosystem, binary, &cmd_parts)
        .map_err(|err| InfynonError::System(format!("Failed to execute '{}': {}", binary, err)))?;
    let install_ok = status.success();
    let child_exit_code = exit_code_from_status(status);
    if !install_ok {
        if !machine_output {
            Logger::error(&format!("Command failed: {}", cmd));
        }
        emit_agent_result(
            machine_output,
            install_packages,
            &hits,
            install_ok,
            &cmd,
            Some(child_exit_code),
        );
        std::process::exit(child_exit_code);
    }
    emit_agent_result(
        machine_output,
        install_packages,
        &hits,
        install_ok,
        &cmd,
        None,
    );
    Ok(())
}

fn handle_install_check_error(
    args: &PkgArgs,
    install_packages: &[String],
    err: String,
) -> Result<(), InfynonError> {
    if args.machine_output() {
        crate::utils::print_json_pretty(
            &serde_json::json!({"schema_version":"infynon.pkg.install.v1","status":"error","error":err,"packages_checked":install_packages,"installed":false}),
        );
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
        crate::utils::print_json_pretty(
            &serde_json::json!({"schema_version":"infynon.pkg.install.v1","status":"blocked","packages_checked":install_packages,"vulnerabilities":vulns,"installed":false,"blocked_by":format!("--strict {}", strict)}),
        );
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
        crate::utils::print_json_pretty(
            &serde_json::json!({"schema_version":"infynon.pkg.install.v1","status":"input_required","error":"Vulnerable packages require an explicit non-interactive decision. Use --yes, --skip-vulnerable, --auto-fix, or --strict.","packages_checked":install_packages,"vulnerabilities":vulns,"installed":false}),
        );
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

fn run_passthrough_command(
    ecosystem: &str,
    binary: &str,
    args: &[String],
) -> Result<(), InfynonError> {
    let (program, actual_args) = crate::cli::proxy_pkg_invocation(ecosystem, binary, args);
    let cmd = crate::cli::format_pkg_cmd(&program, &actual_args);
    println!();
    Logger::step(&format!("Running: {}", cmd));
    println!();
    match crate::cli::proxy_pkg_cmd(ecosystem, binary, args) {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => {
            let exit_code = exit_code_from_status(status);
            Logger::error(&format!(
                "Command failed with exit code {}: {}",
                exit_code, cmd
            ));
            std::process::exit(exit_code);
        }
        Err(err) => Err(InfynonError::System(format!(
            "Failed to execute '{}': {}",
            binary, err
        ))),
    }
}

fn emit_agent_result(
    agent: bool,
    install_packages: &[String],
    hits: &[scan::VulnHit],
    installed: bool,
    cmd: &str,
    command_exit_code: Option<i32>,
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
    let exit_code = if let Some(code) = command_exit_code {
        code
    } else if hits.is_empty() {
        0
    } else if has_medium_plus {
        2
    } else {
        1
    };
    crate::utils::print_json_pretty(
        &serde_json::json!({"schema_version":"infynon.pkg.install.v1","status":status,"packages_checked":install_packages,"vulnerabilities":hits.iter().map(hit_to_json).collect::<Vec<_>>(),"installed":installed,"install_cmd":cmd}),
    );
    std::process::exit(exit_code);
}

fn exit_code_from_status(status: std::process::ExitStatus) -> i32 {
    status.code().unwrap_or(EXIT_COMMAND_ERROR)
}

fn hit_to_json(hit: &scan::VulnHit) -> serde_json::Value {
    serde_json::json!({"package":hit.package,"current_version":"","cve_id":hit.cve_id,"severity":hit.severity,"summary":hit.summary,"safe_version":hit.fixed_version,"fix_cmd":hit.upgrade_cmd})
}

#[cfg(test)]
mod tests {
    use super::{detect_install_request, InstallRequest};

    fn strings(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn detects_install_after_leading_package_manager_options() {
        let args = strings(&["--prefix", "app", "install", "lodash"]);
        let request = detect_install_request("npm", &args).expect("install request");

        assert_eq!(request.action_index, 2);
        assert_eq!(request.install_packages(&args), strings(&["lodash"]));
    }

    #[test]
    fn skips_flags_and_option_values_when_collecting_install_packages() {
        let args = strings(&[
            "install",
            "--save-dev",
            "lodash",
            "--workspace",
            "web",
            "-w",
            "admin",
            "@types/node@20.0.0",
        ]);
        let request = detect_install_request("npm", &args).expect("install request");

        assert_eq!(
            request.install_packages(&args),
            strings(&["lodash", "@types/node@20.0.0"])
        );
    }

    #[test]
    fn does_not_gate_non_install_commands_that_mention_install_later() {
        let args = strings(&["run", "install", "lodash"]);

        assert!(detect_install_request("npm", &args).is_none());
    }

    #[test]
    fn ignores_requirement_files_and_local_path_operands() {
        let pip_args = strings(&["install", "-r", "requirements.txt"]);
        assert!(detect_install_request("pip", &pip_args).is_none());

        let npm_args = strings(&["install", ".", "./local-pkg", "file:../pkg", "react"]);
        let request = detect_install_request("npm", &npm_args).expect("install request");
        assert_eq!(request.install_packages(&npm_args), strings(&["react"]));
    }

    #[test]
    fn rewrite_preserves_original_forwarded_args_when_packages_are_unchanged() {
        let args = strings(&["install", "--save-dev", "lodash", "--workspace", "web"]);
        let request = InstallRequest {
            action_index: 0,
            package_indices: vec![2],
        };

        assert_eq!(
            request.rewrite_forwarded_args(&args, &strings(&["lodash"])),
            args
        );
    }

    #[test]
    fn rewrite_replaces_only_checked_package_positions() {
        let args = strings(&[
            "install",
            "--save-dev",
            "lodash",
            "--workspace",
            "web",
            "react",
        ]);
        let request = InstallRequest {
            action_index: 0,
            package_indices: vec![2, 5],
        };

        assert_eq!(
            request.rewrite_forwarded_args(&args, &strings(&["lodash"])),
            strings(&["install", "--save-dev", "lodash", "--workspace", "web"])
        );
    }
}
