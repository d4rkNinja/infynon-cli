#!/usr/bin/env bash
# Build and publish an INFYNON release from a self-hosted Linux machine.
#
# This is a standalone fallback for when GitHub-hosted Actions cannot run.
# It uses the same asset names as .github/workflows/release.yml.
#
# Examples:
#   scripts/self-hosted-release.sh v0.2.12 --install-deps --install-macos-sdk 26.4 --build-all --publish-github --publish-npm
#   scripts/self-hosted-release.sh v0.2.12 --build-linux --publish-github
#   scripts/self-hosted-release.sh v0.2.12 --assets-dir /srv/infynon/v0.2.12 --publish-github --publish-npm
#
# Required for publishing:
#   GH_TOKEN with release access to this repository
#   NPM_TOKEN when --publish-npm is used

set -euo pipefail

export PATH="$HOME/.local/bin:$HOME/.cargo/bin:$PATH"

RELEASE_REPO="${RELEASE_REPO:-d4rkNinja/infynon-cli}"
DIST_DIR="${DIST_DIR:-release-bundle}"
INSTALL_DEPS=false
INSTALL_MACOS_SDK=false
MACOS_SDK_VERSION="${MACOS_SDK_VERSION:-26.4}"
MACOS_SDK_CACHE_DIR="${MACOS_SDK_CACHE_DIR:-$HOME/.cache/infynon/macos-sdk}"
BUILD_ALL=false
BUILD_LINUX=false
BUILD_WINDOWS=false
BUILD_MACOS=false
PUBLISH_GITHUB=false
PUBLISH_NPM=false
ALLOW_PARTIAL=false
ASSETS_DIR=""

usage() {
  cat <<'EOF'
Usage:
  scripts/self-hosted-release.sh <vX.Y.Z> [options]

Options:
  --install-deps      Install Ubuntu/Rust helper dependencies for cross-builds.
  --install-macos-sdk [VERSION]
                      Download a macOS SDK release and set SDKROOT.
                      Default version: 26.4.
  --build-all         Build Linux, Windows, and macOS release assets.
  --build-linux       Build Linux x64 and ARM64 musl assets locally.
  --build-windows     Build the Windows x64 asset from Linux.
  --build-macos       Build macOS x64 and ARM64 assets from Linux.
  --assets-dir DIR    Copy prebuilt release assets from DIR before publishing.
  --publish-github    Publish GitHub Release assets.
  --publish-npm       Publish all npm platform packages and the main npm package atomically.
  --allow-partial     Do not require all five platform binaries.
  -h, --help          Show this help.

Environment:
  RELEASE_REPO               GitHub repository, default d4rkNinja/infynon-cli.
  GH_TOKEN                   Token used by gh release commands.
  NPM_TOKEN                  npm automation token for --publish-npm.
  NPM_PROVENANCE             true/false, default false for self-hosted publishing.
  NPM_ROLLBACK_ON_FAILURE    true/false, default false. npm blocks republishing
                             unpublished package versions for 24 hours.
  MACOS_SDK_VERSION          Default SDK version for --install-macos-sdk.
  MACOS_SDK_CACHE_DIR        SDK install cache, default ~/.cache/infynon/macos-sdk.

The complete public release expects these assets:
  infynon-x86_64-pc-windows-msvc.exe
  infynon-x86_64-unknown-linux-musl
  infynon-aarch64-unknown-linux-musl
  infynon-x86_64-apple-darwin
  infynon-aarch64-apple-darwin
EOF
}

die() {
  echo "error: $*" >&2
  exit 1
}

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || die "missing required command: $1"
}

run() {
  echo
  echo "==> $*"
  "$@"
}

abs_dir() {
  local dir="$1"
  mkdir -p "$dir"
  (cd "$dir" && pwd -P)
}

stage_assets() {
  local source_dir="$1"
  local dest_dir="$2"
  local source_abs dest_abs files

  [[ -d "$source_dir" ]] || die "assets directory does not exist: $source_dir"
  source_abs="$(abs_dir "$source_dir")"
  dest_abs="$(abs_dir "$dest_dir")"

  if [[ "$source_abs" == "$dest_abs" ]]; then
    echo "Assets directory is already the output directory: $dest_dir"
    return
  fi

  shopt -s nullglob
  files=("$source_abs"/infynon-*)
  shopt -u nullglob

  [[ ${#files[@]} -gt 0 ]] || die "assets directory has no infynon-* files: $source_dir"
  run cp -f "${files[@]}" "$dest_abs"/
}

ensure_zig_for_zigbuild() {
  if command -v zig >/dev/null 2>&1; then
    return
  fi

  if command -v python-zig >/dev/null 2>&1; then
    mkdir -p "$HOME/.local/bin"
    ln -sf "$(command -v python-zig)" "$HOME/.local/bin/zig"
    export PATH="$HOME/.local/bin:$PATH"
  fi

  command -v zig >/dev/null 2>&1 || die "missing required command: zig. Install it with: pipx install ziglang"
}

ensure_macos_sdk() {
  if [[ "$(uname -s)" == "Darwin" ]] && command -v xcrun >/dev/null 2>&1; then
    return
  fi

  if [[ -n "${SDKROOT:-}" && -d "${SDKROOT:-}" ]]; then
    return
  fi

  cat >&2 <<'EOF'
error: macOS cross-builds require an Apple macOS SDK.

Set SDKROOT to an extracted MacOSX.sdk directory before using --build-macos or --build-all, for example:

  export SDKROOT=/opt/MacOSX.sdk
  scripts/self-hosted-release.sh v0.2.12 --build-macos

The script cannot install the Apple SDK automatically. Build macOS assets on a Mac, or provision a licensed SDK on this Linux server.
EOF
  exit 1
}

install_macos_sdk() {
  local version="$1"
  local sdk_dir archive tmp_dir found_sdk repo_dir sdk_name

  need_cmd gh
  if [[ "$version" == "latest" ]]; then
    version="$(gh release list -R alexey-lysiuk/macos-sdk --limit 1 --json tagName --jq '.[0].tagName')"
    [[ -n "$version" ]] || die "could not resolve latest macOS SDK release tag"
  fi

  sdk_dir="$MACOS_SDK_CACHE_DIR/MacOSX${version}.sdk"
  if [[ -d "$sdk_dir" ]]; then
    export SDKROOT="$sdk_dir"
    echo "Using cached macOS SDK: $SDKROOT"
    return
  fi

  need_cmd tar

  tmp_dir="$(mktemp -d)"
  mkdir -p "$MACOS_SDK_CACHE_DIR"

  echo "Downloading macOS SDK $version from alexey-lysiuk/macos-sdk..."
  if gh release download "$version" \
    -R alexey-lysiuk/macos-sdk \
    -D "$tmp_dir" \
    --clobber; then
    shopt -s nullglob
    for archive in "$tmp_dir"/*; do
      case "$archive" in
        *.tar|*.tar.gz|*.tgz|*.tar.xz|*.txz|*.zip)
          ;;
        *)
          continue
          ;;
      esac

      case "$archive" in
        *.zip)
          need_cmd unzip
          run unzip -q "$archive" -d "$tmp_dir/extracted"
          ;;
        *)
          mkdir -p "$tmp_dir/extracted"
          run tar -xf "$archive" -C "$tmp_dir/extracted"
          ;;
      esac
    done
    shopt -u nullglob

    found_sdk="$(find "$tmp_dir/extracted" -type d -name 'MacOSX*.sdk' | sort | tail -n 1 || true)"
    [[ -n "$found_sdk" ]] || die "downloaded macOS SDK release did not contain a MacOSX*.sdk directory"
  else
    sdk_name="MacOSX${version}.sdk"
    repo_dir="$tmp_dir/repo"
    echo "No release tag '$version'; sparse-checking out repository directory $sdk_name..."
    run git clone --depth 1 --filter=blob:none --sparse https://github.com/alexey-lysiuk/macos-sdk.git "$repo_dir"
    run git -C "$repo_dir" sparse-checkout set "$sdk_name"
    found_sdk="$repo_dir/$sdk_name"
    [[ -d "$found_sdk" ]] || die "macOS SDK directory not found in repository: $sdk_name"
  fi

  rm -rf "$sdk_dir"
  cp -a "$found_sdk" "$sdk_dir"
  rm -rf "$tmp_dir"

  export SDKROOT="$sdk_dir"
  echo "Installed macOS SDK: $SDKROOT"
}

install_deps() {
  if [[ "$(uname -s)" != "Linux" ]]; then
    die "--install-deps is currently implemented for Linux hosts only"
  fi

  need_cmd sudo

  run sudo rm -f /etc/apt/sources.list.d/github-cli.list
  run sudo apt-get update
  run sudo apt-get install -y \
    build-essential \
    ca-certificates \
    curl \
    git \
    jq \
    wget \
    nodejs \
    npm \
    python3 \
    python3-full \
    python3-venv \
    pipx \
    pkg-config \
    libssl-dev \
    musl-tools \
    mingw-w64 \
    xz-utils \
    unzip

  run pipx ensurepath
  export PATH="$HOME/.local/bin:$HOME/.cargo/bin:$PATH"

  if ! command -v python-zig >/dev/null 2>&1 && ! command -v zig >/dev/null 2>&1; then
    run pipx install ziglang
  fi
  ensure_zig_for_zigbuild

  if ! command -v cargo >/dev/null 2>&1 || ! command -v rustup >/dev/null 2>&1; then
    run sh -c 'curl https://sh.rustup.rs -sSf | sh -s -- -y'
    # shellcheck disable=SC1091
    . "$HOME/.cargo/env"
  fi

  export PATH="$HOME/.local/bin:$HOME/.cargo/bin:$PATH"

  run rustup target add \
    x86_64-unknown-linux-musl \
    aarch64-unknown-linux-musl \
    x86_64-pc-windows-gnu \
    x86_64-apple-darwin \
    aarch64-apple-darwin

  if ! cargo zigbuild --help >/dev/null 2>&1; then
    run cargo install cargo-zigbuild --locked
  fi

  if ! command -v gh >/dev/null 2>&1; then
    local key list url arch
    key="/etc/apt/keyrings/github-cli.gpg"
    list="/etc/apt/sources.list.d/github-cli.list"
    url="https://cli.github.com/packages"
    arch="$(dpkg --print-architecture)"

    run sudo mkdir -p -m 755 /etc/apt/keyrings
    wget -qO- "$url/githubcli-archive-keyring.gpg" | sudo tee "$key" >/dev/null
    run sudo chmod go+r "$key"
    printf 'deb [arch=%s signed-by=%s] %s stable main\n' "$arch" "$key" "$url" \
      | sudo tee "$list" >/dev/null
    run sudo apt-get update
    run sudo apt-get install -y gh
  fi
}

if [[ $# -lt 1 ]]; then
  usage
  exit 2
fi

RELEASE_TAG="$1"
shift

while [[ $# -gt 0 ]]; do
  case "$1" in
    --install-deps)
      INSTALL_DEPS=true
      ;;
    --install-macos-sdk)
      INSTALL_MACOS_SDK=true
      if [[ "${2:-}" =~ ^([0-9]+(\.[0-9]+)*|latest)$ ]]; then
        MACOS_SDK_VERSION="$2"
        shift
      fi
      ;;
    --build-all)
      BUILD_ALL=true
      BUILD_LINUX=true
      BUILD_WINDOWS=true
      BUILD_MACOS=true
      ;;
    --build-linux)
      BUILD_LINUX=true
      ;;
    --build-windows)
      BUILD_WINDOWS=true
      ;;
    --build-macos)
      BUILD_MACOS=true
      ;;
    --assets-dir)
      ASSETS_DIR="${2:-}"
      [[ -n "$ASSETS_DIR" ]] || die "--assets-dir requires a directory"
      shift
      ;;
    --publish-github)
      PUBLISH_GITHUB=true
      ;;
    --publish-npm)
      PUBLISH_NPM=true
      ;;
    --allow-partial)
      ALLOW_PARTIAL=true
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      die "unknown option: $1"
      ;;
  esac
  shift
done

[[ "$RELEASE_TAG" =~ ^v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z][0-9A-Za-z.-]*)?$ ]] \
  || die "release tag must be v-prefixed semver, for example v0.2.12"

VERSION="${RELEASE_TAG#v}"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

if [[ "$INSTALL_DEPS" == true ]]; then
  install_deps
fi

if [[ "$INSTALL_MACOS_SDK" == true ]]; then
  install_macos_sdk "$MACOS_SDK_VERSION"
fi

if [[ "$BUILD_MACOS" == true ]]; then
  ensure_macos_sdk
fi

need_cmd git
need_cmd cargo
need_cmd rustup
need_cmd python3
need_cmd sha256sum
need_cmd find
need_cmd sort

run python3 scripts/verify-release-versions.py "$RELEASE_TAG"

mkdir -p "$DIST_DIR"

if [[ -n "$ASSETS_DIR" ]]; then
  stage_assets "$ASSETS_DIR" "$DIST_DIR"
fi

build_rust_target() {
  local target="$1"
  local ext="${2:-}"
  local asset_target="${3:-$target}"
  local output="$DIST_DIR/infynon-$asset_target$ext"

  if cargo zigbuild --help >/dev/null 2>&1; then
    ensure_zig_for_zigbuild
    run cargo zigbuild --release --target "$target"
  else
    run cargo build --release --target "$target"
  fi

  cp "target/$target/release/infynon$ext" "$output"
  test -s "$output"
}

if [[ "$BUILD_LINUX" == true ]]; then
  build_rust_target "x86_64-unknown-linux-musl" ""
  build_rust_target "aarch64-unknown-linux-musl" ""
fi

if [[ "$BUILD_WINDOWS" == true ]]; then
  build_rust_target "x86_64-pc-windows-gnu" ".exe" "x86_64-pc-windows-msvc"
fi

if [[ "$BUILD_MACOS" == true ]]; then
  build_rust_target "x86_64-apple-darwin" ""
  build_rust_target "aarch64-apple-darwin" ""
fi

declare -a REQUIRED_ASSETS=(
  "infynon-x86_64-pc-windows-msvc.exe"
  "infynon-x86_64-unknown-linux-musl"
  "infynon-aarch64-unknown-linux-musl"
  "infynon-x86_64-apple-darwin"
  "infynon-aarch64-apple-darwin"
)

missing=()
for asset in "${REQUIRED_ASSETS[@]}"; do
  if [[ ! -s "$DIST_DIR/$asset" ]]; then
    missing+=("$asset")
  fi
done

if [[ ${#missing[@]} -gt 0 && "$ALLOW_PARTIAL" != true ]]; then
  printf 'error: missing required release assets:\n' >&2
  printf '  %s\n' "${missing[@]}" >&2
  echo "Provide them with --assets-dir or rerun with --allow-partial for a GitHub-only partial release." >&2
  exit 1
fi

(
  cd "$DIST_DIR"
  find . -maxdepth 1 -type f -name 'infynon-*' -printf '%f\0' \
    | sort -z \
    | xargs -0 sha256sum > checksums.txt

  RELEASE_TAG="$RELEASE_TAG" python3 - <<'PY'
import hashlib
import json
import os
from pathlib import Path

release_tag = os.environ["RELEASE_TAG"]
version = release_tag[1:] if release_tag.startswith("v") else release_tag
assets = []

for path in sorted(Path(".").glob("infynon-*")):
    if not path.is_file():
        continue
    target = path.name.removeprefix("infynon-")
    if target.endswith(".exe"):
        target = target[:-4]
    assets.append(
        {
            "target": target,
            "asset": path.name,
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
            "size": path.stat().st_size,
        }
    )

if not assets:
    raise SystemExit("No release assets found")

Path("release-manifest.json").write_text(
    json.dumps({"schema_version": 1, "version": version, "assets": assets}, indent=2) + "\n",
    encoding="utf-8",
)
PY
)

publish_github() {
  need_cmd gh
  export RELEASE_TAG
  export RELEASE_REPO
  [[ -n "${GH_TOKEN:-}" ]] || die "GH_TOKEN is required"

  if gh release view "$RELEASE_TAG" -R "$RELEASE_REPO" >/dev/null 2>&1; then
    run gh release edit "$RELEASE_TAG" \
      -R "$RELEASE_REPO" \
      --title "INFYNON CLI $RELEASE_TAG" \
      --notes "Stable INFYNON CLI release assets for $RELEASE_TAG."
    run gh release upload "$RELEASE_TAG" "$DIST_DIR"/* \
      -R "$RELEASE_REPO" \
      --clobber
  else
    run gh release create "$RELEASE_TAG" "$DIST_DIR"/* \
      -R "$RELEASE_REPO" \
      --title "INFYNON CLI $RELEASE_TAG" \
      --notes "Stable INFYNON CLI release assets for $RELEASE_TAG."
  fi

  # Go proxy indexing is best-effort and can return 404 immediately after a tag is pushed.
  curl -fsL "https://proxy.golang.org/github.com/${RELEASE_REPO}/go/@v/${RELEASE_TAG}.info" >/dev/null 2>&1 || true
  curl -fsL "https://proxy.golang.org/github.com/${RELEASE_REPO}/go/@v/go%2F${RELEASE_TAG}.info" >/dev/null 2>&1 || true
}

publish_npm() {
  need_cmd node
  need_cmd npm
  [[ -n "${NPM_TOKEN:-}" ]] || die "NPM_TOKEN is required for --publish-npm"
  [[ ${#missing[@]} -eq 0 ]] || die "--publish-npm requires all five platform assets"

  while IFS='|' read -r package asset binary; do
    package_dir="npm/platforms/$package"
    mkdir -p "$package_dir/bin"
    cp "$DIST_DIR/$asset" "$package_dir/bin/$binary"
    cp npm/LICENSE "$package_dir/LICENSE"
    chmod 755 "$package_dir/bin/$binary"
    test -s "$package_dir/bin/$binary"
  done <<'EOF'
cli-windows-x64|infynon-x86_64-pc-windows-msvc.exe|infynon.exe
cli-linux-x64|infynon-x86_64-unknown-linux-musl|infynon
cli-linux-arm64|infynon-aarch64-unknown-linux-musl|infynon
cli-darwin-x64|infynon-x86_64-apple-darwin|infynon
cli-darwin-arm64|infynon-aarch64-apple-darwin|infynon
EOF

  declare -a npm_package_dirs=()
  while IFS= read -r package_dir; do
    npm_package_dirs+=("$package_dir")
    (cd "$package_dir" && npm pack --dry-run)
  done < <(find npm/platforms -mindepth 2 -maxdepth 2 -name package.json -printf '%h\n' | sort)
  npm_package_dirs+=("npm")
  (cd npm && npm pack --dry-run)

  export NODE_AUTH_TOKEN="$NPM_TOKEN"
  npm config set //registry.npmjs.org/:_authToken "$NPM_TOKEN" >/dev/null

  declare -a npm_specs=()
  declare -a npm_existing_specs=()
  declare -a npm_missing_specs=()
  declare -a npm_published_this_run=()

  for package_dir in "${npm_package_dirs[@]}"; do
    name="$(node -p "require('./${package_dir}/package.json').name")"
    package_version="$(node -p "require('./${package_dir}/package.json').version")"
    [[ "$package_version" == "$VERSION" ]] || die "npm package $name has version $package_version; expected $VERSION"
    [[ "$name" != @* ]] || die "npm package $name is scoped. Use unscoped package names for self-hosted npm publishing."

    npm_specs+=("${name}@${VERSION}")
  done

  for spec in "${npm_specs[@]}"; do
    if published="$(npm view "$spec" version --silent 2>/dev/null)" && [[ "$published" == "$VERSION" ]]; then
      npm_existing_specs+=("$spec")
    else
      npm_missing_specs+=("$spec")
    fi
  done

  if [[ ${#npm_existing_specs[@]} -gt 0 && ${#npm_missing_specs[@]} -gt 0 ]]; then
    printf 'error: npm registry already has a partial %s release.\n' "$VERSION" >&2
    printf 'Already published:\n' >&2
    printf '  %s\n' "${npm_existing_specs[@]}" >&2
    printf 'Missing:\n' >&2
    printf '  %s\n' "${npm_missing_specs[@]}" >&2
    echo "Refusing to publish because npm package versions must stay consistent." >&2
    exit 1
  fi

  if [[ ${#npm_missing_specs[@]} -eq 0 ]]; then
    echo "All npm packages for $VERSION are already published; skipping npm publish."
  else
    rollback_npm_publish() {
      local spec
      if [[ ${#npm_published_this_run[@]} -eq 0 ]]; then
        return
      fi

      if [[ "${NPM_ROLLBACK_ON_FAILURE:-false}" != "true" ]]; then
        echo "npm publish failed after these packages were published:" >&2
        printf '  %s\n' "${npm_published_this_run[@]}" >&2
        echo "Leaving them published to avoid npm's 24-hour unpublished-version republish lock." >&2
        echo "Set NPM_ROLLBACK_ON_FAILURE=true only when you intentionally want npm unpublish rollback." >&2
        return
      fi

      echo "Rolling back npm packages published during this run..." >&2
      for ((idx=${#npm_published_this_run[@]}-1; idx>=0; idx--)); do
        spec="${npm_published_this_run[$idx]}"
        echo "Unpublishing $spec" >&2
        npm unpublish "$spec" --force || echo "warning: failed to unpublish $spec; manual cleanup required" >&2
      done
    }

    publish_required() {
      local package_dir="$1"
      local name package_version spec

      name="$(node -p "require('./${package_dir}/package.json').name")"
      package_version="$(node -p "require('./${package_dir}/package.json').version")"
      spec="${name}@${package_version}"

      local publish_args=(--access public)
      if [[ "${NPM_PROVENANCE:-false}" == "true" ]]; then
        publish_args+=(--provenance)
      else
        publish_args+=(--provenance=false)
      fi

      echo "Publishing $spec from ${package_dir}"
      if (cd "$package_dir" && npm publish "${publish_args[@]}"); then
        npm_published_this_run+=("$spec")
        return
      fi

      rollback_npm_publish
      die "npm publish failed for $spec"
    }

    for package_dir in "${npm_package_dirs[@]}"; do
      publish_required "$package_dir"
    done
  fi
}

if [[ "$PUBLISH_NPM" == true ]]; then
  publish_npm
fi

if [[ "$PUBLISH_GITHUB" == true ]]; then
  publish_github
fi

echo
echo "Self-hosted release pipeline complete for $RELEASE_TAG."
echo "Assets:"
ls -lh "$DIST_DIR"
