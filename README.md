<p align="center">
  <h1 align="center">🛡️ INFYNON</h1>
</p>

<p align="center">
  <strong>Stop trusting installs, traffic, and API flows blindly.</strong>
</p>

<p align="center">
  A <strong>security-first CLI</strong> that intercepts threats at every layer of your stack:<br/><br/>
  • 📦 <strong>Dependency Firewall</strong> — CVE scan before any package touches your disk<br/>
  • 🛡️ <strong>Network Firewall</strong> — reverse proxy WAF between the internet and your backend<br/>
  • 🧪 <strong>API Flow Tester</strong> — node-based integration testing with built-in security probes<br/><br/>
  → One binary. Three shields. Security <strong>before execution</strong>, not after damage.
</p>

<p align="center">
  <a href="https://github.com/d4rkNinja/infynon-cli/stargazers">
    <img src="https://img.shields.io/github/stars/d4rkNinja/infynon-cli?style=for-the-badge" />
  </a>
  <a href="https://github.com/d4rkNinja/infynon-cli/blob/main/LICENSE">
    <img src="https://img.shields.io/github/license/d4rkNinja/infynon-cli?style=for-the-badge" />
  </a>
  <img src="https://img.shields.io/badge/ecosystems-14-blue?style=for-the-badge" />
  <img src="https://img.shields.io/badge/version-0.2.0--beta.7.2-orange?style=for-the-badge" />
  <a href="https://www.npmjs.com/package/infynon">
    <img src="https://img.shields.io/npm/v/infynon?style=for-the-badge&logo=npm&label=npm" />
  </a>
</p>

<p align="center">
  <a href="#-what-is-infynon">What It Is</a> •
  <a href="#-why-infynon-exists">Why It Exists</a> •
  <a href="#-quick-start">Quick Start</a> •
  <a href="#-core-capabilities">Features</a> •
  <a href="#-installation">Install</a> •
  <a href="#-ci--ai-agent-mode">CI / Agents</a> •
  <a href="https://cli.infynon.com/docs">Docs</a>
</p>

---

## 📦 What is INFYNON?

INFYNON is a single Rust binary with three modes — each one plugging a gap that traditional tools leave open.

### Module 1 — `infynon pkg` · Dependency Firewall

Intercepts package install commands across **14 ecosystems** and runs a 3-layer CVE check **before anything touches your disk**. Drop-in wrapper — same commands, same behavior, zero vulnerable packages.

```bash
infynon pkg npm install express          # instead of: npm install express
infynon pkg cargo add serde             # instead of: cargo add serde
infynon pkg pip install requests        # instead of: pip install requests
```

**Ecosystems:** `npm · yarn · pnpm · bun · pip · uv · poetry · cargo · go · gem · composer · nuget · hex · pub`

---

### Module 2 — `infynon` · Network Firewall

A self-hosted reverse proxy WAF with a real-time TUI dashboard. Sits between the internet and your backend — filtering HTTP traffic in real time.

```
Internet → INFYNON WAF → Your App Server
```

```bash
infynon init --port 8080 --upstream-port 3000
infynon start                        # firewall + TUI dashboard
infynon start --headless             # for servers, no TUI
```

---

### Module 3 — `infynon weave` · API Flow Testing

Test your entire API as a **connected flow**, not as isolated endpoints. Model each endpoint as a node in a directed graph — authentication tokens and extracted values thread automatically between nodes. No more manual token copy-paste between test steps.

```bash
infynon weave env set BASE_URL http://localhost:8001
infynon weave node create --ai "POST /auth/login — extracts token"
infynon weave node create --ai "POST /orders — creates order, extracts order_id"
infynon weave flow create "checkout" --ai "login then create order"
infynon weave flow run checkout
infynon weave ai probe checkout      # auth bypass, rate limit, SQLi probes
infynon weave tui                    # 10-tab TUI dashboard
```

---

## 🚨 Why INFYNON Exists

### Packages: AI installs. You don't verify.

Claude Code, Cursor, Copilot — they suggest packages and run installs. You don't check every transitive dependency for CVEs. INFYNON sits in between: every install gets scanned, blocked, or auto-fixed before it runs.

### Traffic: Your backend is exposed. You don't see it.

Rate abuse, SQL injection, path traversal — happening on your server while you're writing code. INFYNON WAF sits in front and filters in real time, with a live TUI feed and configurable rules.

### API Workflows: Postman collections rot. You maintain them.

Multi-step API tests break when tokens expire, request bodies change, or auth flows evolve. Weave models flows as a graph — AI-generated, context-threaded, re-runnable. Security probes run automatically after every flow.

---

## ⚡ Quick Start

### Dependency Firewall

```bash
npm install -g infynon

# Scan your project's lock files right now
infynon pkg scan

# Secure install — intercepts before running the package manager
infynon pkg npm install express lodash

# Auto-fix all vulnerable deps
infynon pkg fix --auto

# Deep audit with full dependency tree
infynon pkg audit
```

### Network Firewall

```bash
# Initialize config for port 8080 → upstream at 3000
infynon init --port 8080 --upstream-port 3000

# Start with TUI dashboard
infynon start

# Block a suspicious IP
infynon block 203.0.113.50

# View recent blocked requests
infynon logs --verdict block --count 50
```

### API Flow Testing (Weave)

```bash
# Set base URL for this project
infynon weave env set BASE_URL http://localhost:8001

# Create nodes from natural language
infynon weave node create --ai "POST /auth/login with email and password, extracts token and user_id"
infynon weave node create --ai "GET /users/{user_id} — returns user profile"
infynon weave node create --ai "POST /orders — creates order, extracts order_id"

# Wire into a flow
infynon weave flow create "user-journey" --ai "login, get profile, create order"

# Run — tokens flow automatically
infynon weave flow run user-journey

# Run built-in security probes on the flow
infynon weave ai probe user-journey
```

---

## 🧪 Traditional API Testing vs INFYNON Weave

| | Traditional (Postman / Insomnia / pytest) | INFYNON Weave |
|---|---|---|
| **Flow model** | Isolated requests or linear collections | Directed graph — nodes wire together |
| **Token handling** | Manual: copy from one test, paste into next | Automatic: extracted values thread between nodes |
| **Dynamic inputs** | Hardcoded or env vars set ahead of time | Runtime prompts (OTP, 2FA, password) mid-flow |
| **Security testing** | Separate tool (Burp, manual scripts) | Built-in probes: auth bypass, rate limit, SQLi |
| **Flow creation** | Manual: click to configure each request | AI-generated from natural language description |
| **CI integration** | Complex setup, credential management | `--set KEY=val` flags or `--default` values |
| **Visibility** | Static report after the fact | Live TUI: step-by-step feed, latency, diffs |
| **Prompt inputs** | N/A | `text · boolean · select · multiselect` types |

---

## 🚀 Core Capabilities

### 🔐 Dependency Security

- **Pre-install CVE scanning** via OSV.dev — before the package hits your disk
- **Blocks vulnerable packages** — interactive decision or fully automated
- **Auto-fix** — upgrade to nearest safe version automatically
- **15+ lock file parsers** — works with your existing project, zero config
- **CI enforcement** — non-zero exit on violation, configurable severity threshold
- **`--agent` mode** — machine-readable JSON for AI agents and CI parsers

### 🛡️ Network Protection

- **Reverse proxy WAF** — SQLi, XSS, path traversal, command injection detection
- **Rate limiting** — per-IP, per-path, and global sliding window limits
- **IP filtering** — blocklist, allowlist, CIDR ranges, auto-reputation banning
- **Multi-upstream routing** — route paths to different backend services
- **Hot config reload** — changes applied in seconds, no restart needed
- **Email alerts** — SMTP/SES notifications on suspicious activity + daily digest

### 🧪 API Flow Testing (Weave)

- **Node-based flows** — each node is one HTTP request; graph edges thread values forward
- **Context threading** — tokens, IDs, extracted values flow automatically between nodes
- **Runtime prompt inputs** — pause and ask for OTPs, passwords, dynamic values mid-flow
- **4 prompt types** — `text`, `boolean`, `select`, `multiselect`
- **AI flow builder** — describe in English, Weave wires the graph
- **Security probes** — auth bypass, rate limit, SQL injection checks out of the box
- **10-tab TUI dashboard** — live execution feed, latency profiler, security results, env manager
- **CI ready** — `--default` values or `--set KEY=val` for fully non-interactive runs

### 🔬 Dependency Intelligence

| Command | Description |
|---------|-------------|
| `infynon pkg audit` | Recursive dependency tree with CVE annotations |
| `infynon pkg why <pkg>` | Trace why a package is in your tree |
| `infynon pkg outdated` | Detect outdated deps across all ecosystems |
| `infynon pkg diff <pkg> v1 v2` | Compare versions: size, deps, scripts, CVEs |
| `infynon pkg doctor` | Health check: dupes, unused, phantoms, missing locks |
| `infynon pkg size <pkg>` | Install weight and transitive dep count |
| `infynon pkg search <query>` | Cross-ecosystem search |
| `infynon pkg clean` | Find and remove unused dependencies |
| `infynon pkg migrate <from> <to>` | Migrate between package managers |
| `infynon pkg scan --output pdf` | Export full report as Markdown or PDF |

---

## 🤖 CI & AI Agent Mode

### Non-Interactive Flags

```bash
# Fail the build if any critical or high vulnerability is found
infynon pkg npm install express --strict high

# Auto-upgrade to safe versions silently
infynon pkg npm install express --auto-fix

# Skip all vulnerable packages, install only clean ones
infynon pkg npm install express --skip-vulnerable

# Install everything regardless (audit-only CI)
infynon pkg npm install express --yes
```

| Flag | Behavior | Exit Code |
|------|----------|-----------|
| `--strict [LEVEL]` | Block if vulnerabilities at or above level are found | `3` on block |
| `--auto-fix` | Upgrade to safe versions, skip unfixable | `0` |
| `--skip-vulnerable` | Skip vulnerable packages, install clean ones | `0` |
| `--yes` | Install all packages including vulnerable | `0` |

Severity levels: `critical · high · medium · low · all`

### `--agent` Mode (Structured JSON)

Add `--agent` to any command for machine-readable JSON output — designed for Claude Code, Cursor, CI parsers, and shell scripts.

```bash
infynon pkg scan --agent
infynon pkg npm install express lodash --agent --strict high
infynon pkg uv add fastapi sqlalchemy --agent --auto-fix
```

```json
{
  "status": "vulnerable",
  "packages_scanned": 142,
  "vulnerabilities": [
    {
      "package": "requests",
      "ecosystem": "PyPI",
      "current_version": "2.28.0",
      "severity": "MEDIUM",
      "safe_version": "2.31.0",
      "fix_cmd": "pip install requests==2.31.0"
    }
  ],
  "summary": { "critical": 0, "high": 0, "medium": 1, "low": 0, "total": 1 }
}
```

**Exit codes:** `0` clean · `1` warnings only · `2` vulnerabilities found · `3` blocked by `--strict`

### GitHub Actions

```yaml
- name: Secure install
  run: infynon pkg npm install --strict high
```

### Weave in CI

```bash
# Pre-seed prompt values — no interactive prompts
infynon weave flow run auth-flow \
  --set email=ci@example.com \
  --set password=Test@1234

# Or set defaults on each prompt input at definition time
infynon weave node prompt register add email --label "Email" --default "ci@example.com"
```

---

## 👥 Who Is This For

- **AI coding users** (Claude Code, Cursor, Copilot) — AI suggests and installs packages; INFYNON intercepts and verifies them before anything touches disk
- **AI agents running autonomously** — `--agent` mode emits structured JSON so agents can parse results and react without screen-scraping
- **Backend engineers** testing APIs that have multi-step auth flows and context dependencies
- **DevOps / security teams** enforcing CVE policies in CI pipelines
- **Anyone installing dependencies** without auditing every transitive package

---

## 🔥 Installation

### npm (recommended — all platforms)

```bash
npm install -g infynon
```

Downloads the right pre-built binary for your OS automatically. Requires Node.js 14+.

**Supported:** Windows x64 · Linux x64 · Linux ARM64 · macOS x64 · macOS ARM64

```bash
# Uninstall (removes binary + config files from ~/.infynon/)
npm uninstall -g infynon
```

### Linux / macOS

```bash
curl -fsSL https://raw.githubusercontent.com/d4rkNinja/infynon-cli/main/scripts/install.sh | bash
```

### Windows

```powershell
irm https://raw.githubusercontent.com/d4rkNinja/infynon-cli/main/scripts/install.ps1 | iex
```

### Build from Source

```bash
cargo install --git https://github.com/d4rkNinja/infynon-cli
```

---

## 🦅 Eagle Eye — Scheduled Monitoring

Background CVE scanner for your projects. Runs on a configurable timer, scans all lock files, sends email alerts when vulnerabilities match your risk threshold.

```bash
infynon pkg eagle-eye setup    # Interactive setup: SMTP, paths, risk level, schedule
infynon pkg eagle-eye start    # Start monitoring in foreground
infynon pkg eagle-eye status   # View current configuration
```

---

## 💡 Core Idea

Security should happen **before execution**, not after damage.

INFYNON enforces that — at the dependency level, the network level, and the API flow level.

---

## 🧬 Development Channel

```bash
# Latest features (may have breaking changes)
cargo install --git https://github.com/d4rkNinja/infynon-cli --branch development
```

---

## 🔮 Upcoming

- SBOM generation (CycloneDX / SPDX) after every install
- Typosquatting / hallucinated-package detection
- License compliance gate (`--deny-license GPL-3.0`)
- Policy-as-code (`.infynon-policy.toml`)
- Geo-IP blocking (MaxMind GeoLite2)
- Webhook alerts (Slack, Discord, Teams)
- TLS termination support

---

**[cli.infynon.com/docs](https://cli.infynon.com/docs)** · [github.com/d4rkNinja/infynon-cli](https://github.com/d4rkNinja/infynon-cli)
