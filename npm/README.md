# INFYNON

**Stop trusting installs, traffic, and API flows blindly.**

A security-first CLI — one binary, three shields:

- 📦 **Dependency Firewall** — pre-install CVE scanner across 14 ecosystems
- 🛡️ **Network Firewall** — reverse proxy WAF with real-time TUI dashboard
- 🧪 **API Flow Tester** — node-based integration testing with security probes

[![npm](https://img.shields.io/npm/v/infynon?style=flat-square&logo=npm)](https://www.npmjs.com/package/infynon)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](https://github.com/d4rkNinja/infynon-cli/blob/main/LICENSE)
[![GitHub](https://img.shields.io/badge/source-GitHub-black?style=flat-square&logo=github)](https://github.com/d4rkNinja/infynon-cli)

> ⚠️ AI installs packages. You don't verify them. That's the risk.
> **INFYNON fixes that — blocks threats before they reach your system.**
> Use `--agent` for structured JSON output inside AI agents or CI pipelines.

---

## Install

```bash
npm install -g infynon
```

Downloads the right pre-built native binary for your OS and architecture automatically. Requires Node.js 14+.

**Supported platforms:** Windows x64 · Linux x64 · Linux ARM64 · macOS x64 · macOS ARM64

```bash
npm uninstall -g infynon    # uninstall and clean up all config files
```

---

## Module 1 — `infynon pkg` · Dependency Firewall

Intercepts install commands across **14 ecosystems** and runs a 3-layer CVE check before anything touches your disk.

```bash
# Scan your project's lock files for CVEs
infynon pkg scan

# Secure install — drop-in wrapper around your package manager
infynon pkg npm install express
infynon pkg cargo add serde
infynon pkg pip install requests
infynon pkg yarn add lodash

# Auto-fix all vulnerable dependencies
infynon pkg fix --auto

# Deep audit with full dependency tree
infynon pkg audit

# CI / non-interactive flags
infynon pkg npm install express --strict high       # fail on critical/high (exit 3)
infynon pkg npm install express --auto-fix          # auto-upgrade to safe versions
infynon pkg npm install express --skip-vulnerable   # skip bad packages silently
infynon pkg npm install express --yes               # install everything (audit-only CI)

# AI agent mode — structured JSON for AI tools and CI parsers
infynon pkg scan --agent
infynon pkg npm install express --agent --strict high
infynon pkg uv add fastapi --agent --auto-fix
```

**Ecosystems:** npm · yarn · pnpm · bun · pip · uv · poetry · cargo · go · gem · composer · nuget · hex · pub

---

## Module 2 — `infynon` · Network Firewall

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

## Module 3 — `infynon weave` · API Flow Testing

Test your entire API as a connected flow. Model endpoints as a directed graph — authentication tokens and extracted values thread automatically between nodes.

```bash
# Set your API base URL once
infynon weave env set BASE_URL http://localhost:8001

# Create nodes from natural language
infynon weave node create --ai "POST /auth/login with email and password, extracts token"
infynon weave node create --ai "POST /orders — creates order, extracts order_id"

# Wire into a flow and run
infynon weave flow create "checkout" --ai "login then create order"
infynon weave flow run checkout

# Run security probes (auth bypass, rate limit, SQL injection)
infynon weave ai probe checkout

# Open the 10-tab TUI dashboard
infynon weave tui
```

**Runtime prompt inputs** — pause and ask for OTPs, passwords, and dynamic values mid-flow:

```bash
infynon weave node prompt verify-otp add otp_code --label "OTP Code" --secret
infynon weave node prompt create-order add env --type select --options "staging,production,dev"
infynon weave node prompt confirm-delete add confirm --type boolean --default false
infynon weave node prompt create-token add scopes --type multiselect --options "read,write,admin"
```

**Prompt types:** `text · boolean · select · multiselect`

**CI ready** — use `--default` values or `--set KEY=val` for fully non-interactive runs:

```bash
infynon weave flow run auth-flow --set email=ci@example.com --set password=Test@1234
```

---

## Traditional Testing vs INFYNON Weave

| | Traditional (Postman / pytest) | INFYNON Weave |
|---|---|---|
| **Token handling** | Manual copy-paste between requests | Automatic — extracted values thread forward |
| **Dynamic inputs** | Hardcoded env vars | Runtime prompts (OTP, 2FA, password) |
| **Security testing** | Separate tool (Burp, manual) | Built-in probes: auth bypass, rate limit, SQLi |
| **Flow creation** | Manual configuration | AI-generated from natural language |
| **CI integration** | Complex credential management | `--set KEY=val` or `--default` flags |

---

## Commands Reference

### Package Security
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

### API Flow Testing (Weave)
| Command | Description |
|---------|-------------|
| `infynon weave node create --ai "..."` | Create a node from natural language |
| `infynon weave flow create "name" --ai "..."` | Build a flow from description |
| `infynon weave flow run <id>` | Run a flow with live step output |
| `infynon weave flow run <id> --set key=val` | Pre-seed context vars (skip prompts) |
| `infynon weave ai probe <id>` | Run auth bypass / rate limit / SQLi probes |
| `infynon weave ai explain <id>` | Diagnose the last failed run |
| `infynon weave validate` | Validate all nodes and flows |
| `infynon weave tui` | Open 10-tab TUI dashboard |

---

## Claude Code Integration

INFYNON has dedicated Claude Code plugins at [d4rkNinja/code-guardian](https://github.com/d4rkNinja/code-guardian). Install them so Claude knows how to use every INFYNON command correctly — right mode, right flags, right ecosystem.

```bash
/plugin marketplace add d4rkNinja/code-guardian
/plugin install infynon-pkg@d4rkNinja
/plugin install infynon-firewall@d4rkNinja
/plugin install infynon-weave@d4rkNinja
/reload-plugins
```

Once installed, Claude automatically routes installs through `infynon pkg`, picks the right CI flag, detects lock files, helps write firewall configs, explains CVE findings, and designs API test flows.

| Plugin | Skills included |
|--------|----------------|
| `infynon-pkg` | package-security · cve-triage · eagle-eye-monitor |
| `infynon-firewall` | firewall-setup · attack-response · rule-writer |
| `infynon-weave` | api-testing |

---

## Full Documentation

**[cli.infynon.com/docs](https://cli.infynon.com/docs)**

Source: [github.com/d4rkNinja/infynon-cli](https://github.com/d4rkNinja/infynon-cli)

---

## License

MIT
