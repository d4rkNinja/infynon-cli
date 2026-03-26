# INFYNON — Universal Package Security Manager

<div align="center">
  <strong>A dual-personality security CLI that acts as both a vulnerability scanner and a secure package proxy for 14+ ecosystems.</strong>
</div>

---

## 🏗️ Repository Layout

```
infynon-cli/
├── src/
│   ├── main.rs              # Binary personality router (infynon vs infynon-pkg)
│   ├── cli/
│   │   ├── args.rs          # Clap v4 argument definitions
│   │   ├── commands.rs      # Command dispatch & install security gate
│   │   ├── scan.rs          # OSV scan orchestrator + install checker
│   │   └── mod.rs
│   ├── engine/
│   │   ├── osv.rs           # OSV.dev batch API client
│   │   ├── scanner.rs       # Lock-file / manifest parsers (14 ecosystems)
│   │   ├── reporter.rs      # Markdown & PDF report generator
│   │   └── mod.rs
│   ├── tui/
│   │   ├── logger.rs        # Centralized terminal styling (Logger::*)
│   │   ├── loaders.rs       # Animated install progress bars
│   │   └── dashboard.rs     # Ratatui dashboard stub
│   ├── ecosystems/
│   │   └── detector.rs      # Binary detection (where/which) + install hints
│   ├── daemon/
│   │   └── updater.rs       # Nightly intelligence pipeline stub
│   └── error/
│       └── types.rs         # InfynonError
├── AGENTS.md                # Architecture rules & agent directives
├── Cargo.toml
└── README.md
```

---

## ⚡ Two Tools in One Binary

The same binary morphs based on **how it is invoked**:

| Invocation | Personality | Purpose |
|---|---|---|
| `infynon` | Firewall / daemon | WAF intercept, threat intelligence, dashboard |
| `infynon-pkg` | Secure package proxy | Vulnerability-gated installs + full CVE scanner |

---

## 🛡️ `infynon-pkg` — Secure Package Proxy

### Install Commands

Every install is intercepted, checked against the OSV database, and only proceeds if clear (or with a countdown warning):

```bash
# Explicit ecosystem
infynon-pkg npm install express
infynon-pkg yarn add lodash@4.17.21
infynon-pkg pnpm add react@18.2.0
infynon-pkg bun add hono

infynon-pkg pip install requests==2.31.0
infynon-pkg uv install fastapi>=0.100.0
infynon-pkg poetry add django~=4.2.0

infynon-pkg cargo add serde@1.0
infynon-pkg go get golang.org/x/net@v0.25.0

infynon-pkg gem install rails:7.1.0
infynon-pkg composer require laravel/framework:^10.0
infynon-pkg nuget Microsoft.Extensions.Logging --version 8.0.0
infynon-pkg hex phoenix
infynon-pkg pub http

# Auto-detect ecosystem from project files
infynon-pkg install <package>

# CI/CD strict mode (block on ANY CVE hit)
infynon-pkg --strict npm install express
```

### Version Spec Parsing

`infynon-pkg` understands **every ecosystem's native spec format**:

| Ecosystem | Example input | Parsed |
|---|---|---|
| npm/yarn/pnpm/bun | `picomatch@4.0.3` | name=`picomatch`, ver=`4.0.3` |
| Scoped npm | `@types/node@20.0.0` | name=`@types/node`, ver=`20.0.0` |
| pip/uv/poetry | `requests==2.28.0` | name=`requests`, ver=`2.28.0` |
| pip range | `flask>=2.0,<3.0` | name=`flask`, ver=`2.0` |
| pip extras | `requests[security]==2.31.0` | name=`requests`, ver=`2.31.0` |
| cargo/go | `serde@1.0.190` | name=`serde`, ver=`1.0.190` |
| Go modules | `golang.org/x/net@v0.25.0` | name=`golang.org/x/net`, ver=`v0.25.0` |
| gem | `rails:7.1.0` | name=`rails`, ver=`7.1.0` |
| composer | `laravel/framework:^10.0` | name=`laravel/framework`, ver=`^10.0` |

### What Happens When a Vulnerability is Found

```
⚠ 'picomatch@4.0.3' has 2 known vulnerability(ies):
     INFORMATIONAL   GHSA-3v7f-55p6-f55p   Method Injection in POSIX Character Classes
          → safe version: 4.0.4   npm install picomatch@4.0.4
     INFORMATIONAL   GHSA-c2c7-rcm5-vvqj   ReDoS vulnerability via extglob quantifiers
          → safe version: 4.0.4   npm install picomatch@4.0.4

  💡 Safe alternatives:
     →  npm install picomatch@4.0.4

  ⏱ Proceeding in 5 seconds... Press Ctrl+C to abort
```

- **Normal mode**: 5-second countdown abort window, then proceeds
- **`--strict` mode**: Hard block — installation is refused until CVEs are resolved
- **CRITICAL/HIGH detected**: Extra ⛔ HIGH RISK warning before countdown

---

## 🔍 `infynon-pkg scan` — Full CVE Scanner

Reads all lock/manifest files in the current directory, batch-queries OSV for every pinned version, and presents a full report.

```bash
# Inline report only (no files written)
infynon-pkg scan

# Filter by severity
infynon-pkg scan --fix                     # show + auto-fix all severities
infynon-pkg scan --fix high                # show + auto-fix CRITICAL and HIGH only
infynon-pkg scan --fix critical            # only CRITICAL

# Save reports
infynon-pkg scan --output markdown         # infynon-scan-report.md
infynon-pkg scan --output pdf              # infynon-scan-report.pdf
infynon-pkg scan --output both             # both files

# Custom lock file
infynon-pkg scan --pkg-file ./Cargo.lock
infynon-pkg scan --pkg-file ./requirements.txt
```

### Supported Lock / Manifest Files

| Ecosystem | Files Parsed |
|---|---|
| **npm** | `package-lock.json` (v1/v2/v3) |
| **yarn** | `yarn.lock` |
| **pnpm** | `pnpm-lock.yaml` |
| **bun** | `package.json` (bun fallback) |
| **pip** | `requirements.txt` |
| **uv** | `uv.lock` |
| **poetry** | `poetry.lock`, `pyproject.toml` |
| **cargo** | `Cargo.lock` |
| **go** | `go.sum`, `go.mod` |
| **gem** | `Gemfile.lock` |
| **composer** | `composer.lock` |
| **nuget** | `packages.lock.json` |
| **hex** | `mix.lock` |
| **pub** | `pubspec.lock` |

### `--fix` Auto-Remediation

When `--fix` is passed, infynon-pkg **automatically executes** the upgrade command for every fixable package:

```
⚡ Auto-Fix  Executing 2 remediation command(s)...

  ✔  picomatch 4.0.3 → 4.0.4  fixed
  ✔  brace-expansion 1.1.12 → 5.0.5  fixed

  Auto-fix complete  2 succeeded  0 failed
```

Failed commands show the exact `stderr` output so you know why.

---

## 🔥 `infynon` — Firewall Mode

```bash
infynon                    # Show firewall splash
infynon daemon             # Start background intelligence service
infynon dashboard          # Open real-time ratatui dashboard
infynon update-intel       # Pull latest OSV threat feeds
```

---

## 🌐 OSV API Integration

Uses the [OSV.dev](https://osv.dev) batch API (`/v1/querybatch`) for high-throughput scanning:

- **Batch query**: All 800+ packages sent in a single HTTP request
- **Detail fetch**: Individual CVE records fetched for severity/version/summary
- **Affected ranges**: `SEMVER` and `ECOSYSTEM` ranges parsed to extract exact fix versions
- **Fail-open**: OSV errors skip the check rather than blocking installs

---

## 🔧 Binary Detection

`infynon-pkg` uses the OS's **native resolver** to check if package managers are installed:

| OS | Method |
|---|---|
| Windows | `where <binary>` — resolves `.cmd`, `.bat`, `.exe` in PATH |
| macOS | `which <binary>` — POSIX PATH resolver |
| Linux | `which <binary>` — POSIX PATH resolver |

Alternative names are also probed: `pip3`, `python3`, `nodejs`, `dotnet-host`, etc.

When a binary is not found, OS-appropriate install instructions are shown:

```
  ✘ Package manager 'npm' is not installed on this system.
  ℹ  npm ships bundled with Node.js — install Node.js to get npm.

  Install command: brew install node   OR   nvm install --lts   (macOS)
  Official docs:   https://nodejs.org/en/download
```

---

## 🏗️ Building

```bash
# Development
cargo build

# Release (produces infynon.exe / infynon)
cargo build --release

# Install both personalities to PATH
cp target/release/infynon ~/.cargo/bin/infynon
cp target/release/infynon ~/.cargo/bin/infynon-pkg
```

**Zero-warning requirement**: `cargo build` must emit no warnings. All unused stubs must be annotated `#[allow(dead_code)]`.

---

## 🧠 Architecture Principles

See [AGENTS.md](./AGENTS.md) for the full agent directive spec.

- **No logic in root**: `main.rs` delegates immediately to `src/cli`, `src/engine`, `src/tui`
- **No inline styling**: All output goes through `Logger::*` — never raw ANSI in business logic
- **No ANSI in tabled cells**: Color breaks width calculations; use plain text + tabled's `Color` modifier
- **Dual-personality routing**: `std::env::args().next()` determines mode at startup
#   i n f y n o n - c l i  
 