use crate::domain::error::DomainError;
use crate::domain::ports::{CommandOutput, SystemIOPort};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct LocalSystemIO;

impl LocalSystemIO {
    pub fn new() -> Self {
        Self {}
    }
}

impl SystemIOPort for LocalSystemIO {
    fn write_file(&self, path: &str, content: &str) -> Result<(), DomainError> {
        // Ensure parent directory exists to make writes robust for callers.
        if let Some(parent) = Path::new(path).parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| DomainError::FileWriteError {
                    path: path.to_string(),
                    message: format!("Failed to create parent directory: {}", e),
                })?;
            }
        }

        fs::write(path, content).map_err(|e| DomainError::FileWriteError {
            path: path.to_string(),
            message: e.to_string(),
        })
    }

    fn read_file(&self, path: &str) -> Result<String, DomainError> {
        fs::read_to_string(path).map_err(|e| DomainError::FileReadError {
            path: path.to_string(),
            message: e.to_string(),
        })
    }

    fn path_exists(&self, path: &str) -> bool {
        Path::new(path).exists()
    }

    fn create_dir_all(&self, path: &str) -> Result<(), DomainError> {
        fs::create_dir_all(path).map_err(|e| DomainError::ConfigDirError(format!(
            "Failed to create directory {}: {}",
            path, e
        )))
    }

    fn remove_file(&self, path: &str) -> Result<(), DomainError> {
        fs::remove_file(path).map_err(|e| DomainError::FileWriteError {
            path: path.to_string(),
            message: format!("Failed to remove file: {}", e),
        })
    }

    fn remove_dir(&self, path: &str) -> Result<(), DomainError> {
        fs::remove_dir(path).map_err(|e| DomainError::ConfigDirError(format!(
            "Failed to remove directory {}: {}",
            path, e
        )))
    }

    fn get_env_var(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }

    fn get_home_dir(&self) -> Result<PathBuf, DomainError> {
        if cfg!(windows) {
            std::env::var("USERPROFILE")
                .map(PathBuf::from)
                .map_err(|_| DomainError::EnvVarNotSet("USERPROFILE".to_string()))
        } else {
            std::env::var("HOME")
                .map(PathBuf::from)
                .map_err(|_| DomainError::EnvVarNotSet("HOME".to_string()))
        }
    }

    fn get_device_identifier(&self) -> String {
        let username = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "unknown".to_string());

        let hostname = hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .unwrap_or_else(|| "unknown".to_string());

        format!("{}@{}", username, hostname)
    }

    fn execute_command(&self, program: &str, args: &[&str]) -> Result<CommandOutput, DomainError> {
        let output = Command::new(program).args(args).output().map_err(|e| {
            DomainError::CommandExecutionFailed {
                command: format!("{} {}", program, args.join(" ")),
                message: e.to_string(),
            }
        })?;

        Ok(CommandOutput {
            stdout: String::from_utf8_lossy(&output.stdout).trim().to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
            exit_code: output.status.code().unwrap_or(-1),
        })
    }

    fn set_file_permissions(&self, path: &str, mode: u32) -> Result<(), DomainError> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = fs::Permissions::from_mode(mode);
            fs::set_permissions(path, permissions).map_err(|e| DomainError::FileWriteError {
                path: path.to_string(),
                message: format!("Failed to set permissions: {}", e),
            })
        }

        #[cfg(not(unix))]
        {
            // No-op on non-Unix platforms
            let _ = (path, mode);
            Ok(())
        }
    }
}
