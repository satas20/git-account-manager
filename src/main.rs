// Git Account Manager - Terminal-based tool for managing multiple Git identities
mod domain;
mod adapters;

use clap::{Parser, Subcommand};
// load local .env for development (no-op if .env not present)
use dotenvy::dotenv;
use crate::domain::entity::Profile;

#[derive(Parser, Debug)]
#[command(
    name = "git-acc-mngr",
    about = "Git Account Manager - Manage multiple Git identities with ease",
    long_about = "A powerful terminal-based tool for managing multiple Git identities with OAuth authentication and automatic SSH key management.",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Launch the interactive TUI (Terminal User Interface)
    Tui,
    /// Create a demo profile (for testing)
    Profile {
        /// Profile name
        name: String,
        /// Profile email address
        email: String
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // If a `.env` file exists in the project root, load it into the process env.
    // This makes the development flow easier so you can run `source .env` or
    // just rely on `.env` being present.
    dotenv().ok();

    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Tui) => {
            // Explicit TUI command
            let tui = adapters::tui::TuiAdapter::new();
            if let Err(e) = tui.run() {
                eprintln!("TUI error: {}", e);
            }
        }
        Some(Commands::Profile { name, email }) => {
            // Demo profile command
            let p = Profile::new(name, email);
            println!("Profile: {} <{}> (host={})", p.name, p.email, p.auth_host);
        }
        None => {
            // Default behavior: Launch TUI directly
            let tui = adapters::tui::TuiAdapter::new();
            if let Err(e) = tui.run() {
                eprintln!("TUI error: {}", e);
            }
        }
    }

    Ok(())
}

