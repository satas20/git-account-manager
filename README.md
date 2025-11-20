# AccMngr — Git Account Manager (skeleton)

This repository contains a minimal skeleton of the AccMngr CLI project following a hexagonal architecture (domain + adapters).

Structure created:

- `src/domain` — core entities, ports (traits), and use cases.
- `src/adapters` — system I/O, Git provider, and TUI adapter stubs.

What I created in this step:

- Domain modules: `entity.rs`, `ports.rs`, `use_cases.rs`, and `mod.rs`.
- Adapter modules: `system_io.rs`, `github.rs`, `tui.rs`, and `mod.rs`.
- Updated `src/main.rs` to reference the new modules (see the code changes).

Next steps you might want:

- Add `thiserror`, `anyhow`, `tokio`, and `reqwest` to `Cargo.toml` if you plan to implement async adapters and richer error handling.
- Wire the TUI with `ratatui` and command-line argument parsing with `clap`.

## Local OAuth environment

To run the GitHub OAuth demo in the TUI you must provide GitHub OAuth credentials. You can copy the example env file and provide the secret:

1. Copy the example file and edit the secret:

```sh
cp .env.example .env
# then open .env and set GITHUB_CLIENT_SECRET to the value from GitHub
```

2. Load the environment (Zsh/Bash):

```sh
source .env
```

3. Run the TUI and start the Add→GitHub flow:

```sh
cargo run -- Tui
```

Notes:

- The app expects the callback redirect to be `http://127.0.0.1:8787/callback` by default. If you change the port or host in `.env`, update the OAuth App settings on GitHub accordingly.
- Keep `GITHUB_CLIENT_SECRET` private. Do not commit `.env` to version control.
