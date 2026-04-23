use crate::engine::{osv, reporter, scanner};
use crate::tui::logger::Logger;
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use std::collections::HashMap;
use std::time::Duration;

pub(super) fn run_auto_fix(findings: &[reporter::ScanFinding]) {
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
    let items: Vec<(String, String)> = pkg_map
        .values()
        .filter_map(|(pkg, confirmed, suggested)| {
            let best = if !confirmed.is_empty() {
                osv::max_version(confirmed)
            } else {
                osv::max_version(suggested)
            }?;
            Some((
                format!("{} {} → {}", pkg.name, pkg.version, best),
                upgrade_cmd(pkg, &best),
            ))
        })
        .collect();
    if items.is_empty() {
        return Logger::info("No packages have a known fixed or suggested version available.");
    }
    println!(
        "\n  {} {}\n",
        "⚡ Auto-Fix".bold().truecolor(255, 200, 50),
        format!("Upgrading {} package(s)...", items.len()).truecolor(160, 160, 180)
    );
    let (mut success_count, mut fail_count) = (0usize, 0usize);
    for (label, cmd) in items {
        run_fix_command(&label, &cmd, &mut success_count, &mut fail_count);
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
}

pub fn upgrade_cmd(pkg: &scanner::LockedPackage, fixed: &str) -> String {
    let source = std::path::Path::new(&pkg.source)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(&pkg.source);
    match pkg.ecosystem.as_str() {
        "npm" => match source {
            "yarn.lock" => format!("yarn add {}@{}", pkg.name, fixed),
            "pnpm-lock.yaml" => format!("pnpm add {}@{}", pkg.name, fixed),
            "bun.lockb" | "bun.lock" => format!("bun add {}@{}", pkg.name, fixed),
            _ if std::path::Path::new("bun.lockb").exists() => {
                format!("bun add {}@{}", pkg.name, fixed)
            }
            _ => format!("npm install {}@{}", pkg.name, fixed),
        },
        "crates.io" => format!("cargo add {}@{}", pkg.name, fixed),
        "PyPI" => match source {
            "uv.lock" => format!("uv add {}=={}", pkg.name, fixed),
            "poetry.lock" => format!("poetry add {}=={}", pkg.name, fixed),
            _ if std::path::Path::new("uv.lock").exists() => {
                format!("uv add {}=={}", pkg.name, fixed)
            }
            _ if std::path::Path::new("poetry.lock").exists() => {
                format!("poetry add {}=={}", pkg.name, fixed)
            }
            _ => format!(
                "{} install {}=={}",
                crate::ecosystems::detector::resolve_binary("pip"),
                pkg.name,
                fixed
            ),
        },
        "Go" => format!(
            "go get {}@{}",
            pkg.name,
            if fixed.starts_with('v') {
                fixed.to_string()
            } else {
                format!("v{}", fixed)
            }
        ),
        "RubyGems" => format!(
            "{} install {} -v {}",
            crate::ecosystems::detector::resolve_binary("gem"),
            pkg.name,
            fixed
        ),
        "Packagist" => format!("composer require {}:{}", pkg.name, fixed),
        "NuGet" => format!("dotnet add package {} --version {}", pkg.name, fixed),
        "Hex" => format!("mix deps.update {}", pkg.name),
        "pub.dev" => format!(
            "{} pub upgrade {}",
            crate::ecosystems::detector::resolve_binary("dart"),
            pkg.name
        ),
        _ => format!("upgrade {} to {}", pkg.name, fixed),
    }
}

fn run_fix_command(label: &str, cmd: &str, success_count: &mut usize, fail_count: &mut usize) {
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
        format!("({})", cmd).truecolor(100, 100, 120)
    ));
    match crate::cli::run_pkg_cmd(cmd) {
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
