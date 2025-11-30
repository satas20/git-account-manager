use crate::domain::entity::Profile;
use crate::domain::ports::AuthProviderPort;
use crate::domain::use_cases::ProfilesManager;
use async_trait::async_trait;
use serde::Deserialize;
use std::env;
//FİX:not working correctly 
pub struct GitlabAdapter<'a> {
    profiles_manager: Option<ProfilesManager<'a>>,
}

impl<'a> GitlabAdapter<'a> {
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

    /// Helper method to make authenticated GitLab API requests with automatic token refresh.
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
            .ok_or_else(|| "GitlabAdapter not initialized with ProfilesManager".to_string())?;

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

    /// Async helper that starts the OAuth flow for GitLab:
    /// - opens the user's browser to the GitLab authorize URL
    /// - starts a local HTTP listener on 127.0.0.1:8788 to receive the callback
    /// - exchanges the code for an access token and refresh token
    /// Returns (access_token, refresh_token) on success.
    pub async fn start_oauth_flow_async(&self, _account: &str) -> Result<(String, Option<String>), String> {
        // Try runtime env first (loaded via dotenvy in main.rs from .env file)
        let client_id = env::var("GITLAB_APP_ID")
            .map_err(|_| "GitLab OAuth not configured. Set GITLAB_APP_ID in .env file or as environment variable.".to_string())?;

        let client_secret = env::var("GITLAB_CLIENT_SECRET")
            .map_err(|_| "GitLab OAuth not configured. Set GITLAB_CLIENT_SECRET in .env file or as environment variable.".to_string())?;

        // callback listener
        let redirect_host = "127.0.0.1";
        let redirect_port = 8788u16;
        let redirect_path = "/callback";
        let redirect_uri = format!("http://{}:{}{}", redirect_host, redirect_port, redirect_path);

        // Generate cryptographically secure random state token for CSRF protection
        use rand::Rng;
        let state: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        let scope = "read_user api write_repository";
        let auth_url = format!(
            "https://gitlab.com/oauth/authorize?client_id={}&redirect_uri={}&response_type=code&state={}&scope={}",
            urlencoding::encode(&client_id),
            urlencoding::encode(&redirect_uri),
            urlencoding::encode(&state),
            urlencoding::encode(scope)
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
        let expected_state = state.clone();
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
        })
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
            token_type: Option<String>,
            scope: Option<String>,
            created_at: Option<u64>,
        }

        let client = reqwest::Client::new();
        let params = [
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("code", code.as_str()),
            ("grant_type", "authorization_code"),
            ("redirect_uri", redirect_uri.as_str()),
        ];

        let resp = client
            .post("https://gitlab.com/oauth/token")
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
            username: Option<String>,
            name: Option<String>,
            email: Option<String>,
        }

        let client = reqwest::Client::new();
        let resp = client
            .get("https://gitlab.com/api/v4/user")
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

        let username = ur.username.or(ur.name).unwrap_or_else(|| "unknown".to_string());
        let email = ur.email.unwrap_or_default();

        print!("Fetched GitLab user: {} <{}>", username, email);
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
            .post("https://gitlab.com/api/v4/user/keys")
            .header("User-Agent", "gitt_account_manager")
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
impl<'a> AuthProviderPort for GitlabAdapter<'a> {
    async fn upload_ssh_key(&self, profile_key: &str, public_key: &str) -> Result<u64, String> {
        // Get device identifier before entering the closure
        let device_id = if let Some(mgr) = &self.profiles_manager {
            mgr.get_device_identifier()
        } else {
            return Err("GitlabAdapter not initialized with ProfilesManager".to_string());
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
                    .post("https://gitlab.com/api/v4/user/keys")
                    .header("User-Agent", "gitt_account_manager")
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
            let url = format!("https://gitlab.com/api/v4/user/keys/{}", key_id);
            let resp = client
                .delete(&url)
                .header("User-Agent", "gitt_account_manager")
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
        let client_id = env::var("GITLAB_APP_ID")
            .map_err(|_| "GitLab credentials not configured")?;

        let client_secret = env::var("GITLAB_CLIENT_SECRET")
            .map_err(|_| "GitLab credentials not configured")?;

        #[derive(Deserialize)]
        #[allow(dead_code)]
        struct TokenResponse {
            access_token: String,
            refresh_token: Option<String>,
            expires_in: Option<u64>,
            token_type: Option<String>,
            scope: Option<String>,
            created_at: Option<u64>,
        }

        let client = reqwest::Client::new();
        let params = [
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("refresh_token", refresh_token),
            ("grant_type", "refresh_token"),
        ];

        let resp = client
            .post("https://gitlab.com/oauth/token")
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
            #[derive(Deserialize)]
            struct UserResp {
                username: Option<String>,
                name: Option<String>,
                email: Option<String>,
            }

            let client = reqwest::Client::new();
            let resp = client
                .get("https://gitlab.com/api/v4/user")
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

            let username = ur.username.or(ur.name).unwrap_or_else(|| "unknown".to_string());
            let email = ur.email.unwrap_or_default();
            print!("Fetched GitLab user: {} <{}>", username, email);
            Ok(Profile::new(&username, &email))
        }).await
    }
}
