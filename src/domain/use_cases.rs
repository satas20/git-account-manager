use crate::domain::entity::Profile;
use crate::domain::error::{DomainError, Result};
use crate::domain::ports::SystemIOPort;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// ProfileRecord is the persisted representation stored in the single JSON file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileRecord {
    pub name: String,
    pub email: String,
    pub auth_host: String,
    /// adapter used to create this profile, e.g. "github"
    pub adapter: Option<String>,
    /// relative or absolute path to the ssh key (private key file) for this profile
    pub ssh_key: Option<String>,
    /// GitHub ID of the uploaded SSH key (used for deletion)
    pub ssh_key_id: Option<u64>,
    /// encrypted access token (base64 of nonce+ciphertext)
    pub token_encrypted: Option<String>,
    /// encrypted refresh token (base64 of nonce+ciphertext)
    pub refresh_token_encrypted: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ProfilesFile {
    pub profiles: BTreeMap<String, ProfileRecord>,
}

/// Domain-level manager for profiles. It performs CRUD on a single JSON file
/// and delegates all IO operations to the provided `SystemIOPort` implementation.
/// This module now contains PURE business logic with NO direct dependencies on
/// std::fs, std::env, or std::process::Command - all such operations go through ports.
pub struct ProfilesManager<'a> {
    storage: &'a dyn SystemIOPort,
    path: PathBuf,
}

impl<'a> ProfilesManager<'a> {
    /// Create a manager that will persist to `path`. If `path` is None, a default
    /// XDG or HOME-based path is used: `$XDG_CONFIG_HOME/git-account-manager/profiles.json`
    pub fn new(storage: &'a dyn SystemIOPort, path: Option<PathBuf>) -> Result<Self> {
        let path = match path {
            Some(p) => p,
            None => Self::default_profiles_path(storage)?,
        };

        Ok(Self { storage, path })
    }

    /// Get a device identifier based on the system username and hostname.
    /// Returns in format "username@hostname" (e.g., "ata@ata-ThinkPad-E15-Ge")
    pub fn get_device_identifier(&self) -> String {
        self.storage.get_device_identifier()
    }

    fn default_profiles_path(storage: &dyn SystemIOPort) -> Result<PathBuf> {
        if let Some(xdg) = storage.get_env_var("XDG_CONFIG_HOME") {
            let mut p = PathBuf::from(xdg);
            p.push("git-account-manager");
            p.push("profiles.json");
            return Ok(p);
        }
        let home = storage.get_home_dir()?;
        let mut p = home;
        p.push(".config");
        p.push("git-account-manager");
        p.push("profiles.json");
        Ok(p)
    }

    fn load(&self) -> Result<ProfilesFile> {
        let path_str = self.path.to_string_lossy().to_string();
        match self.storage.read_file(&path_str) {
            Ok(s) => {
                let pf = serde_json::from_str::<ProfilesFile>(&s)
                    .map_err(|e| DomainError::InvalidProfilesJson(e.to_string()))?;
                Ok(pf)
            }
            Err(_) => Ok(ProfilesFile::default()),
        }
    }

    fn save(&self, pf: &ProfilesFile) -> Result<()> {
        // Ensure directory exists
        if let Some(dir) = self.path.parent() {
            let dir_str = dir.to_string_lossy().to_string();
            self.storage.create_dir_all(&dir_str)?;
        }

        let json = serde_json::to_string_pretty(pf)
            .map_err(|e| DomainError::SerializationError(e.to_string()))?;
        let path_str = self.path.to_string_lossy().to_string();
        self.storage.write_file(&path_str, &json)
    }

    /// List all profile keys (e.g. "alice@github")
    pub fn list_keys(&self) -> Result<Vec<String>> {
        let pf = self.load()?;
        Ok(pf.profiles.keys().cloned().collect())
    }

    /// Get a profile by key.
    pub fn get_profile(&self, key: &str) -> Result<Option<ProfileRecord>> {
        let pf = self.load()?;
        Ok(pf.profiles.get(key).cloned())
    }

    /// Add or update a profile. The key should follow the pattern "name@adapter".
    pub fn upsert_profile(&self, key: &str, rec: ProfileRecord) -> Result<()> {
        let mut pf = self.load()?;
        pf.profiles.insert(key.to_string(), rec);
        self.save(&pf)
    }

    /// Remove a profile by key. Returns true if removed.
    /// Also deletes the associated SSH key files if they exist.
    pub fn remove_profile(&self, key: &str) -> Result<bool> {
        let mut pf = self.load()?;

        // If profile exists, check for ssh keys and delete them
        if let Some(rec) = pf.profiles.get(key) {
            if let Some(priv_path) = &rec.ssh_key {
                let p = PathBuf::from(priv_path);
                let priv_path_str = p.to_string_lossy().to_string();

                if self.storage.path_exists(&priv_path_str) {
                    let _ = self.storage.remove_file(&priv_path_str);
                }

                let pub_p = p.with_extension("pub");
                let pub_path_str = pub_p.to_string_lossy().to_string();
                if self.storage.path_exists(&pub_path_str) {
                    let _ = self.storage.remove_file(&pub_path_str);
                }

                // Try to remove the parent directory if it's empty
                if let Some(parent) = p.parent() {
                    let parent_str = parent.to_string_lossy().to_string();
                    let _ = self.storage.remove_dir(&parent_str);
                }
            }
        }

        let removed = pf.profiles.remove(key).is_some();
        if removed {
            self.save(&pf)?;
        }
        Ok(removed)
    }

    /// Convenience: create a profile from the domain `Profile` entity.
    /// Uses adapter name (e.g. "github") as part of the key.
    pub fn create_from_entity(
        &self,
        profile: &Profile,
        adapter: Option<String>,
        ssh_key: Option<String>,
    ) -> Result<String> {
        let key = format!(
            "{}@{}",
            profile.name,
            adapter.clone().unwrap_or_else(|| "local".to_string())
        );
        let rec = ProfileRecord {
            name: profile.name.clone(),
            email: profile.email.clone(),
            auth_host: profile.auth_host.clone(),
            adapter,
            ssh_key,
            ssh_key_id: None,
            token_encrypted: None,
            refresh_token_encrypted: None,
        };
        self.upsert_profile(&key, rec)?;
        Ok(key)
    }

    /// Obtain or create a master key stored in the config directory.
    /// The master key is used to symmetrically encrypt/decrypt profile tokens.
    fn get_or_create_master_key(&self) -> Result<[u8; 32]> {
        use base64::engine::general_purpose::STANDARD;
        use base64::Engine as _;
        use rand::RngCore;

        // Store master key in config dir as a file with restricted perms
        let key_path = if let Some(xdg) = self.storage.get_env_var("XDG_CONFIG_HOME") {
            let mut p = PathBuf::from(xdg);
            p.push("git-account-manager");
            p.push("master.key");
            p
        } else {
            let home = self.storage.get_home_dir()?;
            let mut p = home;
            p.push(".config");
            p.push("git-account-manager");
            p.push("master.key");
            p
        };

        let key_path_str = key_path.to_string_lossy().to_string();

        if let Ok(s) = self.storage.read_file(&key_path_str) {
            let decoded = STANDARD
                .decode(s.trim())
                .map_err(|e| DomainError::MasterKeyError(format!("Invalid master key: {}", e)))?;
            if decoded.len() != 32 {
                return Err(DomainError::MasterKeyError(
                    "Master key has invalid length".to_string(),
                ));
            }
            let mut k = [0u8; 32];
            k.copy_from_slice(&decoded);
            return Ok(k);
        }

        // Generate and store new key
        if let Some(dir) = key_path.parent() {
            let dir_str = dir.to_string_lossy().to_string();
            self.storage.create_dir_all(&dir_str)?;
        }

        let mut key = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut key);
        let encoded = STANDARD.encode(&key);

        self.storage.write_file(&key_path_str, &encoded)?;

        // Set restrictive permissions on Unix
        let _ = self.storage.set_file_permissions(&key_path_str, 0o600);

        Ok(key)
    }

    fn encrypt_token(&self, plain: &str) -> Result<String> {
        use base64::engine::general_purpose::STANDARD;
        use base64::Engine as _;
        use chacha20poly1305::aead::{Aead, KeyInit};
        use chacha20poly1305::{ChaCha20Poly1305, Nonce};
        use rand::RngCore;

        let key = self.get_or_create_master_key()?;
        let cipher = ChaCha20Poly1305::new(&chacha20poly1305::Key::from_slice(&key));
        let mut nonce = [0u8; 12];
        rand::rngs::OsRng.fill_bytes(&mut nonce);
        let ciphertext = cipher
            .encrypt(Nonce::from_slice(&nonce), plain.as_bytes())
            .map_err(|e| DomainError::EncryptionError(e.to_string()))?;

        // Store as base64(nonce || ciphertext)
        let mut out = Vec::with_capacity(nonce.len() + ciphertext.len());
        out.extend_from_slice(&nonce);
        out.extend_from_slice(&ciphertext);
        Ok(STANDARD.encode(&out))
    }

    fn decrypt_token(&self, enc: &str) -> Result<String> {
        use base64::engine::general_purpose::STANDARD;
        use base64::Engine as _;
        use chacha20poly1305::aead::{Aead, KeyInit};
        use chacha20poly1305::{ChaCha20Poly1305, Nonce};

        let data = STANDARD
            .decode(enc)
            .map_err(|e| DomainError::InvalidEncryptedData(format!("Invalid base64: {}", e)))?;

        if data.len() < 12 {
            return Err(DomainError::InvalidEncryptedData(
                "Encrypted token too short".to_string(),
            ));
        }

        let (nonce, ct) = data.split_at(12);
        let key = self.get_or_create_master_key()?;
        let cipher = ChaCha20Poly1305::new(&chacha20poly1305::Key::from_slice(&key));
        let plain = cipher
            .decrypt(Nonce::from_slice(nonce), ct)
            .map_err(|e| DomainError::DecryptionError(e.to_string()))?;

        String::from_utf8(plain)
            .map_err(|e| DomainError::DecryptionError(format!("Not UTF-8: {}", e)))
    }

    /// Set (encrypt and persist) auth token for a profile key.
    pub fn set_token_for_profile(&self, key: &str, token: &str) -> Result<()> {
        let mut pf = self.load()?;
        let rec = pf
            .profiles
            .get_mut(key)
            .ok_or_else(|| DomainError::ProfileNotFound(key.to_string()))?;
        let enc = self.encrypt_token(token)?;
        rec.token_encrypted = Some(enc);
        self.save(&pf)
    }

    /// Set (encrypt and persist) refresh token for a profile key.
    pub fn set_refresh_token_for_profile(&self, key: &str, refresh_token: &str) -> Result<()> {
        let mut pf = self.load()?;
        let rec = pf
            .profiles
            .get_mut(key)
            .ok_or_else(|| DomainError::ProfileNotFound(key.to_string()))?;
        let enc = self.encrypt_token(refresh_token)?;
        rec.refresh_token_encrypted = Some(enc);
        self.save(&pf)
    }

    /// Get the decrypted refresh token for a profile if present.
    pub fn get_refresh_token(&self, key: &str) -> Result<Option<String>> {
        let pf = self.load()?;
        let rec = pf
            .profiles
            .get(key)
            .ok_or_else(|| DomainError::ProfileNotFound(key.to_string()))?;

        match &rec.refresh_token_encrypted {
            Some(enc) => Ok(Some(self.decrypt_token(enc)?)),
            None => Ok(None),
        }
    }

    /// Get the decrypted auth token for a profile if present.
    pub fn get_auth_token(&self, key: &str) -> Result<Option<String>> {
        let pf = self.load()?;
        let rec = match pf.profiles.get(key) {
            Some(r) => r,
            None => return Ok(None),
        };

        match &rec.token_encrypted {
            Some(enc) => {
                let token = self.decrypt_token(enc)?;
                Ok(Some(token))
            }
            None => Ok(None),
        }
    }

    /// Query the local git configuration for a key (e.g. "user.name" or "user.email").
    /// Returns `None` when `git` is not available or the value is empty.
    pub fn get_git_config(&self, key: &str) -> Option<String> {
        let output = self
            .storage
            .execute_command("git", &["config", key])
            .ok()?;

        if output.exit_code == 0 && !output.stdout.is_empty() {
            Some(output.stdout)
        } else {
            None
        }
    }

    /// Generate an Ed25519 SSH keypair for the given profile key.
    /// The keys are stored in `$XDG_CONFIG_HOME/git-account-manager/keys/<sanitized_key>/`.
    /// Updates the profile record with the path to the private key.
    pub fn generate_and_assign_ssh_key(&self, key: &str) -> Result<String> {
        // Compute keys dir: $XDG_CONFIG_HOME/git-account-manager/keys/<sanitized_key>
        let cfg_base = if let Some(xdg) = self.storage.get_env_var("XDG_CONFIG_HOME") {
            let mut p = PathBuf::from(xdg);
            p.push("git-account-manager");
            p
        } else {
            let home = self.storage.get_home_dir()?;
            let mut p = home;
            p.push(".config");
            p.push("git-account-manager");
            p
        };

        let mut keys_dir = cfg_base;
        // Sanitize key for filesystem (replace @ with _at_)
        let safe_key = key.replace("@", "_at_");
        keys_dir.push("keys");
        keys_dir.push(&safe_key);

        let keys_dir_str = keys_dir.to_string_lossy().to_string();
        self.storage.create_dir_all(&keys_dir_str)?;

        let mut priv_path = keys_dir.clone();
        priv_path.push("id_ed25519");
        let priv_path_str = priv_path.to_string_lossy().to_string();

        // Remove existing key if present to avoid ssh-keygen prompt failure
        if self.storage.path_exists(&priv_path_str) {
            let _ = self.storage.remove_file(&priv_path_str);
        }
        let pub_path = priv_path.with_extension("pub");
        let pub_path_str = pub_path.to_string_lossy().to_string();
        if self.storage.path_exists(&pub_path_str) {
            let _ = self.storage.remove_file(&pub_path_str);
        }

        // Run ssh-keygen -t ed25519 -f <priv_path> -N "" -q
        let output = self.storage.execute_command(
            "ssh-keygen",
            &["-t", "ed25519", "-f", &priv_path_str, "-N", "", "-q"],
        )?;

        if output.exit_code != 0 {
            return Err(DomainError::SshError(format!(
                "ssh-keygen failed with exit code {}: {}",
                output.exit_code, output.stderr
            )));
        }

        // Update profile record with ssh_key path
        let mut pf = self.load()?;
        if let Some(rec) = pf.profiles.get_mut(key) {
            rec.ssh_key = Some(priv_path_str.clone());
            self.save(&pf)?;
            Ok(priv_path_str)
        } else {
            Err(DomainError::ProfileNotFound(key.to_string()))
        }
    }

    /// Retrieve the content of the public SSH key for a profile.
    /// Assumes the public key is at `<private_key_path>.pub`.
    pub fn get_public_key_content(&self, key: &str) -> Result<String> {
        let pf = self.load()?;
        let rec = pf
            .profiles
            .get(key)
            .ok_or_else(|| DomainError::ProfileNotFound(key.to_string()))?;
        let priv_path = rec
            .ssh_key
            .as_ref()
            .ok_or_else(|| DomainError::SshKeyNotAssigned(key.to_string()))?;

        let pub_path = format!("{}.pub", priv_path);
        self.storage.read_file(&pub_path)
    }

    /// Switch to the specified profile.
    /// 1. Updates local git config (user.name, user.email).
    /// 2. Updates ~/.ssh/config to use the profile's SSH key for the auth host.
    /// 3. Adds the SSH key to the ssh-agent (optional, skipped if agent not available).
    pub fn switch_profile(&self, key: &str) -> Result<()> {
        let pf = self.load()?;
        let rec = pf
            .profiles
            .get(key)
            .ok_or_else(|| DomainError::ProfileNotFound(key.to_string()))?;
        let ssh_key_path = rec
            .ssh_key
            .as_ref()
            .ok_or_else(|| DomainError::SshKeyNotAssigned(key.to_string()))?;

        // 1. Update git config
        let output_name = self
            .storage
            .execute_command("git", &["config", "--global", "user.name", &rec.name])?;

        if output_name.exit_code != 0 {
            return Err(DomainError::GitError(format!(
                "Failed to set user.name: {}",
                output_name.stderr
            )));
        }

        let output_email = self
            .storage
            .execute_command("git", &["config", "--global", "user.email", &rec.email])?;

        if output_email.exit_code != 0 {
            return Err(DomainError::GitError(format!(
                "Failed to set user.email: {}",
                output_email.stderr
            )));
        }

        // 2. Update SSH config (cross-platform)
        let home = self.storage.get_home_dir()?;
        let mut ssh_config_path = home;
        ssh_config_path.push(".ssh");

        let ssh_dir_str = ssh_config_path.to_string_lossy().to_string();
        if !self.storage.path_exists(&ssh_dir_str) {
            self.storage.create_dir_all(&ssh_dir_str)?;
        }

        ssh_config_path.push("config");
        let ssh_config_str = ssh_config_path.to_string_lossy().to_string();

        let config_content = match self.storage.read_file(&ssh_config_str) {
            Ok(content) => content,
            Err(_) => String::new(),
        };

        // Convert Windows paths to forward slashes for SSH config
        let ssh_key_config_path = if cfg!(windows) {
            ssh_key_path.replace("\\", "/")
        } else {
            ssh_key_path.clone()
        };

        let host_entry = format!(
            "Host {}\n  HostName {}\n  PreferredAuthentications publickey\n  IdentityFile {}\n  IdentitiesOnly yes\n",
            rec.auth_host, rec.auth_host, ssh_key_config_path
        );

        // Remove existing block for this host if present
        let mut new_lines = Vec::new();
        let mut skip: bool = false;
        for line in config_content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("Host ") {
                if trimmed == format!("Host {}", rec.auth_host) {
                    skip = true;
                } else {
                    skip = false;
                }
            }
            if !skip {
                new_lines.push(line);
            }
        }

        // Append our new block
        new_lines.push(""); // ensure separation
        new_lines.push(&host_entry);

        let new_config = new_lines.join("\n");
        self.storage.write_file(&ssh_config_str, &new_config)?;

        // 3. Try to add key to ssh-agent (optional, may fail if agent not running)
        let _ = self.storage.execute_command("ssh-add", &[ssh_key_path]);
        // Ignore ssh-add failures - SSH config is sufficient for git operations

        Ok(())
    }

    /// Remove a profile and also delete its SSH key from the remote provider (if configured).
    /// This is a high-level use case that coordinates between local storage and remote API.
    pub async fn remove_profile_with_remote_cleanup<A>(
        &self,
        key: &str,
        auth_provider: &A,
    ) -> Result<()>
    where
        A: crate::domain::ports::AuthProviderPort,
    {
        // 1. Try to delete from remote provider if we have key_id
        if let Ok(Some(rec)) = self.get_profile(key) {
            if let Some(key_id) = rec.ssh_key_id {
                // Attempt to delete from remote, but don't fail if it errors
                // (the key might already be deleted, or network might be down)
                let _ = auth_provider.delete_ssh_key(key, key_id).await;
            }
        }

        // 2. Remove locally (files and config)
        self.remove_profile(key)?;
        Ok(())
    }
}

/// High-level use case functions that coordinate between domain logic and adapters.
/// These functions encapsulate common workflows and keep the TUI/CLI layers thin.

/// Add a new GitHub profile: runs OAuth, fetches user info, generates SSH keys, and uploads them.
pub async fn add_github_profile_use_case(
    storage: &dyn crate::domain::ports::SystemIOPort,
    github_adapter: &crate::adapters::github::GithubAdapter<'_>,
) -> Result<String> {
    // 1) Run OAuth and get token
    let (token, refresh_token) = github_adapter.start_oauth_flow_async("default").await
        .map_err(|e| DomainError::GitError(format!("OAuth failed: {}", e)))?;

    // 2) Fetch user profile from GitHub
    let profile = github_adapter.fetch_profile_with_token(&token).await
        .map_err(|e| DomainError::GitError(format!("Failed to fetch profile: {}", e)))?;

    // 3) Create ProfilesManager and persist profile
    let mgr = ProfilesManager::new(storage, None)?;
    let key = mgr.create_from_entity(&profile, Some("github".to_string()), None)?;

    // 4) Store tokens encrypted
    mgr.set_token_for_profile(&key, &token)?;
    if let Some(rt) = refresh_token {
        mgr.set_refresh_token_for_profile(&key, &rt)?;
    }

    // 5) Generate SSH keypair
    mgr.generate_and_assign_ssh_key(&key)?;

    // 6) Upload public key to GitHub
    let device_id = mgr.get_device_identifier();
    let pub_key = mgr.get_public_key_content(&key)?;
    let key_id = github_adapter.upload_ssh_key_with_token(&token, &pub_key, &device_id).await
        .map_err(|e| DomainError::SshError(format!("Failed to upload SSH key: {}", e)))?;

    // 7) Update profile with key_id
    if let Ok(Some(mut rec)) = mgr.get_profile(&key) {
        rec.ssh_key_id = Some(key_id);
        mgr.upsert_profile(&key, rec)?;
    }

    Ok(key)
}

/// List all profiles with their display information (name, email, current status).
pub fn list_profiles_use_case(
    storage: &dyn crate::domain::ports::SystemIOPort,
) -> Result<Vec<(String, String, String, bool)>> {
    let mgr = ProfilesManager::new(storage, None)?;
    let keys = mgr.list_keys()?;

    let git_name = mgr.get_git_config("user.name");
    let git_email = mgr.get_git_config("user.email");

    let mut result = Vec::new();
    for key in keys {
        if let Ok(Some(rec)) = mgr.get_profile(&key) {
            let is_current =
                git_name.as_deref().map(|s| s == rec.name).unwrap_or(false) &&
                git_email.as_deref().map(|s| s == rec.email).unwrap_or(false);
            result.push((key, rec.name, rec.email, is_current));
        }
    }

    Ok(result)
}

/// Switch to a profile (updates git config and SSH config).
pub fn switch_profile_use_case(
    storage: &dyn crate::domain::ports::SystemIOPort,
    key: &str,
) -> Result<()> {
    let mgr = ProfilesManager::new(storage, None)?;
    mgr.switch_profile(key)
}

/// Remove a profile including remote SSH key cleanup.
pub async fn remove_profile_use_case(
    storage: &dyn crate::domain::ports::SystemIOPort,
    key: &str,
) -> Result<()> {
    let mgr = ProfilesManager::new(storage, None)?;

    // Create GithubAdapter with the manager for token refresh
    let github_adapter = crate::adapters::github::GithubAdapter::with_manager(mgr);

    // Get a new manager reference (since we moved mgr into github_adapter)
    let mgr2 = ProfilesManager::new(storage, None)?;

    // Use the high-level method
    mgr2.remove_profile_with_remote_cleanup(key, &github_adapter).await
}
