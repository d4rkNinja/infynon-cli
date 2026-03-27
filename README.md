<p align="center">
  <strong>INFYNON</strong><br>
  Universal Package Security Manager
</p>

<p align="center">
  Single binary. 14 ecosystems. Automatic CVE scanning before every install.
</p>

<p align="center">
  <a href="#installation">Install</a> &middot;
  <a href="#quick-start">Quick Start</a> &middot;
  <a href="#commands">Commands</a> &middot;
  <a href="#uninstallation">Uninstall</a>
</p>

---

### Scan your dependencies for CVEs

<p align="center">
  <img src="assets/scan-demo.png" alt="infynon pkg scan" width="700">
</p>

### Secure install with interactive vulnerability prompts

<p align="center">
  <img src="assets/install-demo.png" alt="infynon pkg npm install" width="700">
</p>

---

## What It Does Today

- **Single binary** — one `infynon` binary, two modes: `infynon pkg` for package security, `infynon` for firewall info
- **14 ecosystem support** — npm, yarn, pnpm, bun, pip, uv, poetry, cargo, go, gem, composer, nuget, hex, pub
- **Auto-detection** — detects your ecosystem from lock/manifest files (`package-lock.json`, `Cargo.lock`, `go.sum`, `requirements.txt`, etc.)
- **OSV vulnerability scanning** — batch queries [OSV.dev](https://osv.dev) API to check every dependency in your lock file for known CVEs
- **Lock file parsing** — parses 15 lock file formats (npm, yarn, pnpm, pip, poetry, uv, cargo, go.sum, go.mod, Gemfile.lock, composer.lock, packages.lock.json, mix.lock, pubspec.lock, pyproject.toml)
- **Install-time interception** — checks packages against OSV *before* installation, shows severity badges, CVE details, and safe versions
- **Interactive prompts** — per-package approve/skip/upgrade decisions with apply-to-all shortcut
- **Auto-fix** — `--fix` executes upgrade commands automatically for all fixable vulnerabilities
- **Strict mode** — `--strict` hard-blocks all vulnerable packages and exits (CI-ready)
- **Report generation** — export scan results as Markdown or styled PDF with severity tables and upgrade commands
- **Registry version resolution** — fetches latest versions from 9 registries (npm, PyPI, crates.io, Go proxy, RubyGems, Packagist, NuGet, Hex, pub.dev) when no version is specified
- **Binary detection** — OS-native detection of package manager binaries with install instructions (winget/brew/apt)

## Upcoming

- **3-layer verification pipeline** — blocklist trie lookup, static heuristic scan (preinstall scripts, typosquatting, package age), LLM deep-code analysis via local Ollama
- **Firewall engine** — `infynon daemon` for nightly CVE intelligence crawling, `infynon dashboard` for real-time TUI, `infynon update-intel` for manual intel refresh
- **SBOM generation** — CycloneDX format after every install
- **Configuration** — `.infynon.toml` project-level config, custom blocklists, Ollama endpoint settings
- **Ecosystem adapters** — native dependency resolution and install routing per ecosystem

---

## Installation

### One-liner — Linux / macOS

```bash
curl -fsSL https://raw.githubusercontent.com/d4rkNinja/infynon-cli/main/scripts/install.sh | bash
```

### One-liner — Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/d4rkNinja/infynon-cli/main/scripts/install.ps1 | iex
```

### One-liner — Any OS with Rust

```bash
cargo install --git https://github.com/d4rkNinja/infynon-cli
```

### Manual download

Grab the single binary for your platform from [Releases](https://github.com/d4rkNinja/infynon-cli/releases):

| Platform | File |
|----------|------|
| Windows x86_64 | `infynon-x86_64-pc-windows-msvc.exe` |
| Linux x86_64 | `infynon-x86_64-unknown-linux-musl` |
| Linux ARM64 | `infynon-aarch64-unknown-linux-musl` |
| macOS Intel | `infynon-x86_64-apple-darwin` |
| macOS Apple Silicon | `infynon-aarch64-apple-darwin` |

```bash
# Linux / macOS
chmod +x infynon
sudo mv infynon /usr/local/bin/
```

```powershell
# Windows — move to a folder in your PATH
Move-Item infynon.exe "$env:USERPROFILE\.infynon\bin\infynon.exe"
```

---

## Quick Start

### Scan your project

```bash
infynon pkg scan                          # auto-detect lock files
infynon pkg scan --pkg-file ./Cargo.lock  # specific file
infynon pkg scan --output pdf             # export report (markdown|pdf|both)
infynon pkg scan --fix critical           # auto-fix critical+ vulns
infynon pkg scan --fix                    # auto-fix all fixable vulns
```

### Secure install

```bash
# JavaScript
infynon pkg npm install express
infynon pkg yarn add lodash
infynon pkg pnpm add react
infynon pkg bun add axios

# Python
infynon pkg pip install requests
infynon pkg uv pip install fastapi
infynon pkg poetry add django

# Rust
infynon pkg cargo add serde

# Go
infynon pkg go get golang.org/x/crypto

# Ruby
infynon pkg gem install rails

# PHP
infynon pkg composer require laravel/framework

# .NET
infynon pkg nuget add Newtonsoft.Json

# Elixir
infynon pkg hex deps.get

# Dart / Flutter
infynon pkg pub add http
```

### Auto-detect (no ecosystem prefix)

```bash
infynon pkg install express       # detects npm from package.json
infynon pkg add serde             # detects cargo from Cargo.toml
```

### Strict mode (CI)

```bash
infynon pkg --strict npm install express   # exit 1 on any CVE
```

---

## Commands

### `infynon pkg` — Package Security

| Command | Description |
|---------|-------------|
| `infynon pkg scan` | Scan lock/manifest files for known CVEs |
| `infynon pkg scan --output <FORMAT>` | Export: `markdown`, `pdf`, `both` |
| `infynon pkg scan --fix [LEVEL]` | Auto-fix: `critical` `high` `medium` `low` `all` |
| `infynon pkg scan --pkg-file <PATH>` | Scan specific file |
| `infynon pkg <ecosystem> install <pkg>` | Secure install with CVE check |
| `infynon pkg --strict ...` | Block all vulnerable packages |

### `infynon` — Firewall Engine (Upcoming)

| Command | Status |
|---------|--------|
| `infynon` | Shows project info and available commands |
| `infynon daemon` | Planned — nightly CVE intelligence pipeline |
| `infynon dashboard` | Planned — real-time TUI security dashboard |
| `infynon update-intel` | Planned — manual CVE intel refresh |

---

## Supported Lock Files

| Ecosystem | Files Parsed |
|-----------|-------------|
| npm | `package-lock.json` |
| yarn | `yarn.lock` |
| pnpm | `pnpm-lock.yaml` |
| pip / uv | `requirements.txt`, `pyproject.toml` |
| poetry | `poetry.lock` |
| uv | `uv.lock` |
| cargo | `Cargo.lock` |
| go | `go.sum`, `go.mod` |
| gem | `Gemfile.lock` |
| composer | `composer.lock` |
| nuget | `packages.lock.json` |
| hex | `mix.lock` |
| pub | `pubspec.lock` |

---

## Use with AI / Agentic Coding Tools

Add this to your AI coding tool's system prompt or rules file (`.cursorrules`, `.claude/CLAUDE.md`, `.github/copilot-instructions.md`, etc.) to make it use INFYNON for all package installs:

### Prompt

```
When installing any package or dependency, ALWAYS use `infynon pkg` as a secure proxy instead of calling the package manager directly. This automatically scans for CVEs before installation.

Examples:
  - Instead of `npm install express`     → `infynon pkg npm install express`
  - Instead of `pip install requests`    → `infynon pkg pip install requests`
  - Instead of `yarn add lodash`         → `infynon pkg yarn add lodash`
  - Instead of `cargo add serde`         → `infynon pkg cargo add serde`
  - Instead of `go get golang.org/x/net` → `infynon pkg go get golang.org/x/net`
  - Instead of `uv pip install fastapi`  → `infynon pkg uv pip install fastapi`
  - Instead of `poetry add django`       → `infynon pkg poetry add django`
  - Instead of `composer require laravel/framework` → `infynon pkg composer require laravel/framework`
  - Instead of `gem install rails`       → `infynon pkg gem install rails`
  - Instead of `dotnet add package X`    → `infynon pkg nuget add X`
  - Instead of `dart pub add http`       → `infynon pkg pub add http`
  - Instead of `mix deps.get`            → `infynon pkg hex deps.get`

After adding dependencies, run `infynon pkg scan` to audit the full dependency tree.
Use `infynon pkg scan --fix` to auto-remediate all fixable vulnerabilities.
Use `infynon pkg scan --output both` to export Markdown + PDF reports.

For strict CI pipelines, use `infynon pkg --strict <ecosystem> install <pkg>` to hard-block any vulnerable package.

For full command reference, see: https://github.com/d4rkNinja/infynon-cli/tree/main/docs
```

### Example `.cursorrules` / `.claude/CLAUDE.md`

```
# Package Security

All package installations MUST go through infynon:
  infynon pkg <ecosystem> <command> <packages>

Run `infynon pkg scan` after modifying dependencies.
Run `infynon pkg scan --fix` to auto-fix vulnerabilities.
Run `infynon pkg scan --output both` to generate security reports.
```

---

## Building From Source

```bash
git clone https://github.com/d4rkNinja/infynon-cli.git
cd infynon-cli
cargo build --release
# Binary → target/release/infynon
```

Cross-compile with [cross](https://github.com/cross-rs/cross):

```bash
cargo install cross
cross build --release --target x86_64-unknown-linux-musl
cross build --release --target aarch64-unknown-linux-musl
cross build --release --target x86_64-apple-darwin
cross build --release --target aarch64-apple-darwin
```

---

## Uninstallation

### Installed via cargo

```bash
cargo uninstall infynon
```

### Installed via install script

**Linux / macOS:**
```bash
sudo rm /usr/local/bin/infynon
```

**Windows:**
```powershell
Remove-Item "$env:USERPROFILE\.infynon\bin\infynon.exe"
# Remove from PATH: Settings → Environment Variables → remove .infynon\bin entry
```

---

## License

MIT

---

Built by **d4rkninja** & **whit3ninj4**

### Special Thanks

Huge shoutout to **whit3ninj4** for the relentless contributions, ideas, and late-night debugging sessions that shaped INFYNON into what it is today.
