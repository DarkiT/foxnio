use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, FromRow, PgPool};
use std::collections::{HashMap, HashSet};

/// Account service for managing AI service accounts
pub struct AccountService {
    pool: PgPool,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Account {
    pub id: i64,
    pub name: String,
    pub provider: String,
    pub status: String,
    pub credentials: String,
    pub models: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AccountStatus {
    Active,
    Inactive,
    Suspended,
    Deleted,
}

#[derive(Debug, thiserror::Error)]
pub enum AccountError {
    #[error("Account not found")]
    NotFound,
    #[error("Account inactive")]
    Inactive,
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

impl AccountService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get account by ID
    pub async fn get(&self, id: i64) -> Result<Account, AccountError> {
        let account = query_as::<_, Account>("SELECT * FROM accounts WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(AccountError::NotFound)?;

        Ok(account)
    }

    /// List active accounts
    pub async fn list_active(&self) -> Result<Vec<Account>, AccountError> {
        let accounts = query_as::<_, Account>(
            "SELECT * FROM accounts WHERE status = 'active' ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(accounts)
    }

    /// Create account
    pub async fn create(
        &self,
        name: String,
        provider: String,
        credentials: String,
        models: Vec<String>,
    ) -> Result<Account, AccountError> {
        let account = query_as::<_, Account>(r#"
            INSERT INTO accounts (name, provider, status, credentials, models, created_at, updated_at)
            VALUES ($1, $2, 'active', $3, $4, NOW(), NOW())
            RETURNING *
            "#)
            .bind(name)
            .bind(provider)
            .bind(credentials)
            .bind(&models)
            .fetch_one(&self.pool)
            .await?;

        Ok(account)
    }

    /// Update account status
    pub async fn update_status(&self, id: i64, status: AccountStatus) -> Result<(), AccountError> {
        let status_str = serde_json::to_string(&status).unwrap();

        query("UPDATE accounts SET status = $1, updated_at = NOW() WHERE id = $2")
            .bind(status_str)
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Delete account
    pub async fn delete(&self, id: i64) -> Result<(), AccountError> {
        self.update_status(id, AccountStatus::Deleted).await
    }

    /// Select account for model with model routing support
    /// 
    /// 模型路由调度逻辑：
    /// 1. 如果分组启用了模型路由且配置了路由规则，优先从路由列表中选择账号
    /// 2. 支持精确匹配和通配符匹配（如 "claude-opus-*"）
    /// 3. 排除已失败的账号 ID
    /// 4. 如果没有路由规则或路由列表中没有可用账号，则返回 None
    pub async fn select_account_for_model_with_exclusions(
        &self,
        group_id: i64,
        requested_model: &str,
        excluded_ids: &HashSet<i64>,
    ) -> Result<Option<Account>, AccountError> {
        // 1. 获取分组的模型路由配置
        let routing_config = self.get_group_model_routing(group_id).await?;
        
        // 2. 如果启用了模型路由，尝试从路由列表中选择账号
        if routing_config.enabled {
            if let Some(account_ids) = Self::match_model_routing(&routing_config.routing, requested_model) {
                // 从路由列表中选择第一个可用的账号
                for account_id in account_ids {
                    if !excluded_ids.contains(&account_id) {
                        if let Ok(account) = self.get(account_id).await {
                            if account.status == "active" {
                                return Ok(Some(account));
                            }
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    /// Get group's model routing configuration
    async fn get_group_model_routing(&self, group_id: i64) -> Result<ModelRoutingConfig, AccountError> {
        let row = query_as::<_, (Option<serde_json::Value>, bool)>(
            "SELECT model_routing, model_routing_enabled FROM groups WHERE id = $1"
        )
        .bind(group_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(AccountError::NotFound)?;

        let routing = row.0
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();

        Ok(ModelRoutingConfig {
            routing,
            enabled: row.1,
        })
    }

    /// Match model against routing configuration
    /// Returns account IDs if matched, None otherwise
    fn match_model_routing(
        routing: &HashMap<String, Vec<i64>>,
        requested_model: &str,
    ) -> Option<Vec<i64>> {
        if requested_model.is_empty() {
            return None;
        }

        // 1. 精确匹配优先
        if let Some(account_ids) = routing.get(requested_model) {
            if !account_ids.is_empty() {
                return Some(account_ids.clone());
            }
        }

        // 2. 通配符匹配（前缀匹配）
        for (pattern, account_ids) in routing {
            if Self::match_model_pattern(pattern, requested_model) && !account_ids.is_empty() {
                return Some(account_ids.clone());
            }
        }

        None
    }

    /// Check if model matches pattern
    /// Supports * wildcard at the end, e.g., "claude-opus-*" matches "claude-opus-4-20250514"
    fn match_model_pattern(pattern: &str, model: &str) -> bool {
        if pattern == model {
            return true;
        }

        // Handle * wildcard (only at the end)
        if pattern.ends_with('*') {
            let prefix = &pattern[..pattern.len() - 1];
            return model.starts_with(prefix);
        }

        false
    }

    /// Get accounts by group ID
    pub async fn get_accounts_by_group(&self, group_id: i64) -> Result<Vec<Account>, AccountError> {
        let accounts = query_as::<_, Account>(
            r#"
            SELECT a.* 
            FROM accounts a
            JOIN account_groups ag ON a.id = ag.account_id
            WHERE ag.group_id = $1 AND a.status = 'active'
            ORDER BY a.created_at DESC
            "#,
        )
        .bind(group_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(accounts)
    }
}

/// Model routing configuration
struct ModelRoutingConfig {
    routing: HashMap<String, Vec<i64>>,
    enabled: bool,
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_account_service() {
        // Test would require database connection
    }
}
