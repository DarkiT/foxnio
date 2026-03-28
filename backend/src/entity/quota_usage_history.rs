//! Quota usage history entity

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "quota_usage_history")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub user_id: i64,
    pub api_key_id: Option<Uuid>,
    pub account_id: Option<i64>,
    pub model: String,
    pub amount: Decimal,
    pub tokens_in: Option<i32>,
    pub tokens_out: Option<i32>,
    pub request_type: Option<String>,
    pub metadata: Option<Json>,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
