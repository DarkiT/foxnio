//! Model Sync Service
//!
//! 模型自动同步服务，从各 AI 服务商获取最新模型信息

#![allow(dead_code)]

use anyhow::{bail, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::model_registry::{CreateModelRequest, ModelRegistry, RuntimeModelConfig};

/// 模型同步服务
pub struct ModelSyncService {
    db: sea_orm::DatabaseConnection,
    model_registry: Arc<ModelRegistry>,
    http_client: reqwest::Client,
    sync_state: Arc<RwLock<SyncState>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncState {
    pub last_sync: Option<DateTime<Utc>>,
    pub last_success: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub in_progress: bool,
    pub provider_status: HashMap<String, ProviderSyncStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderSyncStatus {
    pub provider: String,
    pub last_sync: Option<DateTime<Utc>>,
    pub models_count: usize,
    pub last_error: Option<String>,
}

/// 同步结果
#[derive(Debug, Serialize, Deserialize)]
pub struct SyncResult {
    pub provider: String,
    pub new_models: Vec<String>,
    pub updated_models: Vec<String>,
    pub deprecated_models: Vec<String>,
    pub price_changes: Vec<PriceChange>,
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceChange {
    pub model_name: String,
    pub old_input_price: f64,
    pub new_input_price: f64,
    pub old_output_price: f64,
    pub new_output_price: f64,
}

/// OpenAI 模型列表响应
#[derive(Debug, Deserialize)]
struct OpenAIModelsResponse {
    data: Vec<OpenAIModel>,
}

#[derive(Debug, Deserialize)]
struct OpenAIModel {
    id: String,
    object: String,
    created: i64,
    owned_by: String,
}

/// Anthropic 模型信息（从文档或 API 获取）
#[derive(Debug, Serialize, Deserialize)]
struct AnthropicModelInfo {
    name: String,
    display_name: String,
    input_price: f64,
    output_price: f64,
    context_window: i32,
    max_tokens: i32,
}

impl ModelSyncService {
    pub fn new(db: sea_orm::DatabaseConnection, model_registry: Arc<ModelRegistry>) -> Self {
        Self {
            db,
            model_registry,
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap(),
            sync_state: Arc::new(RwLock::new(SyncState::default())),
        }
    }
    
    /// 同步所有提供商的模型
    pub async fn sync_all(&self) -> Result<Vec<SyncResult>> {
        let mut state = self.sync_state.write().await;
        if state.in_progress {
            bail!("Sync already in progress");
        }
        state.in_progress = true;
        drop(state);
        
        let mut results = Vec::new();
        
        // 同步各提供商
        for provider in &["openai", "anthropic", "google", "deepseek", "mistral", "cohere"] {
            match self.sync_provider(provider).await {
                Ok(result) => {
                    results.push(result);
                }
                Err(e) => {
                    warn!("Failed to sync provider {}: {}", provider, e);
                    results.push(SyncResult {
                        provider: provider.to_string(),
                        new_models: vec![],
                        updated_models: vec![],
                        deprecated_models: vec![],
                        price_changes: vec![],
                        errors: vec![e.to_string()],
                    });
                }
            }
        }
        
        let mut state = self.sync_state.write().await;
        state.in_progress = false;
        state.last_sync = Some(Utc::now());
        
        Ok(results)
    }
    
    /// 同步单个提供商的模型
    pub async fn sync_provider(&self, provider: &str) -> Result<SyncResult> {
        info!("Syncing models for provider: {}", provider);
        
        let result = match provider {
            "openai" => self.sync_openai().await?,
            "anthropic" => self.sync_anthropic().await?,
            "google" => self.sync_google().await?,
            "deepseek" => self.sync_deepseek().await?,
            "mistral" => self.sync_mistral().await?,
            "cohere" => self.sync_cohere().await?,
            _ => bail!("Unknown provider: {}", provider),
        };
        
        // 更新提供商状态
        let mut state = self.sync_state.write().await;
        state.provider_status.insert(
            provider.to_string(),
            ProviderSyncStatus {
                provider: provider.to_string(),
                last_sync: Some(Utc::now()),
                models_count: result.new_models.len() + result.updated_models.len(),
                last_error: if result.errors.is_empty() { None } else { Some(result.errors.join(", ")) },
            },
        );
        
        Ok(result)
    }
    
    /// 从 OpenAI 同步模型
    async fn sync_openai(&self) -> Result<SyncResult> {
        // 1. 调用 OpenAI API 获取模型列表
        let url = "https://api.openai.com/v1/models";
        let response = self.http_client
            .get(url)
            .header("Authorization", "Bearer YOUR_API_KEY") // 需要配置
            .send()
            .await?;
        
        if !response.status().is_success() {
            bail!("OpenAI API error: {}", response.status());
        }
        
        let models: OpenAIModelsResponse = response.json().await?;
        
        // 2. 过滤出聊天模型
        let chat_models: Vec<String> = models.data
            .into_iter()
            .filter(|m| {
                m.id.starts_with("gpt-") || 
                m.id.starts_with("chatgpt-") ||
                m.id.starts_with("o1-") ||
                m.id.starts_with("o3-")
            })
            .map(|m| m.id)
            .collect();
        
        // 3. 对比数据库中的模型
        let existing_models = self.model_registry.list_models().await?;
        let existing_names: Vec<String> = existing_models.iter()
            .filter(|m| m.provider == "openai")
            .map(|m| m.name.clone())
            .collect();
        
        let new_models: Vec<String> = chat_models.iter()
            .filter(|m| !existing_names.contains(m))
            .cloned()
            .collect();
        
        // 4. 添加新模型
        for model_name in &new_models {
            let default_config = self.get_default_model_config("openai", model_name)?;
            match self.model_registry.create(default_config).await {
                Ok(_) => info!("Added new OpenAI model: {}", model_name),
                Err(e) => warn!("Failed to add model {}: {}", model_name, e),
            }
        }
        
        Ok(SyncResult {
            provider: "openai".to_string(),
            new_models,
            updated_models: vec![],
            deprecated_models: vec![],
            price_changes: vec![],
            errors: vec![],
        })
    }
    
    /// 从 Anthropic 同步模型（基于已知模型列表）
    async fn sync_anthropic(&self) -> Result<SyncResult> {
        // Anthropic 没有公开的模型列表 API，使用已知模型
        let known_models = vec![
            AnthropicModelInfo {
                name: "claude-3-5-sonnet-20241022".into(),
                display_name: "Claude 3.5 Sonnet".into(),
                input_price: 3.0,
                output_price: 15.0,
                context_window: 200000,
                max_tokens: 8192,
            },
            AnthropicModelInfo {
                name: "claude-3-5-haiku-20241022".into(),
                display_name: "Claude 3.5 Haiku".into(),
                input_price: 0.8,
                output_price: 4.0,
                context_window: 200000,
                max_tokens: 8192,
            },
            AnthropicModelInfo {
                name: "claude-3-opus-20240229".into(),
                display_name: "Claude 3 Opus".into(),
                input_price: 15.0,
                output_price: 75.0,
                context_window: 200000,
                max_tokens: 4096,
            },
        ];
        
        // 对比并更新
        let mut new_models = vec![];
        let mut price_changes = vec![];
        
        let existing_models = self.model_registry.list_models().await?;
        let existing: HashMap<String, &RuntimeModelConfig> = existing_models.iter()
            .filter(|m| m.provider == "anthropic")
            .map(|m| (m.name.clone(), m))
            .collect();
        
        for model_info in known_models {
            if let Some(existing) = existing.get(&model_info.name) {
                // 检查价格变化
                if (existing.input_price - model_info.input_price).abs() > 0.01 ||
                   (existing.output_price - model_info.output_price).abs() > 0.01 {
                    price_changes.push(PriceChange {
                        model_name: model_info.name.clone(),
                        old_input_price: existing.input_price,
                        new_input_price: model_info.input_price,
                        old_output_price: existing.output_price,
                        new_output_price: model_info.output_price,
                    });
                }
            } else {
                // 新模型
                new_models.push(model_info.name.clone());
                let req = CreateModelRequest {
                    name: model_info.name,
                    aliases: vec![],
                    provider: "anthropic".into(),
                    api_name: None,
                    display_name: Some(model_info.display_name),
                    input_price: model_info.input_price,
                    output_price: model_info.output_price,
                    max_tokens: model_info.max_tokens,
                    context_window: model_info.context_window,
                    max_concurrent: 5,
                    fallback_models: vec![],
                    capabilities: Default::default(),
                    supports_streaming: true,
                    supports_function_calling: true,
                    supports_vision: true,
                    enabled: true,
                    priority: 100,
                };
                self.model_registry.create(req).await?;
            }
        }
        
        Ok(SyncResult {
            provider: "anthropic".to_string(),
            new_models,
            updated_models: vec![],
            deprecated_models: vec![],
            price_changes,
            errors: vec![],
        })
    }
    
    /// 从 Google 同步模型
    async fn sync_google(&self) -> Result<SyncResult> {
        // Google Gemini API 需要调用 list models endpoint
        // https://generativelanguage.googleapis.com/v1beta/models
        
        // 简化实现：使用已知模型
        let known_models = vec!["gemini-2.0-flash-exp", "gemini-1.5-pro", "gemini-1.5-flash"];
        
        let existing_models = self.model_registry.list_models().await?;
        let existing_names: Vec<String> = existing_models.iter()
            .filter(|m| m.provider == "google")
            .map(|m| m.name.clone())
            .collect();
        
        let new_models: Vec<String> = known_models.iter()
            .filter(|m| !existing_names.contains(m))
            .cloned()
            .map(|s| s.to_string())
            .collect();
        
        for model_name in &new_models {
            let default_config = self.get_default_model_config("google", model_name)?;
            self.model_registry.create(default_config).await?;
        }
        
        Ok(SyncResult {
            provider: "google".to_string(),
            new_models,
            updated_models: vec![],
            deprecated_models: vec![],
            price_changes: vec![],
            errors: vec![],
        })
    }
    
    /// 从 DeepSeek 同步模型
    async fn sync_deepseek(&self) -> Result<SyncResult> {
        // DeepSeek 模型列表
        let known_models = vec!["deepseek-chat", "deepseek-reasoner"];
        
        let existing_models = self.model_registry.list_models().await?;
        let existing_names: Vec<String> = existing_models.iter()
            .filter(|m| m.provider == "deepseek")
            .map(|m| m.name.clone())
            .collect();
        
        let new_models: Vec<String> = known_models.iter()
            .filter(|m| !existing_names.contains(m))
            .cloned()
            .map(|s| s.to_string())
            .collect();
        
        for model_name in &new_models {
            let default_config = self.get_default_model_config("deepseek", model_name)?;
            self.model_registry.create(default_config).await?;
        }
        
        Ok(SyncResult {
            provider: "deepseek".to_string(),
            new_models,
            updated_models: vec![],
            deprecated_models: vec![],
            price_changes: vec![],
            errors: vec![],
        })
    }
    
    /// 从 Mistral 同步模型
    async fn sync_mistral(&self) -> Result<SyncResult> {
        // TODO: 实现 Mistral 模型同步
        Ok(SyncResult {
            provider: "mistral".to_string(),
            new_models: vec![],
            updated_models: vec![],
            deprecated_models: vec![],
            price_changes: vec![],
            errors: vec!["Not implemented".into()],
        })
    }
    
    /// 从 Cohere 同步模型
    async fn sync_cohere(&self) -> Result<SyncResult> {
        // TODO: 实现 Cohere 模型同步
        Ok(SyncResult {
            provider: "cohere".to_string(),
            new_models: vec![],
            updated_models: vec![],
            deprecated_models: vec![],
            price_changes: vec![],
            errors: vec!["Not implemented".into()],
        })
    }
    
    /// 获取默认模型配置
    fn get_default_model_config(&self, provider: &str, model_name: &str) -> Result<CreateModelRequest> {
        Ok(CreateModelRequest {
            name: model_name.to_string(),
            aliases: vec![],
            provider: provider.to_string(),
            api_name: None,
            display_name: Some(model_name.to_string()),
            input_price: 0.0, // 需要从价格 API 获取
            output_price: 0.0,
            max_tokens: 4096,
            context_window: 128000,
            max_concurrent: 5,
            fallback_models: vec![],
            capabilities: Default::default(),
            supports_streaming: true,
            supports_function_calling: false,
            supports_vision: false,
            enabled: true,
            priority: 100,
        })
    }
    
    /// 获取同步状态
    pub async fn get_sync_state(&self) -> SyncState {
        self.sync_state.read().await.clone()
    }
    
    /// 启动定时同步任务
    pub async fn start_sync_scheduler(self: Arc<Self>, interval: std::time::Duration) {
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;
                
                info!("Starting scheduled model sync");
                match self.sync_all().await {
                    Ok(results) => {
                        let total_new: usize = results.iter().map(|r| r.new_models.len()).sum();
                        let total_updated: usize = results.iter().map(|r| r.updated_models.len()).sum();
                        info!("Model sync completed: {} new, {} updated", total_new, total_updated);
                    }
                    Err(e) => {
                        warn!("Scheduled model sync failed: {}", e);
                    }
                }
            }
        });
    }
}

impl Default for SyncState {
    fn default() -> Self {
        Self {
            last_sync: None,
            last_success: None,
            last_error: None,
            in_progress: false,
            provider_status: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sync_state_default() {
        let state = SyncState::default();
        assert!(!state.in_progress);
        assert!(state.last_sync.is_none());
    }
}
