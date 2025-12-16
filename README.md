# Git Account Manager 🔐

<p align="center">
  <img src="git-accmngr-logo.png" alt="Git Account Manager Logo" width="200"/>
</p>

```
                          :==
                        :*****=
                        +*******+
                     +:   ********=
                   +****.     =*****:
                 +*******      *******
               -*********=     :********
             .************-  .   -********
            **************-  :*-     -*****=
           -**************-  :**      ******
             +************-  :**=    =****+
              :***********-  :+        +*
                =********.    :-.    .-
                  =******     .******.
                    =****=.  .=****-
                      ++        *+
                       .*-.  .=+
                         :***+
```

A powerful terminal-based tool for managing multiple Git identities with OAuth authentication and automatic SSH key management. Switch between GitHub and GitLab accounts seamlessly with encrypted token storage and profile-based SSH configuration.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)
![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-lightgrey.svg)

## ✨ Features

- 🔐 **OAuth Authentication** - Secure authentication with GitHub and GitLab
- 🔑 **Automatic SSH Key Management** - Generates and uploads Ed25519 SSH keys per profile
- 🔒 **Encrypted Token Storage** - ChaCha20-Poly1305 encryption for OAuth tokens
- 🔄 **Profile Switching** - Instantly switch between different Git identities
- 🎨 **Beautiful TUI** - Modern terminal interface built with Ratatui
- 🖥️ **Cross-Platform** - Works on Linux, macOS, and Windows

## 🚀 Installation

### Quick Install

#### Linux & macOS

```bash
curl -fsSL https://raw.githubusercontent.com/satas20/git-account-manager/main/install.sh | bash
```

#### Windows (Recommended: WSL)

1. **Install WSL** (PowerShell as Administrator):

   ```powershell
   wsl --install
   ```

2. **Restart** your computer

3. **Open WSL** and run:
   ```bash
   curl -fsSL https://raw.githubusercontent.com/satas20/git-account-manager/main/install.sh | bash
   ```

> **Why WSL?** Better Git/SSH integration and consistent environment.

### Windows (Native - PowerShell)

```powershell
# Download and extract
Invoke-WebRequest -Uri "https://github.com/satas20/git-account-manager/releases/latest/download/git-acc-mngr-windows-x86_64.exe.tar.gz" -OutFile "git-acc-mngr.tar.gz"
tar -xzf git-acc-mngr.tar.gz

# Install
New-Item -ItemType Directory -Force -Path "$env:USERPROFILE\.local\bin"
Move-Item -Force git-acc-mngr-windows-x86_64.exe "$env:USERPROFILE\.local\bin\git-acc-mngr.exe"

# Add to PATH (requires restart)
[Environment]::SetEnvironmentVariable("Path", $env:Path + ";$env:USERPROFILE\.local\bin", [EnvironmentVariableTarget]::User)
```

After installation, restart PowerShell and verify:

```powershell
git-acc-mngr --version
```

### Manual Download

Prefer to download manually? Get the latest release for your platform:

**[📥 Download from GitHub Releases](https://github.com/satas20/git-account-manager/releases/latest)**

Available binaries:

- `git-acc-mngr-linux-x86_64.tar.gz` - Linux
- `git-acc-mngr-macos-x86_64.tar.gz` - macOS
- `git-acc-mngr-windows-x86_64.exe.tar.gz` - Windows

Extract and move to your PATH, or follow the platform-specific instructions above.

## 📖 Usage

### Launch the Application

```bash
git-acc-mngr
```

The TUI (Terminal User Interface) launches automatically!

### Adding a Profile

1. Press `1` to enter Profiles menu
2. Press `0` to add a new profile
3. Select your provider (GitHub or GitLab)
4. Your browser opens for OAuth authentication
5. Grant permissions and return to the terminal

The application automatically:

- Fetches your profile information
- Generates an SSH key pair
- Uploads the public key to your account
- Stores encrypted tokens

### Switching Profiles

1. In the Profiles menu, select a profile by number
2. Press `1` to switch
3. Your git config and SSH settings update automatically

### Syncing Account

Refreshes profile data and rotates SSH keys:

1. Select a profile
2. Press `3` to sync
3. The application will:
   - Delete the old SSH key from the remote provider
   - Fetch updated profile information
   - Generate a new SSH key pair
   - Upload the new public key

### Removing a Profile

1. Select a profile
2. Press `2` to remove
3. Cleans up all local and remote data

## 🗂️ File Structure

```
~/.config/git-account-manager/
├── profiles.json          # Profile metadata (encrypted tokens)
├── master.key            # Encryption master key (keep secure!)
└── keys/
    ├── username_at_github/
    │   ├── id_ed25519    # Private SSH key
    │   └── id_ed25519.pub # Public SSH key
    └── username_at_gitlab/
        ├── id_ed25519
        └── id_ed25519.pub

~/.ssh/
└── config               # Updated with profile SSH configurations

~/.gitconfig            # Updated with current profile
```

## 🔒 Security

- **Encrypted Storage**: OAuth tokens encrypted using ChaCha20-Poly1305
- **Device-Specific Keys**: Encryption keys generated per device
- **Secure Permissions**: Key files have restricted permissions (0600)
- **No Plaintext Secrets**: Tokens never stored in plaintext
- **CSRF Protection**: OAuth flow includes state parameter validation

## 🗑️ Uninstalling

```bash
curl -fsSL https://raw.githubusercontent.com/satas20/git-account-manager/main/uninstall.sh | bash
```

Or manually:

```bash
# Remove the binary
rm -f ~/.local/bin/git-acc-mngr
sudo rm -f /usr/local/bin/git-acc-mngr

# (Optional) Remove all profiles and data
rm -rf ~/.config/git-account-manager
```

**Note:** Removing `~/.config/git-account-manager` deletes all your profiles, SSH keys, and encrypted tokens.

## 🛠️ Development

Want to contribute or build from source? See [DEVELOPMENT.md](DEVELOPMENT.md) for setup instructions.

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

**Made with ❤️ and Rust**
