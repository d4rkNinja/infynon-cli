# Install INFYNON

## macOS and Linux

```bash
curl -fsSL https://raw.githubusercontent.com/d4rkNinja/infynon-cli/main/install.sh | bash
```

## Windows

```powershell
iwr https://raw.githubusercontent.com/d4rkNinja/infynon-cli/main/install.ps1 -useb | iex
```

## npm

```bash
npm install -g infynon
```

The npm package downloads the matching native binary for the current platform.

## Go

```bash
go install github.com/d4rkNinja/infynon-cli/go/cmd/infynon@latest
```

The Go wrapper downloads the matching release binary on first run.
