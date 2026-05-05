# npm Install

The `infynon` npm package is the official npm entry point for INFYNON. It is a thin wrapper around the native binary distribution, not a JavaScript reimplementation.

```bash
npm install -g infynon
```

## What Gets Installed

The public distribution repo includes installers, npm/go wrappers, docs, and release assets; the core Rust implementation is not included.

The npm package provides the `infynon` and `infynon-pkg` commands. npm installs the matching optional native package when it is available for the current OS and CPU. If that optional package is unavailable, the wrapper downloads and verifies the matching GitHub Release binary on first launch.

## Optional Platform Packages

Current npm releases can use optional platform packages for native binaries:

- `infynon-windows-x64`
- `infynon-linux-x64`
- `infynon-linux-arm64`
- `infynon-darwin-x64`
- `infynon-darwin-arm64`

npm installs only the optional package matching the current OS and CPU. If optional package installation is unavailable, the wrapper can fall back to the GitHub Release asset for the same version and platform.

## Provenance

INFYNON npm packages are configured for npm provenance when they are published from a public GitHub Actions source repository. npm currently rejects provenance bundles from private GitHub Actions source repositories, so private-source release runs publish without provenance and rely on the GitHub Release manifest plus SHA-256 checksums.

Use provenance together with normal package controls:

- pin versions in CI and managed developer images
- keep lockfiles when installing INFYNON as a project dependency
- review the npm package page for provenance status and package metadata
- verify GitHub Release downloads with `checksums.txt` when installing outside npm

## Troubleshooting

- Use Node.js 18 or newer.
- Run `npm prefix -g` to see where global packages are installed.
- Confirm npm's global bin directory is on `PATH`.
- If a corporate proxy blocks binary download fallback, allow `github.com`, `api.github.com`, and `objects.githubusercontent.com`.
