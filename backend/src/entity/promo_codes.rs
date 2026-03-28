//! Promo Code Entity

#![allow(dead_code)]

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// 优惠码状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromoCodeStatus {
    Active,
    Disabled,
}

impl PromoCodeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Disabled => "disabled",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "active" => Self::Active,
            "disabled" => Self::Disabled,
            _ => Self::Active,
        }
    }
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "promo_codes")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(unique)]
    pub code: String,
    pub bonus_amount: f64,
    pub max_uses: i32,
    pub used_count: i32,
    pub status: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::promo_code_usages::Entity")]
    UsageRecords,
}

impl Related<super::promo_code_usages::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UsageRecords.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// 检查优惠码是否可用
    pub fn is_valid(&self) -> bool {
        // 检查状态
        if self.status != PromoCodeStatus::Active.as_str() {
            return false;
        }

        // 检查使用次数限制
        if self.max_uses > 0 && self.used_count >= self.max_uses {
            return false;
        }

        // 检查过期时间
        if let Some(expires_at) = self.expires_at {
            if expires_at < Utc::now() {
                return false;
            }
        }

        true
    }
}
