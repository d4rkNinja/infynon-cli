# INFYNON CLI

INFYNON is a security-first CLI for package intelligence, API flow testing, and repository memory.

This public repository is the official distribution channel for INFYNON binaries, installers, checksums, and user documentation.

The source code is not included in this repository.

## Install

macOS and Linux:

```bash
curl -fsSL https://raw.githubusercontent.com/d4rkNinja/infynon-cli/main/install.sh | bash
```

Windows:

```powershell
iwr https://raw.githubusercontent.com/d4rkNinja/infynon-cli/main/install.ps1 -useb | iex
```

npm:

```bash
npm install -g infynon
```

Go wrapper:

```bash
go install github.com/d4rkNinja/infynon-cli/go/cmd/infynon@latest
```

## Docs

- [docs/overview.md](docs/overview.md)
- [docs/install.md](docs/install.md)
- [docs/commands.md](docs/commands.md)
- [docs/verification.md](docs/verification.md)

## Release Assets

Each release includes:

- Windows x64 binary
- Linux x64 binary
- Linux arm64 binary
- macOS x64 binary
- macOS arm64 binary
- `checksums.txt`

## Repository Contents

This public repo contains:

- GitHub release assets
- installer scripts
- npm wrapper package
- Go wrapper package
- end-user documentation

This public repo does not contain:

- Rust source code
- internal CI/CD code beyond distribution wrappers
- proprietary implementation details

## Verify Downloads

Use the `checksums.txt` file attached to each GitHub Release to verify binaries before installation.
