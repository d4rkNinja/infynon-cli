use crate::cli::args::{EagleEyeAction, PkgArgs, PkgCommands, RootArgs, RootCommands};
use crate::cli::features;
use crate::cli::scan::{run_scan, FixLevel, OutputFormat};
use crate::error::types::InfynonError;
use crate::tui::logger::Logger;
use clap::Parser;

pub fn execute_pkg_mode() -> Result<(), InfynonError> {
    let mut args;
    let raw_args: Vec<String> = std::env::args().collect();
    args = if raw_args.len() > 1 && raw_args[1] == "pkg" {
        let filtered: Vec<String> = std::iter::once(raw_args[0].clone())
            .chain(raw_args[2..].iter().cloned())
            .collect();
        PkgArgs::parse_from(filtered)
    } else {
        PkgArgs::parse()
    };

    if let Err(message) = crate::cli::validate::validate_pkg_args(&args) {
        Logger::error(&message);
        std::process::exit(2);
    }

    if let Some(command) = args.command.take() {
        return route_pkg_command(args, command);
    }
    if args.passthrough_args.is_empty() {
        Logger::splash_pkg();
        return Ok(());
    }
    super::execute_pkg_passthrough(&args)
}

pub fn execute_root_mode() -> Result<(), InfynonError> {
    let args = RootArgs::parse();
    match args.command {
        None => Logger::splash_root(),
        Some(RootCommands::Pkg { .. }) => return execute_pkg_mode(),
        Some(RootCommands::Weave { action }) => super::execute_api_command(action),
        Some(RootCommands::Trace { action }) => {
            std::process::exit(crate::trace::commands::execute(action))
        }
    }
    Ok(())
}

fn route_pkg_command(args: PkgArgs, command: PkgCommands) -> Result<(), InfynonError> {
    match command {
        PkgCommands::Scan {
            output,
            fix,
            pkg_file,
        } => {
            let machine_output = args.machine_output();
            let has_fix = fix.is_some();
            let format = output
                .as_deref()
                .map(|value| match value.to_ascii_lowercase().as_str() {
                    "pdf" => OutputFormat::Pdf,
                    "both" => OutputFormat::Both,
                    _ => OutputFormat::Markdown,
                });
            let code = run_scan(
                format,
                fix.map(|value| FixLevel::from_str(&value)),
                pkg_file.or(args.pkg_file).as_deref(),
                machine_output,
            );
            if code != 0 && (machine_output || has_fix) {
                std::process::exit(code);
            }
        }
        PkgCommands::Audit { pkg_file } => {
            features::cmd_audit_deep(pkg_file.or(args.pkg_file).as_deref())
        }
        PkgCommands::Why { package, pkg_file } => {
            features::cmd_why(&package, pkg_file.or(args.pkg_file).as_deref())
        }
        PkgCommands::Explain {
            package,
            ecosystem,
            pkg_file,
        } => features::cmd_explain(
            &package,
            ecosystem.as_deref(),
            pkg_file.or(args.pkg_file).as_deref(),
        ),
        PkgCommands::Outdated { pkg_file } => {
            features::cmd_outdated(pkg_file.or(args.pkg_file).as_deref())
        }
        PkgCommands::Diff {
            package,
            v1,
            v2,
            ecosystem,
        } => features::cmd_diff(&package, &v1, &v2, ecosystem.as_deref()),
        PkgCommands::Doctor { pkg_file } => {
            features::cmd_doctor(pkg_file.or(args.pkg_file).as_deref())
        }
        PkgCommands::Size {
            packages,
            ecosystem,
        } => {
            if packages.is_empty() {
                Logger::error("Please specify at least one package name.");
            } else {
                features::cmd_size(&packages, ecosystem.as_deref());
            }
        }
        PkgCommands::Search { query, ecosystem } => {
            features::cmd_search(&query, ecosystem.as_deref())
        }
        PkgCommands::Fix { auto: _, pkg_file } => {
            let code = features::cmd_fix_auto(pkg_file.or(args.pkg_file).as_deref());
            if code != 0 {
                std::process::exit(code);
            }
        }
        PkgCommands::Clean { pkg_file } => {
            features::cmd_clean(pkg_file.or(args.pkg_file).as_deref())
        }
        PkgCommands::Migrate { from, to } => features::cmd_migrate(&from, &to),
        PkgCommands::EagleEye { action } => match action {
            EagleEyeAction::Setup => features::eagle_eye::cmd_setup(),
            EagleEyeAction::Start => features::eagle_eye::cmd_start(),
            EagleEyeAction::Status => features::eagle_eye::cmd_status(),
            EagleEyeAction::Enable => features::eagle_eye::cmd_enable(),
            EagleEyeAction::Disable => features::eagle_eye::cmd_disable(),
        },
    }
    Ok(())
}
