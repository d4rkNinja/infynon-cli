# INFYNON

[![npm version](https://img.shields.io/npm/v/infynon?style=flat-square&logo=npm)](https://www.npmjs.com/package/infynon)
[![Crates.io](https://img.shields.io/crates/v/infynon?style=flat-square&logo=rust)](https://crates.io/crates/infynon)
[![MIT License](https://img.shields.io/badge/license-MIT-0f172a?style=flat-square)](LICENSE)
[![Docs](https://img.shields.io/badge/docs-cli.infynon.com-14b8a6?style=flat-square)](https://cli.infynon.com/docs)
[![Claude Code](https://img.shields.io/badge/Claude%20Code-code--guardian-7c3aed?style=flat-square)](https://github.com/d4rkNinja/code-guardian)

INFYNON is a Rust CLI for three workflow problems:

- package security
- API flow testing
- repo memory & provenance

If your team installs dependencies fast, tests APIs through real workflows, and keeps losing context across branches, PRs, and machines, INFYNON is built for that exact shape of work.

Website: [cli.infynon.com](https://cli.infynon.com)

Claude Code companion:
[d4rkNinja/code-guardian](https://github.com/d4rkNinja/code-guardian)

## Good Fit For

- teams doing AI-assisted or high-speed coding
- backend teams testing stateful API workflows
- repos where package ownership and handoff context matter
- developers who want one CLI instead of three disconnected tools

## What INFYNON Includes

| Area | Command | Best For | What It Solves |
|---|---|---|---|
| Package Security | `infynon pkg` | scanning, safe installs, remediation, monitoring | risky dependencies, invisible installs, version exposure |
| API Flow Testing | `infynon weave` | multi-step API execution and validation | brittle request scripts, missing flow context, runtime probes |
| Repo Memory & Provenance | `infynon trace` | handoffs, package ownership, branch/PR/file/package notes, TUI inspection | lost context across people, PRs, branches, and machines |

## How the workflow fits together

- `pkg` checks what is entering the system
- `weave` tests how the real API path behaves
- `trace` preserves who changed what, why it changed, and what the team knew at the time

## Why I Built INFYNON

Most tooling only covers one slice of the workflow.

- dependency scanners tell you what is risky
- API tools let you hit endpoints
- notes and handoffs live in chat, PR comments, or someone's head

That leaves a gap.

Modern teams need one place to:

- inspect dependency risk before it spreads
- test behavior across real request chains
- keep structured repo context visible and queryable

That is why INFYNON is organized into three product areas instead of one overloaded command set.

## Recommended With Trace

If you want Trace to feel native inside Claude Code, use `code-guardian` as the companion layer:

- retrieve the latest Trace context before work starts
- write back team or package notes after work ends
- connect it with Claude Code hooks so the memory flow becomes automatic

Practical setup:

```text
Claude Code + code-guardian + INFYNON Trace
```

That gives you:

- `infynon trace` for storage, retrieval, sync, compact, and TUI inspection
- `code-guardian` for agent-side retrieval and update behavior

## Quick Comparison

| Problem | Without INFYNON | With INFYNON |
|---|---|---|
| Package installs | you install first, inspect later | `pkg` lets you scan, audit, and control install-time workflows |
| API verification | isolated requests miss full behavior | `weave` models full flows with context threading |
| Repo context | provenance gets lost in chat and PR comments | `trace` keeps it structured, searchable, and inspectable |

## Product Areas

### `infynon pkg`

Use `pkg` when the question is about dependencies.

What it gives you:

- CVE scanning across 14 ecosystems
- secure install wrapper
- audit / why / outdated / diff / doctor / fix / clean / migrate
- scheduled monitoring with Eagle Eye

```bash
infynon pkg scan
infynon pkg audit
infynon pkg npm install express --strict high
infynon pkg fix --auto
```

### `infynon weave`

Use `weave` when the question is about real API behavior.

What it gives you:

- node-based API flow testing
- context threading between requests
- OpenAPI import
- runtime prompt inputs
- live execution, run diff, and built-in security probes

```bash
infynon weave env set BASE_URL http://localhost:8001
infynon weave node create --ai "POST /auth/login extracts token"
infynon weave flow create "checkout" --ai "login then create order"
infynon weave flow run checkout
infynon weave ai probe checkout
```

### `infynon trace`

Use `trace` when the question is about repo memory and provenance.

What it gives you:

- Redis for fast live retrieval and session-style coordination
- SQL for durable notes, structured queries, and long-term canonical memory
- canonical / team / user memory layers
- PR / branch / file / package notes with package ownership history
- compaction and reconciliation
- TUI-based inspection, note browsing, and package risk ownership
- first-class integration with the `code-guardian` Claude Code companion

```bash
infynon trace init --owner team --user alien
infynon trace source add-sql team-db --engine sqlite --url sqlite://.infynon/trace/trace.db --user alien --default
infynon trace note add repo-handoff --title "Auth changed" --body "Refresh moved into middleware"
infynon trace sync --direction both
infynon trace tui
```

Claude Code companion:
[d4rkNinja/code-guardian](https://github.com/d4rkNinja/code-guardian)

## Head-to-Head Comparison

### `infynon pkg` vs Alternatives

| Feature | infynon pkg | npm audit | Snyk CLI | Socket CLI | OSV-scanner |
|---|:---:|:---:|:---:|:---:|:---:|
| Secure install wrapper | ✓ | — | — | ~ npm only | — |
| 14 ecosystems in one scan | ✓ | — | ~ | ~ | ~ |
| Block installs via strict mode | ✓ | — | — | ~ npm only | — |
| Scheduled CLI monitoring | ✓ | — | ~ server-side | — | — |
| PDF + Markdown report export | ✓ | — | — | — | — |
| Package version diff | ✓ | — | — | — | — |
| Per-package install decisions | ✓ | — | — | — | — |
| Auto-fix / remediation | ✓ | ~ basic | ✓ | ✓ | — |
| No SaaS account required | ✓ | ✓ | — | ~ | ✓ |

### `infynon weave` vs Alternatives

| Feature | infynon weave | Postman | Hoppscotch | Bruno | Insomnia |
|---|:---:|:---:|:---:|:---:|:---:|
| Terminal TUI | ✓ | — | — | — | — |
| Runtime prompts (OTP / 2FA) | ✓ | — | ~ | — | — |
| Built-in AI security probes | ✓ | — | — | — | — |
| AI-assisted flow creation (CLI) | ✓ | ~ GUI only | ~ GUI / alpha | — | — |
| Visual flow graph in terminal | ✓ | — | — | — | — |
| Run diff (side-by-side) | ✓ | — | — | ~ paid | — |
| Context threading between nodes | ✓ | ✓ | ✓ | ✓ | ✓ |
| OpenAPI / Swagger import | ✓ | ✓ | ✓ | ✓ | ✓ |
| Offline, no account required | ✓ | — | ✓ | ✓ | ~ |

### `infynon trace` vs Alternatives

| Feature | infynon trace | GitHub Wiki | Notion | Confluence | Obsidian |
|---|:---:|:---:|:---:|:---:|:---:|
| Native CLI | ✓ | — | ~ 3rd party | ~ ACLI | ✓ |
| Branch / file / package scoping | ✓ | — | — | — | — |
| Package ownership tracking | ✓ | — | — | — | — |
| Redis + SQL backend choice | ✓ | — | — | — | — |
| Multi-layer memory (team / user / canonical) | ✓ | — | — | — | — |
| Terminal TUI inspection | ✓ | — | — | — | — |
| Claude Code native integration | ✓ | — | ~ MCP | — | ~ |
| Structured retrieval by scope | ✓ | — | — | — | — |
| Bidirectional sync via CLI | ✓ | — | ~ | ~ | ✓ |

`✓` = supported · `~` = partial or limited · `—` = not supported

## Command Style

INFYNON keeps the root command simple:

```bash
infynon pkg <subcommand>
infynon weave <subcommand>
infynon trace <subcommand>
```

## Install

### npm

```bash
npm install -g infynon
```

### Linux / macOS

```bash
curl -fsSL https://raw.githubusercontent.com/d4rkNinja/infynon-cli/main/scripts/install.sh | bash
```

### Windows

```powershell
irm https://raw.githubusercontent.com/d4rkNinja/infynon-cli/main/scripts/install.ps1 | iex
```

### Source

```bash
cargo install --git https://github.com/d4rkNinja/infynon-cli
```

## Docs

- docs home: [cli.infynon.com/docs](https://cli.infynon.com/docs)
- command reference: [docs/commands.md](docs/commands.md)
- Trace guide: [docs/trace.md](docs/trace.md)
- Weave guide: [docs/weave.md](docs/weave.md)
- scan guide: [docs/scan.md](docs/scan.md)
- install guide: [docs/install.md](docs/install.md)

Claude Code companion:
- [code-guardian](https://github.com/d4rkNinja/code-guardian)

## Comparison Blogs

- [One CLI vs fragmented tooling](https://cli.infynon.com/blog/why-infynon-over-fragmented-tooling)
- [`pkg` vs `npm audit`](https://cli.infynon.com/blog/infynon-vs-npm-audit)
- [`pkg` vs Snyk CLI](https://cli.infynon.com/blog/infynon-vs-snyk-cli)
- [`pkg` vs Socket.dev](https://cli.infynon.com/blog/infynon-vs-socket-dev)
- [Why Trace exists](https://cli.infynon.com/blog/why-i-built-loom)
- [Why repo memory matters](https://cli.infynon.com/blog/agentic-coding-context-problem)

Claude Code companion:
- [code-guardian](https://github.com/d4rkNinja/code-guardian) — gives Claude Code a practical Trace bridge
