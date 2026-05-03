# infynon

INFYNON is a security-first CLI for package intelligence, API flow testing, repository memory, and bounded AI task execution.

This npm package is the official installer wrapper for the INFYNON native binary. It downloads the matching prebuilt binary from GitHub Releases during installation.

## Install

```bash
npm install -g infynon
```

## Command Areas

| Area | Command | Purpose |
|---|---|---|
| Package intelligence | `infynon pkg` | Scan dependencies, inspect package risk, audit package changes, and support safer install workflows. |
| API flow testing | `infynon weave` | Run multi-step API flows from the terminal. |
| Repository memory | `infynon trace` | Preserve handoff notes, branch context, package ownership, and repo memory. |
| Agent task contracts | `infynon task` | Create GCCD task briefs with a goal, context, constraints, and completion criteria for AI agents. |

## Supported Platforms

- Windows x64
- Linux x64
- Linux arm64
- macOS x64
- macOS arm64

## Quick Start

```bash
infynon --help
infynon pkg scan
infynon pkg audit
infynon weave flow run checkout
infynon trace tui
infynon task create task_001 --mutate --workspace . --prompt "Ship the settings API patch"
```

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
- https://github.com/d4rkNinja/infynon-cli/blob/main/docs/gccd.md
- https://github.com/d4rkNinja/infynon-cli/releases

## Source Availability

This package distributes the INFYNON binary and installer wrapper only. The Rust source code is proprietary and is not bundled in this package.
