# INFYNON

INFYNON is a CLI for:

- package intelligence with `infynon pkg`
- API flow testing with `infynon weave`
- shared coding memory with `infynon loom`

[![npm](https://img.shields.io/npm/v/infynon?style=flat-square&logo=npm)](https://www.npmjs.com/package/infynon)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](https://github.com/d4rkNinja/infynon-cli/blob/main/LICENSE)
[![GitHub](https://img.shields.io/badge/source-GitHub-black?style=flat-square&logo=github)](https://github.com/d4rkNinja/infynon-cli)
[![Docs](https://img.shields.io/badge/docs-cli.infynon.com-14b8a6?style=flat-square)](https://cli.infynon.com/docs)
[![Claude Code Skill](https://img.shields.io/badge/Claude%20Code-code--guardian-7c3aed?style=flat-square)](https://github.com/d4rkNinja/code-guardian)

Website: `https://cli.infynon.com`
Recommended Loom skill: `https://github.com/d4rkNinja/code-guardian`

## Install

```bash
npm install -g infynon
```

This package downloads the matching native binary for your OS and architecture.

## Why INFYNON Exists

INFYNON was created because modern repos usually hit three problems at the same time:

- dependencies move faster than teams can review them
- API testing breaks when workflows span multiple requests
- context gets lost between branches, PRs, and different machines

Instead of solving only one of those, INFYNON groups them under one CLI.

## Quick View

| Area | Command | Use It When You Need |
|---|---|---|
| Package intelligence | `infynon pkg` | scanning, safe installs, remediation, monitoring |
| API flow testing | `infynon weave` | stateful API workflows, validation, probes |
| Shared coding memory | `infynon loom` | handoffs, package ownership, repo memory, TUI inspection |

## Best With Claude Code

Loom works best with the `code-guardian` skill when you want Claude Code to pull the latest shared memory before work and update it again after the task.

- skill repo: `https://github.com/d4rkNinja/code-guardian`
- good fit for Claude Code hook-based Loom workflows
- gives Loom a practical agent-side bridge instead of leaving memory updates fully manual

## Comparison Table

| Workflow Need | Typical Pain | INFYNON Answer |
|---|---|---|
| dependency safety | install first, understand later | `pkg` scans, audits, and supports stricter install workflows |
| API flow confidence | one request works, the full workflow fails | `weave` models and runs the whole flow |
| team memory | context is scattered and stale | `loom` keeps it structured, queryable, and inspectable |

## Command Areas

### `infynon pkg`

Friendly summary:
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

Friendly summary:
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

### `infynon loom`

Friendly summary:
Use this when the problem is team context and repo memory.

- canonical, team, and user memory layers
- Redis or SQL backends
- package notes that can identify who introduced a compromised dependency
- sync, retrieve, compact, and TUI inspection
- designed to pair with the `code-guardian` Claude Code skill

```bash
infynon loom init --owner team --user alien
infynon loom source add-sql team-db --engine sqlite --url sqlite://.infynon/loom/loom.db --user alien --default
infynon loom note add repo-handoff --title "Auth changed" --body "Refresh moved into middleware"
infynon loom sync --direction both
```

Claude Code companion:
`https://github.com/d4rkNinja/code-guardian`

## Backend Choice For Loom

| Backend | Better For |
|---|---|
| Redis | fast live retrieval, active session state, lower-latency coordination |
| SQL | durable structured history, stronger filtering, canonical memory |

## Documentation

- docs home: `https://cli.infynon.com/docs`
- root README: `README.md`
- command reference: `docs/commands.md`
- Loom guide: `docs/loom.md`
- Weave guide: `docs/weave.md`
- Claude Code Loom skill: `https://github.com/d4rkNinja/code-guardian`

## Comparison Blogs

Useful reading if you are comparing INFYNON to existing tools or workflows:

- one CLI vs fragmented tooling:
  `https://cli.infynon.com/blog/why-infynon-over-fragmented-tooling`
- `pkg` vs `npm audit`:
  `https://cli.infynon.com/blog/infynon-vs-npm-audit`
- `pkg` vs `Snyk CLI`:
  `https://cli.infynon.com/blog/infynon-vs-snyk-cli`
- `pkg` vs `Socket.dev`:
  `https://cli.infynon.com/blog/infynon-vs-socket-dev`
- Loom product story:
  `https://cli.infynon.com/blog/why-i-built-loom`
- coding memory problem story:
  `https://cli.infynon.com/blog/agentic-coding-context-problem`

## Recommended Stack

```text
INFYNON CLI + Loom + code-guardian
```

Use that stack when you want:

- package risk scanning with `pkg`
- workflow-level API testing with `weave`
- shared repo memory with `loom`
- Claude Code automation around Loom retrieval and updates through `code-guardian`

## License

MIT
