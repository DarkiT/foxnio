use reqwest::Client;
use serde::{Deserialize, Serialize};

/// OAuth refresh API client
pub struct OAuthRefreshApi {
    client: Client,
    config: OAuthConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConfig {
    pub token_url: String,
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Serialize)]
struct RefreshRequest {
    grant_type: String,
    refresh_token: String,
    client_id: String,
    client_secret: String,
}

#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: i64,
    pub token_type: String,
}

#[derive(Debug, thiserror::Error)]
pub enum OAuthError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Token refresh failed")]
    RefreshFailed,
}

impl OAuthRefreshApi {
    pub fn new(config: OAuthConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    /// Refresh access token
    pub async fn refresh(&self, refresh_token: &str) -> Result<TokenResponse, OAuthError> {
        let request = RefreshRequest {
            grant_type: "refresh_token".to_string(),
            refresh_token: refresh_token.to_string(),
            client_id: self.config.client_id.clone(),
            client_secret: self.config.client_secret.clone(),
        };

        let response = self
            .client
            .post(&self.config.token_url)
            .form(&request)
            .send()
            .await?
            .json::<TokenResponse>()
            .await?;

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth_config() {
        let config = OAuthConfig {
            token_url: "https://oauth.example.com/token".to_string(),
            client_id: "test_client".to_string(),
            client_secret: "test_secret".to_string(),
        };

        assert_eq!(config.client_id, "test_client");
    }
}
