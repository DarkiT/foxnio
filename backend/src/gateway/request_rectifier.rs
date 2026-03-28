//! 请求整流器 - API Key 签名和 Wire Casing 支持
//!
//! P1 功能：为 API Key 账号提供签名验证和请求包装支持

#![allow(dead_code)]

use anyhow::{bail, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use reqwest::Request;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// 请求签名器
///
/// 为 API Key 账号生成和验证请求签名
#[derive(Debug, Clone)]
pub struct RequestSigner {
    /// API Key
    api_key: String,
    /// API Secret
    api_secret: String,
}

impl RequestSigner {
    /// 创建新的签名器
    pub fn new(api_key: String, api_secret: String) -> Self {
        Self {
            api_key,
            api_secret,
        }
    }

    /// 为请求生成签名
    ///
    /// 签名算法：
    /// 1. 提取请求方法、路径、时间戳
    /// 2. 拼接字符串: `{method}\n{path}\n{timestamp}\n{body_hash}`
    /// 3. 使用 HMAC-SHA256 生成签名
    /// 4. Base64 编码签名
    pub fn sign(&self, request: &mut Request, timestamp: i64) -> Result<String> {
        let method = request.method().as_str();
        let path = request.url().path();
        let body_hash = self.hash_body(request)?;

        // 构建签名字符串
        let sign_string = format!("{}\n{}\n{}\n{}", method, path, timestamp, body_hash);

        // HMAC-SHA256 签名
        let signature = self.hmac_sha256(&sign_string)?;
        let signature_b64 = BASE64.encode(&signature);

        // 添加签名头
        let headers = request.headers_mut();
        headers.insert("X-API-Key", self.api_key.parse()?);
        headers.insert("X-Timestamp", timestamp.to_string().parse()?);
        headers.insert("X-Signature", signature_b64.parse()?);

        Ok(signature_b64)
    }

    /// 验证请求签名
    pub fn verify(&self, request: &Request, max_age_seconds: i64) -> Result<bool> {
        let headers = request.headers();

        // 提取签名头
        let api_key = headers
            .get("X-API-Key")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        let timestamp_str = headers
            .get("X-Timestamp")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("0");

        let signature_b64 = headers
            .get("X-Signature")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        // 验证 API Key
        if api_key != self.api_key {
            bail!("Invalid API key");
        }

        // 解析时间戳
        let timestamp: i64 = timestamp_str.parse()?;
        let now = chrono::Utc::now().timestamp();

        // 验证时间戳有效性
        if (now - timestamp).abs() > max_age_seconds {
            bail!("Request expired");
        }

        // 计算预期签名
        let method = request.method().as_str();
        let path = request.url().path();
        let body_hash = self.hash_body(request)?;
        let sign_string = format!("{}\n{}\n{}\n{}", method, path, timestamp, body_hash);

        let expected_signature = self.hmac_sha256(&sign_string)?;
        let expected_signature_b64 = BASE64.encode(&expected_signature);

        // 验证签名
        if signature_b64 != expected_signature_b64 {
            bail!("Invalid signature");
        }

        Ok(true)
    }

    /// 计算请求体的哈希
    fn hash_body(&self, request: &Request) -> Result<String> {
        let body_bytes = if let Some(body) = request.body() {
            body.as_bytes().unwrap_or(&[]).to_vec()
        } else {
            vec![]
        };
        let mut hasher = Sha256::new();
        hasher.update(&body_bytes);
        let hash = hasher.finalize();
        Ok(BASE64.encode(hash))
    }

    /// HMAC-SHA256 签名
    fn hmac_sha256(&self, data: &str) -> Result<Vec<u8>> {
        use hmac::{Hmac, Mac};
        type HmacSha256 = Hmac<Sha256>;

        let mut mac = HmacSha256::new_from_slice(self.api_secret.as_bytes())?;
        mac.update(data.as_bytes());
        Ok(mac.finalize().into_bytes().to_vec())
    }
}

/// Wire Casing 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireCasingConfig {
    /// 是否保持原始格式
    pub preserve_original_format: bool,
    /// 是否移除敏感头
    pub remove_sensitive_headers: bool,
    /// 是否添加追踪头
    pub add_trace_headers: bool,
}

impl Default for WireCasingConfig {
    fn default() -> Self {
        Self {
            preserve_original_format: true,
            remove_sensitive_headers: true,
            add_trace_headers: true,
        }
    }
}

/// Wire Casing 处理器
///
/// 保持请求的原始格式，同时提供安全性和可追踪性
#[derive(Debug, Clone)]
pub struct WireCasing {
    config: WireCasingConfig,
}

impl WireCasing {
    /// 创建新的 Wire Casing 处理器
    pub fn new(config: WireCasingConfig) -> Self {
        Self { config }
    }

    /// 创建默认配置的处理器
    pub fn default_casing() -> Self {
        Self::new(WireCasingConfig::default())
    }

    /// 包装请求
    ///
    /// 1. 保存原始请求格式
    /// 2. 移除敏感头（如 Authorization）
    /// 3. 添加追踪头
    pub fn wrap(&self, request: &mut Request, trace_id: &str) -> Result<()> {
        if self.config.add_trace_headers {
            let headers = request.headers_mut();
            headers.insert("X-Trace-ID", trace_id.parse()?);
            headers.insert("X-Forwarded-For", "foxnio".parse()?);
        }

        if self.config.remove_sensitive_headers {
            let headers = request.headers_mut();
            headers.remove("Authorization");
            headers.remove("Cookie");
        }

        Ok(())
    }

    /// 解包请求
    ///
    /// 恢复原始请求格式（如果需要）
    pub fn unwrap(&self, _request: &mut Request) -> Result<()> {
        // 目前不需要做任何事情，因为我们保持了原始格式
        Ok(())
    }
}

/// 转发行为配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForwardBehavior {
    /// 是否启用重试
    pub enable_retry: bool,
    /// 最大重试次数
    pub max_retries: u32,
    /// 重试延迟（毫秒）
    pub retry_delay_ms: u64,
    /// 是否启用超时
    pub enable_timeout: bool,
    /// 超时时间（秒）
    pub timeout_seconds: u64,
    /// 是否启用熔断
    pub enable_circuit_breaker: bool,
    /// 熔断阈值（失败次数）
    pub circuit_breaker_threshold: u32,
    /// 熔断恢复时间（秒）
    pub circuit_breaker_recovery_seconds: u64,
}

impl Default for ForwardBehavior {
    fn default() -> Self {
        Self {
            enable_retry: true,
            max_retries: 3,
            retry_delay_ms: 1000,
            enable_timeout: true,
            timeout_seconds: 30,
            enable_circuit_breaker: true,
            circuit_breaker_threshold: 5,
            circuit_breaker_recovery_seconds: 60,
        }
    }
}

/// 请求整流器
///
/// 统一处理请求签名、Wire Casing 和转发行为
#[derive(Debug, Clone)]
pub struct RequestRectifier {
    /// 签名器（可选）
    signer: Option<RequestSigner>,
    /// Wire Casing 处理器
    wire_casing: WireCasing,
    /// 转发行为配置
    forward_behavior: ForwardBehavior,
}

impl RequestRectifier {
    /// 创建新的请求整流器
    pub fn new(
        signer: Option<RequestSigner>,
        wire_casing: WireCasing,
        forward_behavior: ForwardBehavior,
    ) -> Self {
        Self {
            signer,
            wire_casing,
            forward_behavior,
        }
    }

    /// 创建默认配置的整流器
    pub fn default_rectifier() -> Self {
        Self::new(
            None,
            WireCasing::default_casing(),
            ForwardBehavior::default(),
        )
    }

    /// 处理请求
    ///
    /// 1. 签名（如果配置）
    /// 2. Wire Casing 处理
    /// 3. 添加追踪信息
    pub fn process(&self, request: &mut Request, trace_id: &str) -> Result<()> {
        // 签名
        if let Some(signer) = &self.signer {
            let timestamp = chrono::Utc::now().timestamp();
            signer.sign(request, timestamp)?;
        }

        // Wire Casing
        self.wire_casing.wrap(request, trace_id)?;

        Ok(())
    }

    /// 获取转发行为配置
    pub fn get_forward_behavior(&self) -> &ForwardBehavior {
        &self.forward_behavior
    }

    /// 是否需要重试
    pub fn should_retry(&self, attempt: u32, status_code: u16) -> bool {
        self.forward_behavior.enable_retry
            && attempt < self.forward_behavior.max_retries
            && self.is_retryable_status(status_code)
    }

    /// 判断状态码是否可重试
    fn is_retryable_status(&self, status_code: u16) -> bool {
        matches!(status_code, 429 | 500 | 502 | 503 | 504)
    }

    /// 获取重试延迟（毫秒）
    pub fn get_retry_delay(&self, attempt: u32) -> u64 {
        // 指数退避
        self.forward_behavior.retry_delay_ms * (2u64.pow(attempt))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::Method;

    #[test]
    fn test_request_signer_creation() {
        let signer = RequestSigner::new("test-key".to_string(), "test-secret".to_string());
        assert_eq!(signer.api_key, "test-key");
        assert_eq!(signer.api_secret, "test-secret");
    }

    #[test]
    fn test_wire_casing_config_default() {
        let config = WireCasingConfig::default();
        assert!(config.preserve_original_format);
        assert!(config.remove_sensitive_headers);
        assert!(config.add_trace_headers);
    }

    #[test]
    fn test_wire_casing_creation() {
        let casing = WireCasing::default_casing();
        assert!(casing.config.preserve_original_format);
    }

    #[test]
    fn test_forward_behavior_default() {
        let behavior = ForwardBehavior::default();
        assert!(behavior.enable_retry);
        assert_eq!(behavior.max_retries, 3);
        assert_eq!(behavior.retry_delay_ms, 1000);
        assert!(behavior.enable_timeout);
        assert_eq!(behavior.timeout_seconds, 30);
    }

    #[test]
    fn test_request_rectifier_creation() {
        let rectifier = RequestRectifier::default_rectifier();
        assert!(rectifier.signer.is_none());
        assert!(rectifier.forward_behavior.enable_retry);
    }

    #[test]
    fn test_should_retry() {
        let rectifier = RequestRectifier::default_rectifier();

        // 可重试的状态码
        assert!(rectifier.should_retry(0, 429));
        assert!(rectifier.should_retry(0, 500));
        assert!(rectifier.should_retry(0, 502));
        assert!(rectifier.should_retry(0, 503));
        assert!(rectifier.should_retry(0, 504));

        // 不可重试的状态码
        assert!(!rectifier.should_retry(0, 200));
        assert!(!rectifier.should_retry(0, 400));
        assert!(!rectifier.should_retry(0, 401));

        // 超过最大重试次数
        assert!(!rectifier.should_retry(3, 429));
    }

    #[test]
    fn test_get_retry_delay() {
        let rectifier = RequestRectifier::default_rectifier();

        assert_eq!(rectifier.get_retry_delay(0), 1000);
        assert_eq!(rectifier.get_retry_delay(1), 2000);
        assert_eq!(rectifier.get_retry_delay(2), 4000);
    }

    #[test]
    fn test_request_signer_hash_body() {
        let signer = RequestSigner::new("key".to_string(), "secret".to_string());
        let request = reqwest::Request::new(Method::GET, "https://example.com".parse().unwrap());
        let hash = signer.hash_body(&request).unwrap();
        // 空body的hash应该是固定的
        assert!(!hash.is_empty());
    }
}
