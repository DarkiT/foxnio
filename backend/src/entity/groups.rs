//! Group Entity - 账号分组管理

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// 分组状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GroupStatus {
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "inactive")]
    Inactive,
    #[serde(rename = "suspended")]
    Suspended,
}

impl std::fmt::Display for GroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GroupStatus::Active => write!(f, "active"),
            GroupStatus::Inactive => write!(f, "inactive"),
            GroupStatus::Suspended => write!(f, "suspended"),
        }
    }
}

impl From<&str> for GroupStatus {
    fn from(s: &str) -> Self {
        match s {
            "active" => GroupStatus::Active,
            "inactive" => GroupStatus::Inactive,
            "suspended" => GroupStatus::Suspended,
            _ => GroupStatus::Active,
        }
    }
}

/// 平台类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Platform {
    #[serde(rename = "anthropic")]
    Anthropic,
    #[serde(rename = "openai")]
    OpenAI,
    #[serde(rename = "gemini")]
    Gemini,
    #[serde(rename = "antigravity")]
    Antigravity,
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Platform::Anthropic => write!(f, "anthropic"),
            Platform::OpenAI => write!(f, "openai"),
            Platform::Gemini => write!(f, "gemini"),
            Platform::Antigravity => write!(f, "antigravity"),
        }
    }
}

impl From<&str> for Platform {
    fn from(s: &str) -> Self {
        match s {
            "anthropic" => Platform::Anthropic,
            "openai" => Platform::OpenAI,
            "gemini" => Platform::Gemini,
            "antigravity" => Platform::Antigravity,
            _ => Platform::OpenAI,
        }
    }
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "groups")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub platform: String,
    pub status: String,

    // 配额管理
    pub daily_limit_usd: Option<f64>,
    pub weekly_limit_usd: Option<f64>,
    pub monthly_limit_usd: Option<f64>,

    // 速率限制
    pub rate_multiplier: f64,

    // 模型路由配置
    pub model_routing: Option<JsonValue>, // HashMap<String, Vec<i64>> -> JSON
    pub model_routing_enabled: bool,

    // 降级配置
    pub fallback_group_id: Option<i64>,

    // Claude Code 限制
    pub claude_code_only: bool,
    pub fallback_group_id_on_invalid_request: Option<i64>,

    // 排序和显示
    pub sort_order: i32,
    pub is_exclusive: bool,

    // 时间戳
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::accounts::Entity")]
    Accounts,
}

impl Related<super::accounts::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Accounts.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// 检查分组是否激活
    pub fn is_active(&self) -> bool {
        self.status == "active" && self.deleted_at.is_none()
    }

    /// 获取平台类型
    pub fn platform_type(&self) -> Platform {
        Platform::from(self.platform.as_str())
    }

    /// 获取状态类型
    pub fn status_type(&self) -> GroupStatus {
        GroupStatus::from(self.status.as_str())
    }

    /// 获取模型路由配置
    pub fn get_model_routing(&self) -> std::collections::HashMap<String, Vec<i64>> {
        self.model_routing
            .as_ref()
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default()
    }

    /// 检查是否有日限额
    pub fn has_daily_limit(&self) -> bool {
        self.daily_limit_usd.is_some()
    }

    /// 检查是否有月限额
    pub fn has_monthly_limit(&self) -> bool {
        self.monthly_limit_usd.is_some()
    }
}
