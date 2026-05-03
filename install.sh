#!/usr/bin/env bash
set -euo pipefail

REPO="d4rkNinja/infynon-cli"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

os="$(uname -s)"
arch="$(uname -m)"

case "$os/$arch" in
  Linux/x86_64) target="x86_64-unknown-linux-musl" ;;
  Linux/aarch64|Linux/arm64) target="aarch64-unknown-linux-musl" ;;
  Darwin/x86_64) target="x86_64-apple-darwin" ;;
  Darwin/arm64) target="aarch64-apple-darwin" ;;
  *)
    echo "[infynon] Unsupported platform: $os/$arch" >&2
    echo "[infynon] Download a release manually from https://github.com/$REPO/releases" >&2
    exit 1
    ;;
esac

tag="$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | sed -n 's/.*"tag_name": "\(v[^"]*\)".*/\1/p' | head -n 1)"
if [[ -z "$tag" ]]; then
  echo "[infynon] Could not determine the latest release tag." >&2
  exit 1
fi

asset="infynon-${target}"
url="https://github.com/$REPO/releases/download/$tag/$asset"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

curl -fsSL "$url" -o "$tmp_dir/infynon"
chmod +x "$tmp_dir/infynon"

if mkdir -p "$INSTALL_DIR" 2>/dev/null && [[ -w "$INSTALL_DIR" ]]; then
  install -m 755 "$tmp_dir/infynon" "$INSTALL_DIR/infynon"
  ln -sf "$INSTALL_DIR/infynon" "$INSTALL_DIR/infynon-pkg"
else
  sudo install -d "$INSTALL_DIR"
  sudo install -m 755 "$tmp_dir/infynon" "$INSTALL_DIR/infynon"
  sudo ln -sf "$INSTALL_DIR/infynon" "$INSTALL_DIR/infynon-pkg"
fi

echo "[infynon] Installed $tag to $INSTALL_DIR/infynon"

