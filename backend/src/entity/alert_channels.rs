//! Alert Channels Entity - 告警通道实体

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "alert_channels")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub channel_id: String,
    pub name: String,
    pub channel_type: String,
    pub config: JsonValue,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// 获取通道类型
    pub fn get_channel_type(&self) -> Option<crate::alert::AlertChannelType> {
        crate::alert::AlertChannelType::parse(&self.channel_type)
    }
}
