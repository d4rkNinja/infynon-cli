#!/usr/bin/env bash
# Build infynon binary for the current platform.
# Single binary — use `infynon pkg` for package manager mode.
#
# Usage:
#   ./scripts/build-all.sh          # build for current platform
#   ./scripts/build-all.sh all      # attempt all targets (requires cross-compilers)

set -euo pipefail

DIST="dist"
VERSION="$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')"
mkdir -p "$DIST"

build_target() {
    local target="$1"
    local suffix="$2"
    echo "==> Building for $target ..."
    cargo build --release --target "$target"
    cp "target/$target/release/infynon$suffix" "$DIST/infynon-$target$suffix"
    echo "    -> $DIST/infynon-$target$suffix"
}

if [[ "${1:-}" == "all" ]]; then
    build_target "x86_64-pc-windows-msvc"      ".exe"
    build_target "x86_64-unknown-linux-musl"    ""
    build_target "aarch64-unknown-linux-musl"   ""
    build_target "x86_64-apple-darwin"          ""
    build_target "aarch64-apple-darwin"         ""
else
    case "$(uname -s)-$(uname -m)" in
        Linux-x86_64)          build_target "x86_64-unknown-linux-musl"  "" ;;
        Linux-aarch64)         build_target "aarch64-unknown-linux-musl" "" ;;
        Darwin-x86_64)         build_target "x86_64-apple-darwin"        "" ;;
        Darwin-arm64)          build_target "aarch64-apple-darwin"       "" ;;
        MINGW*|MSYS*|CYGWIN*) build_target "x86_64-pc-windows-msvc" ".exe" ;;
        *)
            echo "Unknown platform: $(uname -s)-$(uname -m)"
            cargo build --release
            ;;
    esac
fi

echo ""
echo "Build complete (v$VERSION):"
ls -lh "$DIST"
