#!/usr/bin/env bash
# Test script for the improved installer

echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║   Git Account Manager - Installer Test Suite                 ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo ""

# Test 1: Syntax validation
echo "Test 1: Syntax Validation"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if bash -n install.sh; then
    echo "✅ PASS: Syntax is valid"
else
    echo "❌ FAIL: Syntax errors found"
    exit 1
fi
echo ""

# Test 2: Check for required commands
echo "Test 2: Required Commands Check"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
MISSING=""
for cmd in curl tar mkdir install; do
    if command -v $cmd &> /dev/null; then
        echo "✅ $cmd found"
    else
        echo "❌ $cmd missing"
        MISSING="$MISSING $cmd"
    fi
done

if [ -z "$MISSING" ]; then
    echo "✅ PASS: All required commands available"
else
    echo "❌ FAIL: Missing commands:$MISSING"
    exit 1
fi
echo ""

# Test 3: OS Detection
echo "Test 3: OS Detection"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
OS=$(uname -s)
case "$OS" in
    Linux*)
        echo "✅ PASS: Detected Linux"
        PLATFORM="linux"
        ;;
    Darwin*)
        echo "✅ PASS: Detected macOS"
        PLATFORM="macos"
        ;;
    *)
        echo "❌ FAIL: Unsupported OS: $OS"
        exit 1
        ;;
esac
echo ""

# Test 4: GitHub Release Access
echo "Test 4: GitHub Release Access"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
REPO="satas20/git-account-manager"
VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -n "$VERSION" ]; then
    echo "✅ PASS: Found latest version: $VERSION"
else
    echo "❌ FAIL: Could not fetch release information"
    exit 1
fi
echo ""

# Test 5: Binary Availability
echo "Test 5: Binary Availability for $PLATFORM"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
ASSET_NAME="git-acc-mngr-${PLATFORM}-x86_64.tar.gz"
URL="https://github.com/${REPO}/releases/download/${VERSION}/${ASSET_NAME}"

if curl --head --silent --fail "$URL" > /dev/null; then
    echo "✅ PASS: Binary available at: $URL"
else
    echo "❌ FAIL: Binary not found at: $URL"
    exit 1
fi
echo ""

# Test 6: Installation Paths
echo "Test 6: Installation Path Logic"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check sudo availability
if sudo -n true 2>/dev/null; then
    echo "✅ Passwordless sudo available → Will try /usr/local/bin"
    PREFERRED_PATH="/usr/local/bin"
elif [ -w "/usr/local/bin" ] 2>/dev/null; then
    echo "✅ /usr/local/bin writable → Will install there"
    PREFERRED_PATH="/usr/local/bin"
else
    echo "⚠️  No sudo/write access → Will use ~/.local/bin"
    PREFERRED_PATH="$HOME/.local/bin"
fi
echo ""

# Test 7: Shell Detection
echo "Test 7: Shell Detection"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ -n "$BASH_VERSION" ]; then
    echo "✅ Running in bash → Will use ~/.bashrc"
    SHELL_CONFIG="$HOME/.bashrc"
elif [ -n "$ZSH_VERSION" ]; then
    echo "✅ Running in zsh → Will use ~/.zshrc"
    SHELL_CONFIG="$HOME/.zshrc"
else
    case "$SHELL" in
        */bash)
            echo "✅ Shell is bash → Will use ~/.bashrc"
            SHELL_CONFIG="$HOME/.bashrc"
            ;;
        */zsh)
            echo "✅ Shell is zsh → Will use ~/.zshrc"
            SHELL_CONFIG="$HOME/.zshrc"
            ;;
        */fish)
            echo "✅ Shell is fish → Will use ~/.config/fish/config.fish"
            SHELL_CONFIG="$HOME/.config/fish/config.fish"
            ;;
        *)
            echo "⚠️  Unknown shell → Will use ~/.profile"
            SHELL_CONFIG="$HOME/.profile"
            ;;
    esac
fi
echo ""

# Test 8: PATH Check
echo "Test 8: Current PATH Configuration"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [[ ":$PATH:" == *":$HOME/.local/bin:"* ]]; then
    echo "✅ ~/.local/bin is in PATH"
else
    echo "⚠️  ~/.local/bin is NOT in PATH"
fi

if [[ ":$PATH:" == *":/usr/local/bin:"* ]]; then
    echo "✅ /usr/local/bin is in PATH"
else
    echo "⚠️  /usr/local/bin is NOT in PATH (unusual)"
fi
echo ""

# Summary
echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║                   Test Summary                                ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo ""
echo "All tests passed! ✅"
echo ""
echo "Installation will:"
echo "  • Target: $PREFERRED_PATH"
echo "  • Shell config: $SHELL_CONFIG"
echo ""
echo "To run the actual installer:"
echo "  bash install.sh"
echo ""
echo "Or use the one-liner:"
echo "  curl -fsSL https://raw.githubusercontent.com/satas20/git-account-manager/main/install.sh | bash"
