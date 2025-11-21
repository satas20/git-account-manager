use crate::domain::ports::SystemIOPort;
use std::fs;

pub struct LocalSystemIO;

impl LocalSystemIO {
    pub fn new() -> Self {
        Self {}
    }
}

impl SystemIOPort for LocalSystemIO {
    fn write_file(&self, path: &str, content: &str) -> Result<(), String> {
        // Ensure parent directory exists to make writes robust for callers.
        if let Some(parent) = std::path::Path::new(path).parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
        }

        fs::write(path, content).map_err(|e| e.to_string())
    }

    fn read_file(&self, path: &str) -> Result<String, String> {
        std::fs::read_to_string(path).map_err(|e| e.to_string())
    }

    fn file_exists(&self, path: &str) -> bool {
        std::path::Path::new(path).exists()
    }
}
