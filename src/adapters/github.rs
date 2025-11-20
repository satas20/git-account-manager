use crate::domain::ports::AuthProviderPort;
use crate::domain::entity::Profile;
use async_trait::async_trait;
use serde::Deserialize;
use std::env;

pub struct GithubAdapter {
    // In the future store tokens/config here
}

impl GithubAdapter {
    pub fn new() -> Self {
        Self {}
    }

    /// Async helper that starts the OAuth flow for GitHub:
    /// - opens the user's browser to the GitHub authorize URL
    /// - starts a local HTTP listener on 127.0.0.1:8787 to receive the callback
    /// - exchanges the code for an access token
    /// Returns the access token on success.
    pub async fn start_oauth_flow_async(&self, _account: &str) -> Result<String, String> {
        // Read client credentials from env
        let client_id = env::var("GITHUB_CLIENT_ID").map_err(|_| "Missing GITHUB_CLIENT_ID environment variable".to_string())?;
        let client_secret = env::var("GITHUB_CLIENT_SECRET").map_err(|_| "Missing GITHUB_CLIENT_SECRET environment variable".to_string())?;

        // callback listener
        let redirect_host = "127.0.0.1";
        let redirect_port = 8787u16;
        let redirect_path = "/callback";
        let redirect_uri = format!("http://{}:{}{}", redirect_host, redirect_port, redirect_path);

        // state token (static for now; in prod, generate random string)
        let state = "state123";

        let scope = "read:user user:email";
        let auth_url = format!(
            "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&scope={}&state={}",
            urlencoding::encode(&client_id),
            urlencoding::encode(&redirect_uri),
            urlencoding::encode(scope),
            urlencoding::encode(state)
        );

        // Try to open the browser
        match webbrowser::open(&auth_url) {
            Ok(_) => {}
            Err(e) => {
                return Err(format!("Failed to open browser: {}. Open this URL manually: {}", e, auth_url));
            }
        }

        // Spawn a blocking listener to wait for the single callback request and extract `code`
        let listen_addr = format!("{}:{}", redirect_host, redirect_port);
        let code = tokio::task::spawn_blocking(move || -> Result<String, String> {
            use std::io::{BufRead, BufReader, Write};
            use std::net::TcpListener;

            let listener = TcpListener::bind(&listen_addr).map_err(|e| format!("Failed to bind {}: {}", listen_addr, e))?;
            // accept a single connection
            let (mut stream, _peer) = listener.accept().map_err(|e| format!("Failed to accept connection: {}", e))?;

            // read the request
            let mut reader = BufReader::new(&mut stream);
            let mut request_line = String::new();
            reader.read_line(&mut request_line).map_err(|e| format!("Failed to read request: {}", e))?;

            // Example: GET /callback?code=...&state=... HTTP/1.1
            let parts: Vec<&str> = request_line.split_whitespace().collect();
            if parts.len() < 2 {
                return Err("Malformed HTTP request".to_string());
            }
            let path = parts[1];
            let code_opt = path.split('?').nth(1).and_then(|q| {
                q.split('&').find_map(|pair| {
                    let mut kv = pair.splitn(2, '=');
                    let k = kv.next()?;
                    let v = kv.next()?;
                    if k == "code" { Some(v.to_string()) } else { None }
                })
            });

            let code = code_opt.ok_or_else(|| format!("code not found in request path: {}", path))?;

            // respond to browser
            let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\n\r\n<html><body><h2>Authentication complete</h2><p>You can close this window and return to the application.</p></body></html>";
            stream.write_all(response.as_bytes()).ok();

            Ok(code)
        })
        .await
        .map_err(|e| format!("Listener task join error: {}", e))?;

        let code = match code {
            Ok(c) => c,
            Err(e) => return Err(e),
        };

        // Exchange the code for an access token
        #[derive(Deserialize)]
        struct TokenResponse {
            access_token: String,
            scope: Option<String>,
            token_type: Option<String>,
        }

        let client = reqwest::Client::new();
        let params = [
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("code", code.as_str()),
            ("redirect_uri", redirect_uri.as_str()),
        ];

        let resp = client
            .post("https://github.com/login/oauth/access_token")
            .header("Accept", "application/json")
            .form(&params)
            .send()
            .await
            .map_err(|e| format!("Token request failed: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("Token exchange failed (status {}): {}", status, text));
        }

        let tr: TokenResponse = resp.json().await.map_err(|e| format!("Invalid token response: {}", e))?;

        Ok(tr.access_token)
    }
}

#[async_trait]
impl AuthProviderPort for GithubAdapter {
    async fn upload_ssh_key(&self, _account: &str, _public_key: &str) -> Result<(), String> {
        // Placeholder - a real implementation would call the GitHub API to upload the key.
        Ok(())
    }

    async fn start_oauth_flow(&self, account: &str) -> Result<String, String> {
        // Delegate to the async helper already implemented in this adapter.
        self.start_oauth_flow_async(account).await
    }

    async fn fetch_profile(&self, token: &str) -> Result<Profile, String> {
        // Call GitHub API: GET /user
        #[derive(Deserialize)]
        struct UserResp {
            login: Option<String>,
            name: Option<String>,
            email: Option<String>,
        }

        let client = reqwest::Client::new();
        let resp = client
            .get("https://api.github.com/user")
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "gitt_account_manager")
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| format!("Failed to call /user: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("Failed to fetch user (status {}): {}", status, text));
        }

        let ur: UserResp = resp.json().await.map_err(|e| format!("Invalid user response: {}", e))?;

        let username = ur.login.or(ur.name).unwrap_or_else(|| "unknown".to_string());
        let email = ur.email.unwrap_or_else(|| "".to_string());

        Ok(Profile::new(&username, &email))
    }
}
