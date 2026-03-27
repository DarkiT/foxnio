//! 告警管理 API 处理器

use axum::{extract::Path, http::StatusCode, Extension, Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;

use super::ApiError;
use crate::alert::{
    rules::{AlertCondition, AlertRule},
    Alert, AlertChannelType, AlertLevel, SilenceRule,
};
use crate::gateway::middleware::permission::check_permission;
use crate::gateway::SharedState;
use crate::service::permission::{Permission, PermissionService};
use crate::service::user::Claims;

/// 创建规则请求
#[derive(Debug, Deserialize)]
pub struct CreateRuleRequest {
    /// 规则名称
    pub name: String,
    /// 规则描述
    #[serde(default)]
    pub description: String,
    /// 告警条件
    pub condition: AlertConditionRequest,
    /// 持续时间（秒）
    #[serde(default)]
    pub duration_secs: u64,
    /// 告警级别
    #[serde(default)]
    pub level: String,
    /// 告警通道
    pub channels: Vec<String>,
    /// 是否启用
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// 标签
    #[serde(default)]
    pub labels: std::collections::HashMap<String, String>,
}

fn default_true() -> bool {
    true
}

/// 告警条件请求
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AlertConditionRequest {
    ErrorRateAbove { threshold: f64 },
    LatencyAbove { threshold_ms: u64 },
    ConnectionCountBelow { threshold: u32 },
    AccountBalanceBelow { threshold: i64 },
    CpuUsageAbove { threshold: f64 },
    MemoryUsageAbove { threshold: f64 },
    DiskUsageAbove { threshold: f64 },
    RequestRateAbove { threshold: f64 },
    Custom { expression: String },
}

/// 创建静默请求
#[derive(Debug, Deserialize)]
pub struct CreateSilenceRequest {
    /// 规则匹配模式
    pub rule_pattern: String,
    /// 静默持续时间（秒）
    pub duration_secs: u64,
    /// 静默原因
    pub reason: String,
}

/// 注册通道请求
#[derive(Debug, Deserialize)]
pub struct RegisterChannelRequest {
    /// 通道 ID
    pub id: String,
    /// 通道类型
    #[serde(rename = "type")]
    pub channel_type: String,
    /// 通道名称
    #[serde(default)]
    pub name: String,
    /// 通道配置
    pub config: Value,
}

/// 测试告警请求
#[derive(Debug, Deserialize)]
pub struct TestAlertRequest {
    /// 通道 ID（可选，不传则测试所有通道）
    pub channel_id: Option<String>,
    /// 告警级别（可选）
    pub level: Option<String>,
    /// 告警标题（可选）
    pub title: Option<String>,
}

/// 规则响应
#[derive(Debug, Serialize)]
pub struct RuleResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub condition: Value,
    pub duration_secs: u64,
    pub level: String,
    pub channels: Vec<String>,
    pub enabled: bool,
    pub labels: std::collections::HashMap<String, String>,
    pub trigger_count: u64,
    pub last_triggered_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// 静默响应
#[derive(Debug, Serialize)]
pub struct SilenceResponse {
    pub id: String,
    pub rule_pattern: String,
    pub start_time: String,
    pub end_time: String,
    pub reason: String,
    pub created_by: Option<String>,
    pub is_active: bool,
}

/// 历史记录响应
#[derive(Debug, Serialize)]
pub struct HistoryResponse {
    pub id: String,
    pub alert: Value,
    pub results: Vec<Value>,
    pub rule_id: Option<String>,
    pub rule_name: Option<String>,
    pub silenced: bool,
    pub created_at: String,
}

/// 统计响应
#[derive(Debug, Serialize)]
pub struct StatsResponse {
    pub total_count: u64,
    pub by_level: std::collections::HashMap<String, u64>,
    pub by_source: std::collections::HashMap<String, u64>,
    pub success_count: u64,
    pub failure_count: u64,
    pub silenced_count: u64,
    pub start_time: String,
    pub end_time: String,
}

// ============ 规则管理 API ============

/// GET /api/v1/admin/alerts/rules - 查询告警规则
pub async fn list_rules(
    Extension(state): Extension<SharedState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Value>, ApiError> {
    // 权限检查
    check_permission(&claims, Permission::AdminRead)
        .await
        .map_err(|e| ApiError(StatusCode::FORBIDDEN, e))?;

    // TODO: 从全局 AlertManager 获取规则
    // 目前返回空列表
    Ok(Json(json!({
        "object": "list",
        "data": []
    })))
}

/// POST /api/v1/admin/alerts/rules - 创建告警规则
pub async fn create_rule(
    Extension(state): Extension<SharedState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateRuleRequest>,
) -> Result<Json<Value>, ApiError> {
    // 权限检查
    check_permission(&claims, Permission::AdminWrite)
        .await
        .map_err(|e| ApiError(StatusCode::FORBIDDEN, e))?;

    // 验证请求
    if req.name.is_empty() {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "Rule name is required".into(),
        ));
    }

    if req.channels.is_empty() {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "At least one channel is required".into(),
        ));
    }

    // 解析告警级别
    let level = AlertLevel::from_str(&req.level).unwrap_or(AlertLevel::Warning);

    // 解析告警通道
    let channels: Vec<AlertChannelType> = req
        .channels
        .iter()
        .filter_map(|c| AlertChannelType::from_str(c))
        .collect();

    if channels.is_empty() {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "Invalid channel types".into(),
        ));
    }

    // 构建告警条件
    let condition = match req.condition {
        AlertConditionRequest::ErrorRateAbove { threshold } => {
            AlertCondition::ErrorRateAbove { threshold }
        }
        AlertConditionRequest::LatencyAbove { threshold_ms } => {
            AlertCondition::LatencyAbove { threshold_ms }
        }
        AlertConditionRequest::ConnectionCountBelow { threshold } => {
            AlertCondition::ConnectionCountBelow { threshold }
        }
        AlertConditionRequest::AccountBalanceBelow { threshold } => {
            AlertCondition::AccountBalanceBelow { threshold }
        }
        AlertConditionRequest::CpuUsageAbove { threshold } => {
            AlertCondition::CpuUsageAbove { threshold }
        }
        AlertConditionRequest::MemoryUsageAbove { threshold } => {
            AlertCondition::MemoryUsageAbove { threshold }
        }
        AlertConditionRequest::DiskUsageAbove { threshold } => {
            AlertCondition::DiskUsageAbove { threshold }
        }
        AlertConditionRequest::RequestRateAbove { threshold } => {
            AlertCondition::RequestRateAbove { threshold }
        }
        AlertConditionRequest::Custom { expression } => AlertCondition::Custom { expression },
    };

    // 创建规则
    let rule = AlertRule::new(&req.name, condition, level, channels)
        .with_description(&req.description)
        .with_duration(Duration::from_secs(req.duration_secs))
        .with_enabled(req.enabled);

    // TODO: 保存到数据库和 AlertManager

    Ok(Json(json!({
        "id": rule.id,
        "name": rule.name,
        "description": rule.description,
        "enabled": rule.enabled,
        "created_at": rule.created_at.to_rfc3339()
    })))
}

/// DELETE /api/v1/admin/alerts/rules/:id - 删除告警规则
pub async fn delete_rule(
    Extension(state): Extension<SharedState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    // 权限检查
    check_permission(&claims, Permission::AdminWrite)
        .await
        .map_err(|e| ApiError(StatusCode::FORBIDDEN, e))?;

    // TODO: 从 AlertManager 删除规则

    Ok(Json(json!({
        "success": true,
        "id": id
    })))
}

// ============ 静默管理 API ============

/// GET /api/v1/admin/alerts/silences - 查询静默规则
pub async fn list_silences(
    Extension(state): Extension<SharedState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Value>, ApiError> {
    // 权限检查
    check_permission(&claims, Permission::AdminRead)
        .await
        .map_err(|e| ApiError(StatusCode::FORBIDDEN, e))?;

    // TODO: 从 AlertManager 获取静默规则

    Ok(Json(json!({
        "object": "list",
        "data": []
    })))
}

/// POST /api/v1/admin/alerts/silences - 创建静默规则
pub async fn create_silence(
    Extension(state): Extension<SharedState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateSilenceRequest>,
) -> Result<Json<Value>, ApiError> {
    // 权限检查
    check_permission(&claims, Permission::AdminWrite)
        .await
        .map_err(|e| ApiError(StatusCode::FORBIDDEN, e))?;

    // 验证请求
    if req.rule_pattern.is_empty() {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "Rule pattern is required".into(),
        ));
    }

    if req.duration_secs == 0 {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "Duration must be greater than 0".into(),
        ));
    }

    // 创建静默规则
    let silence = SilenceRule {
        id: uuid::Uuid::new_v4().to_string(),
        rule_pattern: req.rule_pattern,
        start_time: chrono::Utc::now(),
        end_time: chrono::Utc::now() + chrono::Duration::seconds(req.duration_secs as i64),
        reason: req.reason,
        created_by: Some(claims.sub.clone()),
    };

    // TODO: 保存到 AlertManager

    Ok(Json(json!({
        "id": silence.id,
        "rule_pattern": silence.rule_pattern,
        "start_time": silence.start_time.to_rfc3339(),
        "end_time": silence.end_time.to_rfc3339(),
        "reason": silence.reason,
        "is_active": silence.is_active()
    })))
}

/// DELETE /api/v1/admin/alerts/silences/:id - 删除静默规则
pub async fn delete_silence(
    Extension(state): Extension<SharedState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    // 权限检查
    check_permission(&claims, Permission::AdminWrite)
        .await
        .map_err(|e| ApiError(StatusCode::FORBIDDEN, e))?;

    // TODO: 从 AlertManager 删除静默规则

    Ok(Json(json!({
        "success": true,
        "id": id
    })))
}

// ============ 历史记录 API ============

/// GET /api/v1/admin/alerts/history - 查询告警历史
pub async fn list_history(
    Extension(state): Extension<SharedState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Value>, ApiError> {
    // 权限检查
    check_permission(&claims, Permission::AdminRead)
        .await
        .map_err(|e| ApiError(StatusCode::FORBIDDEN, e))?;

    // TODO: 从 AlertManager 获取历史记录

    Ok(Json(json!({
        "object": "list",
        "data": []
    })))
}

/// GET /api/v1/admin/alerts/stats - 查询告警统计
pub async fn get_stats(
    Extension(state): Extension<SharedState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Value>, ApiError> {
    // 权限检查
    check_permission(&claims, Permission::AdminRead)
        .await
        .map_err(|e| ApiError(StatusCode::FORBIDDEN, e))?;

    // TODO: 从 AlertManager 获取统计信息

    Ok(Json(json!({
        "total_count": 0,
        "by_level": {},
        "by_source": {},
        "success_count": 0,
        "failure_count": 0,
        "silenced_count": 0
    })))
}

// ============ 通道管理 API ============

/// GET /api/v1/admin/alerts/channels - 查询告警通道
pub async fn list_channels(
    Extension(state): Extension<SharedState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Value>, ApiError> {
    // 权限检查
    check_permission(&claims, Permission::AdminRead)
        .await
        .map_err(|e| ApiError(StatusCode::FORBIDDEN, e))?;

    // TODO: 从 AlertManager 获取通道列表

    Ok(Json(json!({
        "object": "list",
        "data": []
    })))
}

/// POST /api/v1/admin/alerts/channels - 注册告警通道
pub async fn register_channel(
    Extension(state): Extension<SharedState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<RegisterChannelRequest>,
) -> Result<Json<Value>, ApiError> {
    // 权限检查
    check_permission(&claims, Permission::AdminWrite)
        .await
        .map_err(|e| ApiError(StatusCode::FORBIDDEN, e))?;

    // 验证请求
    if req.id.is_empty() {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "Channel ID is required".into(),
        ));
    }

    let channel_type = AlertChannelType::from_str(&req.channel_type)
        .ok_or_else(|| ApiError(StatusCode::BAD_REQUEST, "Invalid channel type".into()))?;

    // TODO: 注册到 AlertManager

    Ok(Json(json!({
        "id": req.id,
        "type": channel_type.as_str(),
        "name": req.name,
        "success": true
    })))
}

/// DELETE /api/v1/admin/alerts/channels/:id - 删除告警通道
pub async fn delete_channel(
    Extension(state): Extension<SharedState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    // 权限检查
    check_permission(&claims, Permission::AdminWrite)
        .await
        .map_err(|e| ApiError(StatusCode::FORBIDDEN, e))?;

    // TODO: 从 AlertManager 删除通道

    Ok(Json(json!({
        "success": true,
        "id": id
    })))
}

// ============ 测试 API ============

/// POST /api/v1/admin/alerts/test - 测试告警
pub async fn test_alert(
    Extension(state): Extension<SharedState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<TestAlertRequest>,
) -> Result<Json<Value>, ApiError> {
    // 权限检查
    check_permission(&claims, Permission::AdminWrite)
        .await
        .map_err(|e| ApiError(StatusCode::FORBIDDEN, e))?;

    // 构建测试告警
    let level = req
        .level
        .as_ref()
        .and_then(|l| AlertLevel::from_str(l))
        .unwrap_or(AlertLevel::Info);

    let title = req.title.as_deref().unwrap_or("测试告警");

    let alert = Alert::new(
        level,
        title,
        "这是一条测试告警消息，用于验证告警通道配置是否正确。",
    )
    .with_source("test_api")
    .with_label("triggered_by", &claims.sub);

    // TODO: 发送测试告警

    Ok(Json(json!({
        "success": true,
        "alert": {
            "level": alert.level.as_str(),
            "title": alert.title,
            "message": alert.message
        }
    })))
}
