//! Alert History Entity - 告警历史实体

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "alert_history")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub level: String,
    pub title: String,
    pub message: String,
    pub source: String,
    pub labels: Option<JsonValue>,
    pub rule_id: Option<Uuid>,
    pub rule_name: Option<String>,
    pub send_results: Option<JsonValue>,
    pub silenced: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::alert_rules::Entity",
        from = "Column::RuleId",
        to = "super::alert_rules::Column::Id"
    )]
    Rule,
}

impl Related<super::alert_rules::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Rule.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// 获取告警级别
    pub fn get_level(&self) -> Option<crate::alert::AlertLevel> {
        crate::alert::AlertLevel::parse(&self.level)
    }

    /// 格式化为摘要
    pub fn to_summary(&self) -> String {
        format!(
            "[{}] {} - {}",
            self.level.to_uppercase(),
            self.title,
            self.message
        )
    }
}
