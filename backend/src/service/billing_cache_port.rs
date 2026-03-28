//! Billing cache port (interface)

use async_trait::async_trait;

/// Billing cache entry
#[derive(Debug, Clone)]
pub struct BillingCacheEntry {
    pub user_id: i64,
    pub balance: f64,
    pub last_updated: i64,
}

/// Billing cache port trait
#[async_trait]
pub trait BillingCachePort: Send + Sync {
    async fn get(&self, user_id: i64) -> Option<BillingCacheEntry>;
    async fn set(&self, entry: BillingCacheEntry);
    async fn invalidate(&self, user_id: i64);
}

/// Default implementation
pub struct DefaultBillingCachePort {
    cache: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<i64, BillingCacheEntry>>>,
}

impl Default for DefaultBillingCachePort {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultBillingCachePort {
    pub fn new() -> Self {
        Self {
            cache: std::sync::Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }
}

#[async_trait]
impl BillingCachePort for DefaultBillingCachePort {
    async fn get(&self, user_id: i64) -> Option<BillingCacheEntry> {
        let cache = self.cache.read().await;
        cache.get(&user_id).cloned()
    }

    async fn set(&self, entry: BillingCacheEntry) {
        let mut cache = self.cache.write().await;
        cache.insert(entry.user_id, entry);
    }

    async fn invalidate(&self, user_id: i64) {
        let mut cache = self.cache.write().await;
        cache.remove(&user_id);
    }
}
