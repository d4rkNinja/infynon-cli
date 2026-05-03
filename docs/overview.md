# INFYNON Overview

INFYNON is a production CLI for package intelligence, API flow validation, repository memory, and bounded AI task execution. It is built for teams that work quickly across dependency updates, backend API changes, and AI-assisted development tasks where context needs to remain inspectable after the work moves on.

## Product Areas

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

INFYNON is distributed as prebuilt binaries through GitHub Releases. This public repository includes installation scripts, package-manager wrappers, release checksums, and documentation.

The Rust source code is proprietary and is not included in this repository.

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
- [GCCD Task Contracts](gccd.md)
- [Verification Guide](verification.md)
