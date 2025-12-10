#!/bin/bash
echo "╔═══════════════════════════════════════════════════════════╗"
echo "║   Git Account Manager - Uninstaller                      ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""

# Check if binary exists
FOUND=0
if [ -f /usr/local/bin/git-acc-mngr ]; then
    echo "✓ Found system installation: /usr/local/bin/git-acc-mngr"
    FOUND=1
fi

if [ -f ~/.local/bin/git-acc-mngr ]; then
    echo "✓ Found user installation: ~/.local/bin/git-acc-mngr"
    FOUND=1
fi

if [ $FOUND -eq 0 ]; then
    echo "ℹ️  No installation found"
    exit 0
fi

echo ""
echo "What would you like to remove?"
echo ""
echo "1) Binary only (keep profiles and data)"
echo "2) Binary + all data (complete removal)"
echo "3) Cancel"
echo ""
read -p "Choose [1-3]: " choice

case $choice in
    1)
        echo ""
        echo "Removing binary only..."
        rm -f ~/.local/bin/git-acc-mngr 2>/dev/null && echo "✓ Removed ~/.local/bin/git-acc-mngr"
        sudo rm -f /usr/local/bin/git-acc-mngr 2>/dev/null && echo "✓ Removed /usr/local/bin/git-acc-mngr"
        echo ""
        echo "✅ Binary removed (data preserved in ~/.config/git-account-manager)"
        ;;
    2)
        echo ""
        echo "⚠️  This will remove:"
        echo "   • Binary"
        echo "   • All profiles"
        echo "   • All SSH keys"
        echo "   • All encrypted tokens"
        echo ""
        read -p "Are you sure? [y/N]: " confirm
        if [[ $confirm =~ ^[Yy]$ ]]; then
            rm -f ~/.local/bin/git-acc-mngr 2>/dev/null
            sudo rm -f /usr/local/bin/git-acc-mngr 2>/dev/null
            rm -rf ~/.config/git-account-manager
            echo ""
            echo "✅ Complete removal finished"
        else
            echo "Cancelled"
        fi
        ;;
    3)
        echo "Cancelled"
        exit 0
        ;;
    *)
        echo "Invalid choice"
        exit 1
        ;;
esac

echo ""
if command -v git-acc-mngr &> /dev/null; then
    echo "⚠️  Binary still found in PATH at: $(which git-acc-mngr)"
else
    echo "🎉 Uninstall complete!"
fi
