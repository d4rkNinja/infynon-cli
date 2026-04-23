use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct LockedPackage {
    pub name: String,
    pub version: String,
    pub ecosystem: String,
    pub source: String,
}

pub fn detect_locked_packages(custom_file: Option<&str>) -> Vec<LockedPackage> {
    if let Some(path) = custom_file {
        return parse_custom_file(path);
    }

    let mut packages = Vec::new();
    if Path::new("package-lock.json").exists() {
        packages.extend(parse_npm_lock("package-lock.json"));
    }
    if Path::new("yarn.lock").exists() {
        packages.extend(parse_yarn_lock("yarn.lock"));
    }
    if Path::new("pnpm-lock.yaml").exists() {
        packages.extend(parse_pnpm_lock("pnpm-lock.yaml"));
    }
    if Path::new("bun.lockb").exists()
        || (Path::new("package.json").exists() && !Path::new("package-lock.json").exists())
    {
        packages.extend(parse_package_json("package.json"));
    }
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
    if Path::new("Cargo.lock").exists() {
        packages.extend(parse_cargo_lock("Cargo.lock"));
    }
    if Path::new("go.sum").exists() {
        packages.extend(parse_go_sum("go.sum"));
    }
    if Path::new("go.mod").exists() && !Path::new("go.sum").exists() {
        packages.extend(parse_go_mod("go.mod"));
    }
    if Path::new("Gemfile.lock").exists() {
        packages.extend(parse_gemfile_lock("Gemfile.lock"));
    }
    if Path::new("composer.lock").exists() {
        packages.extend(parse_composer_lock("composer.lock"));
    }
    for entry in ["packages.lock.json", "package.lock.json"] {
        if Path::new(entry).exists() {
            packages.extend(parse_nuget_lock(entry));
        }
    }
    if Path::new("mix.lock").exists() {
        packages.extend(parse_mix_lock("mix.lock"));
    }
    if Path::new("pubspec.lock").exists() {
        packages.extend(parse_pubspec_lock("pubspec.lock"));
    }
    packages
}

fn parse_custom_file(path: &str) -> Vec<LockedPackage> {
    let name = Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    match name {
        "package-lock.json" => parse_npm_lock(path),
        "yarn.lock" => parse_yarn_lock(path),
        "pnpm-lock.yaml" => parse_pnpm_lock(path),
        "package.json" => parse_package_json(path),
        "requirements.txt" => parse_requirements_txt(path),
        "pyproject.toml" => parse_pyproject_toml(path),
        "poetry.lock" => parse_poetry_lock(path),
        "uv.lock" => parse_uv_lock(path),
        "Cargo.lock" => parse_cargo_lock(path),
        "go.sum" => parse_go_sum(path),
        "go.mod" => parse_go_mod(path),
        "Gemfile.lock" => parse_gemfile_lock(path),
        "composer.lock" => parse_composer_lock(path),
        "packages.lock.json" => parse_nuget_lock(path),
        "mix.lock" => parse_mix_lock(path),
        "pubspec.lock" => parse_pubspec_lock(path),
        _ => {
            eprintln!("Unsupported file type: {}", path);
            vec![]
        }
    }
}

include!("scanner/javascript.rs");
include!("scanner/python.rs");
include!("scanner/other.rs");