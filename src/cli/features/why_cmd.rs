use super::*;

pub fn cmd_why(package: &str, pkg_file: Option<&str>) {
    println!();
    Logger::title("INFYNON Why", "blue");
    Logger::step(&format!("Tracing '{}'...", package));

    let packages = load_packages(pkg_file);
    if packages.is_empty() {
        Logger::error("No packages found in lock files.");
        return;
    }

    let found: Vec<&scanner::LockedPackage> = packages.iter()
        .filter(|p| p.name.to_lowercase() == package.to_lowercase())
        .collect();

    if found.is_empty() {
        Logger::error(&format!("'{}' not found in any lock file.", package));
        println!();
        return;
    }

    for pkg in &found {
        println!();
        println!(
            "  {}  {} {} {}",
            "◆".truecolor(0, 210, 255),
            pkg.name.bold().bright_white(),
            format!("v{}", pkg.version).truecolor(120, 220, 120),
            format!("({})", pkg.ecosystem).truecolor(120, 120, 140)
        );
        println!(
            "     {}  Source: {}",
            "→".truecolor(80, 80, 100),
            pkg.source.bold().truecolor(255, 170, 50)
        );
    }

    // Check if direct dependency
    let is_direct = is_direct_dep(package);
    let paths = trace_why(package);

    if is_direct {
        println!();
        println!(
            "  {}  {} is a {} of your project",
            "✔".bright_green(), package.bold(),
            "direct dependency".bold().bright_green()
        );
    } else {
        println!();
        println!(
            "  {}  {} is a {} (pulled in by another package)",
            "ℹ".bright_cyan(), package.bold(),
            "transitive dependency".bold().truecolor(255, 170, 50)
        );
    }

    if !paths.is_empty() {
        println!();
        println!("  {}  Dependency chain(s):\n", "→".truecolor(100, 100, 140));
        for path in &paths {
            print!("     ");
            for (j, step) in path.iter().enumerate() {
                if j > 0 { print!("{}", " → ".truecolor(80, 80, 100)); }
                if j == 0 { print!("{}", step.bold().truecolor(0, 210, 255)); }
                else if j == path.len() - 1 { print!("{}", step.bold().bright_green()); }
                else { print!("{}", step.bold().truecolor(200, 200, 220)); }
            }
            println!();
        }
    }
    println!();
}

fn is_direct_dep(package: &str) -> bool {
    if npm_declared_deps().contains(package) { return true; }
    if cargo_toml_dep_names().iter().any(|n| n == package) { return true; }
    if let Ok(c) = fs::read_to_string("requirements.txt") {
        if c.lines().any(|l| l.trim().to_lowercase().starts_with(&package.to_lowercase())) { return true; }
    }
    for file in &["pyproject.toml", "go.mod", "Gemfile", "composer.json", "pubspec.yaml", "mix.exs"] {
        if let Ok(c) = fs::read_to_string(file) {
            if c.to_lowercase().contains(&package.to_lowercase()) { return true; }
        }
    }
    false
}

fn trace_why(package: &str) -> Vec<Vec<String>> {
    let mut paths = Vec::new();

    // npm: package-lock.json
    if let Ok(content) = fs::read_to_string("package-lock.json") {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(pkgs) = json.get("packages").and_then(|p| p.as_object()) {
                for (key, val) in pkgs {
                    if let Some(deps) = val.get("dependencies").and_then(|d| d.as_object()) {
                        if deps.contains_key(package) {
                            let parent = key.trim_start_matches("node_modules/");
                            if parent.is_empty() {
                                paths.push(vec!["(project)".to_string(), package.to_string()]);
                            } else {
                                paths.push(vec!["(project)".to_string(), parent.to_string(), package.to_string()]);
                            }
                        }
                    }
                }
            }
        }
    }

    // Cargo.lock — reuse shared parser
    let pkg_deps = cargo_lock_deps();
    if !pkg_deps.is_empty() {
        let root = cargo_root_name().unwrap_or_else(|| "(project)".to_string());
        for (parent, deps) in &pkg_deps {
            if deps.iter().any(|d| d.to_lowercase() == package.to_lowercase()) {
                if *parent == root {
                    paths.push(vec![root.clone(), package.to_string()]);
                } else {
                    paths.push(vec![root.clone(), parent.clone(), package.to_string()]);
                }
            }
        }
    }

    paths.truncate(10); // Limit output
    paths
}
