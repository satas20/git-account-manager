# Release v0.2.0 - Improved User Experience

## 🎉 What's New in v0.2.0

### 🚀 **TUI Launches by Default**

The biggest change in this release: **You no longer need to type `tui`!**

**Before (v0.1.0):**
```bash
git-acc-mngr tui
```

**Now (v0.2.0):**
```bash
git-acc-mngr
```

The TUI launches automatically when you run the command without arguments, making it faster and more intuitive!

---

## 📝 Changelog

### Added
- ✨ **Default TUI Launch** - Running `git-acc-mngr` without arguments now launches the TUI directly
- ✨ **Version Flag** - Added `--version` flag to display version information
- 📚 **Improved Help Text** - Better descriptions for all commands
- 🔧 **Smart Installer** - New install.sh with automatic PATH configuration and system-wide install support

### Changed
- 🔄 **Updated CLI Interface** - Better command descriptions and help messages
- 📖 **Updated Documentation** - All README examples now reflect the simpler `git-acc-mngr` command
- 🎯 **Improved Installer Messages** - Clearer feedback about installation location and next steps

### Fixed
- 🐛 No critical bugs fixed in this release (first release was stable!)

---

## 🔧 Installation

### Quick Install (Recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/satas20/git-account-manager/main/install.sh | bash
```

The installer will:
1. Try to install system-wide (`/usr/local/bin`) if you have sudo access
2. Fall back to user install (`~/.local/bin`) if not
3. Automatically configure your PATH (with your permission)
4. Show you exactly how to run the tool

### Usage After Installation

Simply run:
```bash
git-acc-mngr
```

That's it! The TUI launches immediately.

---

## 📦 Available Binaries

Download pre-built binaries for your platform:

- **Linux x86_64**: [git-acc-mngr-linux-x86_64.tar.gz](https://github.com/satas20/git-account-manager/releases/download/v0.2.0/git-acc-mngr-linux-x86_64.tar.gz)
- **macOS x86_64**: [git-acc-mngr-macos-x86_64.tar.gz](https://github.com/satas20/git-account-manager/releases/download/v0.2.0/git-acc-mngr-macos-x86_64.tar.gz)
- **Windows x86_64**: [git-acc-mngr-windows-x86_64.exe.tar.gz](https://github.com/satas20/git-account-manager/releases/download/v0.2.0/git-acc-mngr-windows-x86_64.exe.tar.gz)

---

## 🆙 Upgrading from v0.1.0

If you're already using v0.1.0, upgrading is simple:

### Option 1: Re-run the Installer
```bash
curl -fsSL https://raw.githubusercontent.com/satas20/git-account-manager/main/install.sh | bash
```

This will replace your existing installation with v0.2.0.

### Option 2: Manual Upgrade
```bash
# Download the new version
curl -L https://github.com/satas20/git-account-manager/releases/download/v0.2.0/git-acc-mngr-linux-x86_64.tar.gz -o git-acc-mngr.tar.gz

# Extract
tar -xzf git-acc-mngr.tar.gz

# Replace old binary
mv git-acc-mngr-linux-x86_64 ~/.local/bin/git-acc-mngr
chmod +x ~/.local/bin/git-acc-mngr

# Verify version
git-acc-mngr --version
```

### Your Data is Safe
All your existing profiles, SSH keys, and encrypted tokens are preserved. The upgrade only replaces the binary.

---

## 🎯 User Experience Improvements

### Before v0.2.0:
```bash
$ git-acc-mngr
Created profile: work <work@example.com> (host=github.com)

$ git-acc-mngr tui
# TUI launches
```

Users had to **remember** to type `tui` to launch the interface.

### After v0.2.0:
```bash
$ git-acc-mngr
# TUI launches immediately!
```

Much more intuitive! The default action is what users want 99% of the time.

---

## 🧪 Testing

All changes have been tested on:
- ✅ Ubuntu 22.04 (Linux)
- ✅ GitHub Actions CI/CD
- ✅ Installation script (both system-wide and user install)
- ✅ Binary functionality (help, version, profile commands)
- ✅ Default TUI launch

---

## 📚 Documentation Updates

Updated files:
- ✅ [README.md](README.md) - All examples now use `git-acc-mngr` instead of `git-acc-mngr tui`
- ✅ [install.sh](install.sh) - New smart installer with PATH auto-configuration
- ✅ [INSTALLER_IMPROVEMENTS.md](INSTALLER_IMPROVEMENTS.md) - Detailed installer documentation

---

## 🔗 Links

- **GitHub Repository**: https://github.com/satas20/git-account-manager
- **Installation Guide**: [README.md](README.md)
- **Report Issues**: https://github.com/satas20/git-account-manager/issues

---

## 🙏 Thank You

Thank you for using Git Account Manager! This release focuses on making the tool more intuitive and easier to use.

If you encounter any issues or have suggestions, please [open an issue](https://github.com/satas20/git-account-manager/issues).

---

## 🚦 Release Checklist

Before creating the release, ensure:

- [x] Version bumped to 0.2.0 in Cargo.toml
- [x] main.rs updated with default TUI behavior
- [x] --version flag working
- [x] README.md updated with new command examples
- [x] install.sh updated with new messaging
- [x] Binary builds successfully (`cargo build --release`)
- [x] All commands tested (help, version, profile, default)
- [ ] Changes committed to git
- [ ] Git tag created (v0.2.0)
- [ ] GitHub release created with binaries
- [ ] Release notes published

---

## 🔜 Next Steps for Release

1. **Commit all changes:**
   ```bash
   git add .
   git commit -m "Release v0.2.0: TUI launches by default + improved installer"
   ```

2. **Create git tag:**
   ```bash
   git tag -a v0.2.0 -m "Release v0.2.0 - TUI launches by default"
   ```

3. **Push to GitHub:**
   ```bash
   git push origin main
   git push origin v0.2.0
   ```

4. **GitHub Actions will automatically:**
   - Build binaries for Linux, macOS, Windows
   - Create a GitHub Release
   - Upload binaries to the release

5. **Verify the release:**
   - Check https://github.com/satas20/git-account-manager/releases
   - Test the one-line installer with the new version
   - Verify binaries are available

---

**Ready to release!** 🚀
