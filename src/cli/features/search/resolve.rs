pub(super) fn resolve_search_ecosystems(
    ecosystem: Option<&str>,
) -> Result<Vec<&'static str>, String> {
    let Some(raw) = ecosystem else {
        return Ok(vec![
            "npm",
            "crates.io",
            "PyPI",
            "RubyGems",
            "Packagist",
            "pub.dev",
            "NuGet",
            "Hex",
            "Go",
        ]);
    };

    let normalized = raw.trim().to_ascii_lowercase();
    let canonical = match normalized.as_str() {
        "npm" | "yarn" | "pnpm" | "bun" => "npm",
        "cargo" | "crates.io" => "crates.io",
        "pip" | "pip3" | "pypi" | "uv" | "poetry" => "PyPI",
        "gem" | "rubygems" => "RubyGems",
        "composer" | "packagist" => "Packagist",
        "pub" | "dart" | "pub.dev" => "pub.dev",
        "nuget" | "dotnet" => "NuGet",
        "hex" | "mix" => "Hex",
        "go" | "golang" => "Go",
        other => return Err(format!("Search is not implemented for ecosystem '{}'.", other)),
    };

    Ok(vec![canonical])
}
