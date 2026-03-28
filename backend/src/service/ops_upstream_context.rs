use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Upstream context for operations
pub struct OpsUpstreamContext {
    provider: String,
    model: String,
    account_id: i64,
    metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamInfo {
    pub provider: String,
    pub model: String,
    pub account_id: i64,
    pub endpoint: String,
    pub region: Option<String>,
    pub latency_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamMetrics {
    pub request_count: u64,
    pub success_count: u64,
    pub error_count: u64,
    pub total_latency_ms: u64,
    pub p50_latency_ms: u64,
    pub p95_latency_ms: u64,
    pub p99_latency_ms: u64,
}

impl OpsUpstreamContext {
    pub fn new(provider: String, model: String, account_id: i64) -> Self {
        Self {
            provider,
            model,
            account_id,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Get upstream info
    pub fn to_info(&self, endpoint: String) -> UpstreamInfo {
        UpstreamInfo {
            provider: self.provider.clone(),
            model: self.model.clone(),
            account_id: self.account_id,
            endpoint,
            region: self.metadata.get("region").cloned(),
            latency_ms: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upstream_context() {
        let ctx = OpsUpstreamContext::new("openai".to_string(), "gpt-4".to_string(), 123)
            .with_metadata("region".to_string(), "us-east-1".to_string());

        assert_eq!(ctx.provider, "openai");
        assert_eq!(ctx.metadata.get("region"), Some(&"us-east-1".to_string()));
    }
}
