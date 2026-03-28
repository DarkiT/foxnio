//! Scheduled Test Result Entity

#![allow(dead_code)]

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TestResultStatus {
    Success,
    Failed,
    Timeout,
}

impl TestResultStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Failed => "failed",
            Self::Timeout => "timeout",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "success" => Self::Success,
            "timeout" => Self::Timeout,
            _ => Self::Failed,
        }
    }
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "scheduled_test_results")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub plan_id: i64,
    pub status: String,
    pub result: Option<JsonValue>,
    pub error_message: Option<String>,
    pub duration_ms: Option<i64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::scheduled_test_plans::Entity",
        from = "Column::PlanId",
        to = "super::scheduled_test_plans::Column::Id"
    )]
    Plan,
}

impl Related<super::scheduled_test_plans::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Plan.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
