use std::fmt;

pub type Result<T> = std::result::Result<T, DomainError>;

#[derive(Debug)]
pub enum DomainError {
    FileReadError { path: String, message: String },
    FileWriteError { path: String, message: String },
    InvalidProfilesJson(String),
    SerializationError(String),
    MasterKeyError(String),
    EncryptionError(String),
    InvalidEncryptedData(String),
    DecryptionError(String),
    ProfileNotFound(String),
    SshError(String),
    SshKeyNotAssigned(String),
    GitError(String),
    ConfigDirError(String),
    EnvVarNotSet(String),
    CommandExecutionFailed { command: String, message: String },
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainError::FileReadError { path, message } => {
                write!(f, "Failed to read file '{}': {}", path, message)
            }
            DomainError::FileWriteError { path, message } => {
                write!(f, "Failed to write file '{}': {}", path, message)
            }
            DomainError::InvalidProfilesJson(msg) => {
                write!(f, "Invalid profiles JSON: {}", msg)
            }
            DomainError::SerializationError(msg) => {
                write!(f, "Serialization error: {}", msg)
            }
            DomainError::MasterKeyError(msg) => {
                write!(f, "Master key error: {}", msg)
            }
            DomainError::EncryptionError(msg) => {
                write!(f, "Encryption error: {}", msg)
            }
            DomainError::InvalidEncryptedData(msg) => {
                write!(f, "Invalid encrypted data: {}", msg)
            }
            DomainError::DecryptionError(msg) => {
                write!(f, "Decryption error: {}", msg)
            }
            DomainError::ProfileNotFound(msg) => {
                write!(f, "Profile not found: {}", msg)
            }
            DomainError::SshError(msg) => {
                write!(f, "SSH error: {}", msg)
            }
            DomainError::SshKeyNotAssigned(msg) => {
                write!(f, "SSH key not assigned: {}", msg)
            }
            DomainError::GitError(msg) => {
                write!(f, "Git error: {}", msg)
            }
            DomainError::ConfigDirError(msg) => {
                write!(f, "Config directory error: {}", msg)
            }
            DomainError::EnvVarNotSet(var) => {
                write!(f, "Environment variable not set: {}", var)
            }
            DomainError::CommandExecutionFailed { command, message } => {
                write!(f, "Command '{}' failed: {}", command, message)
            }
        }
    }
}

impl std::error::Error for DomainError {}
