// Domain ports: trait interfaces for adapters to implement.
// The Auth provider needs async operations (network I/O), so use async_trait.
use crate::domain::entity::Profile;
use async_trait::async_trait;

/// Auth provider port - async methods for network-bound operations.
#[async_trait]
pub trait AuthProviderPort {
    /// Upload a public SSH key for a given account identifier.
    async fn upload_ssh_key(&self, account: &str, public_key: &str) -> Result<(), String>;

    /// Start an OAuth flow and return an access token.
    async fn start_oauth_flow(&self, account: &str) -> Result<String, String>;

    /// Fetch profile information (username/email/etc) for the authenticated user.
    async fn fetch_profile(&self, token: &str) -> Result<Profile, String>;
}

pub trait SystemIOPort {
    /// Write the string content to the target path atomically.
    fn write_file(&self, path: &str, content: &str) -> Result<(), String>;

    /// Read the file at path and return its contents as a String.
    fn read_file(&self, path: &str) -> Result<String, String>;
}
