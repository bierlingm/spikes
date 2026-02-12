#!/bin/sh
# Spikes installer - https://spikes.sh
# Usage: curl -fsSL https://spikes.sh/install.sh | sh

set -e

REPO="bierlingm/spikes"
INSTALL_DIR="${SPIKES_INSTALL_DIR:-$HOME/.local/bin}"

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Darwin) OS="apple-darwin" ;;
    Linux) OS="unknown-linux-gnu" ;;
    *) echo "Unsupported OS: $OS"; exit 1 ;;
esac

case "$ARCH" in
    x86_64) ARCH="x86_64" ;;
    aarch64|arm64) ARCH="aarch64" ;;
    *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

TARGET="${ARCH}-${OS}"

# Get latest release
LATEST=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | cut -d'"' -f4)
if [ -z "$LATEST" ]; then
    echo "Failed to get latest release"
    exit 1
fi

echo "Installing spikes ${LATEST} for ${TARGET}..."

# Download binary
URL="https://github.com/${REPO}/releases/download/${LATEST}/spikes-${TARGET}.tar.gz"
TMPDIR=$(mktemp -d)
curl -fsSL "$URL" | tar -xz -C "$TMPDIR"

# Install
mkdir -p "$INSTALL_DIR"
mv "$TMPDIR/spikes" "$INSTALL_DIR/spikes"
chmod +x "$INSTALL_DIR/spikes"
rm -rf "$TMPDIR"

echo ""
echo "Installed spikes to $INSTALL_DIR/spikes"
echo ""

# Check if in PATH
if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
    echo "Add this to your shell config:"
    echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
    echo ""
fi

echo "Get started:"
echo "  spikes --help"
echo ""
