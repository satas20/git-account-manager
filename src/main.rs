mod domain;
mod adapters;

use crate::domain::entity::Profile;

fn main() {
    // Minimal demo: construct a Profile from the domain layer and print it.
    let p = Profile::new("work", "work@example.com");
    println!("Created profile: {} <{}> (host={})", p.name, p.email, p.auth_host);

    // Instantiate adapter stubs (no-op for now).
    let _sys = adapters::system_io::LocalSystemIO::new();
    let _gh = adapters::github::GithubAdapter::new();
    let _tui: adapters::tui::TuiAdapter = adapters::tui::TuiAdapter;

    // In future: wire these together to run actual use cases.
}