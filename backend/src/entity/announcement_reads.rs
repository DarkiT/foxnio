//! Announcement Read Entity

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "announcement_reads")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub announcement_id: i64,
    pub user_id: i64,
    pub read_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::announcements::Entity",
        from = "Column::AnnouncementId",
        to = "super::announcements::Column::Id"
    )]
    Announcement,
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id"
    )]
    User,
}

impl Related<super::announcements::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Announcement.def()
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
