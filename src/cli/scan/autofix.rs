use crate::engine::{osv, reporter, scanner};
use crate::tui::logger::Logger;
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct AutoFixSummary {
    pub success_count: usize,
    pub fail_count: usize,
}

pub(super) fn run_auto_fix(findings: &[reporter::ScanFinding]) -> AutoFixSummary {
    type Key = (String, String, String);
    let mut pkg_map: HashMap<Key, (&scanner::LockedPackage, Vec<String>, Vec<String>)> =
        HashMap::new();
    for finding in findings {
        let key = (
            finding.package.name.clone(),
            finding.package.ecosystem.clone(),
            finding.package.source.clone(),
        );
        let entry = pkg_map
            .entry(key)
            .or_insert_with(|| (&finding.package, Vec::new(), Vec::new()));
        if let Some(ref fixed) = finding.fixed_version {
            entry.1.push(fixed.clone());
        } else if let Some(ref suggested) = finding.suggested_version {
            entry.2.push(suggested.clone());
        }
    }
    let items: Vec<(String, crate::cli::PkgInvocation)> = pkg_map
        .values()
        .filter_map(|(pkg, confirmed, suggested)| {
            let best = if !confirmed.is_empty() {
                osv::max_version(confirmed)
            } else {
                osv::max_version(suggested)
            }?;
            Some((
                format!("{} {} → {}", pkg.name, pkg.version, best),
                upgrade_invocation(pkg, &best),
            ))
        })
        .collect();
    if items.is_empty() {
        Logger::info("No packages have a known fixed or suggested version available.");
        return AutoFixSummary::default();
    }
    println!(
        "\n  {} {}\n",
        "⚡ Auto-Fix".bold().truecolor(255, 200, 50),
        format!("Upgrading {} package(s)...", items.len()).truecolor(160, 160, 180)
    );
    let (mut success_count, mut fail_count) = (0usize, 0usize);
    for (label, invocation) in items {
        run_fix_command(&label, &invocation, &mut success_count, &mut fail_count);
    }
    println!(
        "\n  Auto-fix complete  {}  {}\n",
        format!("{} succeeded", success_count).bold().bright_green(),
        if fail_count > 0 {
            format!("{} failed", fail_count)
                .bold()
                .bright_red()
                .to_string()
        } else {
            "0 failed".truecolor(100, 100, 120).to_string()
        }
    );
    AutoFixSummary {
        success_count,
        fail_count,
    }
}

pub fn upgrade_cmd(pkg: &scanner::LockedPackage, fixed: &str) -> String {
    upgrade_invocation(pkg, fixed).display()
}

fn upgrade_invocation(pkg: &scanner::LockedPackage, fixed: &str) -> crate::cli::PkgInvocation {
    let source = std::path::Path::new(&pkg.source)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(&pkg.source);
    match pkg.ecosystem.as_str() {
        "npm" => match source {
            "yarn.lock" => invocation("yarn", &["add", &format!("{}@{}", pkg.name, fixed)]),
            "pnpm-lock.yaml" => invocation("pnpm", &["add", &format!("{}@{}", pkg.name, fixed)]),
            "bun.lockb" | "bun.lock" => {
                invocation("bun", &["add", &format!("{}@{}", pkg.name, fixed)])
            }
            _ if std::path::Path::new("bun.lockb").exists() => {
                invocation("bun", &["add", &format!("{}@{}", pkg.name, fixed)])
            }
            _ => invocation("npm", &["install", &format!("{}@{}", pkg.name, fixed)]),
        },
        "crates.io" => invocation("cargo", &["add", &format!("{}@{}", pkg.name, fixed)]),
        "PyPI" => match source {
            "uv.lock" => invocation("uv", &["add", &format!("{}=={}", pkg.name, fixed)]),
            "poetry.lock" => invocation("poetry", &["add", &format!("{}=={}", pkg.name, fixed)]),
            _ if std::path::Path::new("uv.lock").exists() => {
                invocation("uv", &["add", &format!("{}=={}", pkg.name, fixed)])
            }
            _ if std::path::Path::new("poetry.lock").exists() => {
                invocation("poetry", &["add", &format!("{}=={}", pkg.name, fixed)])
            }
            _ => invocation(
                &crate::ecosystems::detector::resolve_binary("pip"),
                &["install", &format!("{}=={}", pkg.name, fixed)],
            ),
        },
        "Go" => invocation(
            "go",
            &["get", &format!("{}@{}", pkg.name, go_version(fixed))],
        ),
        "RubyGems" => invocation(
            &crate::ecosystems::detector::resolve_binary("gem"),
            &["install", &pkg.name, "-v", fixed],
        ),
        "Packagist" => invocation("composer", &["require", &format!("{}:{}", pkg.name, fixed)]),
        "NuGet" => invocation("dotnet", &["add", "package", &pkg.name, "--version", fixed]),
        "Hex" => invocation("mix", &["deps.update", &pkg.name]),
        "Pub" | "pub.dev" => invocation(
            &crate::ecosystems::detector::resolve_binary("dart"),
            &["pub", "upgrade", &pkg.name],
        ),
        _ => invocation("echo", &[&format!("upgrade {} to {}", pkg.name, fixed)]),
    }
}

fn invocation(program: &str, args: &[&str]) -> crate::cli::PkgInvocation {
    crate::cli::PkgInvocation::from_args(program, args)
}

fn go_version(fixed: &str) -> String {
    if fixed.starts_with('v') {
        fixed.to_string()
    } else {
        format!("v{}", fixed)
    }
}

fn run_fix_command(
    label: &str,
    invocation: &crate::cli::PkgInvocation,
    success_count: &mut usize,
    fail_count: &mut usize,
) {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::with_template("  {spinner:.green}  {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", "✔"]),
    );
    spinner.enable_steady_tick(Duration::from_millis(60));
    spinner.set_message(format!(
        "{} {}",
        label.bold(),
        format!("({})", invocation.display()).truecolor(100, 100, 120)
    ));
    match invocation.output() {
        Ok(output) if output.status.success() => {
            spinner.finish_and_clear();
            *success_count += 1;
            println!(
                "  {}  {} {}",
                "✔".bright_green().bold(),
                label.bold(),
                "fixed".bright_green()
            );
        }
        Ok(output) => {
            spinner.finish_and_clear();
            *fail_count += 1;
            println!(
                "  {}  {} — command exited with code {}",
                "✘".bright_red().bold(),
                label.bold(),
                output.status.code().unwrap_or(-1)
            );
        }
        Err(err) => {
            spinner.finish_and_clear();
            *fail_count += 1;
            println!(
                "  {}  {} — could not run: {}",
                "✘".bright_red().bold(),
                label.bold(),
                err.to_string().truecolor(200, 80, 80)
            );
        }
    }
}
