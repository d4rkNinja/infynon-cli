# INFYNON Overview

INFYNON is a production CLI control plane for agentic engineering. It brings multi-agent workspace/task orchestration, package intelligence, API flow validation, and repository memory into one terminal workflow.

It is built for teams that work quickly across dependency updates, backend API changes, and AI-assisted development tasks where context needs to remain inspectable after the work moves on.

## Product Areas

### Agent control plane

`infynon workspace`, `infynon task`, and `infynon coding` help a lead developer or agent coordinate Codex, Claude Code, Gemini CLI, and child-agent sessions without losing the working directory, task boundary, status, notes, or result.

Typical uses:

- define user-global workspaces and named folders
- store model slots and an agent root folder
- create GCCD task contracts for parent and child work
- launch Codex, Claude Code, or Gemini CLI into the right workspace
- track task status, pid, notes, result, and session metadata
- keep subagent handoffs reviewable after the terminal closes

### Package intelligence

`infynon pkg` helps teams understand dependency risk and package behavior before risky changes spread through a repository.

Typical uses:

- scan project dependencies
- inspect package risk and vulnerability signals
- explain why a package is present
- compare dependency changes
- run audit and remediation workflows
- support stricter install-time package decisions

### API flow testing

`infynon weave` is for workflows where one HTTP request is not enough. It models API flows as connected steps, carries context between requests, and supports repeatable terminal-based validation.

Typical uses:

- create reusable API nodes and flows
- run multi-step API workflows
- pass state between steps
- validate responses
- run flows in CI-oriented output formats
- test API behavior without relying on a GUI collection runner

### Repository memory

`infynon trace` gives teams a structured way to preserve handoffs, branch context, file notes, package ownership, and long-lived project memory.

Typical uses:

- initialize repo-local memory
- add and retrieve structured notes
- sync context across sources
- inspect project knowledge in a terminal UI
- track branch, file, and package context
- preserve useful context for AI-assisted coding sessions

### Agent task contracts

`infynon task` turns AI work requests into GCCD task contracts. GCCD means Goal, Context, Constraints, and Done When. The format gives an agent enough structure to execute the task without losing the boundaries that matter to the repository.

Typical uses:

- create task briefs from plain prompts
- preserve the goal, context, constraints, and completion criteria for review
- pass bounded work to AI agents
- define stricter child-agent work packages
- make retries and handoffs inspectable

## Distribution Model

INFYNON is distributed as prebuilt binaries through GitHub Releases. The public distribution repo includes installers, npm/go wrappers, docs, and release assets; the core Rust implementation is not included.

## Supported Platforms

Prebuilt releases are produced for:

- Windows x64
- Linux x64
- Linux arm64
- macOS Intel
- macOS Apple Silicon

## Operational Expectations

INFYNON is designed to be run locally by developers and automation. It should be treated like any other developer security tool:

- pin versions in repeatable environments
- verify release checksums for managed deployment
- review command output before applying remediation
- run package and API checks in the same workspace where the relevant lockfiles, manifests, or flow definitions live

## Related Docs

- [Install Guide](install.md)
- [Command Guide](commands.md)
- [Agent Control Plane](agent-control-plane.md)
- [AI Agent Workflow](ai-agent-workflow.md)
- [GCCD Task Contracts](gccd.md)
- [npm Install](npm-install.md)
- [Windows Troubleshooting](windows-troubleshooting.md)
- [Verification Guide](verification.md)
