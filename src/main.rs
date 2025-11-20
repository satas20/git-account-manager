// Small starter: clap CLI + optional ratatui demo to exercise dependencies.
mod domain;
mod adapters;

use clap::{Parser, Subcommand};
// load local .env for development (no-op if .env not present)
use dotenvy::dotenv;
use crate::domain::entity::Profile;

#[derive(Parser, Debug)]
#[command(name = "accmngr", about = "AccMngr - Git Account Manager (skeleton)")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run the minimal TUI demo
    Tui,
    /// Print a demo profile
    Profile { name: String, email: String },
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
            // run the TUI adapter
            let tui = adapters::tui::TuiAdapter::new();
            if let Err(e) = tui.run() {
                eprintln!("TUI error: {}", e);
            }
        }
        Some(Commands::Profile { name, email }) => {
            let p = Profile::new(name, email);
            println!("Profile: {} <{}> (host={})", p.name, p.email, p.auth_host);
        }
        None => {
            // default behaviour: show a sample profile
            let p = Profile::new("work", "work@example.com");
            println!("Created profile: {} <{}> (host={})", p.name, p.email, p.auth_host);
        }
    }

    Ok(())
}

