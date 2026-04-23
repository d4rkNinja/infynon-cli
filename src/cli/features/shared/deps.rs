use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

pub(crate) fn detect_ecosystem() -> &'static str {
    if Path::new("package.json").exists() || Path::new("package-lock.json").exists() {
        "npm"
    } else if Path::new("Cargo.toml").exists() {
        "crates.io"
    } else if Path::new("requirements.txt").exists() || Path::new("pyproject.toml").exists() {
        "PyPI"
    } else if Path::new("go.mod").exists() {
        "Go"
    } else if Path::new("Gemfile").exists() {
        "RubyGems"
    } else if Path::new("composer.json").exists() {
        "Packagist"
    } else if Path::new("pubspec.yaml").exists() {
        "pub.dev"
    } else if Path::new("mix.exs").exists() {
        "Hex"
    } else {
        "npm"
    }
}

pub(crate) fn cargo_root_name() -> Option<String> {
    fs::read_to_string("Cargo.toml").ok().and_then(|content| {
        content
            .lines()
            .find(|line| line.trim().starts_with("name"))
            .and_then(|line| line.split('=').nth(1))
            .map(|name| name.trim().trim_matches('"').to_string())
    })
}

pub(crate) fn cargo_lock_deps() -> HashMap<String, Vec<String>> {
    let mut deps = HashMap::new();
    let Ok(content) = fs::read_to_string("Cargo.lock") else {
        return deps;
    };
    let (mut current_name, mut in_deps) = (None, false);
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "[[package]]" {
            current_name = None;
            in_deps = false;
        } else if let Some(value) = trimmed.strip_prefix("name = ") {
            current_name = Some(value.trim_matches('"').to_string());
        } else if trimmed == "dependencies = [" {
            in_deps = true;
        } else if in_deps && trimmed == "]" {
            in_deps = false;
        } else if in_deps {
            if let Some(ref name) = current_name {
                let dep = trimmed
                    .trim_matches('"')
                    .trim_matches(',')
                    .trim_matches('"');
                let dep_name = dep
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .trim_matches('"');
                if !dep_name.is_empty() {
                    deps.entry(name.clone())
                        .or_default()
                        .push(dep_name.to_string());
                }
            }
        }
    }
    deps
}

pub(crate) fn npm_declared_deps() -> HashSet<String> {
    let mut declared = HashSet::new();
    if let Ok(content) = fs::read_to_string("package.json") {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            for key in &["dependencies", "devDependencies", "peerDependencies"] {
                if let Some(deps) = json.get(key).and_then(|value| value.as_object()) {
                    declared.extend(deps.keys().cloned());
                }
            }
        }
    }
    declared
}

pub(crate) fn cargo_toml_dep_names() -> Vec<String> {
    let mut names = Vec::new();
    let Ok(content) = fs::read_to_string("Cargo.toml") else {
        return names;
    };
    let mut in_deps = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "[dependencies]"
            || trimmed == "[dev-dependencies]"
            || trimmed == "[build-dependencies]"
        {
            in_deps = true;
        } else if trimmed.starts_with('[') {
            in_deps = false;
        } else if in_deps && trimmed.contains('=') {
            let name = trimmed.split('=').next().unwrap_or("").trim();
            if !name.is_empty() {
                names.push(name.to_string());
            }
        }
    }
    names
}

pub(crate) fn format_severity_bar(count: usize, total: usize) -> String {
    if total == 0 || count == 0 {
        String::new()
    } else {
        "█".repeat(((count * 20) / total).max(1))
    }
}
