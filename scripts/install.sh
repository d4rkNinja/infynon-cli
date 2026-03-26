#!/usr/bin/env bash
# One-liner: curl -fsSL https://raw.githubusercontent.com/d4rkNinja/infynon-cli/main/scripts/install.sh | bash
set -euo pipefail

REPO="d4rkNinja/infynon-cli"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

echo ""
echo "  ██╗███╗   ██╗███████╗██╗   ██╗███╗   ██╗ ██████╗ ███╗   ██╗"
echo "  ██║████╗  ██║██╔════╝╚██╗ ██╔╝████╗  ██║██╔═══██╗████╗  ██║"
echo "  ██║██╔██╗ ██║█████╗   ╚████╔╝ ██╔██╗ ██║██║   ██║██╔██╗ ██║"
echo "  ██║██║╚██╗██║██╔══╝    ╚██╔╝  ██║╚██╗██║██║   ██║██║╚██╗██║"
echo "  ██║██║ ╚████║██║        ██║   ██║ ╚████║╚██████╔╝██║ ╚████║"
echo "  ╚═╝╚═╝  ╚═══╝╚═╝        ╚═╝   ╚═╝  ╚═══╝ ╚═════╝ ╚═╝  ╚═══╝"
echo ""
echo "  Universal Package Security Manager — Installer"
echo ""

# Detect platform
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS-$ARCH" in
    Linux-x86_64)   TARGET="x86_64-unknown-linux-musl" ;;
    Linux-aarch64)  TARGET="aarch64-unknown-linux-musl" ;;
    Darwin-x86_64)  TARGET="x86_64-apple-darwin" ;;
    Darwin-arm64)   TARGET="aarch64-apple-darwin" ;;
    *)
        echo "  [!!] Unsupported platform: $OS-$ARCH"
        echo "  Install from source instead: cargo install --git https://github.com/$REPO"
        exit 1
        ;;
esac

echo "  Platform: $OS $ARCH → $TARGET"

# Get latest release tag
echo "  Fetching latest release..."
TAG=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')
if [ -z "$TAG" ]; then
    echo "  [!!] Could not determine latest release. Building from source..."
    if ! command -v cargo &>/dev/null; then
        echo "  Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    fi
    cargo install --git "https://github.com/$REPO"
    # Create symlink for infynon-pkg
    CARGO_BIN="$HOME/.cargo/bin"
    if [ -f "$CARGO_BIN/infynon" ] && [ ! -f "$CARGO_BIN/infynon-pkg" ]; then
        ln -sf "$CARGO_BIN/infynon" "$CARGO_BIN/infynon-pkg"
    fi
    echo ""
    echo "  [OK] Installed! Run: infynon pkg scan"
    exit 0
fi

echo "  Latest release: $TAG"

# Download
BINARY="infynon-${TARGET}"
URL="https://github.com/$REPO/releases/download/$TAG/$BINARY"

echo "  Downloading $URL ..."
TMP=$(mktemp -d)
curl -fsSL "$URL" -o "$TMP/infynon"
chmod +x "$TMP/infynon"

# Install
echo "  Installing to $INSTALL_DIR ..."
if [ -w "$INSTALL_DIR" ]; then
    mv "$TMP/infynon" "$INSTALL_DIR/infynon"
    ln -sf "$INSTALL_DIR/infynon" "$INSTALL_DIR/infynon-pkg"
else
    sudo mv "$TMP/infynon" "$INSTALL_DIR/infynon"
    sudo ln -sf "$INSTALL_DIR/infynon" "$INSTALL_DIR/infynon-pkg"
fi
rm -rf "$TMP"

echo ""
echo "  [OK] infynon $TAG installed to $INSTALL_DIR/infynon"
echo "  [OK] infynon-pkg → symlinked"
echo ""
echo "  Get started:"
echo "    infynon pkg scan              # scan project for CVEs"
echo "    infynon pkg npm install express  # secure install"
echo ""
