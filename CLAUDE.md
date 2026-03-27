# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Infynon CLI is a Rust-based dual-mode security tool:

1. **Network Firewall** (`infynon`): A real-time reverse proxy WAF with TUI dashboard. Sits between the internet and your backend, inspecting and filtering HTTP traffic through a multi-stage pipeline (IP filter → rate limiter → WAF → custom rules).
2. **Package Security Manager** (`infynon pkg`): Intercepts package install commands across 14 ecosystems (npm, yarn, pnpm, bun, pip, uv, poetry, cargo, go, gem, composer, nuget, hex, pub) and runs a 3-layer CVE verification pipeline before allowing installation.

## Build & Run Commands

```bash
cargo build                          # Debug build
cargo build --release                # Release build (LTO, stripped, opt-level z)

# Firewall mode
cargo run -- init                    # Create default infynon.toml
cargo run -- start                   # Start firewall + TUI
cargo run -- start --headless        # Start firewall without TUI
cargo run -- status                  # Show firewall config status
cargo run -- monitor                 # Open TUI monitor

# Package manager mode
cargo run -- pkg <args>              # Run in package manager mode
cargo run -- pkg scan                # Scan lock files for CVEs
```

There are no tests or linting commands configured yet. The project uses default `rustfmt` and has `#![allow(dead_code, unused_variables, unused_imports)]` in main.rs during beta.

## Architecture

### Dual-Mode Binary

The single `infynon` binary operates in two modes, determined at startup in `main.rs`:

- **Firewall mode** (`infynon ...`): Reverse proxy WAF with TUI dashboard. Entry: `cli::run_firewall()`.
- **Package Manager mode** (`infynon pkg ...` or symlinked as `infynon-pkg`): Intercepts install commands, scans packages against OSV.dev before installation, and proxies to native package managers. Entry: `cli::run_package_manager()`.

### Module Layout

- **`firewall/`** — The network firewall engine (NEW in v0.2.0):
  - `config.rs` — TOML-based configuration with full schema (`FirewallConfig`), load/save/init, cross-platform config paths (`~/.infynon/infynon.toml`)
  - `server.rs` — Tokio + Hyper 1.x reverse proxy server with `SharedState` (Arc-shared between proxy, TUI, and cleanup tasks). Handles request forwarding, proxy headers (X-Forwarded-For, X-Real-IP, X-Request-ID), block/rate-limit responses
  - `pipeline.rs` — 4-stage request evaluation pipeline: IP Filter → Rate Limiter → WAF → Custom Rules. Owns all stages, exposes `evaluate()`, `block_ip()`, `unblock_ip()`, `cleanup()`
  - `ip_filter.rs` — IP blocklist/allowlist with CIDR support (via `ipnet`), auto-reputation tracking (dynamic bans after threshold), runtime block/unblock
  - `rate_limiter.rs` — Sliding window rate limiter: per-IP, per-path, and global limits. Periodic cleanup of expired entries
  - `waf.rs` — Web Application Firewall with compiled `RegexSet` patterns for: SQLi (13 patterns), XSS (12 patterns), path traversal (10 patterns), command injection (4 patterns), header injection (3 patterns). Also enforces URL length, body size, HTTP method, blocked extensions/paths, User-Agent rules
  - `rules.rs` — Custom rule engine: `CompiledRule` with `CompiledCondition` (IP match, path prefix/exact/regex, method, header, user-agent, body, content-type, request size) and `RuleAction` (Block, Allow, Flag, RateLimit). Rules compiled from TOML config, sorted by priority, AND logic for conditions
  - `events.rs` — `FirewallEvent` struct (request metadata + verdict + timing) and `Verdict` enum (Allow, Block, RateLimited, Flagged)
  - `stats.rs` — Rolling statistics with ring buffers (60s traffic/blocks sparklines), atomic counters, top-N trackers (IPs, paths, rules), `StatsSnapshot` for TUI consumption
  - `logger.rs` — Async JSONL file logger via tokio channel, writes to access.jsonl and blocked.jsonl
- **`tui/`** — Terminal UI:
  - `firewall_app.rs` — TUI app state machine with 7 views (Dashboard, LiveFeed, Blocked, IpInspector, Rules, Stats, Config), keyboard handling, feed filtering, IP search, config editing
  - `views.rs` — All ratatui rendering: tab bar, status line, dashboard (sparklines, tables), live feed, blocked requests, IP inspector (search + block/unblock), rules, stats, config editor
  - `theme.rs` — INFYNON color palette (Cyan/Red/Green/Yellow/Orange/Purple), verdict colors, common styles
  - `logger.rs` — Styled console logging (titles, steps, success/error, splash screens)
  - `loaders.rs` — Installation spinners
  - `dashboard.rs` — Legacy dashboard stub
- **`cli/`** — Clap-based argument parsing (`args.rs`), command routing (`commands.rs`), scan logic (`scan.rs`), and feature commands (`features/` folder). Firewall commands: init, start, monitor, status, block, unblock, rules, logs, config
- **`ecosystems/`** — `RegistryAdapter` trait in `adapter.rs`, `detector.rs` for binary availability, per-ecosystem: `npm.rs`, `pypi.rs`, `cargo.rs`
- **`engine/`** — Package security pipeline: `layer1_blocklist.rs`, `layer2_static.rs`, `layer3_llm.rs`, `pipeline.rs`, `osv.rs` (OSV.dev API), `scanner.rs` (15+ lock file parsers), `reporter.rs` (Markdown/PDF), `registry.rs` (9 ecosystem registries)
- **`config/`** — Legacy package manager configuration
- **`daemon/`** — Background intelligence aggregation stubs
- **`models/`** — `Package` struct, `Verdict` enum (for pkg mode)
- **`error/`** — `InfynonError` enum via `thiserror`

### Key Patterns

- **Firewall SharedState**: `Arc<SharedState>` is shared between the proxy server (tokio tasks), TUI (main thread), and cleanup task. Contains pipeline, stats, config, event channel, and recent events ring buffer
- **Firewall TUI integration**: Proxy runs on tokio runtime in background thread, TUI runs on main thread via crossterm/ratatui, communicates via shared state. When TUI quits, shutdown signal stops the proxy
- **TOML config with defaults**: `FirewallConfig` uses serde defaults extensively — every field has a sensible default. Config loadable from `./infynon.toml`, `~/.infynon/infynon.toml`, or explicit path
- **Config editable from TUI and file**: The Config view (key 7) shows all settings, allows inline editing. Changes saved to disk apply on next restart
- **Cross-platform**: Uses `cfg(windows)` / `cfg(not(windows))` for home directory resolution. All I/O uses cross-platform APIs. No Unix-specific features
- **Ecosystem auto-detection**: Falls back through manifest file checks (package.json → Cargo.toml → pyproject.toml → go.mod → etc.) in `commands.rs`
- **Interactive vulnerability decisions**: Per-package prompts to install anyway, skip, or install a fixed version
- **Features folder**: `src/cli/features/` has one file per command: `audit.rs`, `why_cmd.rs`, `outdated.rs`, `diff.rs`, `doctor.rs`, `size.rs`, `search.rs`, `fix.rs`, `clean.rs`, `migrate.rs`

### Firewall CLI Commands

```
infynon init [--port N] [--upstream HOST] [--upstream-port N]   Create config
infynon start [--config FILE] [--port N] [--headless]           Start proxy + TUI
infynon monitor [--config FILE]                                  TUI only
infynon status [--config FILE]                                   Show config
infynon block IP                                                 Block an IP
infynon unblock IP                                               Unblock an IP
infynon rules list|enable|disable                                Manage rules
infynon logs [--verdict V] [--ip IP] [--count N]                View logs
infynon config check|show                                        Validate/display config
```

## CI/CD

GitHub Actions (`release.yml`) builds on tag push (`v*`) for 5 targets: Windows x64, Linux x64/ARM64 (musl static), macOS x64/ARM64. Uses `softprops/action-gh-release` for asset upload.

## Dependencies

Key crates:
- **CLI**: `clap` (argument parsing), `dialoguer` (interactive prompts), `owo-colors` (terminal colors), `indicatif` (progress bars), `tabled` (tables)
- **TUI**: `ratatui` (terminal UI framework), `crossterm` (terminal control)
- **HTTP**: `reqwest` (blocking client for pkg mode), `hyper` + `hyper-util` + `http-body-util` (async server/proxy for firewall), `tokio` (async runtime)
- **Data**: `serde` + `serde_json` (serialization), `toml` (config), `chrono` (timestamps), `regex` (WAF patterns), `ipnet` (CIDR), `bytes` (buffers)
- **Output**: `printpdf` (PDF export)
- **Error**: `thiserror`
