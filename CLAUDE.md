# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Infynon CLI is a Rust-based universal package security manager. It intercepts package install commands across 14 ecosystems (npm, yarn, pnpm, bun, pip, uv, poetry, cargo, go, gem, composer, nuget, hex, pub) and runs a 3-layer CVE verification pipeline before allowing installation. It also provides a standalone vulnerability scanner for lock/manifest files.

## Build & Run Commands

```bash
cargo build                          # Debug build
cargo build --release                # Release build (LTO, stripped, opt-level z)
cargo run -- pkg <args>              # Run in package manager mode
cargo run -- pkg scan                # Scan lock files for CVEs
cargo run                            # Run in firewall engine mode
```

There are no tests or linting commands configured yet. The project uses default `rustfmt` and has `#![allow(dead_code, unused_variables, unused_imports)]` in main.rs during beta.

## Architecture

### Dual-Mode Binary

The single `infynon` binary operates in two modes, determined at startup in `main.rs`:

- **Package Manager mode** (`infynon pkg ...` or symlinked as `infynon-pkg`): Intercepts install commands, scans packages against OSV.dev before installation, and proxies to native package managers. Entry: `cli::run_package_manager()`.
- **Firewall Engine mode** (`infynon ...`): Background daemon, TUI dashboard, and intelligence updates. Entry: `cli::run_firewall()`.

### Module Layout

- **`cli/`** — Clap-based argument parsing (`args.rs`), command routing (`commands.rs`), scan logic (`scan.rs`), and feature commands (`features/` folder — see below). The pkg mode uses trailing variadic args for passthrough to native package managers.
- **`ecosystems/`** — `RegistryAdapter` trait in `adapter.rs` provides a polymorphic interface. `detector.rs` handles binary availability checks and auto-detection via manifest file presence. Per-ecosystem implementations in `npm.rs`, `pypi.rs`, `cargo.rs`.
- **`engine/`** — The 3-layer security pipeline:
  - `layer1_blocklist.rs` — Fast in-memory blocklist check (<1ms)
  - `layer2_static.rs` — Static analysis heuristics (<50ms)
  - `layer3_llm.rs` — LLM-based code analysis (<8s)
  - `pipeline.rs` — Orchestrates the three layers
  - `osv.rs` — OSV.dev API integration for CVE lookups
  - `scanner.rs` — Parses 15+ lock/manifest formats (package-lock.json, yarn.lock, pnpm-lock.yaml, Cargo.lock, requirements.txt, pyproject.toml, poetry.lock, uv.lock, go.sum, Gemfile.lock, composer.lock, etc.)
  - `reporter.rs` — Markdown and PDF report generation
- **`tui/`** — Terminal UI: styled logging (`logger.rs`), installation spinners (`loaders.rs`), ratatui dashboard (`dashboard.rs`)
- **`config/`** — Configuration file loading and settings structure
- **`daemon/`** — Background intelligence aggregation and nightly update pipeline
- **`models/`** — `Package` struct (name, version, ecosystem)
- **`error/`** — `InfynonError` enum via `thiserror` (Blocked, System variants)

### Key Patterns

- **Ecosystem auto-detection**: Falls back through manifest file checks (package.json → Cargo.toml → pyproject.toml → go.mod → etc.) in `commands.rs`
- **Interactive vulnerability decisions**: When vulnerabilities are found during install, users get per-package prompts to install anyway, skip, or install a fixed version (`ask_vuln_decisions`)
- **`--strict` flag**: Blocks installation if any CVE meets the specified severity threshold
- **Version spec formatting**: `format_spec_for_ecosystem()` handles ecosystem-specific syntax (npm: `@ver`, pip: `==ver`, gem/composer: `:ver`, nuget: `--version ver`)
- **Dynamic versioning**: Version strings use `env!("CARGO_PKG_VERSION")` — update only in `Cargo.toml`
- **Cross-ecosystem registry APIs**: `registry.rs` queries npm, PyPI, crates.io, Go proxy, RubyGems, Packagist, NuGet, Hex, pub.dev. `features/` extends this with search, size, and diff APIs
- **Features folder**: `src/cli/features/` is a module folder with `mod.rs` (shared HTTP client via `OnceLock`, helpers: `detect_ecosystem`, `cargo_lock_deps`, `npm_declared_deps`, `format_bytes`, `spinner`, `bar`) and one file per command: `audit.rs`, `why_cmd.rs`, `outdated.rs`, `diff.rs`, `doctor.rs`, `size.rs`, `search.rs`, `fix.rs`, `clean.rs`, `migrate.rs`. All `cmd_*` functions are re-exported from `mod.rs`.

## CI/CD

GitHub Actions (`release.yml`) builds on tag push (`v*`) for 5 targets: Windows x64, Linux x64/ARM64 (musl static), macOS x64/ARM64. Uses `softprops/action-gh-release` for asset upload.

## Dependencies

Key crates: `clap` (CLI), `reqwest` (HTTP/blocking with rustls), `serde`/`serde_json` (serialization), `ratatui` (TUI), `indicatif` (progress bars), `owo-colors` (terminal colors), `tabled` (tables), `dialoguer` (interactive prompts), `printpdf` (PDF export), `thiserror` (error handling).
