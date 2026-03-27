<p align="center">
  <h1 align="center">🛡️ INFYNON</h1>
  <p align="center">
    <strong>Dependency Firewall for Developers</strong><br/>
    Prevent vulnerable packages from entering your project — before installation.
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
  <img src="https://img.shields.io/badge/version-0.1.0--beta.7-orange?style=for-the-badge" />
</p>

<p align="center">
  <strong>🚫 Traditional tools scan AFTER install</strong><br/>
  <strong>✅ INFYNON blocks vulnerabilities BEFORE install</strong>
</p>

<p align="center">
  <a href="#-quick-start">Quick Start</a> •
  <a href="#-why-infynon">Why INFYNON</a> •
  <a href="#-how-it-works">How It Works</a> •
  <a href="#-key-features">Features</a> •
  <a href="#-installation">Install</a> •
  <a href="docs/commands.md">Commands</a>
</p>

---

## ⚡ What is INFYNON?

INFYNON is a **universal package security CLI** that acts as a **pre-installation firewall for dependencies**.

Modern development relies heavily on third-party packages — but every install introduces risk.

Most tools like `npm audit`, `pip audit`, or Dependabot:
- scan **after installation**
- notify you **after exposure**
- require manual remediation

INFYNON changes this workflow.

> It **intercepts package installation**, analyzes dependencies in real-time,  
> and blocks or fixes vulnerabilities *before they enter your system*.

---

## 🎯 Why INFYNON?

### The Problem

- Developers install packages blindly
- Vulnerabilities are discovered **too late**
- Supply chain attacks are increasing (typosquatting, malicious updates)
- Existing tools are **reactive, not preventive**

---

### The Shift

INFYNON introduces a **"shift-left security model"**:

| Traditional Flow | INFYNON Flow |
|-----------------|-------------|
| Install → Scan → Fix | Scan → Decide → Install |

This simple shift prevents:
- vulnerable dependencies entering your codebase
- production risks caused by unnoticed CVEs
- wasted time on post-install fixes

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

### Linux / macOS

```bash
curl -fsSL https://raw.githubusercontent.com/d4rkNinja/infynon-cli/main/scripts/install.sh | bash
```

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/d4rkNinja/infynon-cli/main/scripts/install.ps1 | iex
```

### Using Cargo

```bash
cargo install --git https://github.com/d4rkNinja/infynon-cli
```

---

## 🧬 Philosophy

> Security should not be an afterthought.
> It should be enforced by default.

INFYNON ensures that:

* every dependency is verified
* every install is intentional
* every project remains secure by design

---

## ⚠️ Current Scope

INFYNON currently focuses on:

* Known vulnerabilities (CVE-based detection)
* Pre-install interception
* Dependency-level security

---

## 🔮 Upcoming

* LLM-based deep inspection (Layer 3 — local Ollama)
* Firewall daemon & real-time monitoring dashboard
* SBOM generation (CycloneDX) after every install
* Team policies & configuration file
* `infynon pkg bun add` live support

