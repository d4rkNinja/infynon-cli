# INFYNON

**🔥 Stop installing vulnerable dependencies blindly**

A security-first CLI: pre-install CVE scanner for 14 ecosystems + reverse proxy WAF for your backend.

[![npm](https://img.shields.io/npm/v/infynon?style=flat-square&logo=npm)](https://www.npmjs.com/package/infynon)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](https://github.com/d4rkNinja/infynon-cli/blob/main/LICENSE)
[![GitHub](https://img.shields.io/badge/source-GitHub-black?style=flat-square&logo=github)](https://github.com/d4rkNinja/infynon-cli)

> ⚠️ AI installs packages. You don't verify them. That's the risk.
> **INFYNON fixes that — blocks threats before they reach your system.**

---

## Install

```bash
npm install -g infynon
```

Downloads the right pre-built native binary for your OS and architecture automatically. Requires Node.js 14+.

**Supported platforms:** Windows x64 · Linux x64 · Linux ARM64 · macOS x64 · macOS ARM64

To uninstall and clean up all config files:

```bash
npm uninstall -g infynon
```

---

## What is INFYNON?

A single binary with two modes:

### 1. `infynon pkg` — Package Security

Intercepts install commands across **14 ecosystems** and runs a 3-layer CVE check before anything touches your disk.

```bash
# Scan your project's lock files for CVEs
infynon pkg scan

# Secure install — intercepts and checks before running npm
infynon pkg npm install express

# Works with any ecosystem
infynon pkg cargo add serde
infynon pkg pip install requests
infynon pkg yarn add lodash

# Auto-fix all vulnerable dependencies
infynon pkg fix --auto

# Deep audit with full dependency tree
infynon pkg audit

# CI / non-interactive flags (no prompts)
infynon pkg npm install express --strict high      # fail build on critical/high
infynon pkg npm install express --auto-fix         # auto-upgrade to safe versions
infynon pkg npm install express --skip-vulnerable  # skip bad packages silently
infynon pkg npm install express --yes              # install everything (audit-only CI)
```

**Ecosystems:** npm · yarn · pnpm · bun · pip · uv · poetry · cargo · go · gem · composer · nuget · hex · pub

### 2. `infynon` — Network Firewall

A reverse proxy WAF with a real-time TUI dashboard. Sits between the internet and your backend.

```bash
# Initialize config
infynon init --port 8080 --upstream-port 3000

# Start firewall with TUI dashboard
infynon start

# Start headless (no TUI — for servers)
infynon start --headless

# Block an IP
infynon block 203.0.113.50

# View blocked requests
infynon logs --verdict block
```

**Protects against:** SQL injection · XSS · path traversal · command injection · header injection · rate abuse · bad IPs

---

## How It Works (Package Security)

1. You (or an AI tool) runs `infynon pkg npm install express`
2. INFYNON resolves the full dependency tree
3. Queries **OSV.dev** for known CVEs across all packages
4. Presents an interactive decision for any vulnerable package — block, skip, or install a fixed version
5. Only approved packages get installed

---

## More Commands

| Command | Description |
|---------|-------------|
| `infynon pkg scan` | Scan lock files for CVEs |
| `infynon pkg fix --auto` | Auto-upgrade all vulnerable deps |
| `infynon pkg audit` | Full dependency tree with CVE annotations |
| `infynon pkg why <pkg>` | Trace why a package is in your tree |
| `infynon pkg outdated` | Find outdated deps across all ecosystems |
| `infynon pkg diff <pkg> v1 v2` | Compare versions: size, deps, CVEs |
| `infynon pkg doctor` | Health check: dupes, unused, phantoms |
| `infynon pkg size <pkg>` | Install weight and transitive dep count |
| `infynon pkg search <query>` | Cross-ecosystem package search |
| `infynon pkg clean` | Remove unused dependencies |
| `infynon pkg migrate <from> <to>` | Migrate between package managers |
| `infynon pkg eagle-eye setup` | Set up scheduled CVE monitoring with email alerts |

---

## Full Documentation

**[cli.infynon.com/docs](https://cli.infynon.com/docs)**

Source: [github.com/d4rkNinja/infynon-cli](https://github.com/d4rkNinja/infynon-cli)

---

## License

MIT
