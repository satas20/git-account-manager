// Domain ports: trait interfaces for adapters to implement.
// The Auth provider needs async operations (network I/O), so use async_trait.
use crate::domain::entity::Profile;
use async_trait::async_trait;

/// Auth provider port - async methods for network-bound operations.
#[async_trait]
pub trait AuthProviderPort {
    /// Upload a public SSH key for a given profile.
    /// Returns the GitHub ID of the created key.
    async fn upload_ssh_key(&self, profile_key: &str, public_key: &str) -> Result<u64, String>;

    /// Delete an SSH key by its ID for a given profile.
    async fn delete_ssh_key(&self, profile_key: &str, key_id: u64) -> Result<(), String>;

    /// Start an OAuth flow and return an access token and optional refresh token.
    async fn start_oauth_flow(&self, account: &str) -> Result<(String, Option<String>), String>;

    /// Fetch profile information (username/email/etc) for a given profile.
    async fn fetch_profile(&self, profile_key: &str) -> Result<Profile, String>;

    /// Refresh an access token using a refresh token.
    async fn refresh_access_token(&self, refresh_token: &str) -> Result<(String, Option<String>), String>;
}

pub trait SystemIOPort: Send + Sync {
    /// Write the string content to the target path atomically.
    fn write_file(&self, path: &str, content: &str) -> Result<(), String>;

    /// Read the file at path and return its contents as a String.
    fn read_file(&self, path: &str) -> Result<String, String>;

    /// Check if a file exists at the given path.
    fn file_exists(&self, path: &str) -> bool;
}
