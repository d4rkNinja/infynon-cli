fn parse_requirements_txt(path: &str) -> Vec<LockedPackage> {
    let mut out = Vec::new();
    let Ok(content) = fs::read_to_string(path) else {
        return out;
    };
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with('-') {
            continue;
        }
        if let Some((name, version)) = line.split_once("==") {
            if let Some(version) = exact_manifest_version(version) {
                out.push(LockedPackage {
                    name: name.trim().to_string(),
                    version,
                    ecosystem: "PyPI".to_string(),
                    source: path.to_string(),
                });
            }
        }
    }
    out
}

fn parse_pyproject_toml(path: &str) -> Vec<LockedPackage> {
    let mut out = Vec::new();
    let Ok(content) = fs::read_to_string(path) else {
        return out;
    };
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
                            rest.trim_start_matches('[')
                                .trim_end_matches(']')
                                .to_string()
                        } else {
                            // Start of multi-line array
                            in_array = true;
                            rest.trim_start_matches('[').to_string()
                        };
                        for dep in parse_pep621_dep_list(&array_content) {
                            out.push(LockedPackage {
                                name: dep.0,
                                version: dep.1,
                                ecosystem: "PyPI".to_string(),
                                source: src.clone(),
                            });
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
                        val.trim_start_matches('[')
                            .trim_end_matches(']')
                            .to_string()
                    } else {
                        in_array = true;
                        val.trim_start_matches('[').to_string()
                    };
                    for dep in parse_pep621_dep_list(&array_content) {
                        out.push(LockedPackage {
                            name: dep.0,
                            version: dep.1,
                            ecosystem: "PyPI".to_string(),
                            source: src.clone(),
                        });
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
                    out.push(LockedPackage {
                        name: dep.0,
                        version: dep.1,
                        ecosystem: "PyPI".to_string(),
                        source: src.clone(),
                    });
                }
                in_array = false;
                continue;
            }
            for dep in parse_pep621_dep_list(trimmed) {
                out.push(LockedPackage {
                    name: dep.0,
                    version: dep.1,
                    ecosystem: "PyPI".to_string(),
                    source: src.clone(),
                });
            }
            continue;
        }

        // ── Poetry: [tool.poetry.dependencies] key = "version" ──
        if (current_section == "tool.poetry.dependencies"
            || current_section == "tool.poetry.dev-dependencies")
            && trimmed.contains('=') && !trimmed.starts_with('#') {
                let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
                if parts.len() == 2 {
                    let name = parts[0].trim().to_string();
                    if name == "python" {
                        continue;
                    }
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
                    if let Some(version) = exact_manifest_version(ver_str) {
                        out.push(LockedPackage {
                            name,
                            version,
                            ecosystem: "PyPI".to_string(),
                            source: src.clone(),
                        });
                    }
                }
            }
    }
    // Deduplicate by name (keep first occurrence)
    let mut seen = HashSet::new();
    out.retain(|p| seen.insert(p.name.clone()));
    out
}

/// Strip leading version specifier characters and return only the version string.
/// Handles `>=`, `==`, `~=`, `!=`, `>`, `<`, `^`, `~` and takes the first segment before any comma.
/// Parse PEP 621 dependency strings from a comma-separated or newline-separated
/// list of quoted entries, e.g. `"requests>=2.28.0", "click>=8.0"`
fn parse_pep621_dep_list(input: &str) -> Vec<(String, String)> {
    let mut deps = Vec::new();
    // Split by comma, then process each quoted entry
    for entry in input.split(',') {
        let entry = entry.trim().trim_matches('"').trim_matches('\'').trim();
        if entry.is_empty() {
            continue;
        }
        // Strip extras like [security] → "requests[security]>=2.28" → "requests"
        let (name_part, rest) = if let Some(bracket) = entry.find('[') {
            let close = entry.find(']').unwrap_or(bracket + 1);
            (&entry[..bracket], &entry[close + 1..])
        } else {
            // Find where version constraint starts
            let delim = entry.find(|c: char| {
                c == '>' || c == '<' || c == '=' || c == '~' || c == '!' || c == '^'
            });
            match delim {
                Some(pos) => (&entry[..pos], &entry[pos..]),
                None => (entry, ""),
            }
        };
        let name = name_part.trim().to_string();
        if let Some(version) = exact_manifest_version(rest) {
            if !name.is_empty() {
                deps.push((name, version));
            }
        }
    }
    deps
}

fn parse_poetry_lock(path: &str) -> Vec<LockedPackage> {
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
                    ecosystem: "PyPI".to_string(),
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
            ecosystem: "PyPI".to_string(),
            source: path.to_string(),
        });
    }
    out
}

fn parse_uv_lock(path: &str) -> Vec<LockedPackage> {
    // uv.lock is TOML, similar structure to poetry.lock
    parse_poetry_lock(path) // reuse same [[package]] parser, same format
}
