# INFYNON — Security CLI: CVE Scanning, Reverse Proxy WAF & API Flow Testing

<!-- keywords: security cli, vulnerability scanner, cve scanner, npm security, pip security, cargo security, reverse proxy waf, web application firewall, api testing, supply chain security, devsecops, package audit, dependency scanner, rate limiter, ip filter -->

<p align="center">
  <strong>One binary. Three security layers. Zero trust by default.</strong>
</p>

<p align="center">
  INFYNON enforces a <em>verify-before-execute</em> policy across your entire stack —<br/>
  intercepting vulnerable packages before install, testing API flows before trust,<br/>
  and filtering live traffic before exposure.
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
  <img src="https://img.shields.io/github/last-commit/d4rkNinja/infynon-cli?style=for-the-badge" />
</p>

<p align="center">
  <a href="#why-infynon">Why It Exists</a> •
  <a href="#what-infynon-does">What It Does</a> •
  <a href="#who-should-use-this">Who It's For</a> •
  <a href="#-quick-start">Quick Start</a> •
  <a href="#-installation">Install</a> •
  <a href="#-ci--ai-agent-mode">CI / Agents</a> •
  <a href="https://cli.infynon.com/docs">Docs</a>
</p>

---

## Why INFYNON

Modern development is no longer manual.

AI suggests packages.
Dependencies pull hidden trees.
Install scripts execute automatically.
APIs are tested in isolation, not flows.
Traffic is exposed before validation.

Most systems assume trust first.

INFYNON flips that:

**Verify first. Execute later.**

---

## What INFYNON does

### 1. Package Vulnerability Scanner (`infynon pkg`)

- Intercepts installs across 14 ecosystems — CVE check before anything touches disk
- Shows full dependency tree with vulnerability annotations
- Highlights install scripts before they run
- Explains why a package exists in your tree
- Compares versions: size, deps, CVEs side by side
- Detects outdated, unused, and broken dependencies

```bash
infynon pkg audit
infynon pkg why <pkg>
infynon pkg outdated
infynon pkg diff <pkg> v1 v2
infynon pkg doctor
infynon pkg size <pkg>
infynon pkg search <query>
infynon pkg clean
infynon pkg migrate <from> <to>
```

---

### 2. API Flow Security Testing (`infynon weave`)

- Model API workflows as connected nodes — not isolated requests
- Automatically thread tokens and IDs between steps
- Save and replay full flows with one command
- Run built-in security probes: auth bypass, rate limit abuse, SQL injection
- Replace manual Postman chaining

```bash
infynon weave env set BASE_URL http://localhost:8001
infynon weave node create --ai "POST /auth/login — extracts token"
infynon weave node create --ai "POST /orders — creates order, extracts order_id"
infynon weave flow create "checkout" --ai "login then create order"
infynon weave flow run checkout
infynon weave ai probe checkout
infynon weave tui
```

---

### 3. Reverse Proxy WAF (`infynon`)

- Self-hosted reverse proxy web application firewall
- Rate limiting: per-IP, per-path, global sliding window
- IP filtering: blocklist, allowlist, CIDR, auto-reputation banning
- Multi-upstream routing for microservices
- Real-time TUI dashboard — live feed, stats, config editing

```bash
infynon init --port 8080 --upstream-port 3000
infynon start
infynon start --headless
infynon block 203.0.113.50
infynon logs --verdict block --count 50
```

---

<p align="center">
  <img src="assets/infynon-demo-small.gif" alt="INFYNON Demo" width="100%" />
</p>

---

## Supported Ecosystems

`npm` · `yarn` · `pnpm` · `bun` · `pip` · `uv` · `poetry` · `cargo` · `go` · `gem` · `composer` · `nuget` · `hex` · `pub`

14 package managers. One security layer.

---

## Who should use this

- Backend developers
- AI-assisted coding workflows (Claude Code, Cursor, Copilot)
- Security-conscious teams enforcing CVE policy in CI
- API-heavy systems with multi-step auth flows
- CLI-first developers

---

## What INFYNON is not

- Not a package manager
- Not a replacement for npm/pip
- Not just a firewall
- Not just an API tool

It is a control layer before execution.

---

## INFYNON vs Other Security Tools

| Feature | INFYNON | Snyk CLI | Safety CLI | OSV-Scanner |
|---|---|---|---|---|
| **ECOSYSTEM COVERAGE** | | | | |
| npm / yarn / pnpm / bun | ✓ | ✓ | — | ✓ |
| pip / uv / poetry | ✓ | ✓ | ✓ | ✓ |
| cargo | ✓ | ✓ | — | ✓ |
| go / gem / composer / nuget / hex / pub | ✓ | Partial | — | Partial |
| **PACKAGE SECURITY** | | | | |
| **Pre-install interception** | ✓ | — | — | — |
| Interactive install decisions (fix/skip/install) | ✓ | — | — | — |
| Auto-fix to safe version | ✓ | ✓ | — | — |
| Dependency tree audit with risk score | ✓ | ✓ | — | — |
| `why` — trace dependency origin | ✓ | — | — | — |
| Package version diff (size, deps, CVEs) | ✓ | — | — | — |
| Doctor / dependency health check | ✓ | — | — | — |
| Package size & bundle weight analysis | ✓ | — | — | — |
| Cross-ecosystem package search | ✓ | — | — | — |
| Remove unused dependencies (`clean`) | ✓ | — | — | — |
| Migrate between package managers | ✓ | — | — | — |
| Multi-lock file detection + selector | ✓ | — | — | — |
| **MONITORING & ALERTS** | | | | |
| **Eagle Eye — scheduled CVE monitoring** | ✓ | Paid | — | — |
| Email alerts on new vulnerabilities | ✓ | Paid | — | — |
| Daily digest reports | ✓ | Paid | — | — |
| **AI & CI INTEGRATION** | | | | |
| **Claude Code skills + plugins** | ✓ | — | — | — |
| `--agent` structured JSON output | ✓ | — | — | — |
| CI / `--strict [level]` flag | ✓ | ✓ | ✓ | ✓ |
| `--auto-fix` / `--skip-vulnerable` flags | ✓ | ✓ | — | — |
| Self-hosted, no account required | ✓ | — | ✓ | ✓ |
| **FIREWALL & TRAFFIC** | | | | |
| **Reverse proxy WAF** | ✓ | — | — | — |
| Rate limiting (per-IP, per-path, global) | ✓ | — | — | — |
| IP blocking / CIDR / auto-reputation ban | ✓ | — | — | — |
| Custom firewall rules engine | ✓ | — | — | — |
| SQLi / XSS / path traversal detection | ✓ | — | — | — |
| Real-time TUI dashboard | ✓ | — | — | — |
| **API FLOW TESTING** | | | | |
| **Node-based API flow testing** | ✓ | — | — | — |
| AI-generated test flows from natural language | ✓ | — | — | — |
| Auto token threading between steps | ✓ | — | — | — |
| Built-in security probes (auth bypass, SQLi) | ✓ | — | — | — |
| Runtime prompt inputs (OTP, 2FA, mid-flow) | ✓ | — | — | — |

---

## ⚡ Quick Start

### Package Vulnerability Scanning

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

### Reverse Proxy WAF

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

### API Flow Security Testing

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

## 🔬 Dependency Intelligence Commands

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

## 🤖 Claude Code Integration

INFYNON has dedicated Claude Code plugins at [d4rkNinja/code-guardian](https://github.com/d4rkNinja/code-guardian). Install them so Claude automatically knows how to use every INFYNON command — correct flags, right mode, right ecosystem.

### Install plugins

```bash
/plugin marketplace add d4rkNinja/code-guardian
/plugin install infynon-pkg@d4rkNinja
/plugin install infynon-firewall@d4rkNinja
/plugin install infynon-weave@d4rkNinja
/reload-plugins
```

Or load locally for development:

```bash
claude --plugin-dir ./infynon-pkg --plugin-dir ./infynon-firewall --plugin-dir ./infynon-weave
```

### What the plugins do

Once installed, Claude automatically:

- Routes every install through `infynon pkg` — never runs raw `npm install`, `pip install`, etc.
- Picks the right CI flag (`--strict high`, `--auto-fix`, etc.) based on context
- Detects lock files in the project and suggests scanning
- Helps write `infynon.toml` firewall configs and custom WAF rules
- Guides through TUI keyboard shortcuts for both firewall and weave
- Explains CVE findings and how to prioritize fixes
- Designs API test flows with nodes, edges, assertions, and prompt inputs

### Available plugins and skills

| Plugin | Skill | Auto-triggers when |
|--------|-------|--------------------|
| `infynon-pkg` | `package-security` | User asks about packages/CVEs, or lock files detected |
| `infynon-pkg` | `cve-triage` | User has scan results and needs to prioritize |
| `infynon-pkg` | `eagle-eye-monitor` | User wants scheduled CVE monitoring |
| `infynon-firewall` | `firewall-setup` | User asks about WAF/rate limiting, or `infynon.toml` detected |
| `infynon-firewall` | `attack-response` | User is under attack or investigating traffic |
| `infynon-firewall` | `rule-writer` | User wants to write custom firewall rules |
| `infynon-weave` | `api-testing` | User asks about API testing/flows, or `.infynon/api/` detected |

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

## 🦅 Eagle Eye — Scheduled CVE Monitoring

Background dependency scanner for your projects. Runs on a configurable timer, scans all lock files, sends email alerts when vulnerabilities match your risk threshold.

```bash
infynon pkg eagle-eye setup    # Interactive setup: SMTP, paths, risk level, schedule
infynon pkg eagle-eye start    # Start monitoring in foreground
infynon pkg eagle-eye status   # View current configuration
```

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
