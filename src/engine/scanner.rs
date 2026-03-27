use serde_json::Value;
use std::path::Path;
use std::fs;
use std::collections::HashSet;

/// A resolved package entry from a lock file or manifest.
#[derive(Debug, Clone)]
pub struct LockedPackage {
    pub name:      String,
    pub version:   String,
    pub ecosystem: String,
    pub source:    String, // which file it came from
}

/// Detect all lock/manifest files and parse pinned packages.
/// If `custom_file` is provided, only parse that file.
pub fn detect_locked_packages(custom_file: Option<&str>) -> Vec<LockedPackage> {
    if let Some(path) = custom_file {
        return parse_custom_file(path);
    }

    let mut packages = Vec::new();

    // ── JavaScript ───────────────────────────────────────────────────────────
    if Path::new("package-lock.json").exists() {
        packages.extend(parse_npm_lock("package-lock.json"));
    }
    if Path::new("yarn.lock").exists() {
        packages.extend(parse_yarn_lock("yarn.lock"));
    }
    if Path::new("pnpm-lock.yaml").exists() {
        packages.extend(parse_pnpm_lock("pnpm-lock.yaml"));
    }
    // bun.lockb is binary — fall back to package.json for bun
    if Path::new("bun.lockb").exists() || (Path::new("package.json").exists() && !Path::new("package-lock.json").exists()) {
        packages.extend(parse_package_json("package.json"));
    }

    // ── Python ───────────────────────────────────────────────────────────────
    if Path::new("requirements.txt").exists() {
        packages.extend(parse_requirements_txt("requirements.txt"));
    }
    if Path::new("pyproject.toml").exists() {
        packages.extend(parse_pyproject_toml("pyproject.toml"));
    }
    if Path::new("poetry.lock").exists() {
        packages.extend(parse_poetry_lock("poetry.lock"));
    }
    if Path::new("uv.lock").exists() {
        packages.extend(parse_uv_lock("uv.lock"));
    }

    // ── Rust ─────────────────────────────────────────────────────────────────
    if Path::new("Cargo.lock").exists() {
        packages.extend(parse_cargo_lock("Cargo.lock"));
    }

    // ── Go ───────────────────────────────────────────────────────────────────
    if Path::new("go.sum").exists() {
        packages.extend(parse_go_sum("go.sum"));
    }
    if Path::new("go.mod").exists() && !Path::new("go.sum").exists() {
        packages.extend(parse_go_mod("go.mod"));
    }

    // ── Ruby ─────────────────────────────────────────────────────────────────
    if Path::new("Gemfile.lock").exists() {
        packages.extend(parse_gemfile_lock("Gemfile.lock"));
    }

    // ── PHP ──────────────────────────────────────────────────────────────────
    if Path::new("composer.lock").exists() {
        packages.extend(parse_composer_lock("composer.lock"));
    }

    // ── .NET ─────────────────────────────────────────────────────────────────
    // Walk looking for packages.lock.json (NuGet) or *.csproj
    for entry in ["packages.lock.json", "package.lock.json"] {
        if Path::new(entry).exists() {
            packages.extend(parse_nuget_lock(entry));
        }
    }

    // ── Elixir ───────────────────────────────────────────────────────────────
    if Path::new("mix.lock").exists() {
        packages.extend(parse_mix_lock("mix.lock"));
    }

    // ── Dart ─────────────────────────────────────────────────────────────────
    if Path::new("pubspec.lock").exists() {
        packages.extend(parse_pubspec_lock("pubspec.lock"));
    }

    packages
}

/// If user passed `--pkg-file`, detect type by extension/name and parse.
fn parse_custom_file(path: &str) -> Vec<LockedPackage> {
    let name = Path::new(path).file_name().and_then(|n| n.to_str()).unwrap_or("");
    match name {
        "package-lock.json"      => parse_npm_lock(path),
        "yarn.lock"              => parse_yarn_lock(path),
        "pnpm-lock.yaml"         => parse_pnpm_lock(path),
        "package.json"           => parse_package_json(path),
        "requirements.txt"       => parse_requirements_txt(path),
        "pyproject.toml"         => parse_pyproject_toml(path),
        "poetry.lock"            => parse_poetry_lock(path),
        "uv.lock"                => parse_uv_lock(path),
        "Cargo.lock"             => parse_cargo_lock(path),
        "go.sum"                 => parse_go_sum(path),
        "go.mod"                 => parse_go_mod(path),
        "Gemfile.lock"           => parse_gemfile_lock(path),
        "composer.lock"          => parse_composer_lock(path),
        "packages.lock.json"     => parse_nuget_lock(path),
        "mix.lock"               => parse_mix_lock(path),
        "pubspec.lock"           => parse_pubspec_lock(path),
        _ => {
            eprintln!("Unsupported file type: {}", path);
            vec![]
        }
    }
}

// ── Parsers ───────────────────────────────────────────────────────────────────

fn parse_npm_lock(path: &str) -> Vec<LockedPackage> {
    let mut out = Vec::new();
    let Ok(content) = fs::read_to_string(path) else { return out; };
    let Ok(json): Result<Value, _> = serde_json::from_str(&content) else { return out; };
    if let Some(packages) = json.get("packages").and_then(|p| p.as_object()) {
        for (key, val) in packages {
            let name = key.trim_start_matches("node_modules/");
            if name.is_empty() { continue; }
            if let Some(version) = val.get("version").and_then(|v| v.as_str()) {
                out.push(LockedPackage { name: name.to_string(), version: version.to_string(), ecosystem: "npm".to_string(), source: path.to_string() });
            }
        }
    } else if let Some(deps) = json.get("dependencies").and_then(|d| d.as_object()) {
        for (name, val) in deps {
            if let Some(version) = val.get("version").and_then(|v| v.as_str()) {
                out.push(LockedPackage { name: name.clone(), version: version.to_string(), ecosystem: "npm".to_string(), source: path.to_string() });
            }
        }
    }
    out
}

fn parse_package_json(path: &str) -> Vec<LockedPackage> {
    let mut out = Vec::new();
    let Ok(content) = fs::read_to_string(path) else { return out; };
    let Ok(json): Result<Value, _> = serde_json::from_str(&content) else { return out; };
    for key in &["dependencies", "devDependencies", "peerDependencies"] {
        if let Some(deps) = json.get(key).and_then(|d| d.as_object()) {
            for (name, version) in deps {
                let ver = version.as_str().unwrap_or("").trim_start_matches('^').trim_start_matches('~').to_string();
                if !ver.is_empty() && ver != "*" {
                    out.push(LockedPackage { name: name.clone(), version: ver, ecosystem: "npm".to_string(), source: path.to_string() });
                }
            }
        }
    }
    out
}

fn parse_yarn_lock(path: &str) -> Vec<LockedPackage> {
    let mut out = Vec::new();
    let Ok(content) = fs::read_to_string(path) else { return out; };
    let mut current_name: Option<String> = None;
    for line in content.lines() {
        let line = line.trim();
        if line.ends_with(':') && !line.starts_with('#') && !line.starts_with("\"__metadata") {
            let entry = line.trim_end_matches(':').trim_matches('"');
            // Handle scoped packages: @scope/name@version
            // The name is everything up to the LAST '@' (skipping leading '@' for scoped pkgs)
            let name = if entry.starts_with('@') {
                // Scoped: find the last '@' after position 0
                if let Some(pos) = entry[1..].rfind('@') {
                    entry[..pos + 1].to_string()
                } else {
                    // No version separator — use entire entry as name
                    entry.to_string()
                }
            } else if let Some(pos) = entry.find('@') {
                entry[..pos].to_string()
            } else {
                entry.to_string()
            };
            if !name.is_empty() { current_name = Some(name); }
        } else if line.starts_with("version") {
            if let Some(ref name) = current_name {
                let version = line.trim_start_matches("version").trim().trim_matches('"').to_string();
                if !version.is_empty() {
                    out.push(LockedPackage { name: name.clone(), version, ecosystem: "npm".to_string(), source: path.to_string() });
                }
            }
        }
    }
    out
}

fn parse_pnpm_lock(path: &str) -> Vec<LockedPackage> {
    let mut out = Vec::new();
    let Ok(content) = fs::read_to_string(path) else { return out; };
    // YAML: look for `    /name@version:` patterns under `packages:`
    let mut in_packages = false;
    for line in content.lines() {
        if line.starts_with("packages:") { in_packages = true; continue; }
        if in_packages && !line.starts_with(' ') { in_packages = false; }
        if in_packages && line.trim().ends_with(':') {
            let entry = line.trim().trim_end_matches(':').trim_start_matches('/');
            if let Some(at_pos) = entry.rfind('@') {
                let name = &entry[..at_pos];
                let version = &entry[at_pos+1..];
                if !name.is_empty() && !version.is_empty() {
                    out.push(LockedPackage { name: name.to_string(), version: version.to_string(), ecosystem: "npm".to_string(), source: path.to_string() });
                }
            }
        }
    }
    out
}

fn parse_requirements_txt(path: &str) -> Vec<LockedPackage> {
    let mut out = Vec::new();
    let Ok(content) = fs::read_to_string(path) else { return out; };
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with('-') { continue; }
        if let Some((name, version)) = line.split_once("==") {
            out.push(LockedPackage { name: name.trim().to_string(), version: version.trim().to_string(), ecosystem: "PyPI".to_string(), source: path.to_string() });
        }
    }
    out
}

fn parse_pyproject_toml(path: &str) -> Vec<LockedPackage> {
    let mut out = Vec::new();
    let Ok(content) = fs::read_to_string(path) else { return out; };
    let src = path.to_string();
    let lines: Vec<&str> = content.lines().collect();

    // Track which TOML section we're in
    let mut current_section = String::new();
    // Track whether we're inside a multi-line array (dependencies = [...])
    let mut in_array = false;

    for line in &lines {
        let trimmed = line.trim();

        // Track section headers like [project], [tool.poetry.dependencies], etc.
        if trimmed.starts_with('[') && !trimmed.starts_with("[[") {
            let section = trimmed.trim_start_matches('[').trim_end_matches(']').trim();
            current_section = section.to_string();
            in_array = false;
            continue;
        }

        // ── PEP 621: dependencies = ["pkg>=ver", ...] under [project] ──
        if current_section == "project" && !in_array {
            if let Some(rest) = trimmed.strip_prefix("dependencies") {
                let rest = rest.trim();
                if let Some(rest) = rest.strip_prefix('=') {
                    let rest = rest.trim();
                    if rest.starts_with('[') {
                        // Could be single-line or start of multi-line array
                        let array_content = if rest.contains(']') {
                            // Single-line: dependencies = ["a>=1", "b>=2"]
                            rest.trim_start_matches('[').trim_end_matches(']').to_string()
                        } else {
                            // Start of multi-line array
                            in_array = true;
                            rest.trim_start_matches('[').to_string()
                        };
                        for dep in parse_pep621_dep_list(&array_content) {
                            out.push(LockedPackage { name: dep.0, version: dep.1, ecosystem: "PyPI".to_string(), source: src.clone() });
                        }
                        continue;
                    }
                }
            }
        }

        // ── PEP 621: [project.optional-dependencies.X] arrays ──
        if current_section.starts_with("project.optional-dependencies") && !in_array {
            let rest = trimmed.trim();
            // Lines like: group = ["pkg>=ver", ...]
            if let Some((_key, val)) = rest.split_once('=') {
                let val = val.trim();
                if val.starts_with('[') {
                    let array_content = if val.contains(']') {
                        val.trim_start_matches('[').trim_end_matches(']').to_string()
                    } else {
                        in_array = true;
                        val.trim_start_matches('[').to_string()
                    };
                    for dep in parse_pep621_dep_list(&array_content) {
                        out.push(LockedPackage { name: dep.0, version: dep.1, ecosystem: "PyPI".to_string(), source: src.clone() });
                    }
                    continue;
                }
            }
        }

        // ── Continue collecting multi-line array entries ──
        if in_array {
            if trimmed.contains(']') {
                let part = trimmed.trim_end_matches(']').trim_end_matches(',');
                for dep in parse_pep621_dep_list(part) {
                    out.push(LockedPackage { name: dep.0, version: dep.1, ecosystem: "PyPI".to_string(), source: src.clone() });
                }
                in_array = false;
                continue;
            }
            for dep in parse_pep621_dep_list(trimmed) {
                out.push(LockedPackage { name: dep.0, version: dep.1, ecosystem: "PyPI".to_string(), source: src.clone() });
            }
            continue;
        }

        // ── Poetry: [tool.poetry.dependencies] key = "version" ──
        if current_section == "tool.poetry.dependencies" || current_section == "tool.poetry.dev-dependencies" {
            if trimmed.contains('=') && !trimmed.starts_with('#') {
                let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
                if parts.len() == 2 {
                    let name = parts[0].trim().to_string();
                    if name == "python" { continue; }
                    let raw = parts[1].trim().trim_matches('"').trim_matches('\'');
                    // Handle table syntax like {version = "^1.0", ...}
                    let ver_str = if raw.starts_with('{') {
                        raw.split("version")
                            .nth(1)
                            .and_then(|s| s.split('"').nth(1))
                            .unwrap_or("")
                    } else {
                        raw
                    };
                    let version = ver_str
                        .trim_start_matches(">=").trim_start_matches("==")
                        .trim_start_matches("~=").trim_start_matches('^')
                        .trim_start_matches('~')
                        .split(',').next().unwrap_or("").trim().to_string();
                    if !name.is_empty() && !version.is_empty() && version != "*" {
                        out.push(LockedPackage { name, version, ecosystem: "PyPI".to_string(), source: src.clone() });
                    }
                }
            }
        }
    }
    // Deduplicate by name (keep first occurrence)
    let mut seen = HashSet::new();
    out.retain(|p| seen.insert(p.name.clone()));
    out
}

/// Parse PEP 621 dependency strings from a comma-separated or newline-separated
/// list of quoted entries, e.g. `"requests>=2.28.0", "click>=8.0"`
fn parse_pep621_dep_list(input: &str) -> Vec<(String, String)> {
    let mut deps = Vec::new();
    // Split by comma, then process each quoted entry
    for entry in input.split(',') {
        let entry = entry.trim().trim_matches('"').trim_matches('\'').trim();
        if entry.is_empty() { continue; }
        // Strip extras like [security] → "requests[security]>=2.28" → "requests"
        let (name_part, rest) = if let Some(bracket) = entry.find('[') {
            let close = entry.find(']').unwrap_or(bracket + 1);
            (&entry[..bracket], &entry[close + 1..])
        } else {
            // Find where version constraint starts
            let delim = entry.find(|c: char| c == '>' || c == '<' || c == '=' || c == '~' || c == '!' || c == '^');
            match delim {
                Some(pos) => (&entry[..pos], &entry[pos..]),
                None => (entry, ""),
            }
        };
        let name = name_part.trim().to_string();
        let version = rest
            .trim_start_matches(">=").trim_start_matches("==")
            .trim_start_matches("~=").trim_start_matches("!=")
            .trim_start_matches('>').trim_start_matches('<')
            .trim_start_matches('^').trim_start_matches('~')
            .split(',').next().unwrap_or("").trim().to_string();
        if !name.is_empty() {
            deps.push((name, version));
        }
    }
    deps
}

fn parse_poetry_lock(path: &str) -> Vec<LockedPackage> {
    let mut out = Vec::new();
    let Ok(content) = fs::read_to_string(path) else { return out; };
    let mut name: Option<String> = None;
    let mut version: Option<String> = None;
    for line in content.lines() {
        let line = line.trim();
        if line == "[[package]]" {
            if let (Some(n), Some(v)) = (name.take(), version.take()) {
                out.push(LockedPackage { name: n, version: v, ecosystem: "PyPI".to_string(), source: path.to_string() });
            }
        } else if let Some(val) = line.strip_prefix("name = ") {
            name = Some(val.trim_matches('"').to_string());
        } else if let Some(val) = line.strip_prefix("version = ") {
            version = Some(val.trim_matches('"').to_string());
        }
    }
    if let (Some(n), Some(v)) = (name, version) {
        out.push(LockedPackage { name: n, version: v, ecosystem: "PyPI".to_string(), source: path.to_string() });
    }
    out
}

fn parse_uv_lock(path: &str) -> Vec<LockedPackage> {
    // uv.lock is TOML, similar structure to poetry.lock
    parse_poetry_lock(path) // reuse same [[package]] parser, same format
}

fn parse_cargo_lock(path: &str) -> Vec<LockedPackage> {
    let mut out = Vec::new();
    let Ok(content) = fs::read_to_string(path) else { return out; };
    let mut name: Option<String> = None;
    let mut version: Option<String> = None;
    for line in content.lines() {
        let line = line.trim();
        if line == "[[package]]" {
            if let (Some(n), Some(v)) = (name.take(), version.take()) {
                out.push(LockedPackage { name: n, version: v, ecosystem: "crates.io".to_string(), source: path.to_string() });
            }
        } else if let Some(val) = line.strip_prefix("name = ") {
            name = Some(val.trim_matches('"').to_string());
        } else if let Some(val) = line.strip_prefix("version = ") {
            version = Some(val.trim_matches('"').to_string());
        }
    }
    if let (Some(n), Some(v)) = (name, version) {
        out.push(LockedPackage { name: n, version: v, ecosystem: "crates.io".to_string(), source: path.to_string() });
    }
    out
}

fn parse_go_sum(path: &str) -> Vec<LockedPackage> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    let Ok(content) = fs::read_to_string(path) else { return out; };
    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let name = parts[0];
            let version = parts[1].trim_start_matches('v').split('/').next().unwrap_or(parts[1]);
            let key = format!("{name}@{version}");
            if seen.insert(key) {
                out.push(LockedPackage { name: name.to_string(), version: version.to_string(), ecosystem: "Go".to_string(), source: path.to_string() });
            }
        }
    }
    out
}

fn parse_go_mod(path: &str) -> Vec<LockedPackage> {
    let mut out = Vec::new();
    let Ok(content) = fs::read_to_string(path) else { return out; };
    let mut in_require = false;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("require (") || line == "require (" { in_require = true; continue; }
        if in_require && line == ")" { in_require = false; continue; }
        if in_require || line.starts_with("require ") {
            let entry = line.trim_start_matches("require").trim();
            let parts: Vec<&str> = entry.split_whitespace().collect();
            if parts.len() >= 2 {
                out.push(LockedPackage { name: parts[0].to_string(), version: parts[1].trim_start_matches('v').to_string(), ecosystem: "Go".to_string(), source: path.to_string() });
            }
        }
    }
    out
}

fn parse_gemfile_lock(path: &str) -> Vec<LockedPackage> {
    let mut out = Vec::new();
    let Ok(content) = fs::read_to_string(path) else { return out; };
    let mut in_gems = false;
    for line in content.lines() {
        if line.trim() == "GEM" || line.trim() == "specs:" { in_gems = true; continue; }
        if in_gems && (line.trim().is_empty() || (!line.starts_with("    ") && !line.starts_with("  "))) { in_gems = false; }
        if in_gems && line.starts_with("    ") && !line.starts_with("      ") {
            let entry = line.trim();
            if let Some(start) = entry.find('(') {
                let name = entry[..start].trim().to_string();
                let version = entry[start+1..].trim_end_matches(')').to_string();
                out.push(LockedPackage { name, version, ecosystem: "RubyGems".to_string(), source: path.to_string() });
            }
        }
    }
    out
}

fn parse_composer_lock(path: &str) -> Vec<LockedPackage> {
    let mut out = Vec::new();
    let Ok(content) = fs::read_to_string(path) else { return out; };
    let Ok(json): Result<Value, _> = serde_json::from_str(&content) else { return out; };
    for key in &["packages", "packages-dev"] {
        if let Some(pkgs) = json.get(key).and_then(|p| p.as_array()) {
            for pkg in pkgs {
                let name    = pkg.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string();
                let version = pkg.get("version").and_then(|v| v.as_str()).unwrap_or("").trim_start_matches('v').to_string();
                if !name.is_empty() {
                    out.push(LockedPackage { name, version, ecosystem: "Packagist".to_string(), source: path.to_string() });
                }
            }
        }
    }
    out
}

fn parse_nuget_lock(path: &str) -> Vec<LockedPackage> {
    let mut out = Vec::new();
    let Ok(content) = fs::read_to_string(path) else { return out; };
    let Ok(json): Result<Value, _> = serde_json::from_str(&content) else { return out; };
    if let Some(deps) = json.get("dependencies").and_then(|d| d.as_object()) {
        for (_framework, packages) in deps {
            if let Some(pkgs) = packages.as_object() {
                for (name, meta) in pkgs {
                    let version = meta.get("resolved").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    out.push(LockedPackage { name: name.clone(), version, ecosystem: "NuGet".to_string(), source: path.to_string() });
                }
            }
        }
    }
    out
}

fn parse_mix_lock(path: &str) -> Vec<LockedPackage> {
    let mut out = Vec::new();
    let Ok(content) = fs::read_to_string(path) else { return out; };
    // mix.lock format: `  "package": {:hex, :name, "version", ...}`
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('"') || line.starts_with(':') {
            if let Some(colon) = line.find(':') {
                let name = line[..colon].trim().trim_matches('"').to_string();
                // version is the 3rd quoted string in the line
                let parts: Vec<&str> = line.split('"').collect();
                if parts.len() >= 6 {
                    let version = parts[5].to_string();
                    if !name.is_empty() && !version.is_empty() {
                        out.push(LockedPackage { name, version, ecosystem: "Hex".to_string(), source: path.to_string() });
                    }
                }
            }
        }
    }
    out
}

fn parse_pubspec_lock(path: &str) -> Vec<LockedPackage> {
    let mut out = Vec::new();
    let Ok(content) = fs::read_to_string(path) else { return out; };
    let mut in_packages = false;
    let mut current_name: Option<String> = None;
    for line in content.lines() {
        if line.trim() == "packages:" { in_packages = true; continue; }
        if in_packages && !line.starts_with(' ') { in_packages = false; }
        if in_packages {
            // Package name line has exactly 2 spaces indent + name + ":"
            if line.starts_with("  ") && !line.starts_with("   ") && line.trim().ends_with(':') {
                current_name = Some(line.trim().trim_end_matches(':').to_string());
            }
            if let Some(ref name) = current_name {
                if line.trim().starts_with("version:") {
                    let version = line.trim().trim_start_matches("version:").trim().trim_matches('"').to_string();
                    out.push(LockedPackage { name: name.clone(), version, ecosystem: "pub.dev".to_string(), source: path.to_string() });
                }
            }
        }
    }
    out
}

/// Detect which lock/manifest files exist in the current directory.
/// Returns a list of (file_path, ecosystem_label) for each found file.
pub fn detect_lock_files() -> Vec<(&'static str, &'static str)> {
    let candidates: Vec<(&str, &str)> = vec![
        ("package-lock.json", "npm"),
        ("yarn.lock",         "yarn"),
        ("pnpm-lock.yaml",   "pnpm"),
        ("bun.lockb",        "bun"),
        ("package.json",     "npm (manifest)"),
        ("requirements.txt", "pip"),
        ("pyproject.toml",   "pip/uv/poetry"),
        ("poetry.lock",      "poetry"),
        ("uv.lock",          "uv"),
        ("Cargo.lock",       "cargo"),
        ("go.sum",           "go"),
        ("go.mod",           "go (manifest)"),
        ("Gemfile.lock",     "gem"),
        ("composer.lock",    "composer"),
        ("packages.lock.json", "nuget"),
        ("mix.lock",         "hex"),
        ("pubspec.lock",     "pub"),
    ];
    candidates.into_iter()
        .filter(|(path, _)| Path::new(path).exists())
        .collect()
}

/// Parse packages from a specific list of lock files (user-selected subset).
pub fn parse_selected_files(files: &[&str]) -> Vec<LockedPackage> {
    let mut packages = Vec::new();
    for file in files {
        packages.extend(parse_custom_file(file));
    }
    packages
}

/// Return human-readable list of all supported lock/manifest files.
pub fn supported_files() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("JavaScript", "npm",      "package-lock.json, package.json"),
        ("JavaScript", "yarn",     "yarn.lock"),
        ("JavaScript", "pnpm",     "pnpm-lock.yaml"),
        ("JavaScript", "bun",      "bun.lockb (package.json fallback)"),
        ("Python",     "pip",      "requirements.txt"),
        ("Python",     "poetry",   "poetry.lock, pyproject.toml"),
        ("Python",     "uv",       "uv.lock, pyproject.toml"),
        ("Rust",       "cargo",    "Cargo.lock"),
        ("Go",         "go mod",   "go.sum, go.mod"),
        ("Ruby",       "gem",      "Gemfile.lock"),
        ("PHP",        "composer", "composer.lock"),
        (".NET",       "nuget",    "packages.lock.json"),
        ("Elixir",     "hex/mix",  "mix.lock"),
        ("Dart",       "pub",      "pubspec.lock"),
    ]
}
