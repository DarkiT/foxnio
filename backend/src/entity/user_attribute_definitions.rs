//! User Attribute Definition Entity

#![allow(dead_code)]

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// 属性类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttributeType {
    Text,
    Textarea,
    Number,
    Email,
    Url,
    Date,
    Select,
    MultiSelect,
}

impl AttributeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Textarea => "textarea",
            Self::Number => "number",
            Self::Email => "email",
            Self::Url => "url",
            Self::Date => "date",
            Self::Select => "select",
            Self::MultiSelect => "multi_select",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "textarea" => Self::Textarea,
            "number" => Self::Number,
            "email" => Self::Email,
            "url" => Self::Url,
            "date" => Self::Date,
            "select" => Self::Select,
            "multi_select" => Self::MultiSelect,
            _ => Self::Text,
        }
    }
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "user_attribute_definitions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub key: String,
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub options: JsonValue,
    pub required: bool,
    pub validation: JsonValue,
    pub placeholder: String,
    pub display_order: i32,
    pub enabled: bool,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::user_attribute_values::Entity")]
    Values,
}

impl Related<super::user_attribute_values::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Values.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
