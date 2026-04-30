#!/bin/sh
set -e

REPO="bituann/insighta-cli"
BINARY="insighta"
INSTALL_DIR="/usr/local/bin"

# Detect OS and architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS" in
  linux)
    case "$ARCH" in
      x86_64)  ARTIFACT="insighta-linux-x86_64" ;;
      aarch64) ARTIFACT="insighta-linux-arm64" ;;
      *) echo "Unsupported architecture: $ARCH" && exit 1 ;;
    esac
    ;;
  darwin)
    case "$ARCH" in
      x86_64)  ARTIFACT="insighta-macos-x86_64" ;;
      arm64)   ARTIFACT="insighta-macos-arm64" ;;
      *) echo "Unsupported architecture: $ARCH" && exit 1 ;;
    esac
    ;;
  *)
    echo "Unsupported OS: $OS"
    echo "For Windows, download the .exe from https://github.com/$REPO/releases/latest"
    exit 1
    ;;
esac

# Get latest release version
echo "→ Fetching latest release..."
VERSION=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
  | grep '"tag_name"' \
  | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$VERSION" ]; then
  echo "Could not determine latest version"
  exit 1
fi

echo "→ Installing insighta $VERSION..."

URL="https://github.com/$REPO/releases/download/$VERSION/$ARTIFACT"
TMP=$(mktemp)

curl -fsSL "$URL" -o "$TMP"
chmod +x "$TMP"

# Install to /usr/local/bin or ~/.local/bin if no sudo
if [ -w "$INSTALL_DIR" ]; then
  mv "$TMP" "$INSTALL_DIR/$BINARY"
else
  echo "→ No write access to $INSTALL_DIR, installing to ~/.local/bin"
  mkdir -p "$HOME/.local/bin"
  mv "$TMP" "$HOME/.local/bin/$BINARY"
  INSTALL_DIR="$HOME/.local/bin"
  echo "  Add this to your shell profile if not already present:"
  echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
fi

echo "✓ Installed to $INSTALL_DIR/$BINARY"
echo ""
echo "  Run: insighta login"