# Install INFYNON

INFYNON is distributed as prebuilt native binaries. The public distribution repo includes installers, npm/go wrappers, docs, and release assets; the core Rust implementation is not included.

## Recommended

### macOS and Linux

```bash
curl -fsSL https://raw.githubusercontent.com/d4rkNinja/infynon-cli/main/install.sh | bash
```

Use `INSTALL_DIR` when you do not want the default install directory:

```bash
INSTALL_DIR="$HOME/.local/bin" curl -fsSL https://raw.githubusercontent.com/d4rkNinja/infynon-cli/main/install.sh | bash
```

### Windows PowerShell

```powershell
iwr https://raw.githubusercontent.com/d4rkNinja/infynon-cli/main/install.ps1 -useb | iex
```

The Windows installer places binaries under `%USERPROFILE%\.infynon\bin` and updates the user `PATH` when needed.

### npm

```bash
npm install -g infynon
```

The npm package is a thin installer wrapper. It downloads the matching release binary during `postinstall`. See [npm-install.md](npm-install.md) for npm provenance and platform-package details.

### Go wrapper

```bash
go install github.com/d4rkNinja/infynon-cli/go/cmd/infynon@latest
```

The Go wrapper installs a small launcher that resolves the current platform and delegates to the native binary.

## Secure Install Syntax

Use `pkg` in front of native package managers when you want dependency checks before the install completes:

```bash
infynon pkg npm install express
infynon pkg npm install express --strict high
infynon pkg pip install requests --strict high
infynon pkg cargo add serde_json --strict high
```

Strict mode blocks vulnerable packages at or above the requested severity threshold.

## Exit Codes

Package install workflows use this contract:

- `0` success
- `1` command, warning, or nonblocking failure
- `2` lookup or upstream scan failure
- `3` strict policy block
- `4` non-interactive decision required

Use `--json` for machine-readable output and `--no-input` in CI when prompts are not allowed.

## Verify

```bash
infynon --help
infynon pkg --help
infynon weave --help
infynon trace --help
```

GitHub Releases include `checksums.txt` for SHA-256 verification. See [verification.md](verification.md).

## Troubleshooting

- Restart your terminal after a Windows install so the updated user `PATH` is loaded.
- Confirm `%USERPROFILE%\.infynon\bin` or your custom install directory is on `PATH`.
- Allow access to `github.com`, `api.github.com`, and `objects.githubusercontent.com` when downloads are blocked.
- See [windows-troubleshooting.md](windows-troubleshooting.md) for Windows-specific fixes.
