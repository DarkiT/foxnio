//! 应用状态

use crate::config::Config;
use crate::db::RedisPool;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

/// 应用状态
#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub redis: Arc<RedisPool>,
    pub config: Arc<Config>,
}

impl AppState {
    pub fn new(db: DatabaseConnection, redis: RedisPool, config: Config) -> Self {
        Self {
            db,
            redis: Arc::new(redis),
            config: Arc::new(config),
        }
    }
}

/// 共享状态
pub type SharedState = Arc<AppState>;
