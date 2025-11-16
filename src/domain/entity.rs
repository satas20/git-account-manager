#[derive(Debug, Clone)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub email: String,
    pub auth_host: String,
}

impl Profile {
    pub fn new(name: &str, email: &str) -> Self {
        Self {
            id: name.to_string(),
            name: name.to_string(),
            email: email.to_string(),
            auth_host: "github.com".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SshKey {
    pub public_key: String,
}

impl SshKey {
    pub fn from_public_key(s: &str) -> Self {
        Self { public_key: s.to_string() }
    }
}
