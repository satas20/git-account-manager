#!/usr/bin/env bash
# Git Account Manager - Smart Installer
# Usage: curl -fsSL https://raw.githubusercontent.com/satas20/git-account-manager/main/install.sh | bash

set -e

REPO="satas20/git-account-manager"
BINARY_NAME="git-acc-mngr"

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

# Extract
tar -xzf "${TEMP_DIR}/${ASSET_NAME}" -C "${TEMP_DIR}"
BINARY_PATH="${TEMP_DIR}/${BINARY_NAME}-${PLATFORM}-x86_64"

# Smart installation: Try system-wide first, fallback to user install
INSTALLED=false

# Option 1: Try system-wide installation (/usr/local/bin)
if [ -w "/usr/local/bin" ] 2>/dev/null || sudo -n true 2>/dev/null; then
    echo "📍 Installing system-wide to /usr/local/bin (available to all users)..."

    if [ -w "/usr/local/bin" ]; then
        # Can write without sudo
        install -m 755 "${BINARY_PATH}" "/usr/local/bin/${BINARY_NAME}"
        INSTALLED=true
        echo "✅ Installed to /usr/local/bin/${BINARY_NAME}"
    else
        # Need sudo
        if sudo -n true 2>/dev/null; then
            # Passwordless sudo available
            sudo install -m 755 "${BINARY_PATH}" "/usr/local/bin/${BINARY_NAME}"
            INSTALLED=true
            echo "✅ Installed to /usr/local/bin/${BINARY_NAME}"
        else
            # Ask for sudo password
            echo "🔐 This will install to /usr/local/bin (requires admin password)"
            if sudo install -m 755 "${BINARY_PATH}" "/usr/local/bin/${BINARY_NAME}" 2>/dev/null; then
                INSTALLED=true
                echo "✅ Installed to /usr/local/bin/${BINARY_NAME}"
            else
                echo "⚠️  Sudo failed, falling back to user installation..."
            fi
        fi
    fi
fi

# Option 2: Fallback to user installation (~/.local/bin)
if [ "$INSTALLED" = false ]; then
    INSTALL_DIR="${HOME}/.local/bin"
    echo "📍 Installing to user directory ${INSTALL_DIR}..."

    mkdir -p "${INSTALL_DIR}"
    install -m 755 "${BINARY_PATH}" "${INSTALL_DIR}/${BINARY_NAME}"
    echo "✅ Installed to ${INSTALL_DIR}/${BINARY_NAME}"

    # Check if PATH needs to be configured
    if [[ ":$PATH:" != *":${INSTALL_DIR}:"* ]]; then
        echo ""
        echo "⚠️  ${INSTALL_DIR} is not in your PATH"

        # Detect shell and config file
        SHELL_CONFIG=""
        SHELL_NAME=""

        if [ -n "$BASH_VERSION" ]; then
            SHELL_CONFIG="$HOME/.bashrc"
            SHELL_NAME="bash"
        elif [ -n "$ZSH_VERSION" ]; then
            SHELL_CONFIG="$HOME/.zshrc"
            SHELL_NAME="zsh"
        else
            # Try to detect from SHELL environment variable
            case "$SHELL" in
                */bash)
                    SHELL_CONFIG="$HOME/.bashrc"
                    SHELL_NAME="bash"
                    ;;
                */zsh)
                    SHELL_CONFIG="$HOME/.zshrc"
                    SHELL_NAME="zsh"
                    ;;
                */fish)
                    SHELL_CONFIG="$HOME/.config/fish/config.fish"
                    SHELL_NAME="fish"
                    ;;
                *)
                    SHELL_CONFIG="$HOME/.profile"
                    SHELL_NAME="shell"
                    ;;
            esac
        fi

        # Check if already configured
        if [ -f "$SHELL_CONFIG" ] && grep -q "${INSTALL_DIR}" "$SHELL_CONFIG" 2>/dev/null; then
            echo "✅ PATH already configured in $SHELL_CONFIG"
        else
            # Ask user permission to auto-configure
            echo ""
            echo "Would you like to add ${INSTALL_DIR} to your PATH automatically?"
            read -p "This will modify $SHELL_CONFIG [y/N]: " -n 1 -r
            echo

            if [[ $REPLY =~ ^[Yy]$ ]]; then
                # Add to PATH in config file
                echo "" >> "$SHELL_CONFIG"
                echo "# Added by git-account-manager installer" >> "$SHELL_CONFIG"

                if [ "$SHELL_NAME" = "fish" ]; then
                    echo "set -gx PATH \$HOME/.local/bin \$PATH" >> "$SHELL_CONFIG"
                else
                    echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$SHELL_CONFIG"
                fi

                echo "✅ Added to $SHELL_CONFIG"
                echo ""
                echo "To use ${BINARY_NAME} in this terminal, run:"
                echo "   source $SHELL_CONFIG"
                echo ""
                echo "Or open a new terminal window."
            else
                # Manual instructions
                echo ""
                echo "To add manually, run:"
                if [ "$SHELL_NAME" = "fish" ]; then
                    echo "   echo 'set -gx PATH \$HOME/.local/bin \$PATH' >> $SHELL_CONFIG"
                else
                    echo "   echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> $SHELL_CONFIG"
                fi
                echo "   source $SHELL_CONFIG"
            fi
        fi
    else
        echo "✅ ${INSTALL_DIR} is already in your PATH"
    fi
fi

# Cleanup
rm -rf "${TEMP_DIR}"

echo ""
echo "🎉 Installation complete!"
echo ""

# Show how to run based on installation type
if [ "$INSTALLED" = true ]; then
    echo "Run anywhere: ${BINARY_NAME} --help"
else
    if [[ ":$PATH:" == *":${HOME}/.local/bin:"* ]]; then
        echo "Run anywhere: ${BINARY_NAME} --help"
    else
        echo "Run with: ${HOME}/.local/bin/${BINARY_NAME} --help"
        echo "(or reload shell to use just: ${BINARY_NAME})"
    fi
fi
echo ""
echo "Get started: ${BINARY_NAME}"
echo "(TUI launches automatically - no arguments needed!)"
