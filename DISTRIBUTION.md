# Distribution Guide - Git Account Manager

## 🎯 Simple Distribution Strategy (x86_64 only)

Your project is now configured for **one-line installation** via curl.

---

## 📦 What Was Configured

### 1. Binary Name

- **Binary:** `git-acc-mngr`
- Users will run: `git-acc-mngr tui`

### 2. GitHub Actions Workflow

- **File:** `.github/workflows/release.yml`
- **Triggers:** When you push a git tag (e.g., `v0.1.0`)
- **Builds for:**
  - Linux x86_64 (musl for maximum compatibility)
  - macOS x86_64 (Intel Macs)
  - Windows x86_64 (MSVC - provided but WSL is recommended)

### 3. Install Script

- **File:** `install.sh`
- Detects OS automatically
- Downloads correct binary from GitHub releases
- Installs to `~/.local/bin`

---

## 🚀 How to Release a New Version

### Step 1: Prepare Release

```bash
# Make sure everything is committed
git status

# Update version in Cargo.toml if needed
# Update CHANGELOG or release notes
```

### Step 2: Create and Push Tag

```bash
# Create a version tag
git tag v0.1.0

# Push the tag to GitHub
git push origin v0.1.0
```

### Step 3: Automatic Build

GitHub Actions will automatically:

1. Build binaries for Linux, macOS, and Windows
2. Create a GitHub Release
3. Attach compiled binaries as release assets

### Step 4: Verify Release

1. Go to: `https://github.com/satas20/git-account-manager/releases`
2. Check that all 3 binaries were uploaded:
   - `git-acc-mngr-linux-x86_64.tar.gz`
   - `git-acc-mngr-macos-x86_64.tar.gz`
   - `git-acc-mngr-windows-x86_64.exe.tar.gz`

---

## 💻 Installation Methods

### Method 1: One-Line Install (Recommended)

**For Linux, macOS, and Windows (WSL):**
```bash
curl -fsSL https://raw.githubusercontent.com/satas20/git-account-manager/main/install.sh | bash
```

**Windows Users:** We strongly recommend using [WSL (Windows Subsystem for Linux)](https://learn.microsoft.com/en-us/windows/wsl/install) for the best experience:
```powershell
# Install WSL (PowerShell as Admin)
wsl --install

# Restart your computer, then in WSL:
curl -fsSL https://raw.githubusercontent.com/satas20/git-account-manager/main/install.sh | bash
```

### Method 2: Manual Install

#### Linux/macOS/WSL

```bash
# Download latest release
wget https://github.com/satas20/git-account-manager/releases/latest/download/git-acc-mngr-linux-x86_64.tar.gz

# Extract
tar -xzf git-acc-mngr-linux-x86_64.tar.gz

# Move to PATH
mv git-acc-mngr-linux-x86_64 ~/.local/bin/git-acc-mngr
chmod +x ~/.local/bin/git-acc-mngr

# Add to PATH (if not already)
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

#### Windows (Native - Not Recommended)

A native Windows binary is available, but WSL is strongly recommended for better Git/SSH integration:
1. Download: `git-acc-mngr-windows-x86_64.exe.tar.gz`
2. Extract the .exe file
3. Add to PATH or run directly

### Method 3: Build from Source

```bash
git clone https://github.com/satas20/git-account-manager.git
cd git-account-manager
cargo build --release
# Binary at: target/release/git-acc-mngr
```

---

## 🛠️ Setup After Installation

### 1. Create Environment File

```bash
mkdir -p ~/.config/git-acc-mngr
nano ~/.config/git-acc-mngr/.env
```

### 2. Add OAuth Credentials

```bash
# GitHub OAuth App Credentials
GITHUB_CLIENT_ID=your_github_client_id
GITHUB_CLIENT_SECRET=your_github_client_secret

# GitLab OAuth App Credentials
GITLAB_APP_ID=your_gitlab_app_id
GITLAB_CLIENT_SECRET=your_gitlab_client_secret
```

### 3. Run the Application

```bash
git-acc-mngr tui
```

---

## 📝 Before Your First Release

### Update These Files:

1. **`install.sh`** (line 6):

   ```bash
   REPO="satas20/git-account-manager"
   ```

2. **`Cargo.toml`** (line 8):

   ```toml
   repository = "https://github.com/satas20/git-account-manager"
   ```

3. **`.github/workflows/release.yml`**:
   - No changes needed, uses repo context automatically

### Create OAuth Apps:

#### GitHub OAuth App

1. Go to: https://github.com/settings/developers
2. Click "New OAuth App"
3. Set **Authorization callback URL**: `http://127.0.0.1:8787/callback`
4. Copy Client ID and Secret

#### GitLab OAuth App

1. Go to: https://gitlab.com/-/profile/applications
2. Click "Add new application"
3. Scopes: `api`, `read_user`
4. Redirect URI: `http://127.0.0.1:8787/callback`
5. Copy Application ID and Secret

---

## 🎉 Marketing Your Release

### Promote With This Command:

```bash
curl -fsSL https://raw.githubusercontent.com/satas20/git-account-manager/main/install.sh | bash
```

### Share On:

- GitHub README.md
- Dev.to / Hashnode blog post
- Twitter / Reddit (r/rust, r/programming)
- Hacker News

### Key Selling Points:

✅ **One-line installation**
✅ **Secure OAuth authentication** (no passwords stored)
✅ **Cross-platform** (Linux, macOS, Windows)
✅ **Zero-dependency binary** (no runtime required)
✅ **Automatic SSH key management**
✅ **Terminal UI** for easy profile switching

---

## 🔧 Maintenance

### Update Release:

1. Make your changes and commit
2. Tag new version: `git tag v0.2.0`
3. Push tag: `git push origin v0.2.0`
4. GitHub Actions builds automatically

### Delete Bad Release:

```bash
# Delete tag locally
git tag -d v0.1.0

# Delete tag on GitHub
git push origin :refs/tags/v0.1.0

# Delete GitHub Release via web UI
```

---

## 🎯 Next Steps (Optional)

### 1. Add to Package Managers

- **Cargo (Rust):** `cargo publish`
- **Homebrew (macOS):** Create tap
- **AUR (Arch Linux):** Create PKGBUILD

### 2. Create Website

- Landing page at: `satas20.github.io/git-account-manager`
- Show demo GIF/video
- Quick start guide

### 3. Add Telemetry (Optional)

- Usage analytics (anonymous)
- Error reporting with Sentry

---

## ✅ Checklist for First Release

- [ ] Update `REPO` in `install.sh`
- [ ] Update `repository` in `Cargo.toml`
- [ ] Create GitHub OAuth App
- [ ] Create GitLab OAuth App
- [ ] Test build: `cargo build --release`
- [ ] Commit and push all changes
- [ ] Create first tag: `git tag v0.1.0`
- [ ] Push tag: `git push origin v0.1.0`
- [ ] Verify GitHub Release created
- [ ] Test installation script
- [ ] Update README.md with install command
- [ ] Announce release! 🎉
