# INFYNON CLI

INFYNON is a security-first command-line tool for teams that need dependency visibility, API workflow validation, and durable repository context in one place.

This repository is the public distribution channel for INFYNON CLI. It contains installers, package-manager wrappers, user documentation, release checksums, and prebuilt binaries. The proprietary Rust source code is not included.

## What INFYNON Solves

Modern development teams move across package managers, API surfaces, and AI-assisted code changes quickly. INFYNON focuses on the parts of that workflow where context usually gets lost:

- dependency risk before and after installation
- package audit, explanation, remediation, and monitoring workflows
- multi-step API flows that need shared state, runtime prompts, and repeatable validation
- repository memory for handoffs, branch context, package ownership, and provenance

INFYNON is organized around three command areas:

| Area | Command | Purpose |
|---|---|---|
| Package intelligence | `infynon pkg` | Scan dependencies, inspect risk, explain packages, audit projects, and support safer install workflows. |
| API flow testing | `infynon weave` | Model, execute, and validate multi-step API flows from the terminal. |
| Repository memory | `infynon trace` | Store, retrieve, and inspect structured repo context, notes, ownership, and handoff history. |

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

### Go wrapper

```bash
go install github.com/d4rkNinja/infynon-cli/go/cmd/infynon@latest
```

## Quick Start

```bash
infynon --help
infynon pkg scan
infynon pkg audit
infynon weave flow run checkout
infynon trace init
infynon trace tui
```

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
- [Verification Guide](docs/verification.md)

## Repository Contents

This public repository contains:

- installation scripts
- npm package wrapper
- Go package wrapper
- release binaries and checksums
- end-user documentation

This public repository does not contain:

- Rust source code
- internal product logic
- proprietary scanner or workflow implementation
- private build internals

## Security and Integrity

Release binaries are distributed through GitHub Releases and accompanied by `checksums.txt`. Verify downloaded assets before deploying them into managed environments.

Security issues should be reported privately to the INFYNON maintainers. Do not publish undisclosed vulnerabilities in public issues.

## License

INFYNON binaries are distributed under the proprietary binary license included in this repository. The public wrappers and docs exist to install and operate the product; they do not make the proprietary source code open source.
