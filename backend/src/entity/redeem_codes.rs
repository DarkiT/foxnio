//! Redeem code entity

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "redeem_codes")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(unique)]
    pub code: String,
    pub batch_id: Option<String>,
    pub amount: Decimal,
    #[sea_orm(default_value = "balance")]
    pub r#type: String,
    #[sea_orm(default_value = "1")]
    pub max_uses: i32,
    #[sea_orm(default_value = "0")]
    pub used_count: i32,
    #[sea_orm(default_value = "active")]
    pub status: String,
    pub expires_at: Option<DateTimeWithTimeZone>,
    pub used_by: Option<Json>,
    pub notes: Option<String>,
    pub created_by: Option<i64>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Check if code is valid for use
    pub fn is_valid(&self) -> bool {
        if self.status != "active" {
            return false;
        }
        if self.used_count >= self.max_uses {
            return false;
        }
        if let Some(expires) = self.expires_at {
            if expires < chrono::Utc::now() {
                return false;
            }
        }
        true
    }

    /// Check if code has remaining uses
    pub fn has_remaining_uses(&self) -> bool {
        self.used_count < self.max_uses
    }

    /// Get remaining uses count
    pub fn remaining_uses(&self) -> i32 {
        (self.max_uses - self.used_count).max(0)
    }
}
