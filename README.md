<p align="center">
  <h1 align="center">🛡️ INFYNON</h1>
  <p align="center">
    <strong>Network Firewall & Dependency Security Manager</strong><br/>
    Real-time reverse proxy WAF with TUI dashboard + pre-install CVE verification for 14 ecosystems.
  </p>
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
  <img src="https://img.shields.io/badge/lockfiles-15-purple?style=for-the-badge" />
  <img src="https://img.shields.io/badge/version-0.2.0--beta.6.3-orange?style=for-the-badge" />
  <a href="https://www.npmjs.com/package/infynon">
    <img src="https://img.shields.io/npm/v/infynon?style=for-the-badge&logo=npm&label=npm" />
  </a>
  <a href="https://github.com/d4rkNinja/infynon-cli/tree/development">
    <img src="https://img.shields.io/badge/channel-development-blueviolet?style=for-the-badge" />
  </a>
</p>

<p align="center">
  <strong>🚫 AI generates code, installs packages — you don't know what's compromised</strong><br/>
  <strong>✅ INFYNON catches it before it touches your system</strong>
</p>

<p align="center">
  <a href="#-quick-start">Quick Start</a> •
  <a href="#-why-infynon">Why INFYNON</a> •
  <a href="#-how-it-works">How It Works</a> •
  <a href="#-key-features">Features</a> •
  <a href="#-firewall-mode-v020">Firewall</a> •
  <a href="#-installation">Install</a> •
  <a href="#-development-channel">Dev Channel</a> •
  <a href="https://cli.infynon.com/docs">Docs</a>
</p>

---

## ⚡ What is INFYNON?

INFYNON is a **security CLI** written in Rust with two modes:

1. **`infynon` — Network Firewall**: A real-time reverse proxy WAF that sits between the internet and your backend. Inspects, filters, and blocks HTTP traffic with a TUI dashboard. Self-hosted Cloudflare alternative.
2. **`infynon pkg` — Package Security**: A pre-installation firewall for dependencies across 14 ecosystems. Stops compromised packages before they touch your system.

### The Problem INFYNON Solves

**In the age of vibe coding and AI-generated code**, developers and AI tools install packages without knowing if they're compromised. An AI assistant writes `npm install some-package` — but that package could be:

- **Typosquatted** (looks like a real package, isn't)
- **Supply-chain attacked** (legitimate package, now hijacked)
- **Carrying known CVEs** that nobody checked

By the time `npm audit` tells you something is wrong, it's already on your disk. **INFYNON intercepts the install command itself** — scanning before the package ever reaches your machine.

### `infynon` — Network Firewall (v0.2.0)

```
Internet → INFYNON Firewall → Nginx / App Server → Your Application
```

A self-hosted Cloudflare WAF — IP filtering, rate limiting, SQL injection detection, XSS protection, custom rules, maintenance mode, multi-upstream routing, all with a real-time TUI monitor.

### `infynon pkg` — Package Security

A **pre-installation firewall for dependencies**.

Traditional tools like `npm audit`, `pip audit`, or Dependabot:
- scan **after installation** — the damage is already done
- notify you **after exposure** — too late
- require manual remediation — wastes your time

> `infynon pkg` **intercepts the install command**, analyzes dependencies in real-time,
> and blocks or fixes vulnerabilities *before they enter your system*.
> Whether you typed the command or an AI did — INFYNON has your back.

---

## 🎯 Why INFYNON?

### The Problem

- **AI tools generate install commands** — developers approve without checking
- **Vibe coding** means moving fast, not verifying every dependency
- Supply chain attacks are increasing (typosquatting, malicious updates, hijacked packages)
- Traditional tools are **reactive** — they tell you AFTER the compromise
- Your server is exposed to scanners, bots, and attacks 24/7 without a WAF

---

### The Shift

INFYNON introduces **preventive security** at two levels:

| Without INFYNON | With INFYNON |
|-----------------|-------------|
| AI runs `npm install x` → compromised package installed → `npm audit` finds it later | AI runs `infynon pkg npm install x` → CVE detected → blocked before install |
| Internet → Your server → attacked | Internet → INFYNON WAF → filtered → Your server |

This prevents:
- compromised packages entering your codebase (whether installed by you or AI)
- SQL injection, XSS, and bot attacks hitting your backend
- production risks caused by unnoticed CVEs

---

## ⚙️ How It Works

1. **Intercept install command**
  ```bash
   infynon pkg npm install express
  ```

2. **Resolve dependency tree**

   * Detects ecosystem automatically
   * Parses lock files or registry metadata

3. **Query vulnerability database**

   * Uses **OSV.dev** for real CVE intelligence
   * Batch scans all dependencies

4. **Analyze & classify**

   * Severity levels (Critical / High / Medium / Low)
   * Affected versions
   * Suggested safe upgrades

5. **Interactive decision layer**

   * Approve / Skip / Upgrade per package
   * Apply rules globally

6. **Execute safe installation**

   * Only installs approved or fixed packages

---

## 🚀 Key Features

### 🔐 Security First

* **Pre-install CVE scanning**
* Blocks vulnerable packages before execution
* OSV-powered vulnerability intelligence

### 🌍 Multi-Ecosystem Support

Supports **14 ecosystems**:

```
npm • yarn • pnpm • bun
pip • uv • poetry
cargo • go
gem • composer • nuget
hex • pub
```

---

### 🧠 Smart Detection

* Auto-detects ecosystem from project files
* Supports **15+ lock file formats**
* Works without configuration

---

### ⚡ Developer Experience

* Interactive install prompts
* Minimal friction workflow
* Single binary — no setup required

---

### 🛠️ Auto Remediation

* `infynon pkg fix --auto` upgrades all vulnerable dependencies
* `infynon pkg scan --fix high` targets critical + high only
* Suggests safe versions from OSV.dev

---

### 🚫 CI Enforcement

```bash
infynon pkg --strict npm install express
```

* Fails build on any vulnerability
* Ideal for pipelines and teams

---

### 📄 Reporting

* Export results as Markdown or PDF
* Includes CVE details, severity breakdown, upgrade suggestions

---

### 🔬 Dependency Intelligence

| Command | Description |
|---------|-------------|
| `infynon pkg audit` | Recursive dependency tree with CVE annotations |
| `infynon pkg why <pkg>` | Trace why a package is in your tree |
| `infynon pkg outdated` | Detect outdated deps across all ecosystems |
| `infynon pkg diff <pkg> v1 v2` | Compare versions: size, deps, scripts, CVEs |
| `infynon pkg doctor` | Health check: dupes, unused, phantoms, missing locks |
| `infynon pkg size <pkg>` | Install weight and transitive dep count |
| `infynon pkg search <query>` | Cross-ecosystem search (npm, crates, PyPI, …) |
| `infynon pkg clean` | Find and remove unused dependencies |
| `infynon pkg migrate <from> <to>` | Migrate between package managers |
| `infynon pkg eagle-eye setup` | Interactive setup for scheduled CVE monitoring |
| `infynon pkg eagle-eye start` | Start Eagle Eye scheduled vulnerability scanner |
| `infynon pkg eagle-eye status` | Show Eagle Eye configuration and status |
| `infynon pkg eagle-eye enable/disable` | Toggle Eagle Eye monitoring |

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

## ⚡ Quick Start

```bash
# Scan project dependencies for CVEs
infynon pkg scan

# Secure install — any ecosystem
infynon pkg npm install express
infynon pkg cargo add serde
infynon pkg pip install requests

# Auto-fix all vulnerable dependencies
infynon pkg fix --auto

# Deep audit with dependency tree
infynon pkg audit

# Why is a package in the tree?
infynon pkg why lodash

# Check for outdated deps
infynon pkg outdated

# Compare two versions of a package
infynon pkg diff express 4.17.1 4.18.2

# Dependency health check
infynon pkg doctor

# Package size & weight
infynon pkg size express

# Cross-ecosystem search
infynon pkg search http-client

# Remove unused deps
infynon pkg clean

# Migrate npm → pnpm
infynon pkg migrate npm pnpm

# Export PDF report
infynon pkg scan --output pdf

# Strict mode for CI
infynon pkg --strict npm install express
```

---

## 🔥 Installation

### npm (recommended — works on all platforms)

```bash
npm install -g infynon
```

Downloads the right pre-built binary for your OS and architecture automatically. Requires Node.js 14+.

To uninstall completely (removes binary + all config files from `~/.infynon/`):

```bash
npm uninstall -g infynon
```

---

### Linux / macOS (shell script)

```bash
curl -fsSL https://raw.githubusercontent.com/d4rkNinja/infynon-cli/main/scripts/install.sh | bash
```

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/d4rkNinja/infynon-cli/main/scripts/install.ps1 | iex
```

### Using Cargo (build from source)

```bash
cargo install --git https://github.com/d4rkNinja/infynon-cli
```

---

## 🧬 Philosophy

> Security should not be an afterthought.
> It should be enforced by default.

INFYNON ensures that:

* every dependency is verified — whether installed by you or an AI
* every HTTP request is inspected before reaching your backend
* every project remains secure by design, not by afterthought

---

## 🔥 Firewall Mode (v0.2.0)

INFYNON now includes a **real network firewall** — a reverse proxy that inspects, filters, and blocks HTTP traffic in real time.

### Quick Start — Firewall

```bash
# Initialize configuration
infynon init --port 8080 --upstream-port 3000

# Start firewall with TUI dashboard
infynon start

# Start in headless mode (no TUI)
infynon start --headless

# View status
infynon status

# Block an IP
infynon block 203.0.113.50

# View logs
infynon logs --verdict block --count 100

# Validate config
infynon config check

# Show effective config
infynon config show

# Enable/disable rules
infynon rules list
infynon rules enable my-rule
infynon rules disable my-rule
```

### Firewall Features

| Feature | Description |
|---------|-------------|
| **Reverse Proxy** | Sits between internet and your backend, forwards clean traffic |
| **Multi-Upstream Routing** | Route requests to different backends based on path prefix |
| **IP Filtering** | Blocklist, allowlist, CIDR range blocking |
| **Auto-Reputation** | Automatically bans IPs that get blocked too many times |
| **Rate Limiting** | Per-IP, per-path, and global rate limits with sliding window |
| **WAF Engine** | SQL injection, XSS, path traversal, command injection, header injection detection |
| **Custom Rules** | IF-THEN rules with combinable conditions and priority ordering |
| **Maintenance Mode** | Toggle maintenance page for all visitors (from TUI or config) |
| **TUI Dashboard** | 7 real-time views: Dashboard, Live Feed, Blocked, IP Inspector, Rules, Stats, Config |
| **Live Config Editing** | Edit all firewall settings directly from the TUI with instant apply |
| **Hot Config Reload** | Edit `infynon.toml` — changes auto-detected and applied within seconds |
| **Email Alerts** | SMTP/AWS SES notifications on suspicious activity + daily digest reports |
| **JSONL Logging** | Structured event logging with separate blocked request log |
| **Cross-Platform** | Works on Linux, macOS, and Windows |

### Multi-Upstream Routing

Route different paths to different backend services:

```toml
# Default upstream (catches everything not matched by routes)
[upstream]
address = "127.0.0.1"
port = 3000

# Additional upstreams for path-based routing
[[upstreams]]
name = "api-server"
path_prefix = "/api"
address = "127.0.0.1"
port = 4000
strip_prefix = false

[[upstreams]]
name = "static-server"
path_prefix = "/static"
address = "127.0.0.1"
port = 5000
strip_prefix = true
```

### TUI Views

| Key | View | Description |
|-----|------|-------------|
| `1` | Dashboard | Live stats, sparklines, top IPs, top rules, recent events |
| `2` | Live Feed | All requests in real time with search and filtering |
| `3` | Blocked | Blocked requests with rule, stage, and reason details |
| `4` | IP Inspector | Search any IP — see full history, block/unblock from TUI |
| `5` | Rules | Custom rules with hit counts + built-in WAF status |
| `6` | Stats | Traffic breakdown, verdicts, status codes, top paths |
| `7` | Config | Edit all settings directly — save to file with `s` |

### TUI Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `1-7` | Switch between views |
| `q` | Quit TUI (firewall keeps running in headless mode) |
| `/` | Search/filter events |
| `?` | Show help overlay |
| `r` | Reload config from file |
| `m` | Toggle maintenance mode |
| `p` | Pause/resume live feed auto-scroll |
| `f` | Cycle feed filter (All/Blocked/Allowed/Flagged) |
| `b` | Block selected IP (in IP Inspector) |
| `u` | Unblock selected IP (in IP Inspector) |
| `Enter` | Edit config field (in Config view) |
| `s` | Save config to file (in Config view) |

### Configuration

Run `infynon init` for interactive setup, or create `infynon.toml` manually. The config supports:

- **Server**: listen address, port, max connections, timeouts, maintenance mode
- **Upstream**: default backend + multiple path-based upstreams
- **IP Filtering**: blocklist/allowlist modes, CIDR ranges, auto-reputation banning
- **Rate Limiting**: global, per-IP, per-path sliding window limits
- **WAF**: SQLi, XSS, path traversal, command injection, header injection detection
- **Custom Rules**: named rules with conditions (IP, path, method, header, body, content-type, size) and actions (block, allow, flag, rate_limit)
- **Logging**: JSONL access/blocked/alert logs with rotation
- **TUI**: refresh rate, default view, theme, max events in memory
- **Responses**: custom block/rate-limit/maintenance pages
- **Email**: SMTP or AWS SES notifications — alert on block threshold, IP ban, daily digest

Config can be edited from the TUI (view 7) or directly in the file. Changes to the file are auto-detected and hot-reloaded within seconds.

### Email Notifications

Configure email alerts to get notified about suspicious activity:

```toml
[email]
enabled = true
provider = "smtp"                      # "smtp" or "ses"
from = "firewall@example.com"
to = ["admin@example.com"]
alert_on_block_threshold = 100         # Alert when blocks/min > 100
alert_on_ip_ban = true                 # Alert when IP is auto-banned
daily_digest = true                    # Daily summary at 8:00 UTC
daily_digest_hour = 8

[email.smtp]
host = "smtp.gmail.com"
port = 587
username = "your-email@gmail.com"
password = "your-app-password"
tls = true
```

Emails are sent with styled HTML templates showing top blocked IPs, triggered rules, and traffic statistics.

### Eagle Eye — Scheduled Package Monitoring

Eagle Eye is a scheduled vulnerability scanner for `infynon pkg`. It monitors multiple project directories on a timer, scans all lock files for CVEs, and sends email alerts when vulnerabilities matching your risk threshold are found.

```bash
# Interactive setup — configure SMTP, paths, risk level, schedule
infynon pkg eagle-eye setup

# Start monitoring (runs in foreground)
infynon pkg eagle-eye start

# Check current configuration
infynon pkg eagle-eye status

# Enable/disable monitoring
infynon pkg eagle-eye enable
infynon pkg eagle-eye disable
```

**Features:**
- Monitor multiple project directories simultaneously
- Configurable scan interval (default: every 24 hours)
- Risk level threshold: choose which severities trigger alerts (CRITICAL, HIGH, MEDIUM, LOW, ALL)
- SMTP email alerts with styled HTML templates showing per-project vulnerability breakdown
- Enable/disable toggle without losing configuration
- Config stored in `~/.infynon/eagle-eye.toml`

---

## ⚠️ Current Scope

INFYNON currently focuses on:

* **Firewall**: Reverse proxy WAF with real-time TUI monitoring, multi-upstream routing, maintenance mode
* **Package Security**: Known vulnerabilities (CVE-based detection), pre-install interception
* **Cross-platform**: Single binary for Linux, macOS, Windows

---

## 🧪 Development Channel

Want to try the latest features before they hit stable? Follow the **development** branch:

```bash
# Clone the development branch
git clone -b development https://github.com/d4rkNinja/infynon-cli.git
cd infynon-cli

# Build from source
cargo build --release

# Or install directly from development branch
cargo install --git https://github.com/d4rkNinja/infynon-cli --branch development
```

The `development` branch contains:
- Bleeding-edge features still under testing
- Firewall TUI improvements and new views
- Experimental WAF rules and pipeline stages
- Performance optimizations before release

> **Note**: The development branch may have breaking changes. For production use, stick to tagged releases on `main`.

**Watch the branch** for updates: [github.com/d4rkNinja/infynon-cli/tree/development](https://github.com/d4rkNinja/infynon-cli/tree/development)

---

## 🤖 Claude Code Plugin

INFYNON integrates with [Claude Code](https://claude.ai/code) via official plugins. Once installed, Claude Code automatically knows how to help you use every INFYNON command — scanning, fixing, firewall setup, rule authoring, and more.

### Install the Plugin

```bash
# 1. Add the Code Guardian marketplace
/plugin marketplace add d4rkNinja/code-guardian

# 2. Install INFYNON plugins
/plugin install infynon-pkg@d4rkNinja
/plugin install infynon-firewall@d4rkNinja

# 3. Reload to activate
/reload-plugins
```

### What You Get

| Plugin | What Claude Code Learns |
|--------|------------------------|
| **infynon-pkg** | All `infynon pkg` commands — scan, fix, audit, why, outdated, diff, doctor, size, search, clean, migrate, eagle-eye. Auto-triggers when it detects lock files in your project. |
| **infynon-firewall** | All `infynon` firewall commands — init, start, monitor, block/unblock, rules, logs, config. Full `infynon.toml` configuration guide, TUI shortcuts. Auto-triggers when it detects `infynon.toml`. |

Once installed, just ask Claude Code things like:
- *"Scan my project for vulnerabilities"*
- *"Set up a firewall for my Express backend on port 3000"*
- *"Fix all critical CVEs in this project"*
- *"Help me write a custom WAF rule to block scanners"*

Claude Code will recommend and explain the right `infynon` commands.

> **Plugin source**: [github.com/d4rkNinja/code-guardian](https://github.com/d4rkNinja/code-guardian)

---

## 🔮 Upcoming

* Geo-IP blocking (MaxMind GeoLite2 integration)
* SQLite event database for historical queries
* Webhook alerts (Slack, Discord, email)
* LLM-based deep inspection (Layer 3 — local Ollama)
* AI-powered anomaly detection and rule suggestion
* SBOM generation (CycloneDX) after every install
* TLS termination support
* Health check endpoints
