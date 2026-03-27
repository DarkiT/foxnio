//! FoxNIO - AI API Gateway
//!
//! 订阅配额分发管理平台

pub mod cache;
pub mod config;
pub mod db;
pub mod gateway;
pub mod handler;
pub mod health;
pub mod model;
pub mod server;
pub mod service;
pub mod utils;

pub use config::Config;
pub use gateway::Gateway;
pub use health::{HealthChecker, HealthStatus, HealthCheck};

// Re-export HTTP/2 configuration types
pub use config::{
    Http2Config, Http2ClientConfig, ServerConfig, TlsConfig, TlsVersion, ClientAuthMode,
};
pub use server::{TlsError, load_certs, load_private_key, build_tls_server_config};
