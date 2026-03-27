//! 告警通道配置表迁移

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AlertChannels::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AlertChannels::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(
                        ColumnDef::new(AlertChannels::ChannelId)
                            .string_len(50)
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(AlertChannels::Name)
                            .string_len(100)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AlertChannels::ChannelType)
                            .string_len(30)
                            .not_null(),
                    )
                    .col(ColumnDef::new(AlertChannels::Config).json().not_null())
                    .col(
                        ColumnDef::new(AlertChannels::Enabled)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(AlertChannels::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(AlertChannels::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .index(
                        Index::create()
                            .name("idx_alert_channels_type")
                            .col(AlertChannels::ChannelType),
                    )
                    .index(
                        Index::create()
                            .name("idx_alert_channels_enabled")
                            .col(AlertChannels::Enabled),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AlertChannels::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum AlertChannels {
    Table,
    Id,
    ChannelId,
    Name,
    ChannelType,
    Config,
    Enabled,
    CreatedAt,
    UpdatedAt,
}
