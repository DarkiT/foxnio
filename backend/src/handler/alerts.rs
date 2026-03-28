//! 告警管理 API 处理器

#![allow(dead_code)]
use axum::{extract::Path, http::StatusCode, Extension, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use std::time::Duration;

use super::ApiError;
use crate::alert::{
    history::AlertHistoryFilter,
    rules::{AlertCondition, AlertRule},
    Alert, AlertChannelType, AlertLevel, SilenceRule,
};
use crate::gateway::middleware::permission::check_permission;
use crate::gateway::SharedState;
use crate::service::permission::Permission;
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

    let rules: Vec<AlertRule> = state.alert_manager.list_rules().await;

    Ok(Json(json!({
        "object": "list",
        "data": rules.iter().map(|r| json!({
            "id": r.id,
            "name": r.name,
            "description": r.description,
            "enabled": r.enabled,
            "level": r.level.as_str(),
            "trigger_count": r.trigger_count,
            "last_triggered_at": r.last_triggered_at.map(|t: chrono::DateTime<chrono::Utc>| t.to_rfc3339()),
            "created_at": r.created_at.to_rfc3339(),
            "updated_at": r.updated_at.to_rfc3339()
        })).collect::<Vec<_>>()
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
    let level = AlertLevel::parse(&req.level).unwrap_or(AlertLevel::Warning);

    // 解析告警通道
    let channels: Vec<AlertChannelType> = req
        .channels
        .iter()
        .filter_map(|c| AlertChannelType::parse(c))
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

    let rule_id = state.alert_manager.add_rule(rule.clone()).await;

    Ok(Json(json!({
        "id": rule_id,
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

    let deleted = state.alert_manager.delete_rule(&id).await;

    Ok(Json(json!({
        "success": deleted,
        "id": id
    })))
}

/// PUT /api/v1/admin/alerts/rules/:id - 更新告警规则
pub async fn update_rule(
    Extension(state): Extension<SharedState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
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

    // 解析告警级别
    let level = AlertLevel::parse(&req.level).unwrap_or(AlertLevel::Warning);

    // 解析告警通道
    let channels: Vec<AlertChannelType> = req
        .channels
        .iter()
        .filter_map(|c| AlertChannelType::parse(c))
        .collect();

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

    // 创建更新后的规则
    let rule = AlertRule::new(&req.name, condition, level, channels)
        .with_description(&req.description)
        .with_duration(Duration::from_secs(req.duration_secs))
        .with_enabled(req.enabled);

    // 删除旧规则并添加新规则
    state.alert_manager.delete_rule(&id).await;
    let new_id = state.alert_manager.add_rule(rule.clone()).await;

    Ok(Json(json!({
        "id": new_id,
        "name": rule.name,
        "description": rule.description,
        "enabled": rule.enabled,
        "updated_at": chrono::Utc::now().to_rfc3339()
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

    let silences: Vec<SilenceRule> = state.alert_manager.list_silences().await;

    Ok(Json(json!({
        "object": "list",
        "data": silences.iter().map(|s| json!({
            "id": s.id,
            "rule_pattern": s.rule_pattern,
            "start_time": s.start_time.to_rfc3339(),
            "end_time": s.end_time.to_rfc3339(),
            "reason": s.reason,
            "created_by": s.created_by,
            "is_active": s.is_active()
        })).collect::<Vec<_>>()
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

    state.alert_manager.add_silence(silence.clone()).await;

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

    let deleted = state.alert_manager.remove_silence(&id).await;

    Ok(Json(json!({
        "success": deleted,
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

    let filter = AlertHistoryFilter::default();
    let history: Vec<crate::alert::history::AlertHistoryEntry> =
        state.alert_manager.query_history(&filter).await;

    Ok(Json(json!({
        "object": "list",
        "data": history.iter().map(|h| json!({
            "id": h.id,
            "alert": {
                "level": h.alert.level.as_str(),
                "title": h.alert.title,
                "message": h.alert.message,
                "source": h.alert.source,
                "timestamp": h.alert.timestamp.to_rfc3339()
            },
            "results": h.results.iter().map(|r| json!({
                "success": r.success,
                "channel_type": r.channel_type.as_str(),
                "error": r.error,
                "timestamp": r.timestamp.to_rfc3339()
            })).collect::<Vec<_>>(),
            "rule_id": h.rule_id,
            "rule_name": h.rule_name,
            "silenced": h.silenced,
            "created_at": h.created_at.to_rfc3339()
        })).collect::<Vec<_>>()
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

    let stats = state.alert_manager.get_stats(None, None).await;

    Ok(Json(json!({
        "total_count": stats.total_count,
        "by_level": stats.by_level,
        "by_source": stats.by_source,
        "success_count": stats.success_count,
        "failure_count": stats.failure_count,
        "silenced_count": stats.silenced_count,
        "start_time": stats.start_time.to_rfc3339(),
        "end_time": stats.end_time.to_rfc3339()
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

    let channels = state.alert_manager.list_channels().await;

    Ok(Json(json!({
        "object": "list",
        "data": channels.iter().map(|(id, channel_type, name)| json!({
            "id": id,
            "type": channel_type.as_str(),
            "name": name
        })).collect::<Vec<_>>()
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

    let channel_type = AlertChannelType::parse(&req.channel_type)
        .ok_or_else(|| ApiError(StatusCode::BAD_REQUEST, "Invalid channel type".into()))?;

    let result = state
        .alert_manager
        .register_channel(req.id.clone(), channel_type.clone(), req.config)
        .await;

    match result {
        Ok(()) => Ok(Json(json!({
            "id": req.id,
            "type": channel_type.as_str(),
            "name": req.name,
            "success": true
        }))),
        Err(e) => Err(ApiError(StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
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

    let deleted = state.alert_manager.remove_channel(&id).await;

    Ok(Json(json!({
        "success": deleted,
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
        .and_then(|l| AlertLevel::parse(l))
        .unwrap_or(AlertLevel::Info);

    let title = req.title.as_deref().unwrap_or("测试告警");

    let alert = Alert::new(
        level,
        title,
        "这是一条测试告警消息，用于验证告警通道配置是否正确。",
    )
    .with_source("test_api")
    .with_label("triggered_by", &claims.sub);

    // 发送测试告警
    let entry = state
        .alert_manager
        .send_alert(alert.clone(), None, None)
        .await;

    Ok(Json(json!({
        "success": true,
        "alert": {
            "level": alert.level.as_str(),
            "title": alert.title,
            "message": alert.message
        },
        "results": entry.results.iter().map(|r| json!({
            "success": r.success,
            "channel_type": r.channel_type.as_str(),
            "error": r.error
        })).collect::<Vec<_>>()
    })))
}
