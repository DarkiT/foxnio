use bytes::Bytes;
use serde::{Deserialize, Serialize};

/// Upstream forwarder for Sora API requests
pub struct SoraUpstreamForwarder {
    config: ForwarderConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForwarderConfig {
    pub upstream_url: String,
    pub timeout_seconds: u64,
    pub max_retries: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum ForwarderError {
    #[error("Upstream error: {0}")]
    UpstreamError(String),
    #[error("Timeout")]
    Timeout,
}

impl SoraUpstreamForwarder {
    pub fn new(config: ForwarderConfig) -> Self {
        Self { config }
    }

    /// Forward request to upstream
    pub async fn forward(&self, request: &[u8]) -> Result<Bytes, ForwarderError> {
        // In real implementation, would make HTTP request
        Ok(Bytes::copy_from_slice(request))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forwarder_config() {
        let config = ForwarderConfig {
            upstream_url: "https://api.openai.com".to_string(),
            timeout_seconds: 30,
            max_retries: 3,
        };

        assert_eq!(config.timeout_seconds, 30);
    }
}
