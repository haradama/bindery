#!/usr/bin/env sh
set -eu

REPO="haradama/bindery"
BIN_NAME="bindery"

OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS" in
  linux|darwin) : ;;
  *) echo "Unsupported OS: $OS" >&2; exit 1 ;;
esac

case "$ARCH" in
  x86_64|amd64) ARCH="x86_64" ;;
  arm64|aarch64) ARCH="aarch64" ;;
  *) echo "Unsupported architecture: $ARCH" >&2; exit 1 ;;
esac

LATEST_URL=$(curl -fsSLI -o /dev/null -w '%{url_effective}' "https://github.com/${REPO}/releases/latest")
TAG=$(printf '%s\n' "$LATEST_URL" | sed -n 's#.*/tag/\(.*\)$#\1#p')

if [ -z "${TAG:-}" ]; then
  echo "Failed to resolve latest release tag" >&2
  exit 1
fi

echo "Installing $BIN_NAME ${TAG} for ${OS}-${ARCH} ..."

TARBALL="${BIN_NAME}-${OS}-${ARCH}.tar.gz"
URL="https://github.com/${REPO}/releases/download/${TAG}/${TARBALL}"

TMP_DIR=$(mktemp -d)
cleanup() { rm -rf "$TMP_DIR"; }
trap cleanup EXIT INT TERM

echo "Downloading: $URL"
curl -fsSL "$URL" -o "$TMP_DIR/$TARBALL"

tar -xzf "$TMP_DIR/$TARBALL" -C "$TMP_DIR"

INSTALL_DIR="${HOME}/.local/bin"
mkdir -p "$INSTALL_DIR"

if [ ! -f "$TMP_DIR/$BIN_NAME" ]; then
  echo "Binary '$BIN_NAME' not found in archive" >&2
  exit 1
fi

chmod +x "$TMP_DIR/$BIN_NAME"
mv "$TMP_DIR/$BIN_NAME" "$INSTALL_DIR/"

echo "Installed: $INSTALL_DIR/$BIN_NAME"

case ":$PATH:" in
  *":$INSTALL_DIR:"*) : ;;
  *)
    echo ""
    echo "NOTE: Add to PATH if necessary:"
    echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
    ;;
esac

echo "Done."
