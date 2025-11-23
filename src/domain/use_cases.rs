use crate::domain::entity::Profile;
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
/// and delegates all IO read/write to the provided `SystemIOPort` implementation.
pub struct ProfilesManager<'a> {
    storage: &'a dyn SystemIOPort,
    path: PathBuf,
}

impl<'a> ProfilesManager<'a> {
    /// Create a manager that will persist to `path`. If `path` is None, a default
    /// XDG or HOME-based path is used: `$XDG_CONFIG_HOME/git-account-manager/profiles.json`
    pub fn new(storage: &'a dyn SystemIOPort, path: Option<PathBuf>) -> Result<Self, String> {
        let path = match path {
            Some(p) => p,
            None => Self::default_profiles_path()?,
        };

        Ok(Self { storage, path })
    }

    /// Get a device identifier based on the system username and hostname.
    /// Returns in format "username@hostname" (e.g., "ata@ata-ThinkPad-E15-Ge")
    pub fn get_device_identifier() -> String {
        let username = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "unknown".to_string());
        
        let hostname = hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .unwrap_or_else(|| "unknown".to_string());
        
        format!("{}@{}", username, hostname)
    }

    fn default_profiles_path() -> Result<PathBuf, String> {
        if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            let mut p = PathBuf::from(xdg);
            p.push("git-account-manager");
            p.push("profiles.json");
            return Ok(p);
        }
        let home = std::env::var("HOME").map_err(|_| "HOME not set".to_string())?;
        let mut p = PathBuf::from(home);
        p.push(".config");
        p.push("git-account-manager");
        p.push("profiles.json");
        Ok(p)
    }

    fn load(&self) -> Result<ProfilesFile, String> {
        let path_str = self.path.to_string_lossy().to_string();
        match self.storage.read_file(&path_str) {
            Ok(s) => {
                let pf = serde_json::from_str::<ProfilesFile>(&s)
                    .map_err(|e| format!("Invalid profiles JSON: {}", e))?;
                Ok(pf)
            }
            Err(_) => Ok(ProfilesFile::default()),
        }
    }

    fn save(&self, pf: &ProfilesFile) -> Result<(), String> {
        // Ensure directory exists. The storage adapter's write_file will create
        // parent directories, but create_dir_all here is harmless and keeps
        // behaviour robust even with other adapters.
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("Failed to create config dir: {}", e))?;
        }

        let json = serde_json::to_string_pretty(pf).map_err(|e| format!("Serialization error: {}", e))?;
        let path_str = self.path.to_string_lossy().to_string();
        self.storage.write_file(&path_str, &json)
    }

    /// List all profile keys (e.g. "alice@github")
    pub fn list_keys(&self) -> Result<Vec<String>, String> {
        let pf = self.load()?;
        Ok(pf.profiles.keys().cloned().collect())
    }

    /// Get a profile by key.
    pub fn get_profile(&self, key: &str) -> Result<Option<ProfileRecord>, String> {
        let pf = self.load()?;
        Ok(pf.profiles.get(key).cloned())
    }

    /// Add or update a profile. The key should follow the pattern "name@adapter".
    pub fn upsert_profile(&self, key: &str, rec: ProfileRecord) -> Result<(), String> {
        let mut pf = self.load()?;
        pf.profiles.insert(key.to_string(), rec);
        self.save(&pf)
    }

    /// Remove a profile by key. Returns true if removed.
    /// Also deletes the associated SSH key files if they exist.
    pub fn remove_profile(&self, key: &str) -> Result<bool, String> {
        let mut pf = self.load()?;
        
        // If profile exists, check for ssh keys and delete them
        if let Some(rec) = pf.profiles.get(key) {
            if let Some(priv_path) = &rec.ssh_key {
                let p = PathBuf::from(priv_path);
                if p.exists() {
                    let _ = std::fs::remove_file(&p);
                }
                let pub_p = p.with_extension("pub");
                if pub_p.exists() {
                    let _ = std::fs::remove_file(&pub_p);
                }
                // Also try to remove the parent directory if it's named after the key (sanitized)
                if let Some(parent) = p.parent() {
                    // simple check: if dir is empty, remove it
                    let _ = std::fs::remove_dir(parent);
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
    pub fn create_from_entity(&self, profile: &Profile, adapter: Option<String>, ssh_key: Option<String>) -> Result<String, String> {
        let key = format!("{}@{}", profile.name, adapter.clone().unwrap_or_else(|| "local".to_string()));
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

    /// Obtain or create a master key stored in the OS keyring. The master key
    /// is used to symmetrically encrypt/decrypt profile tokens.
    fn get_or_create_master_key() -> Result<[u8;32], String> {
    use rand::RngCore;
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine as _;

        // store master key in config dir as a file with restricted perms
        let key_path = if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            let mut p = PathBuf::from(xdg);
            p.push("git-account-manager");
            p.push("master.key");
            p
        } else {
            let home = std::env::var("HOME").map_err(|_| "HOME not set".to_string())?;
            let mut p = PathBuf::from(home);
            p.push(".config");
            p.push("git-account-manager");
            p.push("master.key");
            p
        };

        if let Ok(s) = std::fs::read_to_string(&key_path) {
            let decoded = STANDARD.decode(s.trim()).map_err(|e| format!("Invalid master key in file: {}", e))?;
            if decoded.len() != 32 { return Err("Master key has invalid length".to_string()); }
            let mut k = [0u8;32];
            k.copy_from_slice(&decoded);
            return Ok(k);
        }

        // generate and store
        if let Some(dir) = key_path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("Failed to create config dir: {}", e))?;
        }
        let mut key = [0u8;32];
        rand::rngs::OsRng.fill_bytes(&mut key);
        let encoded = STANDARD.encode(&key);
        // create file with 0o600 perms when possible
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            let mut opts = std::fs::OpenOptions::new();
            opts.create(true).write(true).truncate(true).mode(0o600);
            let mut f = opts.open(&key_path).map_err(|e| format!("Failed to create master key file: {}", e))?;
            use std::io::Write;
            f.write_all(encoded.as_bytes()).map_err(|e| format!("Failed to write master key: {}", e))?;
        }
        #[cfg(not(unix))]
        {
            std::fs::write(&key_path, encoded.as_bytes()).map_err(|e| format!("Failed to write master key: {}", e))?;
        }

        Ok(key)
    }

    fn encrypt_token(plain: &str) -> Result<String, String> {
    use chacha20poly1305::aead::{Aead, KeyInit};
    use chacha20poly1305::ChaCha20Poly1305;
    use chacha20poly1305::Nonce;
    use rand::RngCore;
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine as _;

    let key = Self::get_or_create_master_key()?;
    let cipher = ChaCha20Poly1305::new(&chacha20poly1305::Key::from_slice(&key));
    let mut nonce = [0u8; 12];
    rand::rngs::OsRng.fill_bytes(&mut nonce);
    let ciphertext = cipher.encrypt(Nonce::from_slice(&nonce), plain.as_bytes()).map_err(|e| format!("Encryption failed: {}", e))?;
    // store as base64(nonce || ciphertext)
    let mut out = Vec::with_capacity(nonce.len() + ciphertext.len());
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&ciphertext);
    Ok(STANDARD.encode(&out))
    }

    fn decrypt_token(enc: &str) -> Result<String, String> {
    use chacha20poly1305::aead::{Aead, KeyInit};
    use chacha20poly1305::ChaCha20Poly1305;
    use chacha20poly1305::Nonce;
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine as _;

    let data = STANDARD.decode(enc).map_err(|e| format!("Invalid base64 token: {}", e))?;
        if data.len() < 12 { return Err("Encrypted token too short".to_string()); }
        let (nonce, ct) = data.split_at(12);
        let key = Self::get_or_create_master_key()?;
        let cipher = ChaCha20Poly1305::new(&chacha20poly1305::Key::from_slice(&key));
        let plain = cipher.decrypt(Nonce::from_slice(nonce), ct).map_err(|e| format!("Decryption failed: {}", e))?;
        String::from_utf8(plain).map_err(|e| format!("Decrypted token not UTF-8: {}", e))
    }

    /// Set (encrypt and persist) auth token for a profile key.
    pub fn set_token_for_profile(&self, key: &str, token: &str) -> Result<(), String> {
        let mut pf = self.load()?;
        let rec = pf.profiles.get_mut(key).ok_or_else(|| format!("Profile not found: {}", key))?;
        let enc = Self::encrypt_token(token)?;
        rec.token_encrypted = Some(enc);
        self.save(&pf)
    }

    /// Set (encrypt and persist) refresh token for a profile key.
    pub fn set_refresh_token_for_profile(&self, key: &str, refresh_token: &str) -> Result<(), String> {
        let mut pf = self.load()?;
        let rec = pf.profiles.get_mut(key).ok_or_else(|| format!("Profile not found: {}", key))?;
        let enc = Self::encrypt_token(refresh_token)?;
        rec.refresh_token_encrypted = Some(enc);
        self.save(&pf)
    }

    /// Get the decrypted refresh token for a profile if present.
    pub fn get_refresh_token(&self, key: &str) -> Result<Option<String>, String> {
        let pf = self.load()?;
        let rec = match pf.profiles.get(key) {
            Some(r) => r,
            None => return Err(format!("Profile not found: {}", key)),
        };
        match &rec.refresh_token_encrypted {
            Some(enc) => Ok(Some(Self::decrypt_token(enc)?)),
            None => Ok(None),
        }
    }

    /// Get the decrypted auth token for a profile if present.
    pub fn get_auth_token(&self, key: &str) -> Result<Option<String>, String> {
        let pf = self.load()?;
        let rec = match pf.profiles.get(key) {
            Some(r) => r,
            None => return Ok(None),
        };
        match &rec.token_encrypted {
            Some(enc) => {
                let token = Self::decrypt_token(enc)?;
                Ok(Some(token))
            }
            None => Ok(None),
        }
    }

    /// Query the local git configuration for a key (e.g. "user.name" or "user.email").
    /// Returns `None` when `git` is not available or the value is empty.
    pub fn get_git_config(&self, key: &str) -> Option<String> {
        use std::process::Command;
        Command::new("git")
            .args(["config", key])
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
                    if s.is_empty() { None } else { Some(s) }
                } else {
                    None
                }
            })
    }

    /// Generate an Ed25519 SSH keypair for the given profile key.
    /// The keys are stored in `$XDG_CONFIG_HOME/git-account-manager/keys/<sanitized_key>/`.
    /// Updates the profile record with the path to the private key.
    pub fn generate_and_assign_ssh_key(&self, key: &str) -> Result<String, String> {
        // compute keys dir: $XDG_CONFIG_HOME/git-account-manager/keys/<sanitized_key>
        let cfg_base = if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            let mut p = PathBuf::from(xdg);
            p.push("git-account-manager");
            p
        } else {
            let home = std::env::var("HOME").map_err(|_| "HOME not set".to_string())?;
            let mut p = PathBuf::from(home);
            p.push(".config");
            p.push("git-account-manager");
            p
        };
        let mut keys_dir = cfg_base;
        // sanitize key for filesystem (replace @ with _at_)
        let safe_key = key.replace("@", "_at_");
        keys_dir.push("keys");
        keys_dir.push(&safe_key);

        if let Err(e) = std::fs::create_dir_all(&keys_dir) {
            return Err(format!("Failed to create keys dir: {}", e));
        }

        let mut priv_path = keys_dir.clone();
        priv_path.push("id_ed25519");
        let priv_path_str = priv_path.to_string_lossy().to_string();

        // Remove existing key if present to avoid ssh-keygen prompt failure
        if priv_path.exists() {
            let _ = std::fs::remove_file(&priv_path);
        }
        let pub_path = priv_path.with_extension("pub");
        if pub_path.exists() {
            let _ = std::fs::remove_file(&pub_path);
        }

        // run ssh-keygen -t ed25519 -f <priv_path> -N "" -q
        let status = std::process::Command::new("ssh-keygen")
            .args(["-t", "ed25519", "-f", &priv_path_str, "-N", "", "-q"])
            .status()
            .map_err(|e| format!("Failed to run ssh-keygen: {}", e))?;

        if !status.success() {
            return Err(format!("ssh-keygen failed with status: {}", status));
        }

        // update profile record with ssh_key path
        let mut pf = self.load()?;
        if let Some(rec) = pf.profiles.get_mut(key) {
            rec.ssh_key = Some(priv_path_str.clone());
            self.save(&pf)?;
            Ok(priv_path_str)
        } else {
            Err(format!("Profile not found: {}", key))
        }
    }

    /// Retrieve the content of the public SSH key for a profile.
    /// Assumes the public key is at `<private_key_path>.pub`.
    pub fn get_public_key_content(&self, key: &str) -> Result<String, String> {
        let pf = self.load()?;
        let rec = pf.profiles.get(key).ok_or_else(|| format!("Profile not found: {}", key))?;
        let priv_path = rec.ssh_key.as_ref().ok_or_else(|| "No SSH key assigned to profile".to_string())?;
        
        let pub_path = format!("{}.pub", priv_path);
        self.storage.read_file(&pub_path)
    }

    /// Switch to the specified profile.
    /// 1. Updates local git config (user.name, user.email).
    /// 2. Updates ~/.ssh/config to use the profile's SSH key for the auth host.
    /// 3. Adds the SSH key to the ssh-agent.
    pub fn switch_profile(&self, key: &str) -> Result<(), String> {
        let pf = self.load()?;
        let rec = pf.profiles.get(key).ok_or_else(|| format!("Profile not found: {}", key))?;
        let ssh_key_path = rec.ssh_key.as_ref().ok_or_else(|| "No SSH key assigned to profile".to_string())?;

        // 1. Update git config
        use std::process::Command;
        let status_name = Command::new("git")
            .args(["config", "--global", "user.name", &rec.name])
            .status()
            .map_err(|e| format!("Failed to set git user.name: {}", e))?;
        if !status_name.success() {
            return Err("git config user.name failed".to_string());
        }

        let status_email = Command::new("git")
            .args(["config", "--global", "user.email", &rec.email])
            .status()
            .map_err(|e| format!("Failed to set git user.email: {}", e))?;
        if !status_email.success() {
            return Err("git config user.email failed".to_string());
        }

        // 2. Update ~/.ssh/config
        let home = std::env::var("HOME").map_err(|_| "HOME not set".to_string())?;
        let mut ssh_config_path = PathBuf::from(&home);
        ssh_config_path.push(".ssh");
        if !ssh_config_path.exists() {
            std::fs::create_dir_all(&ssh_config_path).map_err(|e| format!("Failed to create .ssh dir: {}", e))?;
        }
        ssh_config_path.push("config");
        let ssh_config_str = ssh_config_path.to_string_lossy().to_string();

        let config_content = if self.storage.file_exists(&ssh_config_str) {
            self.storage.read_file(&ssh_config_str)?
        } else {
            String::new()
        };

        // We need to replace or add the Host block for the auth_host (e.g. github.com)
        // Simple parsing: look for "Host <auth_host>" and replace the block until next Host or EOF.
        // Or simpler: use a marker comment managed by this tool?
        // For MVP, let's parse blocks separated by "Host ".
        
        let host_entry = format!("Host {}\n  HostName {}\n  PreferredAuthentications publickey\n  IdentityFile {}\n  IdentitiesOnly yes\n", 
            rec.auth_host, rec.auth_host, ssh_key_path);

        // Remove existing block for this host if present
        // This regex approach is a bit fragile but works for standard configs.
        // We look for `Host github.com` ... (until next Host)
        // Since we don't have regex crate in domain, we do manual line processing.
        
        let mut new_lines = Vec::new();
        let mut skip = false;
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

        // 3. Add key to ssh-agent
        // ssh-add <key_path>
        // Note: ssh-agent must be running.
        // Redirect stdout/stderr to null to avoid interfering with TUI
        let status_add = Command::new("ssh-add")
            .arg(ssh_key_path)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
            
        match status_add {
            Ok(s) if s.success() => Ok(()),
            Ok(s) => Err(format!("ssh-add failed with status: {}", s)),
            Err(e) => Err(format!("Failed to run ssh-add: {}", e)),
        }
    }
}
