mod audit;
mod why_cmd;
mod outdated;
mod diff;
mod doctor;
mod size;
mod search;
mod fix;
mod clean;
mod migrate;

pub use audit::cmd_audit_deep;
pub use why_cmd::cmd_why;
pub use outdated::cmd_outdated;
pub use diff::cmd_diff;
pub use doctor::cmd_doctor;
pub use size::cmd_size;
pub use search::cmd_search;
pub use fix::cmd_fix_auto;
pub use clean::cmd_clean;
pub use migrate::cmd_migrate;

use crate::engine::{scanner, registry};
use crate::tui::logger::Logger;
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;
use std::time::Duration;
use std::fs;
use std::path::Path;

// ── Shared utilities ─────────────────────────────────────────────────────────

static HTTP_CLIENT: OnceLock<reqwest::blocking::Client> = OnceLock::new();

pub(crate) fn http_client() -> &'static reqwest::blocking::Client {
    HTTP_CLIENT.get_or_init(|| {
        let ua = format!("infynon/{} (https://github.com/d4rkNinja/infynon-cli)", env!("CARGO_PKG_VERSION"));
        reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent(ua)
            .build()
            .unwrap_or_default()
    })
}

pub(crate) fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

pub(crate) fn spinner() -> ProgressBar {
    let sp = ProgressBar::new_spinner();
    sp.set_style(
        ProgressStyle::with_template("  {spinner:.cyan}  {msg}")
            .unwrap()
            .tick_strings(&["⠋","⠙","⠹","⠸","⠼","⠴","⠦","⠧","⠇","⠏"]),
    );
    sp.enable_steady_tick(Duration::from_millis(60));
    sp
}

pub(crate) fn bar(len: u64) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_style(
        ProgressStyle::with_template(
            "  {spinner:.cyan}  {msg:<40} [{bar:40.cyan/blue}] {pos}/{len}"
        )
        .unwrap()
        .tick_strings(&["⠋","⠙","⠹","⠸","⠼","⠴","⠦","⠧","⠇","⠏"])
        .progress_chars("█▉▊▋▌▍▎▏  "),
    );
    pb.enable_steady_tick(Duration::from_millis(60));
    pb
}

pub(crate) fn detect_ecosystem() -> &'static str {
    if Path::new("package.json").exists() || Path::new("package-lock.json").exists() { "npm" }
    else if Path::new("Cargo.toml").exists() { "crates.io" }
    else if Path::new("requirements.txt").exists() || Path::new("pyproject.toml").exists() { "PyPI" }
    else if Path::new("go.mod").exists() { "Go" }
    else if Path::new("Gemfile").exists() { "RubyGems" }
    else if Path::new("composer.json").exists() { "Packagist" }
    else if Path::new("pubspec.yaml").exists() { "pub.dev" }
    else if Path::new("mix.exs").exists() { "Hex" }
    else { "npm" }
}

/// Extract root crate name from Cargo.toml
pub(crate) fn cargo_root_name() -> Option<String> {
    fs::read_to_string("Cargo.toml").ok()
        .and_then(|c| c.lines().find(|l| l.trim().starts_with("name"))
            .and_then(|l| l.split('=').nth(1))
            .map(|n| n.trim().trim_matches('"').to_string()))
}

/// Parse Cargo.lock into a map of package_name → list of dependency names
pub(crate) fn cargo_lock_deps() -> HashMap<String, Vec<String>> {
    let mut deps: HashMap<String, Vec<String>> = HashMap::new();
    let Ok(content) = fs::read_to_string("Cargo.lock") else { return deps; };
    let mut current_name: Option<String> = None;
    let mut in_deps = false;
    for line in content.lines() {
        let t = line.trim();
        if t == "[[package]]" { current_name = None; in_deps = false; }
        else if let Some(v) = t.strip_prefix("name = ") { current_name = Some(v.trim_matches('"').to_string()); }
        else if t == "dependencies = [" { in_deps = true; }
        else if in_deps && t == "]" { in_deps = false; }
        else if in_deps {
            if let Some(ref name) = current_name {
                let dep = t.trim_matches('"').trim_matches(',').trim_matches('"');
                let dep_name = dep.split_whitespace().next().unwrap_or("").trim_matches('"');
                if !dep_name.is_empty() {
                    deps.entry(name.clone()).or_default().push(dep_name.to_string());
                }
            }
        }
    }
    deps
}

/// Get declared dependency names from package.json (dependencies + devDependencies + peerDependencies)
pub(crate) fn npm_declared_deps() -> HashSet<String> {
    let mut declared = HashSet::new();
    if let Ok(c) = fs::read_to_string("package.json") {
        if let Ok(j) = serde_json::from_str::<serde_json::Value>(&c) {
            for key in &["dependencies", "devDependencies", "peerDependencies"] {
                if let Some(deps) = j.get(key).and_then(|d| d.as_object()) {
                    declared.extend(deps.keys().cloned());
                }
            }
        }
    }
    declared
}

/// Get Cargo.toml declared dependency names under [dependencies] and [dev-dependencies]
pub(crate) fn cargo_toml_dep_names() -> Vec<String> {
    let mut names = Vec::new();
    let Ok(c) = fs::read_to_string("Cargo.toml") else { return names; };
    let mut in_deps = false;
    for line in c.lines() {
        let t = line.trim();
        if t == "[dependencies]" || t == "[dev-dependencies]" || t == "[build-dependencies]" { in_deps = true; continue; }
        if t.starts_with('[') { in_deps = false; continue; }
        if in_deps && t.contains('=') {
            let name = t.split('=').next().unwrap_or("").trim();
            if !name.is_empty() {
                names.push(name.to_string());
            }
        }
    }
    names
}
