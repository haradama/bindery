#!/usr/bin/env bash
set -e

REPO="haradama/bindery"
BIN_NAME="bindery"

# Detect platform
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$ARCH" in
  x86_64) ARCH="x86_64" ;;
  arm64|aarch64) ARCH="aarch64" ;;
  *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

TAG=$(curl -s https://api.github.com/repos/$REPO/releases/latest | grep -Po '"tag_name": "\\K.*?(?=")')

if [ -z "$TAG" ]; then
  echo "Failed to fetch latest release tag"
  exit 1
fi

echo "Installing $BIN_NAME $TAG for $OS-$ARCH ..."

URL="https://github.com/$REPO/releases/download/$TAG/${BIN_NAME}-${OS}-${ARCH}.tar.gz"

TMP_DIR=$(mktemp -d)
curl -fsSL "$URL" -o "$TMP_DIR/$BIN_NAME.tar.gz"
tar -xzf "$TMP_DIR/$BIN_NAME.tar.gz" -C "$TMP_DIR"
chmod +x "$TMP_DIR/$BIN_NAME"

# Default install path: ~/.local/bin
INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"
mv "$TMP_DIR/$BIN_NAME" "$INSTALL_DIR/"

echo "Installed $BIN_NAME to $INSTALL_DIR"
echo "Make sure $INSTALL_DIR is in your PATH"
