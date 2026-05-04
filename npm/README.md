# infynon

[![npm version](https://img.shields.io/npm/v/infynon?style=flat-square&logo=npm)](https://www.npmjs.com/package/infynon)
[![npm downloads](https://img.shields.io/badge/dynamic/json?style=flat-square&logo=npm&label=npm%20downloads&query=%24.downloads&url=https%3A%2F%2Fapi.npmjs.org%2Fdownloads%2Fpoint%2F2025-05-04%3A2050-12-31%2Finfynon&cacheSeconds=3600)](https://www.npmjs.com/package/infynon)
[![GitHub release](https://img.shields.io/github/v/release/d4rkNinja/infynon-cli?style=flat-square&logo=github)](https://github.com/d4rkNinja/infynon-cli/releases)
[![Agent control plane](https://img.shields.io/badge/agent%20control%20plane-Codex%20%7C%20Claude%20%7C%20Gemini-7c3aed?style=flat-square)](https://github.com/d4rkNinja/infynon-cli/blob/main/docs/agent-control-plane.md)
[![Package security](https://img.shields.io/badge/pkg-secure%20installs-ef4444?style=flat-square)](https://github.com/d4rkNinja/infynon-cli/blob/main/docs/commands.md)

INFYNON is a terminal control plane for agentic engineering: multi-agent workspace/task orchestration, package intelligence, API flow testing, and repository memory in one native CLI.

This npm package is the official installer wrapper for the INFYNON native binary. It downloads the matching prebuilt binary from GitHub Releases during installation.

## Install

```bash
npm install -g infynon
```

## Command Areas

| Area | Command | Purpose |
|---|---|---|
| Agent control plane | `infynon workspace`, `infynon task`, `infynon coding` | Coordinate Codex, Claude Code, Gemini CLI, and child-agent sessions through durable workspace and task records. |
| Package intelligence | `infynon pkg` | Scan dependencies, inspect package risk, audit package changes, and support safer install workflows. |
| API flow testing | `infynon weave` | Run multi-step API flows from the terminal. |
| Repository memory | `infynon trace` | Preserve handoff notes, branch context, package ownership, and repo memory. |
| Agent task contracts | GCCD briefs | Create work contracts with a goal, context, constraints, and completion criteria for AI agents. |

## Supported Platforms

- Windows x64
- Linux x64
- Linux arm64
- macOS x64
- macOS arm64

## Quick Start

```bash
infynon --help
infynon workspace agent-root-show
infynon pkg scan
infynon pkg audit
infynon weave flow run checkout
infynon trace tui
infynon task create task_001 --mutate --workspace app --agent codex --prompt "Ship the settings API patch"
```

## Agent Control Plane

Use INFYNON when one lead session needs to coordinate child coding agents.

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

Good fit:

- parent and child agent work
- Codex, Claude Code, and Gemini CLI sessions launched from the right workspace
- task retries where context and completion criteria must stay intact
- reviewable handoffs between agents and humans

## GCCD Task Contracts

INFYNON tasks are structured as execution contracts, not loose prompts:

- Goal: what outcome should exist
- Context: what the agent needs to know
- Constraints: what must not be changed or broken
- Done When: how completion is verified

## Alternative Install Methods

macOS and Linux:

```bash
curl -fsSL https://raw.githubusercontent.com/d4rkNinja/infynon-cli/main/install.sh | bash
```

Windows:

```powershell
iwr https://raw.githubusercontent.com/d4rkNinja/infynon-cli/main/install.ps1 -useb | iex
```

Go wrapper:

```bash
go install github.com/d4rkNinja/infynon-cli/go/cmd/infynon@latest
```

## Documentation

- https://github.com/d4rkNinja/infynon-cli/tree/main/docs
- https://github.com/d4rkNinja/infynon-cli/blob/main/docs/agent-control-plane.md
- https://github.com/d4rkNinja/infynon-cli/blob/main/docs/gccd.md
- https://github.com/d4rkNinja/infynon-cli/releases

## Source Availability

This package distributes the INFYNON binary and installer wrapper only. The Rust source code is proprietary and is not bundled in this package.
