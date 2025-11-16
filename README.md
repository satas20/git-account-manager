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
- Implement concrete use cases and add unit tests for domain logic.
- Wire the TUI with `ratatui` and command-line argument parsing with `clap`.
