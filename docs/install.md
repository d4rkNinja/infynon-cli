# Install INFYNON

INFYNON can be installed from GitHub Releases, npm, or the Go wrapper. All install methods resolve to the same native binary release assets.

## Recommended Install Methods

### macOS and Linux

```bash
curl -fsSL https://raw.githubusercontent.com/d4rkNinja/infynon-cli/main/install.sh | bash
```

The installer detects the current operating system and CPU architecture, downloads the matching release binary, installs `infynon`, and creates the `infynon-pkg` alias when supported.

Default install directory:

```text
/usr/local/bin
```

Override the install directory:

```bash
INSTALL_DIR="$HOME/.local/bin" curl -fsSL https://raw.githubusercontent.com/d4rkNinja/infynon-cli/main/install.sh | bash
```

### Windows PowerShell

```powershell
iwr https://raw.githubusercontent.com/d4rkNinja/infynon-cli/main/install.ps1 -useb | iex
```

The installer downloads the Windows x64 binary into:

```text
%USERPROFILE%\.infynon\bin
```

It also adds that directory to the user `PATH` if it is not already present.

### npm

```bash
npm install -g infynon
```

The npm package is a small installer wrapper. It downloads the native binary for the current platform during `postinstall`.

Use npm when:

- your team already standardizes global CLI installation through npm
- you want a familiar `npm install -g` flow for developers
- you need the `infynon` and `infynon-pkg` commands on PATH through npm's global bin directory

### Go wrapper

```bash
go install github.com/d4rkNinja/infynon-cli/go/cmd/infynon@latest
```

The Go wrapper installs a small launcher. On first run, it downloads the native INFYNON binary for the current platform and then delegates execution to that binary.

Use the Go wrapper when:

- Go is already available in your development environment
- you want installation through `go install`
- you want the launcher managed under `GOBIN` or `GOPATH/bin`

## Manual Download

Manual downloads are available from GitHub Releases:

```text
https://github.com/d4rkNinja/infynon-cli/releases
```

Choose the asset matching your platform:

| Platform | Asset |
|---|---|
| Windows x64 | `infynon-x86_64-pc-windows-msvc.exe` |
| Linux x64 | `infynon-x86_64-unknown-linux-musl` |
| Linux arm64 | `infynon-aarch64-unknown-linux-musl` |
| macOS Intel | `infynon-x86_64-apple-darwin` |
| macOS Apple Silicon | `infynon-aarch64-apple-darwin` |

## Verify Installation

```bash
infynon --help
infynon pkg --help
infynon weave --help
infynon trace --help
```

## Troubleshooting

### Command not found

Make sure the install directory is on `PATH`.

macOS/Linux example:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

Windows users may need to restart the terminal after the installer updates the user `PATH`.

### Unsupported platform

Prebuilt binaries are provided for the supported platform list above. Other platforms require a supported release asset before they can use the public binary distribution.

### Download blocked by proxy or firewall

Allow access to:

```text
github.com
api.github.com
objects.githubusercontent.com
```

## Next Steps

- [Command Guide](commands.md)
- [Verification Guide](verification.md)
