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
checksums_url="https://github.com/$REPO/releases/download/$tag/checksums.txt"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

download() {
  local source_url="$1"
  local output_path="$2"
  curl -fsSL --retry 3 "$source_url" -o "$output_path"
}

sha256_file() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print tolower($1)}'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | awk '{print tolower($1)}'
  else
    echo "[infynon] sha256sum or shasum is required to verify the release asset." >&2
    exit 1
  fi
}

expected_checksum() {
  awk -v asset="$asset" '
    $1 ~ /^[A-Fa-f0-9]{64}$/ {
      name = $2
      sub(/^\*/, "", name)
      count = split(name, parts, "/")
      if (parts[count] == asset) {
        print tolower($1)
        found = 1
        exit
      }
    }
    END {
      if (!found) {
        exit 1
      }
    }
  ' "$tmp_dir/checksums.txt"
}

download "$url" "$tmp_dir/infynon"
download "$checksums_url" "$tmp_dir/checksums.txt"

expected="$(expected_checksum || true)"
if [[ -z "$expected" ]]; then
  echo "[infynon] checksums.txt does not include $asset." >&2
  exit 1
fi

actual="$(sha256_file "$tmp_dir/infynon")"
if [[ "$actual" != "$expected" ]]; then
  echo "[infynon] SHA-256 mismatch for $asset." >&2
  exit 1
fi

chmod +x "$tmp_dir/infynon"

if mkdir -p "$INSTALL_DIR" 2>/dev/null && [[ -w "$INSTALL_DIR" ]]; then
  install -m 755 "$tmp_dir/infynon" "$INSTALL_DIR/infynon"
  ln -sf "$INSTALL_DIR/infynon" "$INSTALL_DIR/infynon-pkg"
else
  sudo install -d "$INSTALL_DIR"
  sudo install -m 755 "$tmp_dir/infynon" "$INSTALL_DIR/infynon"
  sudo ln -sf "$INSTALL_DIR/infynon" "$INSTALL_DIR/infynon-pkg"
fi

expected_version="${tag#v}"
if ! reported_version="$("$INSTALL_DIR/infynon" --version 2>&1)"; then
  echo "[infynon] Installed binary failed to run: $reported_version" >&2
  exit 1
fi
if ! printf '%s\n' "$reported_version" | tr '[:space:]' '\n' | sed 's/^v//' | grep -Fx "$expected_version" >/dev/null; then
  echo "[infynon] Installed binary did not report version $expected_version." >&2
  exit 1
fi

echo "[infynon] Installed $tag to $INSTALL_DIR/infynon"
