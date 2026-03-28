//! Scheduled Test Plan Entity

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "scheduled_test_plans")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub cron_expr: String,
    pub test_config: JsonValue,
    pub last_run_at: Option<DateTime<Utc>>,
    pub next_run_at: Option<DateTime<Utc>>,
    pub last_result: Option<JsonValue>,
    pub created_by: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::scheduled_test_results::Entity")]
    Results,
}

impl Related<super::scheduled_test_results::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Results.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
