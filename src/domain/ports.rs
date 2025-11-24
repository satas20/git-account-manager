// Domain ports: trait interfaces for adapters to implement.
// The Auth provider needs async operations (network I/O), so use async_trait.
use crate::domain::entity::Profile;
use crate::domain::error::DomainError;
use async_trait::async_trait;
use std::path::PathBuf;

/// Result of executing a command.
#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

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

/// System I/O port - abstracts all interactions with the operating system.
/// This includes file operations, environment variables, and command execution.
pub trait SystemIOPort: Send + Sync {
    /// Write the string content to the target path atomically.
    /// Creates parent directories if they don't exist.
    fn write_file(&self, path: &str, content: &str) -> Result<(), DomainError>;

    /// Read the file at path and return its contents as a String.
    fn read_file(&self, path: &str) -> Result<String, DomainError>;

    /// Check if a file or directory exists at the given path.
    fn path_exists(&self, path: &str) -> bool;

    /// Create a directory and all missing parent directories.
    fn create_dir_all(&self, path: &str) -> Result<(), DomainError>;

    /// Delete a file at the given path.
    fn remove_file(&self, path: &str) -> Result<(), DomainError>;

    /// Delete a directory at the given path (only if empty).
    fn remove_dir(&self, path: &str) -> Result<(), DomainError>;

    /// Get the value of an environment variable.
    fn get_env_var(&self, key: &str) -> Option<String>;

    /// Get the user's home directory path (cross-platform).
    fn get_home_dir(&self) -> Result<PathBuf, DomainError>;

    /// Get a unique device identifier (username@hostname).
    fn get_device_identifier(&self) -> String;

    /// Execute a system command and return the output.
    /// Returns stdout, stderr, and exit code.
    fn execute_command(&self, program: &str, args: &[&str]) -> Result<CommandOutput, DomainError>;

    /// Set file permissions (Unix only, no-op on other platforms).
    /// Mode should be in octal format (e.g., 0o600).
    fn set_file_permissions(&self, path: &str, mode: u32) -> Result<(), DomainError>;
}
