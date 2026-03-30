use sea_orm_migration::prelude::*;

use super::m20240327_000003_create_api_keys::ApiKeys;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 添加 IP 白名单字段
        manager
            .alter_table(
                Table::alter()
                    .table(ApiKeys::Table)
                    .add_column(
                        ColumnDef::new(Alias::new("ip_whitelist"))
                            .json()
                            .null()
                            .comment("IP 白名单列表，JSON 数组格式"),
                    )
                    .to_owned(),
            )
            .await?;

        // 添加每日配额字段
        manager
            .alter_table(
                Table::alter()
                    .table(ApiKeys::Table)
                    .add_column(
                        ColumnDef::new(Alias::new("daily_quota"))
                            .big_integer()
                            .null()
                            .comment("每日配额限制（tokens 数量）"),
                    )
                    .to_owned(),
            )
            .await?;

        // 添加每日已使用配额字段（用于跟踪使用情况）
        manager
            .alter_table(
                Table::alter()
                    .table(ApiKeys::Table)
                    .add_column(
                        ColumnDef::new(Alias::new("daily_used_quota"))
                            .big_integer()
                            .null()
                            .default(0)
                            .comment("当日已使用配额"),
                    )
                    .to_owned(),
            )
            .await?;

        // 添加配额重置时间字段
        manager
            .alter_table(
                Table::alter()
                    .table(ApiKeys::Table)
                    .add_column(
                        ColumnDef::new(Alias::new("quota_reset_at"))
                            .timestamp_with_time_zone()
                            .null()
                            .comment("配额重置时间"),
                    )
                    .to_owned(),
            )
            .await?;

        // 添加索引以提高查询性能
        manager
            .create_index(
                Index::create()
                    .name("idx_api_keys_status")
                    .table(ApiKeys::Table)
                    .col(ApiKeys::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_api_keys_user_status")
                    .table(ApiKeys::Table)
                    .col(ApiKeys::UserId)
                    .col(ApiKeys::Status)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 删除索引
        manager
            .drop_index(Index::drop().name("idx_api_keys_user_status").to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_api_keys_status").to_owned())
            .await?;

        // 删除字段
        manager
            .alter_table(
                Table::alter()
                    .table(ApiKeys::Table)
                    .drop_column(Alias::new("quota_reset_at"))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(ApiKeys::Table)
                    .drop_column(Alias::new("daily_used_quota"))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(ApiKeys::Table)
                    .drop_column(Alias::new("daily_quota"))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(ApiKeys::Table)
                    .drop_column(Alias::new("ip_whitelist"))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
