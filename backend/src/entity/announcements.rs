//! Announcement Entity

#![allow(dead_code)]

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// 公告状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnnouncementStatus {
    Draft,
    Active,
    Archived,
}

impl AnnouncementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Active => "active",
            Self::Archived => "archived",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "draft" => Self::Draft,
            "active" => Self::Active,
            "archived" => Self::Archived,
            _ => Self::Draft,
        }
    }
}

/// 通知模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotifyMode {
    Silent,
    Popup,
}

impl NotifyMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Silent => "silent",
            Self::Popup => "popup",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "popup" => Self::Popup,
            _ => Self::Silent,
        }
    }
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "announcements")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub title: String,
    pub content: String,
    pub status: String,
    pub notify_mode: String,
    pub targeting: Option<JsonValue>,
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::announcement_reads::Entity")]
    Reads,
}

impl Related<super::announcement_reads::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Reads.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
