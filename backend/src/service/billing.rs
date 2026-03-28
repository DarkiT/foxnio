//! 计费服务 - 完整实现

#![allow(dead_code)]
use anyhow::Result;
use chrono::{DateTime, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

use super::user::UserService;
use crate::entity::usages;

/// 使用记录参数
#[derive(Debug, Clone)]
pub struct RecordUsageParams {
    pub user_id: Uuid,
    pub api_key_id: Uuid,
    pub model: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub success: bool,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct UsageRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub model: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cost: i64,
    pub success: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct UserStats {
    pub total_requests: i64,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_cost: i64,
}

pub struct BillingService {
    db: DatabaseConnection,
    rate_multiplier: f64,
}

impl BillingService {
    pub fn new(db: DatabaseConnection, rate_multiplier: f64) -> Self {
        Self {
            db,
            rate_multiplier,
        }
    }

    /// 记录用量
    pub async fn record_usage(&self, params: RecordUsageParams) -> Result<UsageRecord> {
        let cost = self.calculate_cost(&params.model, params.input_tokens, params.output_tokens);

        let usage = usages::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(params.user_id),
            api_key_id: Set(params.api_key_id),
            account_id: Set(None),
            model: Set(params.model),
            input_tokens: Set(params.input_tokens),
            output_tokens: Set(params.output_tokens),
            cost: Set(cost),
            request_id: Set(None),
            success: Set(params.success),
            error_message: Set(params.error_message),
            metadata: Set(None),
            created_at: Set(Utc::now()),
        };

        let usage = usage.insert(&self.db).await?;

        // 扣减余额
        if cost > 0 {
            let user_service = UserService::new(self.db.clone(), String::new(), 24);
            user_service.update_balance(params.user_id, -cost).await?;
        }

        Ok(UsageRecord {
            id: usage.id,
            user_id: usage.user_id,
            model: usage.model,
            input_tokens: usage.input_tokens,
            output_tokens: usage.output_tokens,
            cost: usage.cost,
            success: usage.success,
            created_at: usage.created_at,
        })
    }

    /// 计算费用（单位：分）
    pub fn calculate_cost(&self, model: &str, input_tokens: i64, output_tokens: i64) -> i64 {
        Self::calculate_cost_static(model, input_tokens, output_tokens, self.rate_multiplier)
    }

    /// 计算费用（静态方法，用于测试）
    pub fn calculate_cost_static(
        model: &str,
        input_tokens: i64,
        output_tokens: i64,
        rate_multiplier: f64,
    ) -> i64 {
        // 模型定价（每 1K tokens，单位：分）
        let (input_rate, output_rate) = match model {
            // Claude 3
            "claude-3-opus-20240229" => (1500, 7500),
            "claude-3-sonnet-20240229" => (300, 1500),
            "claude-3-haiku-20240307" => (25, 125),
            "claude-3-5-sonnet-20241022" => (300, 1500),

            // GPT-4
            "gpt-4-turbo" | "gpt-4-turbo-preview" | "gpt-4-0125-preview" => (1000, 3000),
            "gpt-4" | "gpt-4-0613" => (3000, 6000),
            "gpt-4o" | "gpt-4o-2024-11-20" => (250, 1000),
            "gpt-4o-mini" => (15, 60),
            "gpt-3.5-turbo" | "gpt-3.5-turbo-0125" => (50, 150),

            // Gemini
            "gemini-1.5-pro" | "gemini-1.5-pro-latest" => (350, 1050),
            "gemini-1.5-flash" | "gemini-1.5-flash-latest" => (35, 105),
            "gemini-2.0-flash-exp" => (0, 0), // 免费

            // DeepSeek
            "deepseek-chat" => (10, 30),
            "deepseek-reasoner" => (55, 220),

            // 默认
            _ => (100, 300),
        };

        let input_cost =
            (input_tokens as f64 / 1000.0 * input_rate as f64 * rate_multiplier) as i64;
        let output_cost =
            (output_tokens as f64 / 1000.0 * output_rate as f64 * rate_multiplier) as i64;

        input_cost + output_cost
    }

    /// 获取用户用量统计
    pub async fn get_user_stats(&self, user_id: Uuid, days: i32) -> Result<UserStats> {
        let start_time = Utc::now() - chrono::Duration::days(days as i64);

        let usages = usages::Entity::find()
            .filter(usages::Column::UserId.eq(user_id))
            .filter(usages::Column::CreatedAt.gte(start_time))
            .all(&self.db)
            .await?;

        let total_requests = usages.len() as i64;
        let total_input_tokens = usages.iter().map(|u| u.input_tokens).sum();
        let total_output_tokens = usages.iter().map(|u| u.output_tokens).sum();
        let total_cost = usages.iter().map(|u| u.cost).sum();

        Ok(UserStats {
            total_requests,
            total_input_tokens,
            total_output_tokens,
            total_cost,
        })
    }

    /// 获取全局统计（管理后台）
    pub async fn get_global_stats(&self, days: i32) -> Result<UserStats> {
        let start_time = Utc::now() - chrono::Duration::days(days as i64);

        let usages = usages::Entity::find()
            .filter(usages::Column::CreatedAt.gte(start_time))
            .all(&self.db)
            .await?;

        let total_requests = usages.len() as i64;
        let total_input_tokens = usages.iter().map(|u| u.input_tokens).sum();
        let total_output_tokens = usages.iter().map(|u| u.output_tokens).sum();
        let total_cost = usages.iter().map(|u| u.cost).sum();

        Ok(UserStats {
            total_requests,
            total_input_tokens,
            total_output_tokens,
            total_cost,
        })
    }
}
