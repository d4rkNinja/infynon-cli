use crate::cli::scan;
use owo_colors::OwoColorize;
use std::collections::{HashMap, HashSet};
use std::io::{self, Write};

#[derive(Debug, Clone)]
enum PkgAction {
    InstallVulnerable,
    Skip,
    InstallFixed(String),
}

pub fn ask_vuln_decisions(
    original_specs: &[String],
    hits: &[scan::VulnHit],
    ecosystem: &str,
) -> Vec<String> {
    let mut fix_map: HashMap<String, (Option<String>, bool)> = HashMap::new();
    for hit in hits {
        let entry = fix_map.entry(hit.package.clone()).or_insert((None, false));
        if hit.fixed_version.is_some() {
            entry.0 = hit.fixed_version.clone();
            entry.1 = hit.fix_is_clean;
        }
    }
    let vuln_names: HashSet<String> = hits.iter().map(|hit| hit.package.clone()).collect();
    println!(
        "\n  {} {} vulnerable package(s) in your install list:\n",
        "⚠".bold().bright_yellow(),
        vuln_names.len()
    );
    for (index, name) in vuln_names.iter().enumerate() {
        print_vuln_summary(index, name, hits, &fix_map);
    }
    println!(
        "\n  {}  Apply same action to ALL infected packages?",
        "→".truecolor(100, 100, 140)
    );
    println!("     {}  Install anyway (vulnerable)   {}  Skip all   {}  Install recommended   {}  Decide per package\n", "[1]".bold().bright_yellow(), "[2]".bold().bright_red(), "[3]".bold().bright_green(), "[4]".bold().bright_cyan());
    print!("  Choice (1/2/3/4): ");
    io::stdout().flush().ok();
    let mut choice = String::new();
    io::stdin().read_line(&mut choice).ok();
    let decisions = if let Some(action) = global_action(choice.trim()) {
        apply_global_action(&vuln_names, &fix_map, action)
    } else {
        ask_per_package(&vuln_names, &fix_map)
    };
    print_decision_summary(&decisions);
    build_final_specs(original_specs, &decisions, ecosystem)
}

pub fn format_spec_for_ecosystem(name: &str, ver: &str, ecosystem: &str) -> String {
    match ecosystem {
        "pip" | "pip3" | "uv" | "poetry" => format!("{}=={}", name, ver),
        "gem" | "composer" => format!("{}:{}", name, ver),
        "nuget" => format!("{} --version {}", name, ver),
        _ => format!("{}@{}", name, ver),
    }
}

fn print_vuln_summary(
    index: usize,
    name: &str,
    hits: &[scan::VulnHit],
    fix_map: &HashMap<String, (Option<String>, bool)>,
) {
    let (fixed, clean) = fix_map
        .get(name)
        .map(|(version, is_clean)| (version.clone(), *is_clean))
        .unwrap_or((None, true));
    let cves: Vec<_> = hits.iter().filter(|hit| hit.package == name).collect();
    let worst = cves
        .iter()
        .map(|hit| hit.severity)
        .fold("INFORMATIONAL", scan::escalate_severity);
    let hint = match fixed.as_deref() {
        Some(version) if clean => format!(" → safe: {}", version.bright_green()),
        Some(version) => format!(
            " → reduced risk: {} {}",
            version.bright_yellow(),
            "(still has CVEs)".truecolor(160, 120, 50)
        ),
        None => " (no fix available)".truecolor(160, 100, 50).to_string(),
    };
    println!(
        "  {}  {}  [{}]  {} CVE(s){}",
        format!("{:>2}.", index + 1).truecolor(80, 80, 100),
        name.bold(),
        scan::severity_colored(worst),
        cves.len(),
        hint
    );
}

fn global_action(choice: &str) -> Option<PkgAction> {
    match choice {
        "1" => Some(PkgAction::InstallVulnerable),
        "2" => Some(PkgAction::Skip),
        "3" => Some(PkgAction::InstallFixed("__per_pkg__".to_string())),
        _ => None,
    }
}

fn apply_global_action(
    vuln_names: &HashSet<String>,
    fix_map: &HashMap<String, (Option<String>, bool)>,
    action: PkgAction,
) -> HashMap<String, PkgAction> {
    let mut decisions = HashMap::new();
    for name in vuln_names {
        let resolved = match action.clone() {
            PkgAction::InstallFixed(_) => {
                match fix_map.get(name).and_then(|(version, _)| version.clone()) {
                    Some(version) => PkgAction::InstallFixed(version),
                    None => {
                        println!(
                            "  {} No fix for {} — falling back to: install vulnerable",
                            "⚠".bright_yellow(),
                            name.bold()
                        );
                        PkgAction::InstallVulnerable
                    }
                }
            }
            other => other,
        };
        decisions.insert(name.clone(), resolved);
    }
    decisions
}

fn ask_per_package(
    vuln_names: &HashSet<String>,
    fix_map: &HashMap<String, (Option<String>, bool)>,
) -> HashMap<String, PkgAction> {
    let mut decisions = HashMap::new();
    println!();
    for name in vuln_names {
        let (fixed, clean) = fix_map
            .get(name)
            .map(|(version, is_clean)| (version.clone(), *is_clean))
            .unwrap_or((None, true));
        println!("\n  Package: {}", name.bold().bright_white());
        match fixed {
            Some(ref version) if clean => println!(
                "  {}  Install anyway   {}  Skip   {}  Install {} {}",
                "[1]".bold().bright_yellow(),
                "[2]".bold().bright_red(),
                "[3]".bold().bright_green(),
                version.bright_green().bold(),
                "(verified clean)".bright_green()
            ),
            Some(ref version) => println!(
                "  {}  Install anyway   {}  Skip   {}  Install {} {}",
                "[1]".bold().bright_yellow(),
                "[2]".bold().bright_red(),
                "[3]".bold().bright_yellow(),
                version.bright_yellow().bold(),
                "(reduces risk, still has CVEs)".truecolor(180, 140, 50)
            ),
            None => println!(
                "  {}  Install anyway (no fix available)   {}  Skip",
                "[1]".bold().bright_yellow(),
                "[2]".bold().bright_red()
            ),
        }
        print!(
            "  Choice ({}): ",
            if fixed.is_some() { "1/2/3" } else { "1/2" }
        );
        io::stdout().flush().ok();
        let mut line = String::new();
        io::stdin().read_line(&mut line).ok();
        let action = match (line.trim(), fixed) {
            ("2", _) => PkgAction::Skip,
            ("3", Some(version)) => PkgAction::InstallFixed(version),
            _ => PkgAction::InstallVulnerable,
        };
        decisions.insert(name.clone(), action);
    }
    decisions
}

fn print_decision_summary(decisions: &HashMap<String, PkgAction>) {
    println!("\n  {}  Decision summary:\n", "✦".truecolor(100, 160, 255));
    for (name, action) in decisions {
        let label = match action {
            PkgAction::InstallVulnerable => "install vulnerable".bright_yellow().to_string(),
            PkgAction::Skip => "skip".bright_red().to_string(),
            PkgAction::InstallFixed(version) => format!("install {}", version.bright_green()),
        };
        println!(
            "     {}  {} → {}",
            "·".truecolor(60, 60, 80),
            name.bold(),
            label
        );
    }
    println!();
}

fn build_final_specs(
    original_specs: &[String],
    decisions: &HashMap<String, PkgAction>,
    ecosystem: &str,
) -> Vec<String> {
    let mut specs = Vec::new();
    for spec in original_specs {
        let (name, _) = scan::parse_pkg_spec(spec);
        match decisions.get(&name) {
            Some(PkgAction::Skip) => println!("  {} Skipping {}", "✘".bright_red(), name.bold()),
            Some(PkgAction::InstallFixed(version)) => {
                let new_spec = format_spec_for_ecosystem(&name, version, ecosystem);
                println!(
                    "  {} {} → {}",
                    "✔".bright_green(),
                    name.bold(),
                    new_spec.bright_green().bold()
                );
                specs.push(new_spec);
            }
            _ => specs.push(spec.clone()),
        }
    }
    specs
}
