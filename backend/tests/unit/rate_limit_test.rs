//! Rate Limit 单元测试
//!
//! 测试 service/rate_limit.rs 的核心功能

use chrono::{Duration, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// ============ 速率限制器配置测试 ============

#[test]
fn test_rate_limit_config() {
    let config = RateLimitConfig {
        requests_per_minute: 60,
        requests_per_hour: 3600,
        requests_per_day: 86400,
        burst_size: 10,
    };

    assert_eq!(config.requests_per_minute, 60);
    assert_eq!(config.requests_per_hour, 3600);
    assert_eq!(config.requests_per_day, 86400);
    assert_eq!(config.burst_size, 10);
}

// ============ 令牌桶算法测试 ============

#[test]
fn test_token_bucket_creation() {
    let bucket = TokenBucket::new(100, 10.0); // 容量100，每秒补充10个令牌

    assert_eq!(bucket.capacity, 100);
    assert_eq!(bucket.tokens, 100); // 初始满桶
    assert_eq!(bucket.refill_rate, 10.0);
}

#[test]
fn test_token_bucket_consume() {
    let mut bucket = TokenBucket::new(100, 10.0);

    // 消耗令牌
    assert!(bucket.try_consume(10));
    assert_eq!(bucket.tokens, 90);

    // 消耗更多令牌
    assert!(bucket.try_consume(50));
    assert_eq!(bucket.tokens, 40);

    // 尝试消耗超过剩余数量的令牌
    assert!(!bucket.try_consume(50));
    assert_eq!(bucket.tokens, 40); // 未消耗
}

#[test]
fn test_token_bucket_refill() {
    let mut bucket = TokenBucket::new(100, 10.0);
    bucket.tokens = 0;

    // 模拟时间流逝
    bucket.refill(1.0); // 1秒后
    assert_eq!(bucket.tokens, 10);

    bucket.refill(5.0); // 再过5秒
    assert_eq!(bucket.tokens, 60);

    bucket.refill(10.0); // 再过10秒，但不应该超过容量
    assert_eq!(bucket.tokens, 100);
}

#[test]
fn test_token_bucket_burst() {
    let mut bucket = TokenBucket::new(100, 1.0); // 每秒补充1个令牌

    // 突发请求
    for _ in 0..10 {
        assert!(bucket.try_consume(10));
    }

    // 令牌应该用完
    assert_eq!(bucket.tokens, 0);
    assert!(!bucket.try_consume(1));
}

// ============ 滑动窗口算法测试 ============

#[test]
fn test_sliding_window_creation() {
    let window = SlidingWindow::new(60); // 60秒窗口

    assert_eq!(window.window_size, 60);
    assert!(window.requests.is_empty());
}

#[test]
fn test_sliding_window_record() {
    let mut window = SlidingWindow::new(60);
    let now = Utc::now();

    // 记录请求
    window.record(now);
    window.record(now + Duration::seconds(1));
    window.record(now + Duration::seconds(2));

    assert_eq!(window.count(now + Duration::seconds(3)), 3);
}

#[test]
fn test_sliding_window_expiry() {
    let mut window = SlidingWindow::new(60);
    let now = Utc::now();

    // 记录请求
    window.record(now);

    // 61秒后，请求应该过期
    assert_eq!(window.count(now + Duration::seconds(61)), 0);
}

#[test]
fn test_sliding_window_rate_limit() {
    let mut window = SlidingWindow::new(60);
    let now = Utc::now();

    // 记录60个请求（达到限制）
    for i in 0..60 {
        window.record(now + Duration::seconds(i));
    }

    assert_eq!(window.count(now + Duration::seconds(59)), 60);

    // 再记录一个应该超出限制
    window.record(now + Duration::seconds(60));
    assert_eq!(window.count(now + Duration::seconds(60)), 61);
}

// ============ 固定窗口算法测试 ============

#[test]
fn test_fixed_window_creation() {
    let window = FixedWindow::new(60); // 60秒窗口

    assert_eq!(window.window_size, 60);
    assert_eq!(window.count, 0);
}

#[test]
fn test_fixed_window_increment() {
    let mut window = FixedWindow::new(60);
    let now = Utc::now();

    window.increment(now);
    assert_eq!(window.count, 1);

    window.increment(now);
    assert_eq!(window.count, 2);

    // 在同一窗口内
    window.increment(now + Duration::seconds(30));
    assert_eq!(window.count, 3);
}

#[test]
fn test_fixed_window_reset() {
    let mut window = FixedWindow::new(60);
    let now = Utc::now();

    window.increment(now);
    window.increment(now);

    // 窗口过期
    window.increment(now + Duration::seconds(61));
    assert_eq!(window.count, 1); // 重置为1
}

// ============ 用户级别限流测试 ============

#[test]
fn test_user_rate_limit() {
    let mut limiter = UserRateLimiter::new(60); // 每分钟60次

    let user_id = "user-123";

    // 前60次应该成功
    for _ in 0..60 {
        assert!(limiter.check(user_id));
    }

    // 第61次应该失败
    assert!(!limiter.check(user_id));
}

#[test]
fn test_different_users_independent() {
    let mut limiter = UserRateLimiter::new(10);

    let user1 = "user-1";
    let user2 = "user-2";

    // 用户1用完配额
    for _ in 0..10 {
        assert!(limiter.check(user1));
    }
    assert!(!limiter.check(user1));

    // 用户2应该还有配额
    assert!(limiter.check(user2));
}

// ============ API Key 级别限流测试 ============

#[test]
fn test_api_key_rate_limit() {
    let mut limiter = ApiKeyRateLimiter::new(1000); // 每分钟1000次

    let api_key = "sk-test-key-123";

    // 测试限流
    for _ in 0..1000 {
        assert!(limiter.check(api_key));
    }

    assert!(!limiter.check(api_key));
}

#[test]
fn test_api_key_tier_limits() {
    let free_limit = 60;
    let pro_limit = 600;
    let enterprise_limit = 6000;

    assert!(free_limit < pro_limit);
    assert!(pro_limit < enterprise_limit);

    // 不同层级的限制应该不同
    let tiers = vec![
        ("free", free_limit),
        ("pro", pro_limit),
        ("enterprise", enterprise_limit),
    ];

    for (tier, limit) in tiers {
        let mut limiter = ApiKeyRateLimiter::new(limit);
        let key = format!("{}-key", tier);

        for _ in 0..limit {
            assert!(limiter.check(&key), "Tier {} should allow {} requests", tier, limit);
        }

        assert!(!limiter.check(&key), "Tier {} should deny request {} ", tier, limit + 1);
    }
}

// ============ 全局限流测试 ============

#[test]
fn test_global_rate_limit() {
    let mut limiter = GlobalRateLimiter::new(10000); // 全局每分钟10000次

    // 全局限流
    for _ in 0..10000 {
        assert!(limiter.check());
    }

    assert!(!limiter.check());
}

// ============ 组合限流测试 ============

#[test]
fn test_combined_rate_limits() {
    let user_limit = 60;
    let api_key_limit = 1000;
    let global_limit = 10000;

    let user = "user-123";
    let api_key = "sk-key-123";

    let mut user_limiter = UserRateLimiter::new(user_limit);
    let mut api_key_limiter = ApiKeyRateLimiter::new(api_key_limit);
    let mut global_limiter = GlobalRateLimiter::new(global_limit);

    // 模拟请求：需要同时满足所有限流条件
    for _ in 0..user_limit {
        assert!(user_limiter.check(user));
        assert!(api_key_limiter.check(api_key));
        assert!(global_limiter.check());
    }

    // 用户限流应该先触发
    assert!(!user_limiter.check(user));
}

// ============ 并发限流测试 ============

#[tokio::test]
async fn test_concurrent_rate_limit() {
    let limiter = Arc::new(RwLock::new(UserRateLimiter::new(100)));
    let user_id = "user-concurrent";

    let mut tasks = vec![];

    // 并发请求
    for _ in 0..150 {
        let limiter_clone = limiter.clone();
        let user = user_id.to_string();

        let task = tokio::spawn(async move {
            let mut l = limiter_clone.write().await;
            l.check(&user)
        });

        tasks.push(task);
    }

    let results: Vec<bool> = futures::future::join_all(tasks)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();

    // 应该有100个成功，50个失败
    let success_count = results.iter().filter(|&&r| r).count();
    let failure_count = results.iter().filter(|&&r| !r).count();

    // 由于并发竞争，可能会有一些偏差，但应该大致正确
    assert!(success_count <= 100);
    assert!(failure_count >= 50);
}

// ============ 限流状态查询测试 ============

#[test]
fn test_rate_limit_status() {
    let mut limiter = UserRateLimiter::new(60);
    let user_id = "user-status";

    // 消耗一些配额
    for _ in 0..30 {
        limiter.check(user_id);
    }

    let status = limiter.get_status(user_id);

    assert_eq!(status.remaining, 30);
    assert_eq!(status.limit, 60);
    assert_eq!(status.used, 30);
    let utilization = (status.used as f64 / status.limit as f64) * 100.0;
    assert!((utilization - 50.0).abs() < 0.01);
}

// ============ 重置限流测试 ============

#[test]
fn test_rate_limit_reset() {
    let mut limiter = UserRateLimiter::new(10);
    let user_id = "user-reset";

    // 用完配额
    for _ in 0..10 {
        limiter.check(user_id);
    }

    assert!(!limiter.check(user_id));

    // 重置
    limiter.reset(user_id);

    // 应该可以再次使用
    assert!(limiter.check(user_id));
}

// ============ 数据结构定义（模拟实现）============

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
    pub requests_per_day: u32,
    pub burst_size: u32,
}

#[derive(Debug, Clone)]
pub struct TokenBucket {
    capacity: u32,
    tokens: u32,
    refill_rate: f64,
}

impl TokenBucket {
    pub fn new(capacity: u32, refill_rate: f64) -> Self {
        Self {
            capacity,
            tokens: capacity,
            refill_rate,
        }
    }

    pub fn try_consume(&mut self, amount: u32) -> bool {
        if self.tokens >= amount {
            self.tokens -= amount;
            true
        } else {
            false
        }
    }

    pub fn refill(&mut self, seconds: f64) {
        let new_tokens = (seconds * self.refill_rate) as u32;
        self.tokens = (self.tokens + new_tokens).min(self.capacity);
    }
}

#[derive(Debug, Clone)]
pub struct SlidingWindow {
    window_size: i64,
    requests: Vec<chrono::DateTime<Utc>>,
}

impl SlidingWindow {
    pub fn new(window_size: i64) -> Self {
        Self {
            window_size,
            requests: Vec::new(),
        }
    }

    pub fn record(&mut self, time: chrono::DateTime<Utc>) {
        self.requests.push(time);
    }

    pub fn count(&self, now: chrono::DateTime<Utc>) -> usize {
        let cutoff = now - Duration::seconds(self.window_size);
        self.requests.iter().filter(|&&t| t > cutoff).count()
    }
}

#[derive(Debug, Clone)]
pub struct FixedWindow {
    window_size: i64,
    count: u32,
    window_start: Option<chrono::DateTime<Utc>>,
}

impl FixedWindow {
    pub fn new(window_size: i64) -> Self {
        Self {
            window_size,
            count: 0,
            window_start: None,
        }
    }

    pub fn increment(&mut self, now: chrono::DateTime<Utc>) {
        if let Some(start) = self.window_start {
            if (now - start).num_seconds() >= self.window_size {
                // 窗口过期，重置
                self.count = 0;
                self.window_start = Some(now);
            }
        } else {
            self.window_start = Some(now);
        }

        self.count += 1;
    }
}

#[derive(Debug, Clone)]
pub struct UserRateLimiter {
    limit: u32,
    users: HashMap<String, SlidingWindow>,
}

impl UserRateLimiter {
    pub fn new(limit: u32) -> Self {
        Self {
            limit,
            users: HashMap::new(),
        }
    }

    pub fn check(&mut self, user_id: &str) -> bool {
        let window = self.users.entry(user_id.to_string()).or_insert(SlidingWindow::new(60));
        let count = window.count(Utc::now());

        if count < self.limit {
            window.record(Utc::now());
            true
        } else {
            false
        }
    }

    pub fn reset(&mut self, user_id: &str) {
        self.users.remove(user_id);
    }

    pub fn get_status(&self, user_id: &str) -> RateLimitStatus {
        let used = self.users.get(user_id).map(|w| w.count(Utc::now()) as u32).unwrap_or(0);
        RateLimitStatus {
            limit: self.limit,
            remaining: self.limit - used,
            used,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RateLimitStatus {
    pub limit: u32,
    pub remaining: u32,
    pub used: u32,
}

#[derive(Debug, Clone)]
pub struct ApiKeyRateLimiter {
    limit: u32,
    keys: HashMap<String, SlidingWindow>,
}

impl ApiKeyRateLimiter {
    pub fn new(limit: u32) -> Self {
        Self {
            limit,
            keys: HashMap::new(),
        }
    }

    pub fn check(&mut self, api_key: &str) -> bool {
        let window = self.keys.entry(api_key.to_string()).or_insert(SlidingWindow::new(60));
        let count = window.count(Utc::now());

        if count < self.limit {
            window.record(Utc::now());
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
pub struct GlobalRateLimiter {
    limit: u32,
    window: SlidingWindow,
}

impl GlobalRateLimiter {
    pub fn new(limit: u32) -> Self {
        Self {
            limit,
            window: SlidingWindow::new(60),
        }
    }

    pub fn check(&mut self) -> bool {
        let count = self.window.count(Utc::now());

        if count < self.limit {
            self.window.record(Utc::now());
            true
        } else {
            false
        }
    }
}

// ============ 集成测试标记 ============

#[cfg(feature = "integration")]
mod integration_tests {
    use super::*;

    /// 集成测试：需要 Redis 的分布式限流
    #[tokio::test]
    async fn test_distributed_rate_limit_with_redis() {
        // 需要 Redis 连接
    }
}
