//! 指标 HTTP 处理器
//!
//! 提供 Prometheus 格式的指标导出和 JSON 格式的管理接口。

use axum::{
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::metrics::{self, get_business_metrics, BusinessMetricsSummary, MetricsSummary};

/// Prometheus 指标端点
/// GET /metrics
pub async fn prometheus_metrics() -> Response {
    let output = metrics::gather_metrics();

    (
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            "text/plain; version=0.0.4; charset=utf-8",
        )],
        output,
    )
        .into_response()
}

/// JSON 格式的指标摘要
/// GET /api/v1/admin/metrics
pub async fn json_metrics() -> Response {
    let prometheus_summary = metrics::get_metrics_summary();
    let business_summary = get_business_metrics().get_summary();

    let response = MetricsResponse {
        timestamp: Utc::now(),
        prometheus: prometheus_summary,
        business: business_summary,
    };

    (StatusCode::OK, Json(response)).into_response()
}

/// 详细指标端点（包含分项统计）
/// GET /api/v1/admin/metrics/detail
pub async fn detailed_metrics() -> Response {
    let business_metrics = get_business_metrics();

    let response = DetailedMetricsResponse {
        timestamp: Utc::now(),
        summary: business_metrics.get_summary(),
        models: business_metrics.get_all_model_stats(),
        providers: business_metrics.get_all_provider_stats(),
        today_cost: business_metrics.get_today_cost(),
        total_cost: business_metrics.get_total_cost(),
    };

    (StatusCode::OK, Json(response)).into_response()
}

/// 健康检查指标端点
/// GET /api/v1/admin/metrics/health
pub async fn metrics_health() -> Response {
    let health = MetricsHealth {
        status: "healthy".to_string(),
        timestamp: Utc::now(),
        metrics_collected: true,
        prometheus_ready: true,
    };

    (StatusCode::OK, Json(health)).into_response()
}

/// 实时指标端点（用于实时监控面板）
/// GET /api/v1/admin/metrics/realtime
pub async fn realtime_metrics() -> Response {
    let business_metrics = get_business_metrics();

    let realtime = RealtimeMetrics {
        timestamp: Utc::now(),
        requests_total: metrics::REQUESTS_TOTAL.get(),
        active_connections: metrics::ACTIVE_CONNECTIONS.get(),
        websocket_connections: metrics::WEBSOCKET_CONNECTIONS.get(),
        cache_hits: metrics::CACHE_HITS.get(),
        cache_misses: metrics::CACHE_MISSES.get(),
        cache_hit_rate: metrics::CacheMetrics::hit_rate(),
        today_requests: business_metrics.get_today_cost().requests,
        today_cost: business_metrics.get_today_cost().cost,
    };

    (StatusCode::OK, Json(realtime)).into_response()
}

/// 成本统计端点
/// GET /api/v1/admin/metrics/cost
pub async fn cost_metrics() -> Response {
    let business_metrics = get_business_metrics();

    let (total_input, total_output) = business_metrics.get_total_tokens();

    let cost = CostMetrics {
        timestamp: Utc::now(),
        total_cost: business_metrics.get_total_cost(),
        today_cost: business_metrics.get_today_cost(),
        total_tokens_input: total_input,
        total_tokens_output: total_output,
        by_model: business_metrics.get_all_model_stats(),
    };

    (StatusCode::OK, Json(cost)).into_response()
}

/// Token 使用量端点
/// GET /api/v1/admin/metrics/tokens
pub async fn token_metrics() -> Response {
    let business_metrics = get_business_metrics();

    let (total_input, total_output) = business_metrics.get_total_tokens();

    let tokens = TokenMetrics {
        timestamp: Utc::now(),
        total_input,
        total_output,
        by_model: business_metrics
            .get_all_model_stats()
            .into_iter()
            .map(|m| ModelTokens {
                model: m.name,
                input: m.tokens_input,
                output: m.tokens_output,
            })
            .collect(),
        by_user: business_metrics
            .get_user_stats("")
            .map(|_| vec![])
            .unwrap_or_default(),
    };

    (StatusCode::OK, Json(tokens)).into_response()
}

/// 账号使用率端点
/// GET /api/v1/admin/metrics/accounts
pub async fn account_metrics() -> Response {
    let business_metrics = get_business_metrics();

    let accounts = AccountMetrics {
        timestamp: Utc::now(),
        usage_ratio: business_metrics.calculate_account_usage_ratio(),
        providers: business_metrics
            .get_all_provider_stats()
            .into_iter()
            .map(|p| ProviderAccountInfo {
                provider: p.name,
                active_accounts: p.active_accounts,
                total_accounts: p.total_accounts,
                error_rate: p.error_rate,
                avg_latency_ms: p.avg_latency_ms,
            })
            .collect(),
    };

    (StatusCode::OK, Json(accounts)).into_response()
}

// ============================================================================
// 响应结构体
// ============================================================================

/// 指标响应
#[derive(Debug, Serialize, Deserialize)]
pub struct MetricsResponse {
    pub timestamp: DateTime<Utc>,
    pub prometheus: MetricsSummary,
    pub business: BusinessMetricsSummary,
}

/// 详细指标响应
#[derive(Debug, Serialize, Deserialize)]
pub struct DetailedMetricsResponse {
    pub timestamp: DateTime<Utc>,
    pub summary: BusinessMetricsSummary,
    pub models: Vec<metrics::business::ModelStats>,
    pub providers: Vec<metrics::business::ProviderStats>,
    pub today_cost: metrics::business::DailyCost,
    pub total_cost: f64,
}

/// 指标健康状态
#[derive(Debug, Serialize, Deserialize)]
pub struct MetricsHealth {
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub metrics_collected: bool,
    pub prometheus_ready: bool,
}

/// 实时指标
#[derive(Debug, Serialize, Deserialize)]
pub struct RealtimeMetrics {
    pub timestamp: DateTime<Utc>,
    pub requests_total: u64,
    pub active_connections: i64,
    pub websocket_connections: i64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub cache_hit_rate: f64,
    pub today_requests: u64,
    pub today_cost: f64,
}

/// 成本指标
#[derive(Debug, Serialize, Deserialize)]
pub struct CostMetrics {
    pub timestamp: DateTime<Utc>,
    pub total_cost: f64,
    pub today_cost: metrics::business::DailyCost,
    pub total_tokens_input: u64,
    pub total_tokens_output: u64,
    pub by_model: Vec<metrics::business::ModelStats>,
}

/// Token 指标
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenMetrics {
    pub timestamp: DateTime<Utc>,
    pub total_input: u64,
    pub total_output: u64,
    pub by_model: Vec<ModelTokens>,
    pub by_user: Vec<UserTokens>,
}

/// 模型 Token 使用
#[derive(Debug, Serialize, Deserialize)]
pub struct ModelTokens {
    pub model: String,
    pub input: u64,
    pub output: u64,
}

/// 用户 Token 使用
#[derive(Debug, Serialize, Deserialize)]
pub struct UserTokens {
    pub user_id: String,
    pub input: u64,
    pub output: u64,
}

/// 账号指标
#[derive(Debug, Serialize, Deserialize)]
pub struct AccountMetrics {
    pub timestamp: DateTime<Utc>,
    pub usage_ratio: f64,
    pub providers: Vec<ProviderAccountInfo>,
}

/// 提供商账号信息
#[derive(Debug, Serialize, Deserialize)]
pub struct ProviderAccountInfo {
    pub provider: String,
    pub active_accounts: u64,
    pub total_accounts: u64,
    pub error_rate: f64,
    pub avg_latency_ms: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_realtime_metrics_structure() {
        let realtime = RealtimeMetrics {
            timestamp: Utc::now(),
            requests_total: 100,
            active_connections: 10,
            websocket_connections: 5,
            cache_hits: 80,
            cache_misses: 20,
            cache_hit_rate: 0.8,
            today_requests: 50,
            today_cost: 1.5,
        };

        let json = serde_json::to_string(&realtime).unwrap();
        assert!(json.contains("requests_total"));
        assert!(json.contains("cache_hit_rate"));
    }

    #[test]
    fn test_metrics_health() {
        let health = MetricsHealth {
            status: "healthy".to_string(),
            timestamp: Utc::now(),
            metrics_collected: true,
            prometheus_ready: true,
        };

        assert_eq!(health.status, "healthy");
        assert!(health.metrics_collected);
    }

    #[test]
    fn test_detailed_metrics_response() {
        let response = DetailedMetricsResponse {
            timestamp: Utc::now(),
            summary: get_business_metrics().get_summary(),
            models: vec![],
            providers: vec![],
            today_cost: metrics::business::DailyCost::default(),
            total_cost: 0.0,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("timestamp"));
        assert!(json.contains("summary"));
    }
}
