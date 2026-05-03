# INFYNON CLI

INFYNON is distributed here as prebuilt binaries, install scripts, and thin package wrappers.

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

## What is in this repo

- GitHub Releases with Windows, Linux, and macOS binaries
- `install.sh` and `install.ps1`
- npm wrapper package sources
- Go wrapper sources
- checksums for every release

## What is not in this repo

- Rust source code
- internal build logic
- proprietary implementation details

## Verify downloads

Every GitHub Release includes `checksums.txt` so users can validate downloaded binaries before running them.
