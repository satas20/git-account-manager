use crate::domain::ports::AuthProviderPort;

pub struct GithubAdapter {
    // In the future store tokens/config here
}

impl GithubAdapter {
    pub fn new() -> Self {
        Self {}
    }
}

impl AuthProviderPort for GithubAdapter {
    fn upload_ssh_key(&self, _account: &str, _public_key: &str) -> Result<(), String> {
        // Placeholder - a real implementation would call the GitHub API.
        Ok(())
    }
}
