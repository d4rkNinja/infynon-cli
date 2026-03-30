<p align="center">
  <h1 align="center">🛡️ INFYNON</h1>
</p>

<p align="center">
  <strong>🔥 Stop installing vulnerable dependencies blindly</strong>
</p>

<p align="center">
  INFYNON is a <strong>security-first CLI</strong> that acts as a:<br/><br/>
  • 🔐 <strong>Firewall for your dependencies</strong> — pre-install CVE scanner<br/>
  • 🛡️ <strong>Firewall for your backend</strong> — WAF + reverse proxy<br/>
  • 🧪 <strong>API flow tester</strong> — node-based integration testing with security probes<br/><br/>
  → Blocks threats <strong>BEFORE they reach your system</strong>
</p>

<p align="center">
  <em>⚠️ AI installs packages. You don't verify them. That's the risk. INFYNON fixes that.</em>
</p>

<p align="center">
  <a href="https://github.com/d4rkNinja/infynon-cli/stargazers">
    <img src="https://img.shields.io/github/stars/d4rkNinja/infynon-cli?style=for-the-badge" />
  </a>
  <a href="https://github.com/d4rkNinja/infynon-cli/issues">
    <img src="https://img.shields.io/github/issues/d4rkNinja/infynon-cli?style=for-the-badge" />
  </a>
  <a href="https://github.com/d4rkNinja/infynon-cli/blob/main/LICENSE">
    <img src="https://img.shields.io/github/license/d4rkNinja/infynon-cli?style=for-the-badge" />
  </a>
  <img src="https://img.shields.io/badge/ecosystems-14-blue?style=for-the-badge" />
  <img src="https://img.shields.io/badge/version-0.2.0--beta.7-orange?style=for-the-badge" />
  <a href="https://www.npmjs.com/package/infynon">
    <img src="https://img.shields.io/npm/v/infynon?style=for-the-badge&logo=npm&label=npm" />
  </a>
</p>

<p align="center">
  <a href="#-30-second-demo">Demo</a> •
  <a href="#-what-infynon-does">What It Does</a> •
  <a href="#-core-capabilities">Features</a> •
  <a href="#-installation">Install</a> •
  <a href="#-ci--automation">CI</a> •
  <a href="https://cli.infynon.com/docs">Docs</a>
</p>

---

## Demo

```bash
# Scan before you install — any ecosystem
infynon pkg npm install express lodash

  🛡️ INFYNON Secure Proxy — Active
  » Ecosystem: npm

  ✔ 142 dependencies analyzed
  ⚠ 2 vulnerabilities found (1 CRITICAL, 1 HIGH)

    1.  lodash   [CRITICAL]   3 CVEs  → safe: 4.17.21
    2.  express  [HIGH]       1 CVE   → safe: 4.18.2

  → Apply same action to ALL infected packages?
     [1] Install anyway   [2] Skip all   [3] Install recommended   [4] Decide per package
```

Nothing gets installed until it's verified. No configuration needed.

---

## 🧪 Try This in Your Project

```bash
npx infynon pkg audit
```

You might be surprised what's already inside your dependencies.

---

## 🚨 Why INFYNON Exists

Modern development is broken:

- **AI suggests packages** — blindly installed with one command
- **Supply chain attacks are rising** — legitimate packages get hijacked
- **CVEs are discovered after installation** — damage already done

You move fast. Security doesn't keep up. INFYNON fixes that gap.

---

## 📦 What INFYNON Does

### 1. Dependency Firewall *(most used)*

Scan and block vulnerable packages **before** they touch your system. Works as a drop-in wrapper around your existing package manager.

```bash
infynon pkg npm install express       # instead of: npm install express
infynon pkg cargo add serde           # instead of: cargo add serde
infynon pkg pip install requests      # instead of: pip install requests
```

Supports **14 ecosystems**: `npm • yarn • pnpm • bun • pip • uv • poetry • cargo • go • gem • composer • nuget • hex • pub`

### 2. Network Firewall *(advanced)*

A self-hosted reverse proxy WAF. Sits between the internet and your backend — filtering HTTP traffic in real time with a TUI dashboard.

```
Internet → INFYNON WAF → Your App Server
```

### 3. Weave — API Flow Testing *(new)*

Model your entire API as a directed graph of HTTP requests. Run multi-step test flows, thread authentication tokens automatically between nodes, and run built-in security probes — all from the terminal.

```bash
# Set your API base URL once
infynon weave env set BASE_URL http://localhost:8001

# Create nodes (AI-generated from a description)
infynon weave node create --ai "POST /auth/login with email and password, extracts token"
infynon weave node create --ai "POST /cart/create extracts cart_id"

# Wire them into a flow
infynon weave flow create "checkout" --ai "login then create cart then checkout"

# Run the flow — live step-by-step output
infynon weave flow run checkout

# Run security probes (auth bypass, rate limit, SQL injection)
infynon weave ai probe checkout

# Open the TUI dashboard
infynon weave tui
```

```
[POST /auth/login] ──token──▶ [POST /cart/create] ──cart_id──▶ [POST /checkout]
      ↑ asks for email/password       ↑ uses token                  ↑ uses token + cart_id
```

---

## 👥 Who Is This For

- **Vibe coders & AI coding users** (Claude Code, Cursor, Copilot) — AI suggests and installs packages, INFYNON intercepts and verifies them before anything touches disk
- **AI agents running autonomously** — `--agent` mode emits structured JSON so agents can parse results and react without screen-scraping
- **Backend engineers** managing APIs exposed to the internet
- **DevOps / security-conscious teams** enforcing CVE policies in CI
- **Anyone installing dependencies** without auditing every transitive package

---

## 🚀 Core Capabilities

### 🔐 Dependency Security

- **Pre-install CVE scanning** via OSV.dev — before the package hits your disk
- **Blocks vulnerable packages** — interactive decision layer or fully automated
- **Auto-fix suggestions** — upgrade to the nearest safe version automatically
- **15+ lock file parsers** — works with your existing project, no setup
- **CI enforcement** — non-zero exit on violation, configurable severity threshold
- **`--agent` mode** — machine-readable JSON output for AI agents and CI parsers

### 🛡️ Network Protection

- **Reverse proxy WAF** — SQL injection, XSS, path traversal, command injection detection
- **Rate limiting** — per-IP, per-path, and global sliding window limits
- **IP filtering** — blocklist, allowlist, CIDR ranges, auto-reputation banning
- **Maintenance mode** — toggleable from TUI or config file
- **Multi-upstream routing** — route paths to different backends

### ⚡ Developer Tools

- **Dependency audit & tree visualization** — see the full transitive dependency graph
- **Outdated detection** — across all ecosystems at once
- **Package diff** — compare versions: size, deps, scripts, CVEs
- **Eagle Eye** — scheduled background scanner with email alerts

### 🧪 API Flow Testing (Weave)

- **Node-based test flows** — model your API as a directed graph: each node is one HTTP request
- **Context threading** — extracted values (tokens, IDs) flow automatically between nodes
- **Runtime prompt inputs** — pause and ask for OTPs, passwords, dynamic data mid-flow
- **4 prompt types** — text, boolean, select, multiselect for structured input collection
- **AI flow builder** — describe your scenario in English, Weave wires the graph
- **Security probes** — auth bypass, rate limit, and SQL injection checks out of the box
- **TUI dashboard** — 10-tab terminal UI: live execution feed, latency profiler, security results, env manager
- **CI ready** — use `--default` values or `--set KEY=val` to run flows fully non-interactively

---

## ⚡ Quick Start

```bash
# Install (recommended)
npm install -g infynon

# Scan your project for CVEs
infynon pkg scan

# Secure install — drop-in for any package manager
infynon pkg npm install express
infynon pkg cargo add serde
infynon pkg pip install requests

# Auto-fix all vulnerable dependencies
infynon pkg fix --auto

# Deep audit with dependency tree
infynon pkg audit

# Dependency health check
infynon pkg doctor
```

---

## 🤖 CI & Automation

All install commands support non-interactive flags — no prompts, works in any pipeline.

```bash
# Fail the build if any critical or high vulnerability is found
infynon pkg npm install express --strict high

# Auto-upgrade to safe versions, skip unfixable packages
infynon pkg npm install express --auto-fix

# Skip all vulnerable packages, install only safe ones
infynon pkg npm install express --skip-vulnerable

# Install everything regardless (audit-only workflows)
infynon pkg npm install express --yes
```

| Flag | Behavior | Exit Code |
|------|----------|-----------|
| `--strict [LEVEL]` | Block if vulnerabilities at or above level are found | `3` on block |
| `--auto-fix` | Upgrade to safe versions silently, skip if no fix exists | `0` |
| `--skip-vulnerable` | Skip vulnerable packages, install clean ones | `0` |
| `--yes` | Install all packages including vulnerable ones | `0` |

Severity levels: `critical` · `high` · `medium` · `low` · `all`

**GitHub Actions example:**

```yaml
- name: Secure install
  run: infynon pkg npm install --strict high
```

---

## 🤖 AI Agent Mode

Add `--agent` to any command for machine-readable JSON output — designed for AI agents (Claude Code, Cursor), CI parsers, and shell scripts that need structured results.

```bash
# Scan — JSON output, auto-scans all lock files, no interactive prompts
infynon pkg scan --agent

# Install with JSON result
infynon pkg npm install express lodash --agent --strict high
infynon pkg uv add fastapi sqlalchemy --agent --auto-fix
infynon pkg cargo add serde tokio --agent --strict high
```

**Scan output:**
```json
{
  "status": "vulnerable",
  "packages_scanned": 142,
  "vulnerabilities": [
    {
      "package": "requests",
      "ecosystem": "PyPI",
      "current_version": "2.28.0",
      "cve_id": "CVE-2023-32681",
      "severity": "MEDIUM",
      "summary": "HTTP library improperly forwards proxy-authorization headers",
      "safe_version": "2.31.0",
      "fix_cmd": "pip install requests==2.31.0"
    }
  ],
  "summary": { "critical": 0, "high": 0, "medium": 1, "low": 0, "informational": 0, "total": 1 }
}
```

**Exit codes in `--agent` mode:**

| Code | Meaning |
|------|---------|
| `0` | Clean — no vulnerabilities |
| `1` | Warnings only (LOW / INFORMATIONAL) |
| `2` | Vulnerabilities found (MEDIUM, HIGH, or CRITICAL) |
| `3` | Blocked by `--strict` |

---

## 🔬 Dependency Intelligence

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

## 👀 Demo

### 🔎 Dependency Scan

<p align="center">
  <img src="assets/scan-demo.png" width="750"/>
</p>

### 🛡️ Secure Installation Flow

<p align="center">
  <img src="assets/install-demo.png" width="750"/>
</p>

---

## 🔥 Installation

### npm (recommended — all platforms)

```bash
npm install -g infynon
```

Downloads the right pre-built binary for your OS automatically. Requires Node.js 14+.

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

## 🛡️ Network Firewall Mode

A self-hosted reverse proxy WAF. Drop it in front of any backend — Nginx, Express, FastAPI, anything.

### Quick Start

```bash
# Initialize config
infynon init --port 8080 --upstream-port 3000

# Start firewall with live TUI dashboard
infynon start

# Start headless (no TUI — for servers)
infynon start --headless

# View status, block IPs, check logs
infynon status
infynon block 203.0.113.50
infynon logs --verdict block --count 100
```

### Features

| Feature | Description |
|---------|-------------|
| **WAF Engine** | SQLi, XSS, path traversal, command injection detection |
| **Rate Limiting** | Per-IP, per-path, and global sliding window |
| **IP Filtering** | Blocklist, allowlist, CIDR, auto-reputation banning |
| **Multi-Upstream** | Route paths to different backend services |
| **TUI Dashboard** | 7 real-time views — feed, stats, IP inspector, config editor |
| **Hot Config Reload** | Edit `infynon.toml` — applied within seconds, no restart |
| **Email Alerts** | SMTP/SES notifications on suspicious activity + daily digest |

Full configuration reference, TUI keyboard shortcuts, and advanced options → [cli.infynon.com/docs](https://cli.infynon.com/docs)

---

## 🧪 Weave — API Flow Testing

Test your entire API as a connected flow, not as isolated endpoints. Weave threads authentication tokens and response data automatically between nodes — you never manually copy-paste tokens.

### Quick Start

```bash
# 1. Set your API base URL (once per project)
infynon weave env set BASE_URL http://localhost:8001

# 2. Create nodes from natural language
infynon weave node create --ai "POST /auth/login with email and password, extracts token and user_id"
infynon weave node create --ai "GET /users/{user_id} returns user profile"
infynon weave node create --ai "POST /orders — creates order, extracts order_id"

# 3. Build a flow
infynon weave flow create "user-journey" --ai "login, get profile, create order"

# 4. Run it — tokens flow automatically between nodes
infynon weave flow run user-journey

# 5. Run security probes after a successful run
infynon weave ai probe user-journey
```

### Prompt Inputs — Runtime Values

For values only the user can know at test time (OTPs, passwords, 2FA codes), use prompt inputs. The flow pauses and asks before each relevant node:

```bash
# Add a text prompt (OTP code)
infynon weave node prompt verify-otp add otp_code --label "OTP Code" --secret

# Add a yes/no confirmation
infynon weave node prompt delete-account add confirm --label "Confirm delete?" --type boolean --default false

# Add a single-choice dropdown
infynon weave node prompt create-order add env --label "Environment" --type select --options "staging,production,dev" --default staging

# Add a multi-choice checklist
infynon weave node prompt create-token add scopes --label "Token scopes" --type multiselect --options "read,write,admin" --default "read,write"
```

In the TUI a modal popup appears at each prompt. In the CLI the terminal pauses inline.

### CI / Non-Interactive Mode

```bash
# Option 1: --default on every prompt input (run uses default automatically)
infynon weave node prompt register add email --label "Email" --default "ci@example.com"
infynon weave node prompt register add password --label "Password" --secret --default "Test@1234"

# Option 2: --set to pre-seed all vars before the flow starts
infynon weave flow run auth-flow \
  --set email=ci@example.com \
  --set password=Test@1234 \
  --set full_name="CI Bot"
```

### TUI Dashboard

```bash
infynon weave tui               # open dashboard
infynon weave tui <flow-id>     # open on a specific flow
```

10 tabs: Overview · Flow Graph · Live Execution · Latency Profiler · Security Probes · Env/Ctx · State Inspector · Run Diff · Node Library · Config

| Key | Action |
|-----|--------|
| `1`–`9`, `0` | Switch tabs |
| `Enter` / `r` | Run selected flow or node |
| `r` (tab 3) | Retry the last run |
| `b` (tab 3) | Edit node body, then retry |
| `q` | Quit |
| `?` | Help overlay |

Full command reference → [docs/weave.md](docs/weave.md)

---

## 🦅 Eagle Eye — Scheduled Monitoring

Background CVE scanner for your projects. Runs on a timer, scans all lock files, sends email alerts when vulnerabilities match your risk threshold.

```bash
infynon pkg eagle-eye setup    # Interactive setup (SMTP, paths, risk level, schedule)
infynon pkg eagle-eye start    # Start monitoring in foreground
infynon pkg eagle-eye status   # View current configuration
```

---

## 💡 Core Idea

Security should happen **before execution**, not after damage.

INFYNON enforces that — at the dependency level and the network level.

---

## 🧬 Development Channel

```bash
# Latest features (may have breaking changes)
cargo install --git https://github.com/d4rkNinja/infynon-cli --branch development
```

Watch [github.com/d4rkNinja/infynon-cli/tree/development](https://github.com/d4rkNinja/infynon-cli/tree/development) for updates.

---

## 🔮 Upcoming

- SBOM generation (CycloneDX / SPDX) after every install
- Typosquatting / hallucinated-package detection
- Phantom package detection (AI-hallucinated names)
- License compliance gate (`--deny-license GPL-3.0`)
- Policy-as-code (`.infynon-policy.toml`)
- Geo-IP blocking (MaxMind GeoLite2)
- Webhook alerts (Slack, Discord, Teams)
- TLS termination support
