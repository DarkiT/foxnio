//! OAuth 认证框架
//!
//! 提供 OAuth 2.0 授权流程的核心抽象和实现。
//!
//! # 模块结构
//! - `mod` - 核心框架和 trait 定义
//! - `claude` - Claude (Anthropic) OAuth 实现
//! - `gemini` - Google Gemini OAuth 实现
//! - `antigravity` - Antigravity (Google Cloud Code) OAuth 实现
//!
//! # 核心功能
//! - OAuth 2.0 授权码流程（Authorization Code Flow）
//! - PKCE 支持（Proof Key for Code Exchange）
//! - Token 自动刷新机制
//! - Token 加密存储
//!
//! 注意：部分功能正在开发中，暂未完全使用

#![allow(dead_code)]

pub mod antigravity;
pub mod antigravity_ext;
pub mod claude;
pub mod gemini;
pub mod linuxdo;

use anyhow::Result;
use base64::Engine;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use uuid::Uuid;

/// OAuth 提供商类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OAuthProviderType {
    Claude,
    OpenAI,
    Gemini,
    Antigravity,
    LinuxDo,
}

impl OAuthProviderType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::OpenAI => "openai",
            Self::Gemini => "gemini",
            Self::Antigravity => "antigravity",
            Self::LinuxDo => "linuxdo",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "claude" | "anthropic" => Self::Claude,
            "openai" => Self::OpenAI,
            "gemini" => Self::Gemini,
            "antigravity" => Self::Antigravity,
            "linuxdo" | "linux.do" => Self::LinuxDo,
            _ => Self::Claude, // 默认
        }
    }
}

impl std::fmt::Display for OAuthProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// 账号类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountType {
    /// API Key 认证
    ApiKey,
    /// OAuth 认证
    OAuth,
    /// Setup Token（一次性设置）
    SetupToken,
    /// 上游代理
    Upstream,
    /// AWS Bedrock
    Bedrock,
}

impl AccountType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ApiKey => "api_key",
            Self::OAuth => "oauth",
            Self::SetupToken => "setup_token",
            Self::Upstream => "upstream",
            Self::Bedrock => "bedrock",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "api_key" => Self::ApiKey,
            "oauth" => Self::OAuth,
            "setup_token" => Self::SetupToken,
            "upstream" => Self::Upstream,
            "bedrock" => Self::Bedrock,
            _ => Self::ApiKey,
        }
    }
}

impl std::fmt::Display for AccountType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// OAuth Token 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthToken {
    /// 访问令牌
    pub access_token: String,
    /// 刷新令牌
    pub refresh_token: Option<String>,
    /// 过期时间
    pub expires_at: DateTime<Utc>,
    /// Token 类型（通常为 "Bearer"）
    pub token_type: String,
    /// OAuth scope
    pub scope: Option<String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 额外元数据
    #[serde(flatten)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl OAuthToken {
    /// 创建新的 OAuth Token
    pub fn new(access_token: String, expires_in: i64, token_type: String) -> Self {
        let now = Utc::now();
        Self {
            access_token,
            refresh_token: None,
            expires_at: now + chrono::Duration::seconds(expires_in),
            token_type,
            scope: None,
            created_at: now,
            metadata: HashMap::new(),
        }
    }

    /// 设置 refresh_token
    pub fn with_refresh_token(mut self, refresh_token: String) -> Self {
        self.refresh_token = Some(refresh_token);
        self
    }

    /// 设置 scope
    pub fn with_scope(mut self, scope: String) -> Self {
        self.scope = Some(scope);
        self
    }

    /// 添加元数据
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// 检查 token 是否已过期
    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at
    }

    /// 检查是否需要刷新（提前 5 分钟刷新）
    pub fn needs_refresh(&self) -> bool {
        Utc::now() >= self.expires_at - chrono::Duration::minutes(5)
    }

    /// 是否有 refresh_token
    pub fn has_refresh_token(&self) -> bool {
        self.refresh_token.is_some()
    }

    /// 计算剩余有效时间（秒）
    pub fn remaining_seconds(&self) -> i64 {
        let now = Utc::now();
        if now >= self.expires_at {
            0
        } else {
            (self.expires_at - now).num_seconds()
        }
    }
}

/// OAuth 授权 URL 生成结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUrlResult {
    /// 授权 URL
    pub auth_url: String,
    /// 会话 ID
    pub session_id: String,
}

/// OAuth Session（用于存储授权流程中间状态）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthSession {
    /// 会话 ID
    pub session_id: String,
    /// Provider 类型
    pub provider: OAuthProviderType,
    /// State 参数（CSRF 防护）
    pub state: String,
    /// PKCE code_verifier
    pub code_verifier: String,
    /// 重定向 URI
    pub redirect_uri: String,
    /// OAuth scope
    pub scope: String,
    /// 代理 URL（可选）
    pub proxy_url: Option<String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 额外参数
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl OAuthSession {
    /// 创建新的 OAuth Session
    pub fn new(
        provider: OAuthProviderType,
        state: String,
        code_verifier: String,
        redirect_uri: String,
        scope: String,
    ) -> Self {
        Self {
            session_id: Uuid::new_v4().to_string(),
            provider,
            state,
            code_verifier,
            redirect_uri,
            scope,
            proxy_url: None,
            created_at: Utc::now(),
            extra: HashMap::new(),
        }
    }

    /// 设置代理 URL
    pub fn with_proxy_url(mut self, proxy_url: String) -> Self {
        self.proxy_url = Some(proxy_url);
        self
    }

    /// 添加额外参数
    pub fn with_extra(mut self, key: String, value: serde_json::Value) -> Self {
        self.extra.insert(key, value);
        self
    }

    /// 检查 session 是否过期（默认 30 分钟）
    pub fn is_expired(&self) -> bool {
        let ttl = chrono::Duration::minutes(30);
        Utc::now() - self.created_at > ttl
    }
}

/// OAuth Session 存储接口
#[async_trait::async_trait]
pub trait OAuthSessionStore: Send + Sync {
    /// 存储 session
    async fn set(&self, session: OAuthSession) -> Result<()>;

    /// 获取 session
    async fn get(&self, session_id: &str) -> Result<Option<OAuthSession>>;

    /// 删除 session
    async fn delete(&self, session_id: &str) -> Result<()>;
}

/// 内存中的 OAuth Session 存储
#[derive(Debug, Default)]
pub struct InMemorySessionStore {
    sessions: Arc<RwLock<HashMap<String, OAuthSession>>>,
}

impl InMemorySessionStore {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 清理过期 session
    pub async fn cleanup_expired(&self) {
        let mut sessions = self.sessions.write().await;
        sessions.retain(|_, session| !session.is_expired());
    }
}

#[async_trait::async_trait]
impl OAuthSessionStore for InMemorySessionStore {
    async fn set(&self, session: OAuthSession) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        sessions.insert(session.session_id.clone(), session);
        Ok(())
    }

    async fn get(&self, session_id: &str) -> Result<Option<OAuthSession>> {
        let sessions = self.sessions.read().await;
        match sessions.get(session_id) {
            Some(session) if !session.is_expired() => Ok(Some(session.clone())),
            _ => Ok(None),
        }
    }

    async fn delete(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(session_id);
        Ok(())
    }
}

/// OAuth Provider trait - 定义 OAuth 提供商的核心接口
#[async_trait::async_trait]
pub trait OAuthProvider: Send + Sync {
    /// 获取 Provider 类型
    fn provider_type(&self) -> OAuthProviderType;

    /// 生成授权 URL
    async fn generate_auth_url(
        &self,
        redirect_uri: &str,
        scope: Option<&str>,
        state: Option<&str>,
        code_challenge: Option<&str>,
    ) -> Result<AuthUrlResult>;

    /// 用授权码换取 Token
    async fn exchange_code(
        &self,
        code: &str,
        code_verifier: &str,
        redirect_uri: &str,
    ) -> Result<OAuthToken>;

    /// 刷新 Token
    async fn refresh_token(&self, refresh_token: &str) -> Result<OAuthToken>;

    /// 验证 Token 是否有效
    async fn validate_token(&self, access_token: &str) -> Result<bool>;

    /// 撤销 Token
    async fn revoke_token(&self, token: &str) -> Result<()> {
        let _ = token;
        // 默认实现：不支持撤销
        Ok(())
    }
}

/// OAuth 配置
#[derive(Debug, Clone)]
pub struct OAuthConfig {
    /// Client ID
    pub client_id: String,
    /// Client Secret（可选，用于需要 secret 的流程）
    pub client_secret: Option<String>,
    /// 授权端点 URL
    pub authorize_url: String,
    /// Token 端点 URL
    pub token_url: String,
    /// 默认 scope
    pub default_scope: String,
    /// 重定向 URI
    pub redirect_uri: Option<String>,
    /// HTTP 客户端超时（秒）
    pub timeout_seconds: u64,
    /// 代理 URL
    pub proxy_url: Option<String>,
}

impl OAuthConfig {
    /// 创建新的 OAuth 配置
    pub fn new(
        client_id: String,
        authorize_url: String,
        token_url: String,
        default_scope: String,
    ) -> Self {
        Self {
            client_id,
            client_secret: None,
            authorize_url,
            token_url,
            default_scope,
            redirect_uri: None,
            timeout_seconds: 30,
            proxy_url: None,
        }
    }

    /// 设置 client_secret
    pub fn with_client_secret(mut self, secret: String) -> Self {
        self.client_secret = Some(secret);
        self
    }

    /// 设置重定向 URI
    pub fn with_redirect_uri(mut self, uri: String) -> Self {
        self.redirect_uri = Some(uri);
        self
    }

    /// 设置代理
    pub fn with_proxy(mut self, proxy_url: String) -> Self {
        self.proxy_url = Some(proxy_url);
        self
    }

    /// 设置超时
    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = seconds;
        self
    }
}

/// OAuth 服务工厂
pub struct OAuthServiceFactory {
    session_store: Arc<dyn OAuthSessionStore>,
    http_client: reqwest::Client,
}

impl OAuthServiceFactory {
    /// 创建新的 OAuth 服务工厂
    pub fn new(session_store: Arc<dyn OAuthSessionStore>) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            session_store,
            http_client,
        }
    }

    /// 获取 session 存储
    pub fn session_store(&self) -> Arc<dyn OAuthSessionStore> {
        self.session_store.clone()
    }

    /// 获取 HTTP 客户端
    pub fn http_client(&self) -> &reqwest::Client {
        &self.http_client
    }

    /// 创建 Claude OAuth Provider
    pub fn create_claude_provider(&self) -> claude::ClaudeOAuthProvider {
        claude::ClaudeOAuthProvider::new(self.session_store.clone(), self.http_client.clone())
    }

    /// 创建 Gemini OAuth Provider
    pub fn create_gemini_provider(&self, config: OAuthConfig) -> gemini::GeminiOAuthProvider {
        gemini::GeminiOAuthProvider::new(
            config,
            self.session_store.clone(),
            self.http_client.clone(),
        )
    }

    /// 创建 Antigravity OAuth Provider
    pub fn create_antigravity_provider(
        &self,
        config: OAuthConfig,
    ) -> antigravity::AntigravityOAuthProvider {
        antigravity::AntigravityOAuthProvider::new(
            config,
            self.session_store.clone(),
            self.http_client.clone(),
        )
    }
}

// ============================================================================
// PKCE 工具函数
// ============================================================================

/// 生成随机字节
pub fn generate_random_bytes(n: usize) -> Result<Vec<u8>> {
    use rand::RngCore;
    let mut bytes = vec![0u8; n];
    rand::thread_rng().fill_bytes(&mut bytes);
    Ok(bytes)
}

/// 生成 OAuth state（用于 CSRF 防护）
pub fn generate_state() -> Result<String> {
    let bytes = generate_random_bytes(32)?;
    Ok(base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&bytes))
}

/// 生成 session ID
pub fn generate_session_id() -> Result<String> {
    Ok(Uuid::new_v4().to_string())
}

/// 生成 PKCE code_verifier
///
/// RFC 7636 compliant: 43-128 characters from [A-Z][a-z][0-9]-._~
pub fn generate_code_verifier() -> Result<String> {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";
    const TARGET_LEN: usize = 64;

    let mut result = String::with_capacity(TARGET_LEN);
    let bytes = generate_random_bytes(TARGET_LEN * 2)?;

    for byte in bytes {
        let idx = (byte as usize) % CHARSET.len();
        result.push(CHARSET[idx] as char);
        if result.len() >= TARGET_LEN {
            break;
        }
    }

    Ok(result)
}

/// 生成 PKCE code_challenge（S256 方法）
///
/// SHA256(code_verifier) -> base64url without padding
pub fn generate_code_challenge(verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let hash = hasher.finalize();

    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hash)
}

/// Base64 URL 编码（无填充）
pub fn base64_url_encode(data: &[u8]) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(data)
}

/// Base64 URL 解码
pub fn base64_url_decode(data: &str) -> Result<Vec<u8>> {
    base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(data)
        .map_err(|e| anyhow::anyhow!("Base64 decode error: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth_provider_type() {
        assert_eq!(OAuthProviderType::Claude.as_str(), "claude");
        assert_eq!(
            OAuthProviderType::parse("anthropic"),
            OAuthProviderType::Claude
        );
        assert_eq!(
            OAuthProviderType::parse("OPENAI"),
            OAuthProviderType::OpenAI
        );
    }

    #[test]
    fn test_account_type() {
        assert_eq!(AccountType::ApiKey.as_str(), "api_key");
        assert_eq!(AccountType::parse("oauth"), AccountType::OAuth);
    }

    #[test]
    fn test_oauth_token() {
        let token = OAuthToken::new("test_access_token".to_string(), 3600, "Bearer".to_string());

        assert!(!token.is_expired());
        assert!(token.remaining_seconds() > 3500);
        assert!(!token.has_refresh_token());
    }

    #[test]
    fn test_oauth_token_with_refresh() {
        let token = OAuthToken::new("test_access_token".to_string(), 3600, "Bearer".to_string())
            .with_refresh_token("test_refresh_token".to_string());

        assert!(token.has_refresh_token());
        assert!(token.refresh_token.is_some());
    }

    #[test]
    fn test_oauth_session() {
        let session = OAuthSession::new(
            OAuthProviderType::Claude,
            "state123".to_string(),
            "verifier123".to_string(),
            "http://localhost/callback".to_string(),
            "user:profile".to_string(),
        );

        assert!(!session.is_expired());
        assert_eq!(session.provider, OAuthProviderType::Claude);
    }

    #[test]
    fn test_generate_state() {
        let state = generate_state().unwrap();
        assert!(!state.is_empty());
        assert_eq!(state.len(), 43); // 32 bytes -> 43 chars in base64url
    }

    #[test]
    fn test_generate_code_verifier() {
        let verifier = generate_code_verifier().unwrap();
        assert!(verifier.len() >= 43);
        assert!(verifier.len() <= 128);
    }

    #[test]
    fn test_generate_code_challenge() {
        let verifier = "test_verifier_string";
        let challenge = generate_code_challenge(verifier);

        // S256 challenge should be 43 chars (SHA256 -> 32 bytes -> 43 chars base64url)
        assert_eq!(challenge.len(), 43);

        // Same verifier should produce same challenge
        let challenge2 = generate_code_challenge(verifier);
        assert_eq!(challenge, challenge2);
    }

    #[tokio::test]
    async fn test_in_memory_session_store() {
        let store = InMemorySessionStore::new();

        let session = OAuthSession::new(
            OAuthProviderType::Claude,
            "state123".to_string(),
            "verifier123".to_string(),
            "http://localhost/callback".to_string(),
            "user:profile".to_string(),
        );

        let session_id = session.session_id.clone();

        // Set
        store.set(session.clone()).await.unwrap();

        // Get
        let retrieved = store.get(&session_id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().state, "state123");

        // Delete
        store.delete(&session_id).await.unwrap();
        let deleted = store.get(&session_id).await.unwrap();
        assert!(deleted.is_none());
    }

    #[test]
    fn test_oauth_config() {
        let config = OAuthConfig::new(
            "client_id".to_string(),
            "https://example.com/authorize".to_string(),
            "https://example.com/token".to_string(),
            "openid profile".to_string(),
        )
        .with_client_secret("secret".to_string())
        .with_redirect_uri("http://localhost/callback".to_string());

        assert_eq!(config.client_id, "client_id");
        assert!(config.client_secret.is_some());
        assert!(config.redirect_uri.is_some());
    }
}
