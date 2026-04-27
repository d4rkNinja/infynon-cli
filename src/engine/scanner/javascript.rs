fn parse_npm_lock(path: &str) -> Vec<LockedPackage> {
    let mut out = Vec::new();
    let Ok(content) = fs::read_to_string(path) else {
        return out;
    };
    let Ok(json): Result<Value, _> = serde_json::from_str(&content) else {
        return out;
    };
    if let Some(packages) = json.get("packages").and_then(|p| p.as_object()) {
        for (key, val) in packages {
            let name = key.trim_start_matches("node_modules/");
            if name.is_empty() {
                continue;
            }
            if let Some(version) = val.get("version").and_then(|v| v.as_str()) {
                out.push(LockedPackage {
                    name: name.to_string(),
                    version: version.to_string(),
                    ecosystem: "npm".to_string(),
                    source: path.to_string(),
                });
            }
        }
    } else if let Some(deps) = json.get("dependencies").and_then(|d| d.as_object()) {
        for (name, val) in deps {
            if let Some(version) = val.get("version").and_then(|v| v.as_str()) {
                out.push(LockedPackage {
                    name: name.clone(),
                    version: version.to_string(),
                    ecosystem: "npm".to_string(),
                    source: path.to_string(),
                });
            }
        }
    }
    out
}

fn parse_package_json(path: &str) -> Vec<LockedPackage> {
    let mut out = Vec::new();
    let Ok(content) = fs::read_to_string(path) else {
        return out;
    };
    let Ok(json): Result<Value, _> = serde_json::from_str(&content) else {
        return out;
    };
    for key in &["dependencies", "devDependencies", "peerDependencies"] {
        if let Some(deps) = json.get(key).and_then(|d| d.as_object()) {
            for (name, version) in deps {
                if let Some(ver) = version.as_str().and_then(exact_manifest_version) {
                    out.push(LockedPackage {
                        name: name.clone(),
                        version: ver,
                        ecosystem: "npm".to_string(),
                        source: path.to_string(),
                    });
                }
            }
        }
    }
    out
}

fn parse_yarn_lock(path: &str) -> Vec<LockedPackage> {
    let mut out = Vec::new();
    let Ok(content) = fs::read_to_string(path) else {
        return out;
    };
    let mut current_name: Option<String> = None;
    for line in content.lines() {
        let line = line.trim();
        if line.ends_with(':') && !line.starts_with('#') && !line.starts_with("\"__metadata") {
            let entry = line.trim_end_matches(':').trim_matches('"');
            // Handle scoped packages: @scope/name@version
            // The name is everything up to the LAST '@' (skipping leading '@' for scoped pkgs)
            let name = if let Some(stripped) = entry.strip_prefix('@') {
                // Scoped: find the last '@' after position 0
                if let Some(pos) = stripped.rfind('@') {
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
            if !name.is_empty() {
                current_name = Some(name);
            }
        } else if line.starts_with("version") {
            if let Some(ref name) = current_name {
                let version = line
                    .trim_start_matches("version")
                    .trim()
                    .trim_matches('"')
                    .to_string();
                if !version.is_empty() {
                    out.push(LockedPackage {
                        name: name.clone(),
                        version,
                        ecosystem: "npm".to_string(),
                        source: path.to_string(),
                    });
                }
            }
        }
    }
    out
}

fn parse_pnpm_lock(path: &str) -> Vec<LockedPackage> {
    let mut out = Vec::new();
    let Ok(content) = fs::read_to_string(path) else {
        return out;
    };
    // YAML: look for `    /name@version:` patterns under `packages:`
    let mut in_packages = false;
    for line in content.lines() {
        if line.starts_with("packages:") {
            in_packages = true;
            continue;
        }
        if in_packages && !line.starts_with(' ') {
            in_packages = false;
        }
        if in_packages && line.trim().ends_with(':') {
            let entry = line.trim().trim_end_matches(':').trim_start_matches('/');
            let package_part = entry.split('(').next().unwrap_or(entry);
            let at_pos = if let Some(stripped) = package_part.strip_prefix('@') {
                stripped.rfind('@').map(|pos| pos + 1)
            } else {
                package_part.rfind('@')
            };
            if let Some(at_pos) = at_pos {
                let name = &package_part[..at_pos];
                let version = &package_part[at_pos + 1..];
                if !name.is_empty() && !version.is_empty() {
                    out.push(LockedPackage {
                        name: name.to_string(),
                        version: version.to_string(),
                        ecosystem: "npm".to_string(),
                        source: path.to_string(),
                    });
                }
            }
        }
    }
    out
}
