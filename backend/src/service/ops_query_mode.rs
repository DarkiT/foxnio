use serde::{Deserialize, Serialize};

/// Query mode configuration for operations service
pub struct OpsQueryMode {
    mode: QueryMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QueryMode {
    Simple,
    Advanced,
    Aggregated,
    RealTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryConfig {
    pub mode: QueryMode,
    pub cache_enabled: bool,
    pub cache_ttl_seconds: u64,
    pub timeout_ms: u64,
}

impl Default for QueryConfig {
    fn default() -> Self {
        Self {
            mode: QueryMode::Simple,
            cache_enabled: true,
            cache_ttl_seconds: 60,
            timeout_ms: 5000,
        }
    }
}

impl OpsQueryMode {
    pub fn new(mode: QueryMode) -> Self {
        Self { mode }
    }

    /// Check if cache should be used
    pub fn should_use_cache(&self) -> bool {
        matches!(self.mode, QueryMode::Simple | QueryMode::Aggregated)
    }

    /// Get timeout for query
    pub fn get_timeout_ms(&self) -> u64 {
        match self.mode {
            QueryMode::Simple => 5000,
            QueryMode::Advanced => 15000,
            QueryMode::Aggregated => 30000,
            QueryMode::RealTime => 60000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_mode_cache() {
        let simple = OpsQueryMode::new(QueryMode::Simple);
        assert!(simple.should_use_cache());

        let realtime = OpsQueryMode::new(QueryMode::RealTime);
        assert!(!realtime.should_use_cache());
    }

    #[test]
    fn test_query_config_default() {
        let config = QueryConfig::default();
        assert_eq!(config.mode, QueryMode::Simple);
        assert!(config.cache_enabled);
    }
}
