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
        fs::write(path, content).map_err(|e| e.to_string())
    }

    fn read_file(&self, path: &str) -> Result<String, String> {
        fs::read_to_string(path).map_err(|e| e.to_string())
    }
}
