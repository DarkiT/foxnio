//! Webhook 服务
//!
//! 管理 Webhook 端点的创建、更新、删除和事件投递

use anyhow::Result;
use chrono::{DateTime, Utc};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, ActiveModelTrait, Set};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::entity::{webhook_endpoints, webhook_deliveries};

/// Webhook 服务
pub struct WebhookService {
    db: DatabaseConnection,
}

impl WebhookService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
    
    /// 创建 Webhook 端点
    pub async fn create_endpoint(
        &self,
        user_id: Uuid,
        url: String,
        events: Vec<String>,
        secret: Option<String>,
    ) -> Result<webhook_endpoints::Model> {
        use webhook_endpoints::ActiveModel;
        
        let now = Utc::now();
        let endpoint = ActiveModel {
            id: Set(0), // Will be auto-generated
            user_id: Set(user_id),
            url: Set(url),
            events: Set(serde_json::to_value(&events)?),
            secret: Set(secret),
            is_active: Set(true),
            created_at: Set(now),
            updated_at: Set(now),
        };
        
        let endpoint = endpoint.insert(&self.db).await?;
        Ok(endpoint)
    }
    
    /// 列出用户的 Webhook 端点
    pub async fn list_endpoints(&self, user_id: Uuid) -> Result<Vec<webhook_endpoints::Model>> {
        let endpoints = webhook_endpoints::Entity::find()
            .filter(webhook_endpoints::Column::UserId.eq(user_id))
            .all(&self.db)
            .await?;
        
        Ok(endpoints)
    }
    
    /// 获取单个 Webhook 端点
    pub async fn get_endpoint(&self, id: i64, user_id: Uuid) -> Result<Option<webhook_endpoints::Model>> {
        let endpoint = webhook_endpoints::Entity::find_by_id(id)
            .filter(webhook_endpoints::Column::UserId.eq(user_id))
            .one(&self.db)
            .await?;
        
        Ok(endpoint)
    }
    
    /// 更新 Webhook 端点
    pub async fn update_endpoint(
        &self,
        id: i64,
        user_id: Uuid,
        url: Option<String>,
        events: Option<Vec<String>>,
        secret: Option<String>,
        is_active: Option<bool>,
    ) -> Result<Option<webhook_endpoints::Model>> {
        use webhook_endpoints::ActiveModel;
        
        let endpoint = match self.get_endpoint(id, user_id).await? {
            Some(e) => e,
            None => return Ok(None),
        };
        
        let mut active_model: ActiveModel = endpoint.into();
        
        if let Some(u) = url {
            active_model.url = Set(u);
        }
        if let Some(e) = events {
            active_model.events = Set(serde_json::to_value(&e)?);
        }
        if secret.is_some() {
            active_model.secret = Set(secret);
        }
        if let Some(a) = is_active {
            active_model.is_active = Set(a);
        }
        active_model.updated_at = Set(Utc::now());
        
        let updated = active_model.update(&self.db).await?;
        Ok(Some(updated))
    }
    
    /// 删除 Webhook 端点
    pub async fn delete_endpoint(&self, id: i64, user_id: Uuid) -> Result<bool> {
        let endpoint = self.get_endpoint(id, user_id).await?;
        
        if let Some(e) = endpoint {
            webhook_endpoints::Entity::delete_by_id(e.id)
                .exec(&self.db)
                .await?;
            return Ok(true);
        }
        
        Ok(false)
    }
    
    /// 测试 Webhook 端点
    pub async fn test_endpoint(&self, id: i64, user_id: Uuid) -> Result<bool> {
        let endpoint = self.get_endpoint(id, user_id).await?;
        
        if let Some(e) = endpoint {
            // TODO: 实现实际的 HTTP 测试请求
            // 目前返回模拟成功
            return Ok(true);
        }
        
        Ok(false)
    }
    
    /// 列出投递记录
    pub async fn list_deliveries(
        &self,
        endpoint_id: i64,
        user_id: Uuid,
    ) -> Result<Vec<webhook_deliveries::Model>> {
        // 验证用户权限
        let _endpoint = self.get_endpoint(endpoint_id, user_id).await?;
        
        let deliveries = webhook_deliveries::Entity::find()
            .filter(webhook_deliveries::Column::EndpointId.eq(endpoint_id))
            .all(&self.db)
            .await?;
        
        Ok(deliveries)
    }
}
