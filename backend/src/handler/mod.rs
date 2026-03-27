//! HTTP 处理器模块

pub mod auth;
pub mod admin;
pub mod health;

pub use health::{
    health_simple,
    health_live,
    health_ready,
    health_detailed,
    health_resources,
    health_database,
    health_redis,
    app_info,
    metrics,
};
