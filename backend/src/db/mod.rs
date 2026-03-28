//! 数据库模块

pub mod pool;
pub mod redis;

pub use pool::init_database;
pub use redis::{init_redis, RedisPool};
