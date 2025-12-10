# Git Account Manager 🔐

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
- 🔄 **Account Sync** - Refresh profile data and rotate SSH keys
- 🖥️ **Cross-Platform** - Works on Linux, macOS, and Windows
- 📦 **Clean Architecture** - Hexagonal architecture for maintainability

## 📋 What It Does

Git Account Manager handles the complexity of managing multiple Git identities by:

1. **OAuth Flow** - Opens your browser to authenticate with GitHub/GitLab
2. **Profile Management** - Stores user information (name, email) for each account
3. **SSH Key Generation** - Creates unique Ed25519 SSH keys for each profile
4. **Automatic Configuration** - Updates `~/.ssh/config` and `~/.gitconfig` when switching
5. **Token Encryption** - Securely stores OAuth tokens with device-specific encryption
6. **Remote SSH Upload** - Automatically uploads public keys to your GitHub/GitLab account

## 🚀 Quick Start

### One-Line Installation (Recommended)

**Linux, macOS, or Windows (WSL):**
```bash
curl -fsSL https://raw.githubusercontent.com/satas20/git-account-manager/main/install.sh | bash
```

Then run:
```bash
git-acc-mngr
```

The TUI (Terminal User Interface) launches automatically!

### Windows Users

We **strongly recommend** using WSL for the best experience:

1. **Install WSL** (PowerShell as Administrator):
   ```powershell
   wsl --install
   ```

2. **Restart** your computer

3. **Open WSL** and run the installation command above

> **Why WSL?** Better Git/SSH integration, consistent environment, and full Linux tooling support. The native Windows binary is available but not recommended for this tool.

### Windows Native Installation (Alternative)

If you prefer not to use WSL, here are the manual installation steps:

**Using Git Bash or PowerShell:**
```bash
# Download the Windows binary
curl -L https://github.com/satas20/git-account-manager/releases/latest/download/git-acc-mngr-windows-x86_64.exe.tar.gz -o git-acc-mngr-windows.tar.gz

# Extract
tar -xzf git-acc-mngr-windows.tar.gz

# Create bin directory if it doesn't exist
mkdir -p $HOME/.local/bin

# Move binary to bin
mv git-acc-mngr-windows-x86_64.exe $HOME/.local/bin/git-acc-mngr.exe

# Add to PATH (add this to your ~/.bashrc or ~/.bash_profile)
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc

# Now you can run it
git-acc-mngr --help
```

### Build from Source

**Prerequisites:**
- Rust 1.70 or later
- Git installed on your system
- SSH installed (comes with most systems)

```bash
# Clone the repository
git clone https://github.com/satas20/git-account-manager.git
cd git-account-manager

# Build the project
cargo build --release

# Run it (TUI launches automatically)
./target/release/git-acc-mngr
```

### Setup OAuth Credentials

Before using the application, you need to register OAuth applications with GitHub and/or GitLab:

#### GitHub Setup

1. Go to [GitHub Developer Settings](https://github.com/settings/developers)
2. Click "New OAuth App"
3. Fill in the details:
   - **Application name**: Git Account Manager
   - **Homepage URL**: `http://localhost`
   - **Authorization callback URL**: `http://127.0.0.1:8787/callback`
4. Copy the **Client ID** and **Client Secret**

#### GitLab Setup

1. Go to [GitLab Applications](https://gitlab.com/-/profile/applications)
2. Click "Add new application"
3. Fill in the details:
   - **Name**: Git Account Manager
   - **Redirect URI**: `http://127.0.0.1:8788/callback`
   - **Scopes**: Select `api` and `read_user`
4. Copy the **Application ID** and **Secret**

#### Configure Environment Variables

```bash
# Copy the example environment file
cp .env.example .env

# Edit .env and fill in your OAuth credentials
nano .env  # or use your preferred editor
```

Your `.env` file should look like:

```bash
GITHUB_CLIENT_ID=your_github_client_id_here
GITHUB_CLIENT_SECRET=your_github_client_secret_here

GITLAB_APP_ID=your_gitlab_application_id_here
GITLAB_CLIENT_SECRET=your_gitlab_client_secret_here
```

**Important**: Keep your `.env` file secure and never commit it to version control!

### Load Environment and Run

```bash
# Load environment variables
source .env

# Run the application (TUI launches automatically)
git-acc-mngr
```

## 📖 Usage

### Adding a Profile

1. Launch the application with `git-acc-mngr` (no arguments needed!)
2. Press `1` to enter Profiles menu
3. Press `0` to add a new profile
4. Select your provider (GitHub or GitLab)
5. Your browser will open for OAuth authentication
6. Grant permissions and return to the terminal

The application will automatically:

- Fetch your profile information
- Generate an SSH key pair
- Upload the public key to your account
- Store encrypted tokens

### Switching Profiles

1. In the Profiles menu, select a profile by number
2. Press `1` to switch
3. Your git config and SSH settings are updated automatically

### Syncing Account

The sync feature refreshes your profile data and rotates SSH keys:

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
3. The application will clean up:
   - Local profile data
   - SSH key files
   - Remote SSH key from provider
   - Git configuration (if current)

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

## 🏗️ Architecture

This project follows **Hexagonal Architecture** (Ports and Adapters):

```
src/
├── domain/              # Core business logic
│   ├── entity.rs       # Domain entities (Profile)
│   ├── ports.rs        # Port traits (interfaces)
│   ├── use_cases.rs    # Business use cases
│   └── error.rs        # Domain errors
├── adapters/           # External integrations
│   ├── tui.rs         # Terminal UI (Ratatui)
│   ├── github.rs      # GitHub OAuth & API
│   ├── gitlab.rs      # GitLab OAuth & API
│   └── system_io.rs   # File system operations
└── main.rs            # Application entry point
```

### Key Design Principles

- **Pure Domain Logic**: No dependencies on external frameworks in domain layer
- **Dependency Inversion**: Adapters depend on domain ports (traits)
- **Testability**: Each layer can be tested independently
- **Flexibility**: Easy to swap implementations (e.g., different storage backends)

## 🔒 Security

- **Encrypted Storage**: OAuth tokens are encrypted using ChaCha20-Poly1305
- **Device-Specific Keys**: Encryption keys are generated per device
- **Secure Permissions**: Key files have restricted permissions (0600)
- **No Plaintext Secrets**: Tokens never stored in plaintext
- **CSRF Protection**: OAuth flow includes state parameter validation

## 🛠️ Development

### Building from Source

```bash
# Development build
cargo build

# Release build with optimizations
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run
```

### Project Dependencies

- **tokio** - Async runtime
- **reqwest** - HTTP client for OAuth
- **ratatui** - Terminal UI framework
- **crossterm** - Cross-platform terminal manipulation
- **chacha20poly1305** - Encryption
- **serde/serde_json** - Serialization
- **clap** - CLI argument parsing

## 🤝 Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on our code of conduct and the process for submitting pull requests.

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## ⚠️ Troubleshooting

### Browser doesn't open during OAuth

The application will print the OAuth URL. Copy and paste it into your browser manually.

### SSH keys not working

Ensure the SSH key was uploaded successfully:

- Check your GitHub/GitLab SSH keys settings
- Verify the key matches: `cat ~/.config/git-account-manager/keys/username_at_github/id_ed25519.pub`

### Permission denied errors

The application needs write access to:

- `~/.config/git-account-manager/` - Profile storage
- `~/.ssh/config` - SSH configuration
- `~/.gitconfig` - Git configuration

### Token expired errors

Use the "Sync account" feature to refresh tokens and rotate SSH keys.

## 🙏 Acknowledgments

- Built with [Ratatui](https://github.com/ratatui-org/ratatui) for the beautiful TUI
- Inspired by the need to manage multiple Git identities efficiently
- Thanks to the Rust community for excellent crates and documentation

## 📧 Support

For issues, questions, or suggestions:

- Open an [issue](https://github.com/satas20/git-account-manager/issues)
- Check existing [discussions](https://github.com/satas20/git-account-manager/discussions)

---

**Made with ❤️ and Rust**
