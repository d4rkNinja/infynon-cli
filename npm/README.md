# INFYNON

INFYNON is a CLI for:

- package security with `infynon pkg`
- API flow testing with `infynon weave`
- repo memory & provenance with `infynon trace`

[![npm](https://img.shields.io/npm/v/infynon?style=flat-square&logo=npm)](https://www.npmjs.com/package/infynon)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](https://github.com/d4rkNinja/infynon-cli/blob/main/LICENSE)
[![GitHub](https://img.shields.io/badge/source-GitHub-black?style=flat-square&logo=github)](https://github.com/d4rkNinja/infynon-cli)
[![Docs](https://img.shields.io/badge/docs-cli.infynon.com-14b8a6?style=flat-square)](https://cli.infynon.com/docs)
[![Claude Code](https://img.shields.io/badge/Claude%20Code-code--guardian-7c3aed?style=flat-square)](https://github.com/d4rkNinja/code-guardian)

Website: [cli.infynon.com](https://cli.infynon.com)
Claude Code companion: [d4rkNinja/code-guardian](https://github.com/d4rkNinja/code-guardian)

## Install

### npm (recommended)

```bash
npm install -g infynon
```

This package downloads the matching native binary for your OS and architecture.

### Other install methods

```bash
cargo install infynon                                          # Rust (crates.io)
go install github.com/d4rkNinja/infynon-cli/go/cmd/infynon@latest # Go
curl -fsSL https://raw.githubusercontent.com/d4rkNinja/infynon-cli/main/scripts/install.sh | bash  # Linux/macOS
```

## Good Fit For

- teams doing AI-assisted or high-speed coding
- backend teams testing stateful API workflows
- repos where package ownership and handoff context matter
- developers who want one CLI instead of three disconnected tools

## Why INFYNON Exists

INFYNON was created because modern repos usually hit three problems at the same time:

- dependencies move faster than teams can review them
- API testing breaks when workflows span multiple requests
- provenance gets lost between branches, PRs, and different machines

Instead of solving only one of those, INFYNON groups them under one CLI.

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

## Best With Claude Code

Trace works best with `code-guardian` when you want Claude Code to pull the latest handoff context before work and update it again after the task.

- Claude Code companion: [d4rkNinja/code-guardian](https://github.com/d4rkNinja/code-guardian)
- good fit for Claude Code hook-based Trace workflows
- gives Trace a practical agent-side bridge instead of leaving context updates fully manual

## Comparison Table

| Workflow Need | Typical Pain | INFYNON Answer |
|---|---|---|
| dependency safety | install first, understand later | `pkg` scans, audits, and supports stricter install workflows |
| API flow confidence | one request works, the full workflow fails | `weave` models and runs the whole flow |
| repo provenance | context is scattered and stale | `trace` keeps it structured, queryable, and inspectable |

## Command Areas

### `infynon pkg`

Use this when the problem is packages.

- scan lockfiles for vulnerable packages
- secure install wrapper for multiple ecosystems
- audit, why, outdated, diff, doctor, fix, clean, migrate
- Eagle Eye scheduled package monitoring

```bash
infynon pkg scan
infynon pkg audit
infynon pkg npm install express --strict high
```

### `infynon weave`

Use this when the problem is API behavior across multiple steps.

- create API nodes and flows
- run connected request chains
- import OpenAPI
- prompt for runtime values
- run AI-assisted security probes

```bash
infynon weave env set BASE_URL http://localhost:8001
infynon weave flow create "checkout" --ai "login then create order"
infynon weave flow run checkout
```

### `infynon trace`

Use this when the problem is repo memory, handoff context, and package provenance.

- canonical, team, and user memory layers
- Redis or SQL backends
- package notes that identify who introduced a compromised dependency
- sync, retrieve, compact, and TUI inspection
- designed to pair with the `code-guardian` Claude Code companion
- branch-wise knowledge graph with auto-build from git history
- graph queries: path finding, impact analysis, orphan detection, branch diff
- export to JSON and Graphviz DOT
- interactive graph TUI with entity/edge editing and branch switching

```bash
infynon trace init --owner team --user alien
infynon trace source add-sql team-db --engine sqlite --url sqlite://.infynon/trace/trace.db --user alien --default
infynon trace note add repo-handoff --title "Auth changed" --body "Refresh moved into middleware"
infynon trace sync --direction both
infynon trace tui
infynon trace graph build
infynon trace graph show --branch main
infynon trace graph tui
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
| Branch-wise knowledge graph | ✓ | — | — | — | — |

`✓` = supported · `~` = partial or limited · `—` = not supported

## Backend Choice For Trace

| Backend | Better For |
|---|---|
| Redis | fast live retrieval, active session state, lower-latency coordination |
| SQL | durable structured history, stronger filtering, canonical memory |

## Documentation

- docs home: [cli.infynon.com/docs](https://cli.infynon.com/docs)
- root README: `README.md`
- command reference: `docs/commands.md`
- internal coding-agent orchestration, saved agent root, and current Codex/Claude/Gemini model guide: `docs/ninja-coding.md`
- Trace guide: `docs/trace.md`
- Weave guide: `docs/weave.md`
- Claude Code companion: [d4rkNinja/code-guardian](https://github.com/d4rkNinja/code-guardian)

Agent root setup:

```bash
infynon workspace agent-root-show
infynon workspace agent-root-set --mutate --path D:/Codeverse/infynon-agent
infynon coding tui
infynon coding codex
```

`infynon coding tui` provides form-driven workspace and task management with human-readable results, including workspace delete, folder management, task lifecycle actions, and model-slot edits.

Creating a Codex, Claude, or Gemini task starts that agent immediately unless the task is explicitly queued or blocked. Completing or failing a task closes its recorded agent terminal by default when INFYNON has a PID; pass `--keep-terminal` only when the terminal must stay open.

## Comparison Blogs

- [One CLI vs fragmented tooling](https://cli.infynon.com/blog/why-infynon-over-fragmented-tooling)
- [`pkg` vs `npm audit`](https://cli.infynon.com/blog/infynon-vs-npm-audit)
- [`pkg` vs Snyk CLI](https://cli.infynon.com/blog/infynon-vs-snyk-cli)
- [`pkg` vs Socket.dev](https://cli.infynon.com/blog/infynon-vs-socket-dev)
- [Why Trace exists](https://cli.infynon.com/blog/why-i-built-trace)
- [Why repo memory matters](https://cli.infynon.com/blog/agentic-coding-context-problem)

## Recommended Stack

```text
INFYNON CLI + Trace + code-guardian
```

Use that stack when you want:

- package risk scanning with `pkg`
- workflow-level API testing with `weave`
- structured repo context with `trace`
- Claude Code automation around Trace retrieval and updates through `code-guardian`

## License

MIT
