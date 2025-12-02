#!/usr/bin/env bash
# Git Account Manager - Simple Installer
# Usage: curl -fsSL https://raw.githubusercontent.com/satas20/git-account-manager/main/install.sh | bash

set -e

REPO="satas20/git-account-manager"
BINARY_NAME="git-acc-mngr"
INSTALL_DIR="${HOME}/.local/bin"

echo "🚀 Installing Git Account Manager..."

# Detect OS
OS=$(uname -s)
case "$OS" in
    Linux*)  PLATFORM="linux";;
    Darwin*) PLATFORM="macos";;
    *)       echo "❌ Unsupported OS: $OS"; exit 1;;
esac

# Get latest version
VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')
if [ -z "$VERSION" ]; then
    echo "❌ Failed to get latest version"
    exit 1
fi

echo "📦 Downloading ${BINARY_NAME} ${VERSION} for ${PLATFORM}..."

# Download
ASSET_NAME="${BINARY_NAME}-${PLATFORM}-x86_64.tar.gz"
URL="https://github.com/${REPO}/releases/download/${VERSION}/${ASSET_NAME}"
TEMP_DIR=$(mktemp -d)

curl -fsSL "$URL" -o "${TEMP_DIR}/${ASSET_NAME}" || {
    echo "❌ Download failed from: $URL"
    exit 1
}

# Extract and install
tar -xzf "${TEMP_DIR}/${ASSET_NAME}" -C "${TEMP_DIR}"
mkdir -p "${INSTALL_DIR}"
mv "${TEMP_DIR}/${BINARY_NAME}-${PLATFORM}-x86_64" "${INSTALL_DIR}/${BINARY_NAME}"
chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
rm -rf "${TEMP_DIR}"

echo "✅ Installed to ${INSTALL_DIR}/${BINARY_NAME}"

# Check PATH
if [[ ":$PATH:" != *":${INSTALL_DIR}:"* ]]; then
    echo ""
    echo "⚠️  Add to your PATH by running:"
    echo "   echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.bashrc"
    echo "   source ~/.bashrc"
fi

echo ""
echo "🎉 Installation complete!"
echo "   Run: ${BINARY_NAME} tui"
