pub const DEFAULT_SEARCH_ECOSYSTEMS: &[&str] = &[
    "npm",
    "crates.io",
    "PyPI",
    "RubyGems",
    "Packagist",
    "pub.dev",
    "NuGet",
    "Hex",
    "Go",
];

pub const KNOWN_ECOSYSTEM_ALIASES: &[&str] = &[
    "npm",
    "yarn",
    "pnpm",
    "bun",
    "pip",
    "pip3",
    "pypi",
    "uv",
    "poetry",
    "cargo",
    "crates.io",
    "go",
    "golang",
    "gem",
    "rubygems",
    "composer",
    "packagist",
    "nuget",
    "dotnet",
    "hex",
    "mix",
    "pub",
    "pub.dev",
    "dart",
];

pub fn canonical_osv_ecosystem(ecosystem: &str) -> Option<&'static str> {
    match ecosystem.trim().to_ascii_lowercase().as_str() {
        "npm" | "yarn" | "pnpm" | "bun" => Some("npm"),
        "pip" | "pip3" | "pypi" | "uv" | "poetry" => Some("PyPI"),
        "cargo" | "crates.io" => Some("crates.io"),
        "go" | "golang" => Some("Go"),
        "gem" | "rubygems" => Some("RubyGems"),
        "composer" | "packagist" => Some("Packagist"),
        "nuget" | "dotnet" => Some("NuGet"),
        "hex" | "mix" => Some("Hex"),
        "pub" | "pub.dev" | "dart" => Some("Pub"),
        _ => None,
    }
}

pub fn canonical_search_ecosystem(ecosystem: &str) -> Option<&'static str> {
    match canonical_osv_ecosystem(ecosystem)? {
        "Pub" => Some("pub.dev"),
        other => Some(other),
    }
}

pub fn is_known_ecosystem(ecosystem: &str) -> bool {
    let normalized = ecosystem.trim().to_ascii_lowercase();
    KNOWN_ECOSYSTEM_ALIASES.contains(&normalized.as_str())
        || matches!(normalized.as_str(), "postgres" | "mysql" | "sqlite")
}
