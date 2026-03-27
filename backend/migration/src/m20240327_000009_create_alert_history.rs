//! 告警历史表迁移

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AlertHistory::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AlertHistory::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(
                        ColumnDef::new(AlertHistory::Level)
                            .string_len(20)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AlertHistory::Title)
                            .string_len(200)
                            .not_null(),
                    )
                    .col(ColumnDef::new(AlertHistory::Message).text().not_null())
                    .col(
                        ColumnDef::new(AlertHistory::Source)
                            .string_len(100)
                            .not_null(),
                    )
                    .col(ColumnDef::new(AlertHistory::Labels).json())
                    .col(ColumnDef::new(AlertHistory::RuleId).uuid())
                    .col(ColumnDef::new(AlertHistory::RuleName).string_len(100))
                    .col(ColumnDef::new(AlertHistory::SendResults).json())
                    .col(
                        ColumnDef::new(AlertHistory::Silenced)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(AlertHistory::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    // Indexes for common queries
                    .index(
                        Index::create()
                            .name("idx_alert_history_level")
                            .col(AlertHistory::Level),
                    )
                    .index(
                        Index::create()
                            .name("idx_alert_history_source")
                            .col(AlertHistory::Source),
                    )
                    .index(
                        Index::create()
                            .name("idx_alert_history_rule_id")
                            .col(AlertHistory::RuleId),
                    )
                    .index(
                        Index::create()
                            .name("idx_alert_history_created_at")
                            .col(AlertHistory::CreatedAt),
                    )
                    .index(
                        Index::create()
                            .name("idx_alert_history_silenced")
                            .col(AlertHistory::Silenced),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AlertHistory::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum AlertHistory {
    Table,
    Id,
    Level,
    Title,
    Message,
    Source,
    Labels,
    RuleId,
    RuleName,
    SendResults,
    Silenced,
    CreatedAt,
}
