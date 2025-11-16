// Domain ports: trait interfaces for adapters to implement.
// Keep them minimal for now so the project compiles. These can be expanded
// to async traits (with async_trait) and concrete error types later.

pub trait AuthProviderPort {
    /// Upload a public SSH key for a given account identifier.
    fn upload_ssh_key(&self, account: &str, public_key: &str) -> Result<(), String>;
}

pub trait SystemIOPort {
    /// Write the string content to the target path atomically.
    fn write_file(&self, path: &str, content: &str) -> Result<(), String>;

    /// Read the file at path and return its contents as a String.
    fn read_file(&self, path: &str) -> Result<String, String>;
}
