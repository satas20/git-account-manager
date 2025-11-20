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
    pub fn remove_profile(&self, key: &str) -> Result<bool, String> {
        let mut pf = self.load()?;
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
        };
        self.upsert_profile(&key, rec)?;
        Ok(key)
    }
}
