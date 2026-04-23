fn parse_cargo_lock(path: &str) -> Vec<LockedPackage> {
    let mut out = Vec::new();
    let Ok(content) = fs::read_to_string(path) else {
        return out;
    };
    let mut name: Option<String> = None;
    let mut version: Option<String> = None;
    for line in content.lines() {
        let line = line.trim();
        if line == "[[package]]" {
            if let (Some(n), Some(v)) = (name.take(), version.take()) {
                out.push(LockedPackage {
                    name: n,
                    version: v,
                    ecosystem: "crates.io".to_string(),
                    source: path.to_string(),
                });
            }
        } else if let Some(val) = line.strip_prefix("name = ") {
            name = Some(val.trim_matches('"').to_string());
        } else if let Some(val) = line.strip_prefix("version = ") {
            version = Some(val.trim_matches('"').to_string());
        }
    }
    if let (Some(n), Some(v)) = (name, version) {
        out.push(LockedPackage {
            name: n,
            version: v,
            ecosystem: "crates.io".to_string(),
            source: path.to_string(),
        });
    }
    out
}

fn parse_go_sum(path: &str) -> Vec<LockedPackage> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    let Ok(content) = fs::read_to_string(path) else {
        return out;
    };
    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let name = parts[0];
            let version = parts[1]
                .trim_start_matches('v')
                .split('/')
                .next()
                .unwrap_or(parts[1]);
            let key = format!("{name}@{version}");
            if seen.insert(key) {
                out.push(LockedPackage {
                    name: name.to_string(),
                    version: version.to_string(),
                    ecosystem: "Go".to_string(),
                    source: path.to_string(),
                });
            }
        }
    }
    out
}

fn parse_go_mod(path: &str) -> Vec<LockedPackage> {
    let mut out = Vec::new();
    let Ok(content) = fs::read_to_string(path) else {
        return out;
    };
    let mut in_require = false;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("require (") || line == "require (" {
            in_require = true;
            continue;
        }
        if in_require && line == ")" {
            in_require = false;
            continue;
        }
        if in_require || line.starts_with("require ") {
            let entry = line.trim_start_matches("require").trim();
            let parts: Vec<&str> = entry.split_whitespace().collect();
            if parts.len() >= 2 {
                out.push(LockedPackage {
                    name: parts[0].to_string(),
                    version: parts[1].trim_start_matches('v').to_string(),
                    ecosystem: "Go".to_string(),
                    source: path.to_string(),
                });
            }
        }
    }
    out
}

fn parse_gemfile_lock(path: &str) -> Vec<LockedPackage> {
    let mut out = Vec::new();
    let Ok(content) = fs::read_to_string(path) else {
        return out;
    };
    let mut in_gems = false;
    for line in content.lines() {
        if line.trim() == "GEM" || line.trim() == "specs:" {
            in_gems = true;
            continue;
        }
        if in_gems
            && (line.trim().is_empty() || (!line.starts_with("    ") && !line.starts_with("  ")))
        {
            in_gems = false;
        }
        if in_gems && line.starts_with("    ") && !line.starts_with("      ") {
            let entry = line.trim();
            if let Some(start) = entry.find('(') {
                let name = entry[..start].trim().to_string();
                let version = entry[start + 1..].trim_end_matches(')').to_string();
                out.push(LockedPackage {
                    name,
                    version,
                    ecosystem: "RubyGems".to_string(),
                    source: path.to_string(),
                });
            }
        }
    }
    out
}

fn parse_composer_lock(path: &str) -> Vec<LockedPackage> {
    let mut out = Vec::new();
    let Ok(content) = fs::read_to_string(path) else {
        return out;
    };
    let Ok(json): Result<Value, _> = serde_json::from_str(&content) else {
        return out;
    };
    for key in &["packages", "packages-dev"] {
        if let Some(pkgs) = json.get(key).and_then(|p| p.as_array()) {
            for pkg in pkgs {
                let name = pkg
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("")
                    .to_string();
                let version = pkg
                    .get("version")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .trim_start_matches('v')
                    .to_string();
                if !name.is_empty() {
                    out.push(LockedPackage {
                        name,
                        version,
                        ecosystem: "Packagist".to_string(),
                        source: path.to_string(),
                    });
                }
            }
        }
    }
    out
}

fn parse_nuget_lock(path: &str) -> Vec<LockedPackage> {
    let mut out = Vec::new();
    let Ok(content) = fs::read_to_string(path) else {
        return out;
    };
    let Ok(json): Result<Value, _> = serde_json::from_str(&content) else {
        return out;
    };
    if let Some(deps) = json.get("dependencies").and_then(|d| d.as_object()) {
        for (_framework, packages) in deps {
            if let Some(pkgs) = packages.as_object() {
                for (name, meta) in pkgs {
                    let version = meta
                        .get("resolved")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    out.push(LockedPackage {
                        name: name.clone(),
                        version,
                        ecosystem: "NuGet".to_string(),
                        source: path.to_string(),
                    });
                }
            }
        }
    }
    out
}

fn parse_mix_lock(path: &str) -> Vec<LockedPackage> {
    let mut out = Vec::new();
    let Ok(content) = fs::read_to_string(path) else {
        return out;
    };
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
                        out.push(LockedPackage {
                            name,
                            version,
                            ecosystem: "Hex".to_string(),
                            source: path.to_string(),
                        });
                    }
                }
            }
        }
    }
    out
}

fn parse_pubspec_lock(path: &str) -> Vec<LockedPackage> {
    let mut out = Vec::new();
    let Ok(content) = fs::read_to_string(path) else {
        return out;
    };
    let mut in_packages = false;
    let mut current_name: Option<String> = None;
    for line in content.lines() {
        if line.trim() == "packages:" {
            in_packages = true;
            continue;
        }
        if in_packages && !line.starts_with(' ') {
            in_packages = false;
        }
        if in_packages {
            // Package name line has exactly 2 spaces indent + name + ":"
            if line.starts_with("  ") && !line.starts_with("   ") && line.trim().ends_with(':') {
                current_name = Some(line.trim().trim_end_matches(':').to_string());
            }
            if let Some(ref name) = current_name {
                if line.trim().starts_with("version:") {
                    let version = line
                        .trim()
                        .trim_start_matches("version:")
                        .trim()
                        .trim_matches('"')
                        .to_string();
                    out.push(LockedPackage {
                        name: name.clone(),
                        version,
                        ecosystem: "pub.dev".to_string(),
                        source: path.to_string(),
                    });
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
        ("yarn.lock", "yarn"),
        ("pnpm-lock.yaml", "pnpm"),
        ("bun.lockb", "bun"),
        ("package.json", "npm (manifest)"),
        ("requirements.txt", "pip"),
        ("pyproject.toml", "pip/uv/poetry"),
        ("poetry.lock", "poetry"),
        ("uv.lock", "uv"),
        ("Cargo.lock", "cargo"),
        ("go.sum", "go"),
        ("go.mod", "go (manifest)"),
        ("Gemfile.lock", "gem"),
        ("composer.lock", "composer"),
        ("packages.lock.json", "nuget"),
        ("mix.lock", "hex"),
        ("pubspec.lock", "pub"),
    ];
    candidates
        .into_iter()
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
        ("JavaScript", "npm", "package-lock.json, package.json"),
        ("JavaScript", "yarn", "yarn.lock"),
        ("JavaScript", "pnpm", "pnpm-lock.yaml"),
        ("JavaScript", "bun", "bun.lockb (package.json fallback)"),
        ("Python", "pip", "requirements.txt"),
        ("Python", "poetry", "poetry.lock, pyproject.toml"),
        ("Python", "uv", "uv.lock, pyproject.toml"),
        ("Rust", "cargo", "Cargo.lock"),
        ("Go", "go mod", "go.sum, go.mod"),
        ("Ruby", "gem", "Gemfile.lock"),
        ("PHP", "composer", "composer.lock"),
        (".NET", "nuget", "packages.lock.json"),
        ("Elixir", "hex/mix", "mix.lock"),
        ("Dart", "pub", "pubspec.lock"),
    ]
}
