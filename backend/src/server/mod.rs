//! HTTP/2 服务器模块
//!
//! 提供 TLS 证书加载和服务器启动功能

use std::path::PathBuf;
use std::sync::Arc;

use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::ServerConfig as RustlsServerConfig;

// Re-export config types for convenience
pub use crate::config::{
    ClientAuthMode, Http2ClientConfig, Http2Config, ServerConfig, TlsConfig, TlsVersion,
};

/// 加载证书文件
pub fn load_certs(path: &PathBuf) -> Result<Vec<CertificateDer<'static>>, TlsError> {
    let cert_file = std::fs::File::open(path)
        .map_err(|e| TlsError::CertificateFileError(path.clone(), e.to_string()))?;
    let mut reader = std::io::BufReader::new(cert_file);

    rustls_pemfile::certs(&mut reader)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| TlsError::CertificateParseError(e.to_string()))
}

/// 加载私钥文件
pub fn load_private_key(path: &PathBuf) -> Result<PrivateKeyDer<'static>, TlsError> {
    let key_file = std::fs::File::open(path)
        .map_err(|e| TlsError::KeyFileError(path.clone(), e.to_string()))?;
    let mut reader = std::io::BufReader::new(key_file);

    // 尝试加载 PKCS#8 格式
    let keys = rustls_pemfile::pkcs8_private_keys(&mut reader)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| TlsError::KeyParseError(e.to_string()))?;

    if let Some(key) = keys.into_iter().next() {
        return Ok(PrivateKeyDer::Pkcs8(key));
    }

    // 尝试加载 RSA 格式
    let key_file = std::fs::File::open(path)
        .map_err(|e| TlsError::KeyFileError(path.clone(), e.to_string()))?;
    let mut reader = std::io::BufReader::new(key_file);

    let keys = rustls_pemfile::rsa_private_keys(&mut reader)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| TlsError::KeyParseError(e.to_string()))?;

    if let Some(key) = keys.into_iter().next() {
        return Ok(PrivateKeyDer::Pkcs1(key));
    }

    Err(TlsError::KeyNotFound)
}

/// 构建 TLS 服务器配置
pub fn build_tls_server_config(tls_config: &TlsConfig) -> Result<Arc<RustlsServerConfig>, TlsError> {
    let cert = load_certs(&tls_config.cert_path)?;
    let key = load_private_key(&tls_config.key_path)?;

    let config = RustlsServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert, key)
        .map_err(|e| TlsError::CertificateError(e.to_string()))?;

    Ok(Arc::new(config))
}

/// TLS 错误类型
#[derive(Debug, thiserror::Error)]
pub enum TlsError {
    #[error("Certificate file error {0}: {1}")]
    CertificateFileError(PathBuf, String),

    #[error("Certificate parse error: {0}")]
    CertificateParseError(String),

    #[error("Key file error {0}: {1}")]
    KeyFileError(PathBuf, String),

    #[error("Key parse error: {0}")]
    KeyParseError(String),

    #[error("Certificate error: {0}")]
    CertificateError(String),

    #[error("No private key found")]
    KeyNotFound,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http2_config_defaults() {
        let config = Http2Config::default();
        assert!(config.enabled);
        assert_eq!(config.max_concurrent_streams, 200);
        assert!(config.adaptive_window);
    }

    #[test]
    fn test_http2_client_config_defaults() {
        let config = Http2ClientConfig::default();
        assert!(config.enabled);
        assert_eq!(config.pool_size, 16);
        assert!(config.auto_negotiate);
    }

    #[test]
    fn test_tls_config_deserialization() {
        let yaml = r#"
enabled: true
cert_path: /path/to/cert.pem
key_path: /path/to/key.pem
min_version: v13
"#;
        let config: TlsConfig = serde_yaml::from_str(yaml).unwrap();
        assert!(config.enabled);
        assert_eq!(config.min_version, TlsVersion::V13);
    }
}
