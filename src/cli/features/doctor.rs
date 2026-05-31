use super::*;
use serde_json::Value;
use std::process::Command;

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

pub fn cmd_doctor_npm() {
    println!();
    Logger::title("INFYNON npm Doctor", "blue");
    Logger::step("Checking the npm wrapper, native binary, shell, and release metadata...\n");

    let wrapper = std::env::var("INFYNON_NPM_WRAPPER").ok();
    let package_dir = std::env::var("INFYNON_NPM_PACKAGE_DIR").ok();
    let binary = std::env::var("INFYNON_NPM_BINARY").ok().or_else(|| {
        std::env::current_exe()
            .ok()
            .map(|path| path.display().to_string())
    });
    let binary_source = std::env::var("INFYNON_NPM_BINARY_SOURCE").ok();

    print_doctor_check(
        "platform",
        "info",
        &format!("{} {}", std::env::consts::OS, std::env::consts::ARCH),
    );
    print_doctor_check(
        "node",
        if command_exists("node") { "ok" } else { "warn" },
        &command_output("node", &["--version"]).unwrap_or_else(|| "not found on PATH".to_string()),
    );
    print_doctor_check(
        "npm",
        if command_exists("npm") { "ok" } else { "warn" },
        &command_output("npm", &["--version"]).unwrap_or_else(|| "not found on PATH".to_string()),
    );
    print_doctor_check(
        "npm prefix",
        "info",
        &command_output("npm", &["prefix", "-g"]).unwrap_or_else(|| "unavailable".to_string()),
    );
    print_doctor_check(
        "npm root",
        "info",
        &command_output("npm", &["root", "-g"]).unwrap_or_else(|| "unavailable".to_string()),
    );
    print_doctor_check(
        "wrapper",
        if wrapper.is_some() { "ok" } else { "warn" },
        wrapper.as_deref().unwrap_or(
            "INFYNON_NPM_WRAPPER is not set; this binary may not be running through npm.",
        ),
    );
    print_doctor_check(
        "package dir",
        if package_dir
            .as_ref()
            .map(|value| Path::new(value).is_dir())
            .unwrap_or(false)
        {
            "ok"
        } else {
            "warn"
        },
        package_dir
            .as_deref()
            .unwrap_or("INFYNON_NPM_PACKAGE_DIR is not set."),
    );

    if let Some(binary_path) = binary.as_deref() {
        let path = Path::new(binary_path);
        print_doctor_check(
            "native binary",
            if path.is_file() { "ok" } else { "fail" },
            binary_path,
        );
        print_doctor_check(
            "binary source",
            "info",
            binary_source.as_deref().unwrap_or("direct-or-unknown"),
        );
        print_doctor_check(
            "binary version",
            "info",
            &command_output(binary_path, &["--version"])
                .unwrap_or_else(|| "unavailable".to_string()),
        );
    } else {
        print_doctor_check(
            "native binary",
            "fail",
            "Unable to resolve native binary path.",
        );
    }

    print_doctor_check("shell", "info", &detect_shell());
    print_doctor_check("PATH entries", "info", &path_entry_summary());

    if cfg!(windows) {
        print_doctor_check(
            "PowerShell policy",
            "info",
            &command_output(
                "powershell",
                &[
                    "-NoProfile",
                    "-Command",
                    "Get-ExecutionPolicy -List | Out-String",
                ],
            )
            .unwrap_or_else(|| "unavailable".to_string()),
        );
        print_doctor_check("Windows long paths", "info", &windows_long_paths_status());
    }

    let manifest = release_manifest_status(binary.as_deref());
    print_doctor_check(&manifest.0, &manifest.1, &manifest.2);

    println!();
    println!("Next steps if npm launch still fails:");
    println!("  npm uninstall -g infynon");
    println!("  npm cache clean --force");
    println!("  npm install -g infynon");
    println!("  infynon doctor npm");
    if cfg!(windows) {
        println!("  Check Windows Defender quarantine and any enterprise antivirus logs.");
    }
}

fn print_doctor_check(name: &str, status: &str, detail: &str) {
    let label = match status {
        "ok" => "OK",
        "warn" => "WARN",
        "fail" => "FAIL",
        _ => "INFO",
    };
    let detail = detail.trim();
    if detail.contains('\n') {
        println!("  [{label}] {name}");
        for line in detail
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
        {
            println!("       {line}");
        }
    } else {
        println!("  [{label}] {name}: {detail}");
    }
}

fn command_exists(command: &str) -> bool {
    let locator = if cfg!(windows) { "where" } else { "which" };
    Command::new(locator)
        .arg(command)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn command_output(command: &str, args: &[&str]) -> Option<String> {
    let output = command_candidates(command)
        .into_iter()
        .find_map(|candidate| Command::new(candidate).args(args).output().ok())?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if output.status.success() {
        Some(if stdout.is_empty() {
            "ok".to_string()
        } else {
            stdout
        })
    } else if stderr.is_empty() {
        Some(format!("exit code {:?}", output.status.code()))
    } else {
        Some(stderr)
    }
}

fn command_candidates(command: &str) -> Vec<String> {
    let has_path_separator = command.contains('\\') || command.contains('/');
    let has_extension = Path::new(command).extension().is_some();
    if !cfg!(windows) || has_path_separator || has_extension {
        return vec![command.to_string()];
    }
    vec![
        command.to_string(),
        format!("{command}.exe"),
        format!("{command}.cmd"),
        format!("{command}.bat"),
    ]
}

fn detect_shell() -> String {
    if cfg!(windows) {
        std::env::var("PSModulePath")
            .ok()
            .map(|_| "PowerShell environment detected".to_string())
            .or_else(|| std::env::var("ComSpec").ok())
            .unwrap_or_else(|| "unknown".to_string())
    } else {
        std::env::var("SHELL").unwrap_or_else(|_| "unknown".to_string())
    }
}

fn path_entry_summary() -> String {
    let separator = if cfg!(windows) { ';' } else { ':' };
    let path = std::env::var("PATH").unwrap_or_default();
    let count = path
        .split(separator)
        .filter(|entry| !entry.is_empty())
        .count();
    format!("{count} entries")
}

fn windows_long_paths_status() -> String {
    let output = command_output(
        "reg",
        &[
            "query",
            r"HKLM\SYSTEM\CurrentControlSet\Control\FileSystem",
            "/v",
            "LongPathsEnabled",
        ],
    )
    .unwrap_or_else(|| "unavailable".to_string());
    if output.contains("0x1") {
        "enabled".to_string()
    } else if output.contains("0x0") {
        "disabled".to_string()
    } else {
        output
    }
}

fn release_manifest_status(binary_path: Option<&str>) -> (String, String, String) {
    let version = env!("CARGO_PKG_VERSION");
    let url = format!(
        "https://github.com/d4rkNinja/infynon-cli/releases/download/v{version}/release-manifest.json"
    );
    let response = match http_client().get(&url).send() {
        Ok(response) => response,
        Err(err) => {
            return (
                "release manifest".to_string(),
                "warn".to_string(),
                format!("unavailable: {err}"),
            )
        }
    };
    if !response.status().is_success() {
        return (
            "release manifest".to_string(),
            "warn".to_string(),
            format!("{} returned HTTP {}", url, response.status()),
        );
    }
    let manifest: Value = match response.json() {
        Ok(value) => value,
        Err(err) => {
            return (
                "release manifest".to_string(),
                "warn".to_string(),
                format!("invalid JSON: {err}"),
            )
        }
    };
    let expected_asset = match release_asset_name_for_host() {
        Some(asset) => asset,
        None => {
            return (
                "release manifest".to_string(),
                "warn".to_string(),
                "unsupported host target for manifest comparison".to_string(),
            )
        }
    };
    let asset = manifest
        .get("assets")
        .and_then(Value::as_array)
        .and_then(|assets| {
            assets.iter().find(|asset| {
                asset
                    .get("asset")
                    .and_then(Value::as_str)
                    .map(|name| name == expected_asset)
                    .unwrap_or(false)
            })
        });
    let Some(asset) = asset else {
        return (
            "release manifest".to_string(),
            "warn".to_string(),
            format!("manifest found, but {expected_asset} is missing"),
        );
    };

    let manifest_sha = asset.get("sha256").and_then(Value::as_str);
    let manifest_size = asset.get("size").and_then(Value::as_u64);
    if let (Some(binary_path), Some(manifest_sha), Some(manifest_size)) =
        (binary_path, manifest_sha, manifest_size)
    {
        let path = Path::new(binary_path);
        let size_ok = path
            .metadata()
            .map(|meta| meta.len() == manifest_size)
            .unwrap_or(false);
        let hash_status = file_sha256(path)
            .map(|sha| {
                if sha.eq_ignore_ascii_case(manifest_sha) {
                    "checksum ok".to_string()
                } else {
                    format!("checksum mismatch: local {sha}, manifest {manifest_sha}")
                }
            })
            .unwrap_or_else(|| "checksum unavailable".to_string());
        let status = if size_ok && hash_status == "checksum ok" {
            "ok"
        } else {
            "warn"
        };
        return (
            "release manifest".to_string(),
            status.to_string(),
            format!(
                "{expected_asset}; size {}; {hash_status}",
                if size_ok { "ok" } else { "mismatch" }
            ),
        );
    }

    (
        "release manifest".to_string(),
        "ok".to_string(),
        format!("{expected_asset} present"),
    )
}

fn release_asset_name_for_host() -> Option<String> {
    let target = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("windows", "x86_64") => "x86_64-pc-windows-msvc.exe",
        ("linux", "x86_64") => "x86_64-unknown-linux-musl",
        ("linux", "aarch64") => "aarch64-unknown-linux-musl",
        ("macos", "x86_64") => "x86_64-apple-darwin",
        ("macos", "aarch64") => "aarch64-apple-darwin",
        _ => return None,
    };
    Some(format!("infynon-{target}"))
}

fn file_sha256(path: &Path) -> Option<String> {
    if cfg!(windows) {
        let escaped = path.display().to_string().replace('\'', "''");
        let script = format!(
            "(Get-FileHash -Algorithm SHA256 -LiteralPath '{}').Hash.ToLowerInvariant()",
            escaped
        );
        command_output("powershell", &["-NoProfile", "-Command", &script])
    } else {
        let output = Command::new("sha256sum").arg(path).output().ok();
        if let Some(output) = output.filter(|output| output.status.success()) {
            return String::from_utf8_lossy(&output.stdout)
                .split_whitespace()
                .next()
                .map(str::to_string);
        }
        let output = Command::new("shasum")
            .args(["-a", "256"])
            .arg(path)
            .output()
            .ok()?;
        if output.status.success() {
            String::from_utf8_lossy(&output.stdout)
                .split_whitespace()
                .next()
                .map(str::to_string)
        } else {
            None
        }
    }
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
