use crate::engine::scanner;
use crate::tui::logger::Logger;
use owo_colors::OwoColorize;
use std::io::{self, Write};

pub fn run_scan(
    output: Option<crate::cli::scan::OutputFormat>,
    fix_level: Option<crate::cli::scan::FixLevel>,
    pkg_file: Option<&str>,
    agent: bool,
) {
    if !agent {
        println!();
        Logger::title("INFYNON Package Scanner", "blue");
    }
    if !agent {
        if let Some(file) = pkg_file {
            Logger::step(&format!("Using custom file: {}", file));
        } else {
            Logger::step("Detecting lock files in current directory...");
        }
    }
    let packages = collect_packages(pkg_file, agent);
    if packages.is_empty() {
        if agent {
            crate::cli::scan::agent::exit_error(
                "No packages found in selected lock/manifest files",
            );
        }
        Logger::error("No packages found in selected lock/manifest files.");
        return;
    }
    if !agent {
        let mut sources: Vec<String> = packages.iter().map(|pkg| pkg.source.clone()).collect();
        sources.sort();
        sources.dedup();
        Logger::success(&format!(
            "Found {} pinned packages from: {}",
            packages.len(),
            sources.join(", ")
        ));
        println!();
    }
    if agent {
        crate::cli::scan::agent::run_agent_scan(&packages, fix_level.as_ref())
    } else {
        crate::cli::scan::human::run_human_scan(&packages, output, fix_level.as_ref())
    }
}

fn collect_packages(pkg_file: Option<&str>, agent: bool) -> Vec<scanner::LockedPackage> {
    if let Some(file) = pkg_file {
        return scanner::detect_locked_packages(Some(file));
    }
    let found = scanner::detect_lock_files();
    if found.is_empty() {
        if agent {
            return Vec::new();
        }
        Logger::error("No packages found in supported lock/manifest files.");
        Logger::info(
            "Supported: package-lock.json · yarn.lock · pnpm-lock.yaml · requirements.txt",
        );
        Logger::info(
            "           poetry.lock · Cargo.lock · go.sum Â· Gemfile.lock Â· composer.lock",
        );
        Logger::info("           mix.lock Â· pubspec.lock  — or pass --pkg-file <path>");
        return Vec::new();
    }
    if found.len() == 1 {
        return scanner::parse_selected_files(&[found[0].0]);
    }
    if agent {
        return scanner::parse_selected_files(
            &found.iter().map(|(file, _)| *file).collect::<Vec<_>>(),
        );
    }
    println!(
        "\n  {} Found {} lock/manifest files:\n",
        "ℹ".bright_cyan().bold(),
        found.len()
    );
    for (index, (file, ecosystem)) in found.iter().enumerate() {
        println!(
            "     {}  {} {}",
            format!("[{}]", index + 1).bold().bright_cyan(),
            file.bold(),
            format!("({})", ecosystem).truecolor(120, 120, 140)
        );
    }
    println!("\n     {}  Scan all files\n", "[A]".bold().bright_green());
    print!("  Select files to scan (e.g. 1,3 or A for all): ");
    io::stdout().flush().ok();
    let mut choice = String::new();
    io::stdin().read_line(&mut choice).ok();
    let selection: Vec<&str> =
        if choice.trim().eq_ignore_ascii_case("a") || choice.trim().is_empty() {
            found.iter().map(|(file, _)| *file).collect()
        } else {
            choice
                .split(',')
                .filter_map(|part| {
                    part.trim()
                        .parse::<usize>()
                        .ok()
                        .and_then(|index| found.get(index - 1).map(|entry| entry.0))
                })
                .collect()
        };
    if selection.is_empty() {
        Logger::error("No valid files selected.");
        Vec::new()
    } else {
        for file in &selection {
            Logger::detail("Â» Scanning:", file);
        }
        scanner::parse_selected_files(&selection)
    }
}
