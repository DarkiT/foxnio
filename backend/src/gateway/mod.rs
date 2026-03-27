//! 网关核心模块

mod test;

pub mod claude;
pub mod failover;
pub mod handler;
pub mod middleware;
pub mod models;
pub mod proxy;
pub mod routes;
pub mod scheduler;
pub mod stream;

pub use claude::{get_beta_header, ClaudeHeaders, TLSFingerprint};
pub use failover::FailoverManager;
pub use handler::GatewayHandler;
pub use middleware::*;
pub use proxy::*;
pub use routes::build_app;
pub use scheduler::{
    AccountInfo, AccountMetrics, AccountStatus, BudgetSummary, CostConfig, CostOptimizer,
    ScheduleContext, ScheduleResult, ScheduleStrategy, Scheduler, SchedulerConfig,
    SchedulerMetrics, SchedulerStats,
};
pub use stream::*;

use crate::config::Config;
use redis::aio::ConnectionManager;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

pub struct AppState {
    pub db: DatabaseConnection,
    pub redis: ConnectionManager,
    pub config: Config,
}

pub type SharedState = Arc<AppState>;
