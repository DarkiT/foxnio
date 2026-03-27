//! 数据库模块

pub mod pool;
pub mod redis;

pub use pool::{init_database, DatabaseConfig, DatabasePool};
pub use redis::{init_redis, RedisConfig, RedisPool};
