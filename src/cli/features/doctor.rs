use super::*;

pub fn cmd_doctor(pkg_file: Option<&str>) {
    println!();
    Logger::title("INFYNON Doctor", "blue");
    Logger::step("Running health checks...\n");

    let packages = load_packages(pkg_file);
    if packages.is_empty() {
        Logger::error("No packages found in lock files.");
        return;
    }

    let mut issues = 0usize;
    let mut warnings = 0usize;

    // ── Check 1: Duplicates ──────────────────────────────────────────────────
    println!(
        "  {}  {}",
        "1".bold().truecolor(0, 210, 255),
        "Duplicate packages (same name, different versions)"
            .bold()
            .white()
    );
    let mut ver_map: HashMap<String, Vec<String>> = HashMap::new();
    for p in &packages {
        ver_map
            .entry(format!("{}:{}", p.ecosystem, p.name))
            .or_default()
            .push(p.version.clone());
    }
    let mut found_dupes = false;
    for (key, mut vers) in ver_map {
        vers.sort();
        vers.dedup();
        if vers.len() > 1 {
            found_dupes = true;
            warnings += 1;
            let name = key.split_once(':').map(|x| x.1).unwrap_or("?");
            println!(
                "     {} {} has {} versions: {}",
                "⚠".bright_yellow(),
                name.bold(),
                vers.len(),
                vers.join(", ").truecolor(255, 170, 50)
            );
        }
    }
    if !found_dupes {
        println!("     {} No duplicates found", "✔".bright_green());
    }

    // ── Check 2: Unused dependencies ─────────────────────────────────────────
    println!();
    println!(
        "  {}  {}",
        "2".bold().truecolor(0, 210, 255),
        "Potentially unused dependencies".bold().white()
    );
    // Pre-compute source content once for both unused and phantom checks
    let source_js = collect_source(&["js", "ts", "jsx", "tsx", "mjs", "cjs"]);
    let source_rs = collect_source(&["rs"]);

    let unused = find_unused_deps(&source_js, &source_rs);
    if unused.is_empty() {
        println!(
            "     {} No unused dependencies detected",
            "✔".bright_green()
        );
    } else {
        for (name, eco) in &unused {
            warnings += 1;
            println!(
                "     {} {} ({}) — declared but no imports found",
                "⚠".bright_yellow(),
                name.bold(),
                eco.truecolor(120, 120, 140)
            );
        }
    }

    // ── Check 3: Phantom dependencies ────────────────────────────────────────
    println!();
    println!(
        "  {}  {}",
        "3".bold().truecolor(0, 210, 255),
        "Phantom dependencies (imported but not declared)"
            .bold()
            .white()
    );
    let phantoms = find_phantom_deps(&source_js);
    if phantoms.is_empty() {
        println!(
            "     {} No phantom dependencies detected",
            "✔".bright_green()
        );
    } else {
        for name in &phantoms {
            issues += 1;
            println!(
                "     {} {} — imported but not in manifest",
                "✘".bright_red(),
                name.bold()
            );
        }
    }

    // ── Check 4: Lock file health ────────────────────────────────────────────
    println!();
    println!(
        "  {}  {}",
        "4".bold().truecolor(0, 210, 255),
        "Lock file presence".bold().white()
    );
    for (msg, ok) in check_lock_health() {
        if ok {
            println!("     {} {}", "✔".bright_green(), msg);
        } else {
            warnings += 1;
            println!("     {} {}", "⚠".bright_yellow(), msg);
        }
    }

    // ── Check 5: Risky scripts ───────────────────────────────────────────────
    println!();
    println!(
        "  {}  {}",
        "5".bold().truecolor(0, 210, 255),
        "Risky install scripts".bold().white()
    );
    let mut found_risky = false;
    if let Ok(c) = fs::read_to_string("package.json") {
        if let Ok(j) = serde_json::from_str::<serde_json::Value>(&c) {
            if let Some(scripts) = j.get("scripts").and_then(|s| s.as_object()) {
                for s in &["preinstall", "postinstall", "preuninstall"] {
                    if scripts.contains_key(*s) {
                        found_risky = true;
                        warnings += 1;
                        println!(
                            "     {} package.json has '{}' script",
                            "⚠".bright_yellow(),
                            s.bold()
                        );
                    }
                }
            }
        }
    }
    if !found_risky {
        println!("     {} No risky install scripts found", "✔".bright_green());
    }

    // Summary
    println!();
    println!("  {}", "─".repeat(66).truecolor(40, 40, 60));
    let health = if issues == 0 && warnings == 0 {
        "HEALTHY".bold().bright_green().to_string()
    } else if issues == 0 {
        "FAIR".bold().bright_yellow().to_string()
    } else {
        "NEEDS ATTENTION".bold().bright_red().to_string()
    };
    println!(
        "\n  {}  Health: {}  ·  {} issues  ·  {} warnings\n",
        "◆".truecolor(0, 210, 255),
        health,
        issues.to_string().bold(),
        warnings.to_string().bold(),
    );
}

pub(crate) fn find_unused_deps(source_js: &str, source_rs: &str) -> Vec<(String, String)> {
    let mut unused = Vec::new();

    // npm
    let npm_deps = npm_declared_deps();
    // Only check "dependencies", not devDependencies/peerDependencies
    if let Ok(c) = fs::read_to_string("package.json") {
        if let Ok(j) = serde_json::from_str::<serde_json::Value>(&c) {
            if let Some(deps) = j.get("dependencies").and_then(|d| d.as_object()) {
                for name in deps.keys() {
                    let pats = [
                        format!("'{}'", name),
                        format!("\"{}\"", name),
                        format!("'{}/", name),
                        format!("\"{}/", name),
                    ];
                    if !pats.iter().any(|p| source_js.contains(p)) {
                        unused.push((name.clone(), "npm".to_string()));
                    }
                }
            }
        }
    }
    // suppress unused variable warning
    let _ = npm_deps;

    // Cargo
    for name in cargo_toml_dep_names() {
        let use_name = name.replace('-', "_");
        let pats = [
            format!("use {}", use_name),
            format!("{}::", use_name),
            format!("extern crate {}", use_name),
        ];
        if !pats.iter().any(|p| source_rs.contains(p)) {
            unused.push((name, "cargo".to_string()));
        }
    }
    unused
}

fn find_phantom_deps(source_js: &str) -> Vec<String> {
    let mut phantom_set: HashSet<String> = HashSet::new();
    if !Path::new("package.json").exists() {
        return phantom_set.into_iter().collect();
    }
    let declared = npm_declared_deps();
    for line in source_js.lines() {
        let line = line.trim();
        for delim in &["'", "\""] {
            if let Some(start) = line
                .find(&format!("require({}", delim))
                .or_else(|| line.find(&format!("from {}", delim)))
            {
                let rest = &line[start..];
                let inner_start = rest.find(*delim).unwrap_or(0) + 1;
                if let Some(inner_end) = rest[inner_start..].find(*delim) {
                    let pkg = &rest[inner_start..inner_start + inner_end];
                    let pkg_name = if pkg.starts_with('@') {
                        pkg.splitn(3, '/').take(2).collect::<Vec<_>>().join("/")
                    } else {
                        pkg.split('/').next().unwrap_or(pkg).to_string()
                    };
                    if !pkg_name.is_empty()
                        && !pkg_name.starts_with('.')
                        && !pkg_name.starts_with('/')
                        && !declared.contains(&pkg_name)
                    {
                        let builtins = [
                            "fs",
                            "path",
                            "os",
                            "http",
                            "https",
                            "crypto",
                            "util",
                            "stream",
                            "events",
                            "child_process",
                            "url",
                            "querystring",
                            "assert",
                            "buffer",
                            "net",
                            "tls",
                            "dns",
                            "cluster",
                            "readline",
                            "zlib",
                            "vm",
                            "worker_threads",
                            "perf_hooks",
                            "process",
                            "module",
                            "console",
                            "timers",
                        ];
                        if !builtins.contains(&pkg_name.as_str()) && !pkg_name.starts_with("node:")
                        {
                            phantom_set.insert(pkg_name);
                        }
                    }
                }
            }
        }
    }
    phantom_set.into_iter().take(20).collect()
}

pub(crate) fn collect_source(extensions: &[&str]) -> String {
    let mut buf = String::new();
    let dirs = ["src", "lib", "app", "pages", "components", "."];
    for dir in &dirs {
        if Path::new(dir).is_dir() {
            walk_source(dir, extensions, &mut buf, 3);
        }
    }
    buf
}

fn walk_source(dir: &str, exts: &[&str], buf: &mut String, depth: usize) {
    if depth == 0 {
        return;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let skip = [
                "node_modules",
                "target",
                "dist",
                "build",
                "__pycache__",
                "vendor",
                ".git",
                ".next",
            ];
            if name.starts_with('.') || skip.contains(&name) {
                continue;
            }
            walk_source(path.to_str().unwrap_or(""), exts, buf, depth - 1);
        } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if exts.contains(&ext) {
                if let Ok(c) = fs::read_to_string(&path) {
                    buf.push_str(&c);
                    buf.push('\n');
                }
            }
        }
    }
}

fn check_lock_health() -> Vec<(String, bool)> {
    let mut checks = Vec::new();
    let manifest_lock: Vec<(&str, &[&str])> = vec![
        (
            "package.json",
            &[
                "package-lock.json",
                "yarn.lock",
                "pnpm-lock.yaml",
                "bun.lockb",
            ],
        ),
        ("Cargo.toml", &["Cargo.lock"]),
        ("go.mod", &["go.sum"]),
        ("Gemfile", &["Gemfile.lock"]),
        ("composer.json", &["composer.lock"]),
        ("pubspec.yaml", &["pubspec.lock"]),
        ("mix.exs", &["mix.lock"]),
    ];
    for (manifest, locks) in &manifest_lock {
        if Path::new(manifest).exists() {
            let has = locks.iter().any(|l| Path::new(l).exists());
            checks.push((
                format!(
                    "{} {} lock file",
                    manifest,
                    if has { "has" } else { "MISSING" }
                ),
                has,
            ));
        }
    }
    if Path::new("pyproject.toml").exists() || Path::new("requirements.txt").exists() {
        let has = Path::new("poetry.lock").exists()
            || Path::new("uv.lock").exists()
            || Path::new("requirements.txt").exists();
        checks.push((
            format!(
                "Python project {} pinned deps",
                if has { "has" } else { "MISSING" }
            ),
            has,
        ));
    }
    if checks.is_empty() {
        checks.push(("No manifest files found".to_string(), false));
    }
    checks
}
