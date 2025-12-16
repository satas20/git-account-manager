use crate::domain::entity::Profile;
use crate::domain::ports::AuthProviderPort;
use crate::domain::use_cases::ProfilesManager;
use async_trait::async_trait;
use serde::Deserialize;
use std::env;

pub struct GithubAdapter<'a> {
    profiles_manager: Option<ProfilesManager<'a>>,
}

impl<'a> GithubAdapter<'a> {
    pub fn new() -> Self {
        Self {
            profiles_manager: None,
        }
    }

    /// Create adapter with a ProfilesManager for token refresh functionality
    pub fn with_manager(profiles_manager: ProfilesManager<'a>) -> Self {
        Self {
            profiles_manager: Some(profiles_manager),
        }
    }

    /// Helper method to make authenticated GitHub API requests with automatic token refresh.
    /// If the request returns 401, it will attempt to refresh the token and retry once.
    /// Requires the adapter to be created with `with_manager()`.
    async fn api_call_with_refresh<F, Fut, T>(
        &self,
        profile_key: &str,
        api_call: F,
    ) -> Result<T, String>
    where
        F: Fn(String) -> Fut,
        Fut: std::future::Future<Output = Result<T, String>>,
    {
        let mgr = self
            .profiles_manager
            .as_ref()
            .ok_or_else(|| "GithubAdapter not initialized with ProfilesManager".to_string())?;

        // Get current access token
        let token: String = mgr
            .get_auth_token(profile_key)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("No access token found for profile: {}", profile_key))?;

        // Try the API call with current token
        match api_call(token.clone()).await {
            Ok(result) => Ok(result),
            Err(e) if e.contains("status 401") || e.contains("Unauthorized") => {
                // Token expired, try to refresh
                let refresh_token = mgr
                    .get_refresh_token(profile_key)
                    .map_err(|e| e.to_string())?
                    .ok_or_else(|| {
                        format!("No refresh token found for profile: {}", profile_key)
                    })?;

                // Refresh the access token
                let (new_access_token, new_refresh_token) =
                    self.refresh_access_token(&refresh_token).await?;

                // Update stored tokens
                mgr.set_token_for_profile(profile_key, &new_access_token)
                    .map_err(|e| e.to_string())?;
                if let Some(new_rt) = new_refresh_token {
                    mgr.set_refresh_token_for_profile(profile_key, &new_rt)
                        .map_err(|e| e.to_string())?;
                }

                // Retry the API call with new token
                api_call(new_access_token).await
            }
            Err(e) => Err(e),
        }
    }

    /// Async helper that starts the OAuth flow for GitHub:
    /// - opens the user's browser to the GitHub authorize URL
    /// - starts a local HTTP listener on 127.0.0.1:8787 to receive the callback
    /// - exchanges the code for an access token and refresh token
    /// Returns (access_token, refresh_token) on success.
    pub async fn start_oauth_flow_async(&self, _account: &str) -> Result<(String, Option<String>), String> {
        // Read client credentials from compile-time env (set during build) or runtime env
        // This allows the binary to work out-of-the-box with embedded credentials,
        // but users can override with their own if needed.
        let client_id = env::var("GITHUB_CLIENT_ID")
            .or_else(|_| option_env!("GITHUB_CLIENT_ID").map(String::from).ok_or(()))
            .map_err(|_| {
                "Missing GITHUB_CLIENT_ID.\n\
                 Please set GITHUB_CLIENT_ID environment variable or rebuild with embedded credentials.\n\
                 See: https://github.com/satas20/git-account-manager#oauth-setup".to_string()
            })?;

        let client_secret = env::var("GITHUB_CLIENT_SECRET")
            .or_else(|_| option_env!("GITHUB_CLIENT_SECRET").map(String::from).ok_or(()))
            .map_err(|_| {
                "Missing GITHUB_CLIENT_SECRET.\n\
                 Please set GITHUB_CLIENT_SECRET environment variable or rebuild with embedded credentials.\n\
                 See: https://github.com/satas20/git-account-manager#oauth-setup".to_string()
            })?;

        // callback listener
        let redirect_host = "127.0.0.1";
        let redirect_port = 8787u16;
        let redirect_path = "/callback";
        let redirect_uri = format!("http://{}:{}{}", redirect_host, redirect_port, redirect_path);

        // Generate cryptographically secure random state token for CSRF protection
        use rand::Rng;
        let state: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        let scope = "read:user user:email write:public_key";
        let auth_url = format!(
            "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&scope={}&state={}",
            urlencoding::encode(&client_id),
            urlencoding::encode(&redirect_uri),
            urlencoding::encode(scope),
            urlencoding::encode(&state)
        );

        // START CALLBACK SERVER FIRST (before trying to open browser)
        // This ensures the server is listening even if browser opening fails
        let listen_addr = format!("{}:{}", redirect_host, redirect_port);
        let expected_state = state.clone();

        // Spawn the listener in background BEFORE opening browser
        let listener_handle = tokio::task::spawn_blocking(move || -> Result<String, String> {
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

            // Parse query parameters
            let query_params: std::collections::HashMap<String, String> = path
                .split('?')
                .nth(1)
                .map(|q| {
                    q.split('&')
                        .filter_map(|pair| {
                            let mut kv = pair.splitn(2, '=');
                            Some((kv.next()?.to_string(), kv.next()?.to_string()))
                        })
                        .collect()
                })
                .unwrap_or_default();

            // Validate state token (CSRF protection)
            let received_state = query_params.get("state").ok_or_else(|| "Missing state parameter".to_string())?;
            if received_state != &expected_state {
                let response = "HTTP/1.1 400 Bad Request\r\nContent-Type: text/html; charset=utf-8\r\n\r\n<html><head><script>window.setTimeout(function(){window.close();},3000);</script></head><body><h2>Authentication failed</h2><p>Invalid state parameter (CSRF check failed). This window will close automatically...</p></body></html>";
                stream.write_all(response.as_bytes()).ok();
                return Err("State parameter mismatch - possible CSRF attack".to_string());
            }

            let code = query_params.get("code").ok_or_else(|| format!("code not found in request path: {}", path))?.clone();

            // respond to browser
            let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\n\r\n<html><body><h2>Authentication complete</h2><p>You can close this window and return to the application.</p></body></html>";
            stream.write_all(response.as_bytes()).ok();

            Ok(code)
        });

        // Give the server a moment to start listening
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // NOW try to open the browser (server is already running)
        match webbrowser::open(&auth_url) {
            Ok(_) => {
                // Browser opened successfully
            }
            Err(e) => {
                // Browser failed to open - try to copy URL to clipboard
                match cli_clipboard::set_contents(auth_url.clone()) {
                    Ok(_) => {
                        return Err(format!(
                            "Failed to open browser: {}.\n\nOAuth URL copied to clipboard!\nPaste it in your browser to continue.\n\nURL: {}",
                            e, auth_url
                        ));
                    }
                    Err(_) => {
                        return Err(format!(
                            "Failed to open browser: {}.\n\nOpen this URL manually: {}",
                            e, auth_url
                        ));
                    }
                }
            }
        }

        // Wait for the callback (server is already running)
        let code = listener_handle
            .await
            .map_err(|e| format!("Listener task join error: {}", e))?;

        let code = match code {
            Ok(c) => c,
            Err(e) => return Err(e),
        };

        // Exchange the code for an access token
        #[derive(Deserialize)]
        #[allow(dead_code)]
        struct TokenResponse {
            access_token: String,
            refresh_token: Option<String>,
            expires_in: Option<u64>,
            refresh_token_expires_in: Option<u64>,
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

        Ok((tr.access_token, tr.refresh_token))
    }

    /// Fetch profile using a direct token (used during initial OAuth flow before profile exists)
    pub async fn fetch_profile_with_token(&self, token: &str) -> Result<Profile, String> {
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
        let mut email = ur.email.unwrap_or_default();
        
        // If email is empty, fetch from /user/emails and use the primary one
        if email.is_empty() {
            #[derive(Deserialize)]
            struct EmailResp {
                email: String,
                primary: bool,
            }

            let emails_resp = client
                .get("https://api.github.com/user/emails")
                .header("Accept", "application/vnd.github+json")
                .header("User-Agent", "gitt_account_manager")
                .bearer_auth(token)
                .send()
                .await
                .map_err(|e| format!("Failed to call /user/emails: {}", e))?;

            if emails_resp.status().is_success() {
                let emails: Vec<EmailResp> = emails_resp
                    .json()
                    .await
                    .map_err(|e| format!("Invalid emails response: {}", e))?;
                
                // Find primary email
                if let Some(primary_email) = emails.iter().find(|e| e.primary) {
                    email = primary_email.email.clone();
                }
            }
        }

        print!("Fetched GitHub user: {} <{}>", username, email);
        Ok(Profile::new(&username, &email))
    }

    /// Upload SSH key using a direct token (used during initial profile setup)
    /// device_id should be in format "username@hostname"
    pub async fn upload_ssh_key_with_token(&self, token: &str, public_key: &str, device_id: &str) -> Result<u64, String> {
        #[derive(serde::Serialize)]
        struct KeyBody<'a> {
            title: &'a str,
            key: &'a str,
        }

        // Use device identifier to make key title unique per device
        let key_title = format!("git-acc-mngr:{}", device_id);

        let body = KeyBody {
            title: &key_title,
            key: public_key,
        };

        let client = reqwest::Client::new();
        let resp = client
            .post("https://api.github.com/user/keys")
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "gitt_account_manager")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .bearer_auth(token)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Failed to upload SSH key: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("Failed to upload SSH key (status {}): {}", status, text));
        }

        #[derive(Deserialize)]
        struct KeyResp {
            id: u64,
        }
        let key_resp: KeyResp = resp.json().await.map_err(|e| format!("Invalid key response: {}", e))?;

        Ok(key_resp.id)
    }
}

#[async_trait]
impl<'a> AuthProviderPort for GithubAdapter<'a> {
    async fn upload_ssh_key(&self, profile_key: &str, public_key: &str) -> Result<u64, String> {
        // Get device identifier before entering the closure
        let device_id = if let Some(mgr) = &self.profiles_manager {
            mgr.get_device_identifier()
        } else {
            return Err("GithubAdapter not initialized with ProfilesManager".to_string());
        };

        self.api_call_with_refresh(profile_key, |token| {
            let device_id = device_id.clone();
            async move {
                #[derive(serde::Serialize)]
                struct KeyBody<'b> {
                    title: &'b str,
                    key: &'b str,
                }

                let key_title = format!("git-acc-mngr:{}", device_id);

            let body = KeyBody {
                title: &key_title,
                key: public_key,
            };

            let client = reqwest::Client::new();
            let resp = client
                .post("https://api.github.com/user/keys")
                .header("Accept", "application/vnd.github+json")
                .header("User-Agent", "gitt_account_manager")
                .header("X-GitHub-Api-Version", "2022-11-28")
                .bearer_auth(&token)
                .json(&body)
                .send()
                .await
                .map_err(|e| format!("Failed to upload SSH key: {}", e))?;

            if !resp.status().is_success() {
                let status = resp.status();
                let text = resp.text().await.unwrap_or_default();
                return Err(format!("Failed to upload SSH key (status {}): {}", status, text));
            }

            #[derive(Deserialize)]
            struct KeyResp {
                id: u64,
            }
                let key_resp: KeyResp = resp.json().await.map_err(|e| format!("Invalid key response: {}", e))?;

                Ok(key_resp.id)
            }
        }).await
    }

    async fn delete_ssh_key(&self, profile_key: &str, key_id: u64) -> Result<(), String> {
        self.api_call_with_refresh(profile_key, |token| async move {
            let client = reqwest::Client::new();
            let url = format!("https://api.github.com/user/keys/{}", key_id);
            let resp = client
                .delete(&url)
                .header("Accept", "application/vnd.github+json")
                .header("User-Agent", "gitt_account_manager")
                .header("X-GitHub-Api-Version", "2022-11-28")
                .bearer_auth(&token)
                .send()
                .await
                .map_err(|e| format!("Failed to delete SSH key: {}", e))?;

            if !resp.status().is_success() {
                let status = resp.status();
                let text = resp.text().await.unwrap_or_default();
                return Err(format!("Failed to delete SSH key (status {}): {}", status, text));
            }

            Ok(())
        }).await
    }

    async fn start_oauth_flow(&self, account: &str) -> Result<(String, Option<String>), String> {
        // Delegate to the async helper already implemented in this adapter.
        self.start_oauth_flow_async(account).await
    }

    async fn refresh_access_token(&self, refresh_token: &str) -> Result<(String, Option<String>), String> {
        let client_id = env::var("GITHUB_CLIENT_ID")
            .map_err(|_| "Missing GITHUB_CLIENT_ID environment variable".to_string())?;
        let client_secret = env::var("GITHUB_CLIENT_SECRET")
            .map_err(|_| "Missing GITHUB_CLIENT_SECRET environment variable".to_string())?;

        #[derive(Deserialize)]
        #[allow(dead_code)]
        struct TokenResponse {
            access_token: String,
            refresh_token: Option<String>,
            expires_in: Option<u64>,
            refresh_token_expires_in: Option<u64>,
        }

        let client = reqwest::Client::new();
        let params = [
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
        ];

        let resp = client
            .post("https://github.com/login/oauth/access_token")
            .header("Accept", "application/json")
            .form(&params)
            .send()
            .await
            .map_err(|e| format!("Token refresh request failed: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("Token refresh failed (status {}): {}", status, text));
        }

        let tr: TokenResponse = resp
            .json()
            .await
            .map_err(|e| format!("Invalid token refresh response: {}", e))?;

        Ok((tr.access_token, tr.refresh_token))
    }

    async fn fetch_profile(&self, profile_key: &str) -> Result<Profile, String> {
        self.api_call_with_refresh(profile_key, |token| async move {
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
                .bearer_auth(&token)
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
            let email = ur.email.unwrap_or_else(|| "".to_string()); //TODO email is not saved correctly 
            print!("Fetched GitHub user: {} <{}>", username, email);
            Ok(Profile::new(&username, &email))
        }).await
    }
}
