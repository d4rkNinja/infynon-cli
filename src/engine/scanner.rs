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
            let name = entry.split('@').next().unwrap_or("").trim_matches('"').to_string();
            if !name.is_empty() { current_name = Some(name); }
        } else if line.starts_with("version") {
            if let Some(ref name) = current_name {
                let version = line.trim_start_matches("version").trim().trim_matches('"').to_string();
                out.push(LockedPackage { name: name.clone(), version, ecosystem: "npm".to_string(), source: path.to_string() });
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
    // Extract [project.dependencies] / [tool.poetry.dependencies] lines
    let mut in_deps = false;
    for line in content.lines() {
        let line = line.trim();
        if line == "[project.dependencies]" || line == "[tool.poetry.dependencies]" || line == "[tool.uv.dependencies]" {
            in_deps = true; continue;
        }
        if line.starts_with('[') { in_deps = false; continue; }
        if in_deps && line.contains('=') {
            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() == 2 {
                let name = parts[0].trim().to_string();
                let version = parts[1].trim().trim_matches('"').trim_matches('\'').trim_start_matches(">=").trim_start_matches("==").trim_start_matches('^').trim_start_matches('~').to_string();
                if !name.is_empty() && !version.is_empty() && version != "*" {
                    out.push(LockedPackage { name, version, ecosystem: "PyPI".to_string(), source: path.to_string() });
                }
            }
        }
    }
    out
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
