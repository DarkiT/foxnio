use reqwest::Client;
use serde::{Deserialize, Serialize};

/// SDK client for Sora API
pub struct SoraSdkClient {
    client: Client,
    api_key: String,
    base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateRequest {
    pub prompt: String,
    pub duration_seconds: Option<u32>,
    pub resolution: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateResponse {
    pub id: String,
    pub status: String,
    pub video_url: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum SdkError {
    #[error("API error: {0}")]
    ApiError(String),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}

impl SoraSdkClient {
    pub fn new(api_key: String, base_url: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url,
        }
    }

    /// Generate video
    pub async fn generate(&self, request: &GenerateRequest) -> Result<GenerateResponse, SdkError> {
        let response = self
            .client
            .post(format!("{}/v1/generate", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(request)
            .send()
            .await?
            .json::<GenerateResponse>()
            .await?;

        Ok(response)
    }

    /// Get generation status
    pub async fn get_status(&self, id: &str) -> Result<GenerateResponse, SdkError> {
        let response = self
            .client
            .get(format!("{}/v1/generate/{}", self.base_url, id))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?
            .json::<GenerateResponse>()
            .await?;

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sdk_client_creation() {
        let client = SoraSdkClient::new("test_key".to_string(), "https://api.sora.ai".to_string());
        assert_eq!(client.base_url, "https://api.sora.ai");
    }
}
