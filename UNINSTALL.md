# Uninstall Git Account Manager

This guide shows you how to completely remove Git Account Manager from your system.

---

## Quick Uninstall

Run this command to remove everything:

```bash
# Remove binary (try both possible locations)
sudo rm -f /usr/local/bin/git-acc-mngr
rm -f ~/.local/bin/git-acc-mngr

# Remove configuration and data (OPTIONAL - keeps your profiles)
rm -rf ~/.config/git-account-manager
```

**Warning:** The last command removes all your profiles, SSH keys, and tokens!

---

## Step-by-Step Uninstall

### Step 1: Remove the Binary

The binary could be in one of two locations:

#### **System-wide installation** (if installed with sudo):
```bash
sudo rm -f /usr/local/bin/git-acc-mngr
```

#### **User installation** (default):
```bash
rm -f ~/.local/bin/git-acc-mngr
```

#### **Check if it's removed:**
```bash
which git-acc-mngr
# Should return nothing if removed successfully
```

---

### Step 2: Remove Configuration (Optional)

Your profiles, SSH keys, and encrypted tokens are stored in:
```
~/.config/git-account-manager/
```

**To keep your data** (for reinstall):
```bash
# Don't delete anything - just reinstall later
```

**To remove everything:**
```bash
rm -rf ~/.config/git-account-manager
```

This removes:
- ❌ All profiles
- ❌ All SSH keys
- ❌ Encrypted OAuth tokens
- ❌ Master encryption key

---

### Step 3: Clean Up PATH (Optional)

If you want to remove the PATH configuration added by the installer:

```bash
# Edit your shell config
nano ~/.bashrc   # or ~/.zshrc for zsh

# Find and remove this line:
# export PATH="$HOME/.local/bin:$PATH"

# Or remove just the git-account-manager comment:
# # Added by git-account-manager installer
# export PATH="$HOME/.local/bin:$PATH"

# Reload shell
source ~/.bashrc
```

**Note:** Only remove the PATH line if you don't use `~/.local/bin` for other tools!

---

## Complete Removal Script

Save this as `uninstall.sh`:

```bash
#!/bin/bash
echo "🗑️  Uninstalling Git Account Manager..."

# Remove binary (both locations)
if [ -f /usr/local/bin/git-acc-mngr ]; then
    echo "Removing system-wide installation..."
    sudo rm -f /usr/local/bin/git-acc-mngr && echo "✅ Removed from /usr/local/bin"
fi

if [ -f ~/.local/bin/git-acc-mngr ]; then
    echo "Removing user installation..."
    rm -f ~/.local/bin/git-acc-mngr && echo "✅ Removed from ~/.local/bin"
fi

# Verify removal
if command -v git-acc-mngr &> /dev/null; then
    echo "⚠️  Binary still found in PATH"
else
    echo "✅ Binary removed successfully"
fi

# Ask about data removal
echo ""
echo "Do you want to remove all profiles and data?"
echo "This will delete:"
echo "  - All Git profiles"
echo "  - All SSH keys"
echo "  - All encrypted tokens"
read -p "Remove all data? [y/N]: " -n 1 -r
echo

if [[ $REPLY =~ ^[Yy]$ ]]; then
    rm -rf ~/.config/git-account-manager
    echo "✅ Removed all data from ~/.config/git-account-manager"
else
    echo "ℹ️  Kept data in ~/.config/git-account-manager"
    echo "   (You can reinstall and keep your profiles)"
fi

echo ""
echo "🎉 Uninstall complete!"
```

Run it:
```bash
chmod +x uninstall.sh
bash uninstall.sh
```

---

## Reinstall After Uninstall

### Fresh Install (Latest Version)
```bash
curl -fsSL https://raw.githubusercontent.com/satas20/git-account-manager/main/install.sh | bash
```

### Install Specific Version
```bash
# Example: Install v0.1.0
curl -L https://github.com/satas20/git-account-manager/releases/download/v0.1.0/git-acc-mngr-linux-x86_64.tar.gz -o git-acc-mngr.tar.gz
tar -xzf git-acc-mngr.tar.gz
sudo mv git-acc-mngr-linux-x86_64 /usr/local/bin/git-acc-mngr
chmod +x /usr/local/bin/git-acc-mngr
```

---

## Verify Uninstall

```bash
# Check if binary is removed
which git-acc-mngr
# Should output nothing

# Check if data is removed (if you chose to remove it)
ls -la ~/.config/git-account-manager
# Should say "No such file or directory"

# Check PATH still works
echo $PATH
# Should still show other paths correctly
```

---

## Troubleshooting

### "Permission denied" when removing system binary
```bash
# Use sudo
sudo rm -f /usr/local/bin/git-acc-mngr
```

### Binary still appears after removal
```bash
# Find all copies
find ~/ -name "git-acc-mngr" 2>/dev/null
find /usr -name "git-acc-mngr" 2>/dev/null

# Remove each one
rm -f <path-to-binary>
```

### Want to keep data but reinstall
```bash
# Just remove binary, keep data
rm -f ~/.local/bin/git-acc-mngr
sudo rm -f /usr/local/bin/git-acc-mngr

# Reinstall - your profiles will still be there!
curl -fsSL https://raw.githubusercontent.com/satas20/git-account-manager/main/install.sh | bash
```

---

## What Gets Removed vs What Stays

| Item | Location | Removed by Uninstall? |
|------|----------|----------------------|
| Binary | `/usr/local/bin/` or `~/.local/bin/` | ✅ Yes |
| Profiles data | `~/.config/git-account-manager/profiles.json` | 🟡 Optional |
| SSH keys | `~/.config/git-account-manager/keys/` | 🟡 Optional |
| Encryption key | `~/.config/git-account-manager/master.key` | 🟡 Optional |
| Git config | `~/.gitconfig` | ❌ No (keeps current user) |
| SSH config | `~/.ssh/config` | ❌ No (keeps SSH entries) |
| PATH setup | `~/.bashrc` | ❌ No (manual removal) |

---

## Quick Reference

```bash
# Remove binary only
rm -f ~/.local/bin/git-acc-mngr
sudo rm -f /usr/local/bin/git-acc-mngr

# Remove everything (including data)
rm -f ~/.local/bin/git-acc-mngr
sudo rm -f /usr/local/bin/git-acc-mngr
rm -rf ~/.config/git-account-manager

# Reinstall
curl -fsSL https://raw.githubusercontent.com/satas20/git-account-manager/main/install.sh | bash
```
