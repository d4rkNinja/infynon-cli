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

## Features

- **Single binary** — one `infynon` binary does everything, no separate tools
- **14 ecosystems** — npm, yarn, pnpm, bun, pip, uv, poetry, cargo, go, gem, composer, nuget, hex, pub
- **Auto-detection** — detects ecosystem from `package.json`, `Cargo.toml`, `go.mod`, `pyproject.toml`, etc.
- **OSV batch scanning** — queries [OSV.dev](https://osv.dev) for every dependency in your lock file
- **3-layer verification pipeline**
  - Layer 1: In-memory blocklist trie (<1ms)
  - Layer 2: Static heuristic scan — preinstall scripts, typosquatting, package age (<50ms)
  - Layer 3: LLM deep-code analysis via local Ollama (flagged packages only)
- **Install-time interception** — checks packages *before* install with interactive approve/skip/upgrade
- **Auto-fix** — `--fix` upgrades vulnerable packages to safe versions automatically
- **Reports** — Markdown and PDF export
- **Strict mode** — `--strict` blocks all vulnerable packages (CI-ready)
- **Local-first** — zero data leaves your machine unless you opt in

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

### Firewall engine

```bash
infynon                    # show info
infynon daemon             # start nightly CVE intelligence daemon
infynon dashboard          # open real-time TUI dashboard
infynon update-intel       # force CVE intel refresh
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
| `infynon pkg <ecosystem> install <pkg>` | Secure install |
| `infynon pkg --strict ...` | Block all vulnerable packages |

### `infynon` — Firewall Engine

| Command | Description |
|---------|-------------|
| `infynon` | Show info and commands |
| `infynon daemon` | Start nightly intelligence pipeline |
| `infynon dashboard` | Open TUI dashboard |
| `infynon update-intel` | Force CVE intel refresh |

---

## Supported Lock Files

| Ecosystem | Files |
|-----------|-------|
| npm / yarn / pnpm / bun | `package-lock.json`, `yarn.lock`, `pnpm-lock.yaml` |
| pip / uv / poetry | `requirements.txt`, `poetry.lock` |
| cargo | `Cargo.lock` |
| go | `go.sum` |
| gem | `Gemfile.lock` |
| composer | `composer.lock` |
| nuget | `packages.lock.json` |
| hex | `mix.lock` |
| pub | `pubspec.lock` |

---

## How It Works

```
  infynon pkg npm install express
                  │
                  ▼
     ┌─────────────────────────┐
     │  Parse package specs    │
     │  Resolve latest version │
     └────────────┬────────────┘
                  │
                  ▼
     ┌─────────────────────────┐
     │  OSV Batch Query        │
     └────────────┬────────────┘
                  │
           CVEs found?
          ╱            ╲
        No              Yes
         │               │
         ▼               ▼
      Install    ┌───────────────┐
      directly   │ [1] Install   │
                 │ [2] Skip      │
                 │ [3] Upgrade   │
                 └───────┬───────┘
                         │
                         ▼
                 Execute native
                 package manager
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

## Known Issues (Beta)

- Layer 3 (LLM analysis) requires local [Ollama](https://ollama.ai) — not yet fully integrated
- Nightly daemon and TUI dashboard are in early development
- SBOM generation planned but not yet implemented
- PDF reports work up to ~200 findings

---

## License

MIT

---

Built by **d4rkninja** & **whit3ninj4**

### Special Thanks

Huge shoutout to **whit3ninj4** for the relentless contributions, ideas, and late-night debugging sessions that shaped INFYNON into what it is today.
