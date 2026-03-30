/// Fetch the latest stable version of a package from its ecosystem registry.
/// Returns `None` on any error (fail-open — don't block installs on lookup failure).
///
/// Supported:
///   npm       → registry.npmjs.org
///   PyPI      → pypi.org
///   crates.io → crates.io
///   Go        → proxy.golang.org
///   RubyGems  → rubygems.org
///   Packagist → repo.packagist.org
///   NuGet     → api.nuget.org
///   Hex       → hex.pm
///   pub.dev   → pub.dev

use reqwest::blocking::Client;
use serde::Deserialize;
use std::time::Duration;

fn client() -> Client {
    let ua = format!("infynon/{} (https://github.com/d4rkNinja/infynon-cli)", env!("CARGO_PKG_VERSION"));
    Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent(ua)
        .build()
        .unwrap_or_default()
}

/// Resolve the latest version for a package in the given OSV ecosystem string.
/// `ecosystem` matches the OSV ecosystem field (e.g. "npm", "PyPI", "crates.io").
pub fn fetch_latest_version(name: &str, ecosystem: &str) -> Option<String> {
    match ecosystem {
        // ── npm family ────────────────────────────────────────────────────────
        // yarn, pnpm, bun all publish to and resolve from the npm registry
        "npm" | "yarn" | "pnpm" | "bun" => npm_latest(name),

        // ── Python family ─────────────────────────────────────────────────────
        // pip, uv, poetry all install from PyPI
        "PyPI" | "pip" | "pip3" | "uv" | "poetry" => pypi_latest(name),

        // ── Rust ──────────────────────────────────────────────────────────────
        "crates.io" => crates_latest(name),

        // ── Go ────────────────────────────────────────────────────────────────
        "Go" => go_latest(name),

        // ── Ruby ──────────────────────────────────────────────────────────────
        "RubyGems" | "gem" => rubygems_latest(name),

        // ── PHP ───────────────────────────────────────────────────────────────
        "Packagist" | "composer" => packagist_latest(name),

        // ── .NET ──────────────────────────────────────────────────────────────
        "NuGet" | "nuget" | "dotnet" => nuget_latest(name),

        // ── Elixir ───────────────────────────────────────────────────────────
        "Hex" | "hex" | "mix" => hex_latest(name),

        // ── Dart / Flutter ───────────────────────────────────────────────────
        "pub.dev" | "pub" | "dart" => pubdev_latest(name),

        // Unknown ecosystem — cannot resolve latest
        _ => None,
    }
}

// ── npm ────────────────────────────────────────────────────────────────────────
// GET https://registry.npmjs.org/<package>/latest
// { "version": "4.19.2", ... }

#[derive(Deserialize)]
struct NpmLatest { version: String }

fn npm_latest(name: &str) -> Option<String> {
    let url = format!("https://registry.npmjs.org/{}/latest", urlenc(name));
    let r: NpmLatest = client().get(&url).send().ok()?.json().ok()?;
    Some(r.version)
}

// ── PyPI ───────────────────────────────────────────────────────────────────────
// GET https://pypi.org/pypi/<package>/json
// { "info": { "version": "2.31.0" } }

#[derive(Deserialize)]
struct PypiRoot { info: PypiInfo }
#[derive(Deserialize)]
struct PypiInfo { version: String }

fn pypi_latest(name: &str) -> Option<String> {
    let url = format!("https://pypi.org/pypi/{}/json", name);
    let r: PypiRoot = client().get(&url).send().ok()?.json().ok()?;
    Some(r.info.version)
}

// ── crates.io ─────────────────────────────────────────────────────────────────
// GET https://crates.io/api/v1/crates/<crate>
// { "crate": { "newest_version": "1.0.196" } }

#[derive(Deserialize)]
struct CratesRoot { #[serde(rename = "crate")] krate: CrateInfo }
#[derive(Deserialize)]
struct CrateInfo { newest_version: String }

fn crates_latest(name: &str) -> Option<String> {
    let url = format!("https://crates.io/api/v1/crates/{}", name);
    let r: CratesRoot = client().get(&url).send().ok()?.json().ok()?;
    Some(r.krate.newest_version)
}

// ── Go proxy ──────────────────────────────────────────────────────────────────
// GET https://proxy.golang.org/<module>/@latest
// { "Version": "v0.25.0", ... }

#[derive(Deserialize)]
struct GoLatest { #[serde(rename = "Version")] version: String }

fn go_latest(name: &str) -> Option<String> {
    // module paths may contain capital letters → use escaped path
    let url = format!("https://proxy.golang.org/{}/@latest", name.to_lowercase());
    let r: GoLatest = client().get(&url).send().ok()?.json().ok()?;
    Some(r.version)
}

// ── RubyGems ──────────────────────────────────────────────────────────────────
// GET https://rubygems.org/api/v1/gems/<gem>.json
// { "version": "7.1.2" }

#[derive(Deserialize)]
struct GemInfo { version: String }

fn rubygems_latest(name: &str) -> Option<String> {
    let url = format!("https://rubygems.org/api/v1/gems/{}.json", name);
    let r: GemInfo = client().get(&url).send().ok()?.json().ok()?;
    Some(r.version)
}

// ── Packagist (Composer) ──────────────────────────────────────────────────────
// GET https://repo.packagist.org/p2/<vendor>/<package>.json
// { "packages": { "<vendor>/<package>": [ { "version": "v10.48.0", ... }, ... ] } }

#[derive(Deserialize)]
struct PackagistRoot {
    packages: std::collections::HashMap<String, Vec<PackagistVersion>>,
}
#[derive(Deserialize)]
struct PackagistVersion { version: String }

fn packagist_latest(name: &str) -> Option<String> {
    // name must be vendor/package format
    let (vendor, pkg) = name.split_once('/')?;
    let url = format!("https://repo.packagist.org/p2/{}/{}.json", vendor, pkg);
    let r: PackagistRoot = client().get(&url).send().ok()?.json().ok()?;
    let versions = r.packages.values().next()?;
    // First entry is latest stable
    let ver = versions.iter()
        .map(|v| v.version.trim_start_matches('v').to_string())
        .find(|v| !v.contains("alpha") && !v.contains("beta") && !v.contains("RC"))?;
    Some(ver)
}

// ── NuGet ─────────────────────────────────────────────────────────────────────
// GET https://api.nuget.org/v3-flatcontainer/<id>/index.json  (lowercase)
// { "versions": ["1.0.0", ..., "8.0.1"] }

#[derive(Deserialize)]
struct NugetIndex { versions: Vec<String> }

fn nuget_latest(name: &str) -> Option<String> {
    let url = format!("https://api.nuget.org/v3-flatcontainer/{}/index.json", name.to_lowercase());
    let r: NugetIndex = client().get(&url).send().ok()?.json().ok()?;
    // Stable versions don't contain '-' (which marks pre-release)
    let stable: Vec<_> = r.versions.iter().filter(|v| !v.contains('-')).collect();
    stable.last().map(|s| s.to_string())
}

// ── Hex (Elixir) ──────────────────────────────────────────────────────────────
// GET https://hex.pm/api/packages/<package>
// { "releases": [{"version": "2.1.0"}, ...], "latest_stable_version": "2.1.0" }

#[derive(Deserialize)]
struct HexPackage { latest_stable_version: Option<String> }

fn hex_latest(name: &str) -> Option<String> {
    let url = format!("https://hex.pm/api/packages/{}", name);
    let r: HexPackage = client().get(&url).send().ok()?.json().ok()?;
    r.latest_stable_version
}

// ── pub.dev (Dart/Flutter) ────────────────────────────────────────────────────
// GET https://pub.dev/api/packages/<package>
// { "latest": { "version": "1.4.0" } }

#[derive(Deserialize)]
struct PubPackage { latest: PubVersion }
#[derive(Deserialize)]
struct PubVersion { version: String }

fn pubdev_latest(name: &str) -> Option<String> {
    let url = format!("https://pub.dev/api/packages/{}", name);
    let r: PubPackage = client().get(&url).send().ok()?.json().ok()?;
    Some(r.latest.version)
}

// ── URL encode ────────────────────────────────────────────────────────────────
// Minimal percent-encoder for package names (handles scoped @scope/pkg)
pub fn urlenc(s: &str) -> String {
    s.replace('@', "%40").replace('/', "%2F")
}
