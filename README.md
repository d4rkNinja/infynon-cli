# INFYNON CLI

[![GitHub release](https://img.shields.io/github/v/release/d4rkNinja/infynon-cli?style=flat-square&logo=github)](https://github.com/d4rkNinja/infynon-cli/releases)
[![npm version](https://img.shields.io/npm/v/infynon?style=flat-square&logo=npm)](https://www.npmjs.com/package/infynon)
[![npm downloads](https://img.shields.io/badge/dynamic/json?style=flat-square&logo=npm&label=npm%20downloads&query=%24.downloads&url=https%3A%2F%2Fapi.npmjs.org%2Fdownloads%2Fpoint%2F2025-05-04%3A2050-12-31%2Finfynon&cacheSeconds=3600)](https://www.npmjs.com/package/infynon)
[![Agent control plane](https://img.shields.io/badge/agent%20control%20plane-Codex%20%7C%20Claude%20%7C%20Gemini-7c3aed?style=flat-square)](docs/agent-control-plane.md)
[![Package security](https://img.shields.io/badge/pkg-secure%20installs-ef4444?style=flat-square)](docs/commands.md)
[![API flows](https://img.shields.io/badge/weave-API%20flows-0ea5e9?style=flat-square)](docs/commands.md)
[![Repo memory](https://img.shields.io/badge/trace-repo%20memory-10b981?style=flat-square)](docs/commands.md)

INFYNON is a terminal control plane for agentic engineering: multi-agent workspace/task orchestration, package intelligence, API workflow validation, durable repository context, and release-ready local automation in one native CLI.

This repository is the public distribution channel for INFYNON CLI. The public distribution repo includes installers, npm/go wrappers, docs, and release assets; the core Rust implementation is not included.

## What INFYNON Solves

Modern development teams move across package managers, API surfaces, and AI-assisted code changes quickly. INFYNON focuses on the parts of that workflow where context usually gets lost:

- Codex, Claude Code, Gemini CLI, and child-agent assignments need durable task boundaries
- agents need to start in the right workspace folder with the right model and prompt
- dependency risk before and after installation
- package audit, explanation, remediation, and monitoring workflows
- multi-step API flows that need shared state, runtime prompts, and repeatable validation
- repository memory for handoffs, branch context, package ownership, and provenance
- GCCD task contracts that turn vague AI work requests into executable, bounded, and verifiable task briefs

INFYNON is organized around five command areas:

| Area | Command | Purpose |
|---|---|---|
| Agent control plane | `infynon workspace`, `infynon task`, `infynon coding` | Coordinate Codex, Claude Code, Gemini CLI, and child-agent sessions through durable workspace and task records. |
| Package intelligence | `infynon pkg` | Scan dependencies, inspect risk, explain packages, audit projects, and support safer install workflows. |
| API flow testing | `infynon weave` | Model, execute, and validate multi-step API flows from the terminal. |
| Repository memory | `infynon trace` | Store, retrieve, and inspect structured repo context, notes, ownership, and handoff history. |
| Agent task contracts | GCCD briefs | Give AI work a clear goal, boundaries, context, and completion criteria. |

## Install

### macOS and Linux

```bash
curl -fsSL https://raw.githubusercontent.com/d4rkNinja/infynon-cli/main/install.sh | bash
```

### Windows

```powershell
iwr https://raw.githubusercontent.com/d4rkNinja/infynon-cli/main/install.ps1 -useb | iex
```

### npm

```bash
npm install -g infynon
```

The npm package is configured for npm provenance and can use optional platform packages such as `@infynon/cli-win32-x64`, `@infynon/cli-linux-x64`, and `@infynon/cli-darwin-arm64` for native binaries.

### Go wrapper

```bash
go install github.com/d4rkNinja/infynon-cli/go/cmd/infynon@latest
```

## Quick Start

```bash
infynon --help
infynon workspace agent-root-show
infynon pkg scan
infynon pkg audit
infynon weave flow run checkout
infynon trace init
infynon trace tui
infynon task create task_001 --mutate --workspace app --agent codex --prompt "Ship the settings API patch"
```

## Agent Control Plane

INFYNON can be used as a parent-agent command center. A lead human or agent creates a workspace, creates child tasks, assigns each task to Codex, Claude Code, or Gemini CLI, and keeps every child result attached to a durable task record.

```bash
infynon workspace agent-root-set --mutate --path D:/Codeverse/infynon-agent
infynon workspace create app --mutate --folder-name web --path D:/Codeverse/app --default

infynon task create task_ui_review \
  --mutate \
  --workspace app \
  --folder-name web \
  --agent claude \
  --prompt "Review the settings UI change. Do not edit backend files. Done when findings are recorded."

infynon coding tui
```

Use this when subagent work needs clear ownership, a real working directory, status tracking, notes, results, and completion criteria instead of scattered terminal tabs and chat messages.

## GCCD Task Contracts

INFYNON tasks are not stored as loose prompts. Task creation can normalize work into a GCCD contract:

- Goal: the outcome the task must produce
- Context: the project, files, APIs, or prior decisions the agent needs to know
- Constraints: boundaries the agent must respect
- Done When: the checks that prove the task is complete

This keeps parent and child agent work reviewable. A child task can be handed to an agent with clear scope instead of an open-ended instruction. See the [GCCD Task Contracts](docs/gccd.md) guide for examples and validation rules.

## Release Assets

Every release publishes these assets:

| Asset | Platform |
|---|---|
| `infynon-x86_64-pc-windows-msvc.exe` | Windows x64 |
| `infynon-x86_64-unknown-linux-musl` | Linux x64 |
| `infynon-aarch64-unknown-linux-musl` | Linux arm64 |
| `infynon-x86_64-apple-darwin` | macOS Intel |
| `infynon-aarch64-apple-darwin` | macOS Apple Silicon |
| `checksums.txt` | SHA-256 checksums for release verification |

## Documentation

- [Overview](docs/overview.md)
- [Install Guide](docs/install.md)
- [Command Guide](docs/commands.md)
- [Agent Control Plane](docs/agent-control-plane.md)
- [AI Agent Workflow](docs/ai-agent-workflow.md)
- [GCCD Task Contracts](docs/gccd.md)
- [npm Install](docs/npm-install.md)
- [Windows Troubleshooting](docs/windows-troubleshooting.md)
- [Verification Guide](docs/verification.md)

## Repository Contents

This public repository contains:

- installation scripts
- npm package wrapper
- Go package wrapper
- release binaries and checksums
- end-user documentation

This public repository does not contain:

- core Rust implementation
- internal product logic
- proprietary scanner or workflow implementation
- private build internals

## Security and Integrity

Release binaries are distributed through GitHub Releases and accompanied by `checksums.txt`. Verify downloaded assets before deploying them into managed environments.

Security issues should be reported privately to the INFYNON maintainers. Do not publish undisclosed vulnerabilities in public issues.

## License

INFYNON binaries are distributed under the proprietary binary license included in this repository. The public wrappers and docs exist to install and operate the product; they do not make the core Rust implementation open source.
