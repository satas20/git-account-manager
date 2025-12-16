# Development Guide

This guide is for developers who want to contribute to Git Account Manager or build from source.

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

## 🛠️ Prerequisites

- **Rust 1.70 or later**
- **Git** installed on your system
- **SSH** installed (comes with most systems)
- GitHub and/or GitLab account for OAuth testing

## 📦 Building from Source

### 1. Clone the Repository

```bash
git clone https://github.com/satas20/git-account-manager.git
cd git-account-manager
```

### 2. Setup OAuth Credentials

You need to register OAuth applications with GitHub and/or GitLab:

#### GitHub OAuth App Setup

1. Go to [GitHub Developer Settings](https://github.com/settings/developers)
2. Click "New OAuth App"
3. Fill in the details:
   - **Application name**: Git Account Manager (Dev)
   - **Homepage URL**: `http://localhost`
   - **Authorization callback URL**: `http://127.0.0.1:8787/callback`
4. Copy the **Client ID** and **Client Secret**

#### GitLab OAuth App Setup

1. Go to [GitLab Applications](https://gitlab.com/-/profile/applications)
2. Click "Add new application"
3. Fill in the details:
   - **Name**: Git Account Manager (Dev)
   - **Redirect URI**: `http://127.0.0.1:8788/callback`
   - **Scopes**: Select `api` and `read_user`
4. Copy the **Application ID** and **Secret**

### 3. Configure Environment Variables

Create a `.env` file in the project root:

```bash
cp .env.example .env
```

Edit `.env` and add your OAuth credentials:

```bash
GITHUB_CLIENT_ID=your_github_client_id_here
GITHUB_CLIENT_SECRET=your_github_client_secret_here

GITLAB_APP_ID=your_gitlab_application_id_here
GITLAB_CLIENT_SECRET=your_gitlab_client_secret_here
```

**Important**: Never commit your `.env` file to version control!

### 4. Build and Run

```bash
# Development build (faster compilation, slower runtime)
cargo build

# Release build (slower compilation, optimized runtime)
cargo build --release

# Run development build
cargo run

# Run release build
./target/release/git-acc-mngr

# Run with debug logging
RUST_LOG=debug cargo run
```

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

## 📚 Project Dependencies

- **tokio** - Async runtime for non-blocking I/O
- **reqwest** - HTTP client for OAuth and API calls
- **ratatui** - Terminal UI framework
- **crossterm** - Cross-platform terminal manipulation
- **chacha20poly1305** - Encryption for token storage
- **serde/serde_json** - Serialization and deserialization
- **clap** - CLI argument parsing
- **cli-clipboard** - Clipboard support for OAuth URLs

## 🔧 Development Workflow

### Code Style

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Fix clippy warnings
cargo clippy --fix
```

### Adding a New Feature

1. Create a feature branch: `git checkout -b feature/your-feature`
2. Implement in the appropriate layer (domain/adapters)
3. Add tests for your feature
4. Run tests: `cargo test`
5. Format and lint: `cargo fmt && cargo clippy`
6. Commit with descriptive message
7. Push and create a pull request

### Debugging

```bash
# Run with debug output
RUST_LOG=debug cargo run

# Run with trace output (very verbose)
RUST_LOG=trace cargo run
```

## 📋 Release Process

Releases are automated via GitHub Actions. To create a release:

1. Update version in `Cargo.toml`
2. Create and push a tag:
   ```bash
   git tag v0.x.x
   git push origin v0.x.x
   ```
3. GitHub Actions will:
   - Build for Linux, macOS, and Windows
   - Embed OAuth credentials from GitHub Secrets
   - Create a GitHub Release with binaries

### GitHub Secrets Required

Add these secrets in repository Settings → Secrets → Actions:

- `OAUTH_GITHUB_CLIENT_ID`
- `OAUTH_GITHUB_CLIENT_SECRET`
- `OAUTH_GITLAB_APP_ID`
- `OAUTH_GITLAB_CLIENT_SECRET`

## 🐛 Troubleshooting

### Browser doesn't open during OAuth

The application will print the OAuth URL. Copy and paste it into your browser manually. The OAuth URL is automatically copied to your clipboard.

### SSH keys not working

- Check your GitHub/GitLab SSH keys settings
- Verify the key matches: `cat ~/.config/git-account-manager/keys/username_at_github/id_ed25519.pub`

### Permission denied errors

The application needs write access to:
- `~/.config/git-account-manager/` - Profile storage
- `~/.ssh/config` - SSH configuration
- `~/.gitconfig` - Git configuration

### Token expired errors

Use the "Sync account" feature to refresh tokens and rotate SSH keys.

### Build errors

```bash
# Clean build artifacts
cargo clean

# Update dependencies
cargo update

# Rebuild
cargo build
```

## 🤝 Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

Make sure all tests pass and code is formatted before submitting.

## 📧 Support

For issues, questions, or suggestions:

- Open an [issue](https://github.com/satas20/git-account-manager/issues)
- Check existing [discussions](https://github.com/satas20/git-account-manager/discussions)

---

**Happy Coding! 🚀**
