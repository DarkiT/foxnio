//! 刷新令牌缓存服务
//!
//! 管理 refresh token 的缓存存储

#![allow(dead_code)]

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::token_cache_key::TokenCacheKey;

/// 缓存条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub refresh_token: String,
    pub account_id: String,
    pub provider: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub use_count: u64,
    pub metadata: serde_json::Value,
}

/// 缓存配置
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub max_entries: usize,
    pub ttl_seconds: u64,
    pub cleanup_interval_seconds: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 100000,
            ttl_seconds: 7 * 24 * 3600, // 7 天
            cleanup_interval_seconds: 3600,
        }
    }
}

/// 缓存统计
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub total_entries: usize,
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub updates: u64,
}

/// 刷新令牌缓存
pub struct RefreshTokenCache {
    config: CacheConfig,
    entries: Arc<RwLock<HashMap<String, CacheEntry>>>,
    stats: Arc<RwLock<CacheStats>>,
}

impl RefreshTokenCache {
    /// 创建新的缓存
    pub fn new(config: CacheConfig) -> Self {
        Self {
            config,
            entries: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }

    /// 存储刷新令牌
    pub async fn store(
        &self,
        key: &TokenCacheKey,
        refresh_token: &str,
        metadata: serde_json::Value,
    ) -> Result<()> {
        let cache_key = key.to_string();
        let now = Utc::now();

        let entry = CacheEntry {
            refresh_token: refresh_token.to_string(),
            account_id: key.account_id.clone(),
            provider: key.provider.clone(),
            created_at: now,
            updated_at: now,
            last_used_at: None,
            use_count: 0,
            metadata,
        };

        let mut entries = self.entries.write().await;
        
        // 检查容量
        if entries.len() >= self.config.max_entries {
            // 移除最久未使用的条目
            if let Some(oldest_key) = entries
                .iter()
                .min_by_key(|(_, e)| e.last_used_at.unwrap_or(e.created_at))
                .map(|(k, _)| k.clone())
            {
                entries.remove(&oldest_key);
                let mut stats = self.stats.write().await;
                stats.evictions += 1;
            }
        }

        entries.insert(cache_key, entry);

        let mut stats = self.stats.write().await;
        stats.total_entries = entries.len();
        stats.updates += 1;

        Ok(())
    }

    /// 获取刷新令牌
    pub async fn get(&self, key: &TokenCacheKey) -> Option<String> {
        let cache_key = key.to_string();
        
        let mut entries = self.entries.write().await;
        
        if let Some(entry) = entries.get_mut(&cache_key) {
            entry.last_used_at = Some(Utc::now());
            entry.use_count += 1;

            let mut stats = self.stats.write().await;
            stats.hits += 1;

            Some(entry.refresh_token.clone())
        } else {
            let mut stats = self.stats.write().await;
            stats.misses += 1;
            None
        }
    }

    /// 获取完整条目
    pub async fn get_entry(&self, key: &TokenCacheKey) -> Option<CacheEntry> {
        let cache_key = key.to_string();
        
        let mut entries = self.entries.write().await;
        
        if let Some(entry) = entries.get_mut(&cache_key) {
            entry.last_used_at = Some(Utc::now());
            entry.use_count += 1;

            let mut stats = self.stats.write().await;
            stats.hits += 1;

            Some(entry.clone())
        } else {
            let mut stats = self.stats.write().await;
            stats.misses += 1;
            None
        }
    }

    /// 更新刷新令牌
    pub async fn update(
        &self,
        key: &TokenCacheKey,
        refresh_token: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<bool> {
        let cache_key = key.to_string();
        
        let mut entries = self.entries.write().await;
        
        if let Some(entry) = entries.get_mut(&cache_key) {
            entry.refresh_token = refresh_token.to_string();
            entry.updated_at = Utc::now();
            
            if let Some(meta) = metadata {
                entry.metadata = meta;
            }

            let mut stats = self.stats.write().await;
            stats.updates += 1;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// 移除条目
    pub async fn remove(&self, key: &TokenCacheKey) -> bool {
        let cache_key = key.to_string();
        
        let mut entries = self.entries.write().await;
        let removed = entries.remove(&cache_key).is_some();
        
        if removed {
            let mut stats = self.stats.write().await;
            stats.total_entries = entries.len();
        }
        
        removed
    }

    /// 检查条目是否存在
    pub async fn contains(&self, key: &TokenCacheKey) -> bool {
        let entries = self.entries.read().await;
        entries.contains_key(&key.to_string())
    }

    /// 清空缓存
    pub async fn clear(&self) {
        let mut entries = self.entries.write().await;
        entries.clear();
        
        let mut stats = self.stats.write().await;
        stats.total_entries = 0;
    }

    /// 清理过期条目
    pub async fn cleanup_expired(&self) -> usize {
        let now = Utc::now();
        let ttl = chrono::Duration::seconds(self.config.ttl_seconds as i64);
        
        let mut entries = self.entries.write().await;
        let before = entries.len();
        
        entries.retain(|_, entry| {
            now - entry.updated_at < ttl
        });
        
        let removed = before - entries.len();
        
        if removed > 0 {
            let mut stats = self.stats.write().await;
            stats.total_entries = entries.len();
            stats.evictions += removed as u64;
        }
        
        removed
    }

    /// 获取缓存大小
    pub async fn size(&self) -> usize {
        self.entries.read().await.len()
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> CacheStats {
        let stats = self.stats.read().await.clone();
        stats
    }

    /// 计算命中率
    pub async fn hit_rate(&self) -> f64 {
        let stats = self.stats.read().await;
        let total = stats.hits + stats.misses;
        if total == 0 {
            return 0.0;
        }
        stats.hits as f64 / total as f64
    }

    /// 获取所有条目的键
    pub async fn keys(&self) -> Vec<String> {
        self.entries.read().await.keys().cloned().collect()
    }

    /// 按提供商获取条目
    pub async fn get_by_provider(&self, provider: &str) -> Vec<CacheEntry> {
        let entries = self.entries.read().await;
        entries
            .values()
            .filter(|e| e.provider == provider)
            .cloned()
            .collect()
    }

    /// 启动后台清理任务
    pub fn start_background_cleanup(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                std::time::Duration::from_secs(self.config.cleanup_interval_seconds)
            );

            loop {
                interval.tick().await;
                let cleaned = self.cleanup_expired().await;
                if cleaned > 0 {
                    tracing::debug!("Cleaned up {} expired refresh tokens", cleaned);
                }
            }
        })
    }
}

impl Default for RefreshTokenCache {
    fn default() -> Self {
        Self::new(CacheConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_store_and_get() {
        let cache = RefreshTokenCache::default();
        let key = TokenCacheKey::new("openai", "account1");
        
        cache.store(
            &key,
            "refresh_token_123",
            serde_json::json!({}),
        ).await.unwrap();
        
        let token = cache.get(&key).await;
        assert_eq!(token, Some("refresh_token_123".to_string()));
    }

    #[tokio::test]
    async fn test_update() {
        let cache = RefreshTokenCache::default();
        let key = TokenCacheKey::new("openai", "account1");
        
        cache.store(
            &key,
            "old_token",
            serde_json::json!({}),
        ).await.unwrap();
        
        let updated = cache.update(
            &key,
            "new_token",
            Some(serde_json::json!({"updated": true})),
        ).await.unwrap();
        
        assert!(updated);
        
        let token = cache.get(&key).await;
        assert_eq!(token, Some("new_token".to_string()));
    }

    #[tokio::test]
    async fn test_remove() {
        let cache = RefreshTokenCache::default();
        let key = TokenCacheKey::new("openai", "account1");
        
        cache.store(&key, "token", serde_json::json!({})).await.unwrap();
        
        let removed = cache.remove(&key).await;
        assert!(removed);
        
        let token = cache.get(&key).await;
        assert!(token.is_none());
    }

    #[tokio::test]
    async fn test_contains() {
        let cache = RefreshTokenCache::default();
        let key = TokenCacheKey::new("openai", "account1");
        
        assert!(!cache.contains(&key).await);
        
        cache.store(&key, "token", serde_json::json!({})).await.unwrap();
        assert!(cache.contains(&key).await);
    }

    #[tokio::test]
    async fn test_hit_rate() {
        let cache = RefreshTokenCache::default();
        let key1 = TokenCacheKey::new("openai", "account1");
        let key2 = TokenCacheKey::new("openai", "account2");
        
        cache.store(&key1, "token1", serde_json::json!({})).await.unwrap();
        
        // 1 hit, 1 miss
        cache.get(&key1).await;
        cache.get(&key2).await;
        
        let rate = cache.hit_rate().await;
        assert!((rate - 0.5).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_get_by_provider() {
        let cache = RefreshTokenCache::default();
        
        cache.store(
            &TokenCacheKey::new("openai", "acc1"),
            "token1",
            serde_json::json!({}),
        ).await.unwrap();
        
        cache.store(
            &TokenCacheKey::new("anthropic", "acc2"),
            "token2",
            serde_json::json!({}),
        ).await.unwrap();
        
        cache.store(
            &TokenCacheKey::new("openai", "acc3"),
            "token3",
            serde_json::json!({}),
        ).await.unwrap();
        
        let openai_entries = cache.get_by_provider("openai").await;
        assert_eq!(openai_entries.len(), 2);
    }
}
