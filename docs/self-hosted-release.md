# Self-Hosted Release Pipeline

Use this when GitHub-hosted Actions cannot run, but you have a Linux server that can build and publish INFYNON release assets.

## Server prerequisites

Install these tools on the Linux server:

```bash
sudo apt-get update
sudo apt-get install -y build-essential curl git jq nodejs npm python3 pipx musl-tools mingw-w64 xz-utils unzip
rustup target add x86_64-unknown-linux-musl aarch64-unknown-linux-musl x86_64-pc-windows-gnu x86_64-apple-darwin aarch64-apple-darwin
cargo install cargo-zigbuild --locked
pipx install ziglang
```

Or let the release script install the common Ubuntu dependencies:

```bash
scripts/self-hosted-release.sh v0.2.12 --install-deps --allow-partial
```

Install GitHub CLI and authenticate it:

```bash
gh auth login
```

For publishing npm packages, create an npm automation token and export it:

```bash
export NPM_TOKEN=...
```

The npm release uses unscoped package names so it can publish from the owner account without an npm organization:

```text
infynon
infynon-windows-x64
infynon-linux-x64
infynon-linux-arm64
infynon-darwin-x64
infynon-darwin-arm64
```

For publishing the GitHub release, export a token:

```bash
export GH_TOKEN=...
```

The token must be able to push to `d4rkNinja/infynon-cli` and create or edit releases there.

## Permanent Server Environment

Set release credentials and the macOS SDK path once on the Linux server. Do not commit real tokens to this repository.

```bash
cat >> ~/.bashrc <<'EOF'

# INFYNON self-hosted release environment
export SDKROOT="$HOME/.cache/infynon/macos-sdk/MacOSX26.4.sdk"
export NPM_TOKEN="replace-with-npm-token"
export GH_TOKEN="replace-with-github-token"
EOF

source ~/.bashrc
```

Use an npm token that can create and publish new unscoped packages for the `alien257` npm account. A token limited to only the existing `infynon` package cannot publish platform packages.

Use a GitHub token that can push to `d4rkNinja/infynon-cli` and create or edit releases. If you prefer GitHub CLI auth instead of a stored token, run this after `gh auth login`:

```bash
cat >> ~/.bashrc <<'EOF'

# INFYNON GitHub token from gh auth
export GH_TOKEN="$(gh auth token)"
EOF
```

Verify the saved environment before a release:

```bash
source ~/.bashrc
test -d "$SDKROOT"
npm whoami --registry=https://registry.npmjs.org/
gh auth status
```

## Build Assets

From the source repo:

```bash
git checkout main
git pull origin main
scripts/self-hosted-release.sh v0.2.12 --build-all
```

This writes assets, `checksums.txt`, and `release-manifest.json` into `release-bundle/`.

You can also build one platform group at a time:

```bash
scripts/self-hosted-release.sh v0.2.12 --build-linux
scripts/self-hosted-release.sh v0.2.12 --build-windows
scripts/self-hosted-release.sh v0.2.12 --build-macos
```

On Linux, the Windows executable is built with Rust target `x86_64-pc-windows-gnu` and staged as `infynon-x86_64-pc-windows-msvc.exe` because existing installers download that release asset name.

macOS cross-builds require an Apple macOS SDK. On a Linux host, you can ask the script to fetch a public SDK from `alexey-lysiuk/macos-sdk`:

```bash
scripts/self-hosted-release.sh v0.2.12 --install-macos-sdk 26.4 --build-macos
```

You can still use `latest` when you want GitHub's current latest release tag:

```bash
scripts/self-hosted-release.sh v0.2.12 --install-macos-sdk latest --build-macos
```

Or set `SDKROOT` to an extracted `MacOSX.sdk` before using `--build-macos` or `--build-all`:

```bash
export SDKROOT=/opt/MacOSX.sdk
scripts/self-hosted-release.sh v0.2.12 --build-macos
```

Make sure your use of any SDK complies with Apple's license terms.

For SDK versions like `26.4` that exist as repository folders but not GitHub release tags, the script falls back to a sparse checkout of `MacOSX26.4.sdk`.

## Publish a Complete Release

The npm installer and public GitHub release expect all five platform assets:

```text
infynon-x86_64-pc-windows-msvc.exe
infynon-x86_64-unknown-linux-musl
infynon-aarch64-unknown-linux-musl
infynon-x86_64-apple-darwin
infynon-aarch64-apple-darwin
```

To build and publish from the Linux server:

```bash
source ~/.bashrc
scripts/self-hosted-release.sh v0.2.12 \
  --install-deps \
  --build-all \
  --publish-github \
  --publish-npm
```

For a GitHub-only partial release, add `--allow-partial`. Do not use partial releases for npm.

## What the script does

- Verifies release metadata with `scripts/verify-release-versions.py`.
- Builds Linux musl assets when `--build-linux` is passed.
- Builds the Windows asset when `--build-windows` is passed.
- Builds macOS assets when `--build-macos` is passed.
- Builds all five assets when `--build-all` is passed.
- Generates `checksums.txt` and `release-manifest.json`.
- When both `--publish-npm` and `--publish-github` are passed, publishes npm first so npm credential failures do not create a GitHub release first.
- Creates or updates the GitHub release and uploads assets.
- Stages platform npm packages and publishes all npm packages for the version as one required set.

## Safety Notes

The script refuses to publish npm unless all five platform binaries are present. This keeps the main npm package from referencing missing optional platform packages.

The npm publish phase is intentionally strict:

- All npm package versions must match the release tag.
- Scoped package names such as `@infynon/cli-*` are rejected for self-hosted publishing.
- Platform npm package names are unscoped `infynon-*` names. Do not use old `@infynon/cli-*`, `infynon-cli-*`, or `infynon-cli-win32-x64` names.
- If any package for the target version is already published while another is missing, the script stops before publishing.
- If a package publish fails after earlier packages were published in the same run, the script leaves those packages published by default and exits with an error. This avoids npm's 24-hour lock on republishing unpublished package versions.
- Set `NPM_ROLLBACK_ON_FAILURE=true` only when you intentionally want `npm unpublish <package>@<version> --force` rollback behavior.
