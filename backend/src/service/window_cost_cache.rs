//! Window Cost Cache 服务
//!
//! 窗口期费用缓存，用于优化实时计费查询性能

use anyhow::Result;
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 窗口期费用缓存
pub struct WindowCostCache {
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    window_duration: Duration,
}

#[derive(Clone, Debug)]
struct CacheEntry {
    cost: f64,
    tokens: i64,
    requests: i64,
    expires_at: DateTime<Utc>,
}

impl WindowCostCache {
    pub fn new(window_duration: Duration) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            window_duration,
        }
    }
    
    /// 获取窗口期费用
    pub async fn get_window_cost(&self, key: &str) -> Option<(f64, i64, i64)> {
        let cache = self.cache.read().await;
        cache.get(key).and_then(|entry| {
            if entry.expires_at > Utc::now() {
                Some((entry.cost, entry.tokens, entry.requests))
            } else {
                None
            }
        })
    }
    
    /// 设置窗口期费用
    pub async fn set_window_cost(
        &self,
        key: String,
        cost: f64,
        tokens: i64,
        requests: i64,
    ) {
        let mut cache = self.cache.write().await;
        cache.insert(key, CacheEntry {
            cost,
            tokens,
            requests,
            expires_at: Utc::now() + self.window_duration,
        });
    }
    
    /// 清理过期缓存
    pub async fn cleanup_expired(&self) {
        let mut cache = self.cache.write().await;
        let now = Utc::now();
        cache.retain(|_, entry| entry.expires_at > now);
    }
    
    /// 清空缓存
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }
    
    /// 获取缓存统计
    pub async fn stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        let total = cache.len();
        let active = cache.values().filter(|e| e.expires_at > Utc::now()).count();
        
        CacheStats {
            total_entries: total,
            active_entries: active,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub active_entries: usize,
}

impl Default for WindowCostCache {
    fn default() -> Self {
        Self::new(Duration::minutes(5))
    }
}
