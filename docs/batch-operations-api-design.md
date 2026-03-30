# FoxNIO 批量操作 API 设计

## 概述

批量操作 API 允许用户一次性创建、更新、删除多个资源，提高操作效率，减少 API 调用次数。

**优先级**: P1（中）
**预计工作量**: 2-3 天
**参考项目**: sub2api 的 BulkUpdateAccounts 实现

## 功能需求

### 1. 支持的批量操作

| 操作 | 资源类型 | 描述 |
|------|---------|------|
| 批量创建 | API Keys | 一次性创建多个 API Key |
| 批量创建 | Users | 批量导入用户 |
| 批量创建 | Models | 批量导入模型配置 |
| 批量更新 | API Keys | 批量更新 API Key 配置 |
| 批量更新 | Accounts | 批量更新账户配置 |
| 批量更新 | Models | 批量更新模型配置 |
| 批量删除 | API Keys | 批量删除 API Key |
| 批量删除 | Users | 批量删除用户 |
| 批量删除 | Models | 批量删除模型配置 |

### 2. 设计原则

1. **幂等性**: 相同请求多次执行结果一致
2. **事务性**: 要么全部成功，要么全部回滚（可配置）
3. **可追踪**: 返回每个操作的详细结果
4. **限流**: 限制单次批量操作的数量

## API 端点设计

### 1. 批量创建 API Keys

```rust
POST /api/v1/admin/api-keys/batch-create
{
    "keys": [
        {
            "name": "key-1",
            "permissions": ["chat", "completion"],
            "rate_limit": 100,
            "expires_at": "2026-12-31T23:59:59Z"
        },
        {
            "name": "key-2",
            "permissions": ["chat"],
            "rate_limit": 50
        }
    ],
    "stop_on_error": false  // 遇到错误是否停止
}

Response 200:
{
    "total": 2,
    "created": 2,
    "failed": 0,
    "results": [
        {
            "index": 0,
            "success": true,
            "key": {
                "id": 1,
                "name": "key-1",
                "key": "foxnio-xxx...",
                "created_at": "2026-03-30T08:20:00Z"
            }
        },
        {
            "index": 1,
            "success": true,
            "key": {
                "id": 2,
                "name": "key-2",
                "key": "foxnio-yyy...",
                "created_at": "2026-03-30T08:20:00Z"
            }
        }
    ]
}
```

### 2. 批量更新账户

```rust
POST /api/v1/admin/accounts/batch-update
{
    "account_ids": [1, 2, 3, 4, 5],
    "updates": {
        "group_ids": [10, 20],           // 分配到多个组
        "enabled": true,
        "priority": 100,
        "max_concurrent": 10,
        "tags": ["production", "vip"]
    },
    "skip_mixed_channel_check": false,   // 是否跳过混合渠道检查
    "stop_on_error": true
}

Response 200:
{
    "total": 5,
    "updated": 4,
    "failed": 1,
    "results": [
        {
            "account_id": 1,
            "success": true,
            "message": "Updated successfully"
        },
        {
            "account_id": 2,
            "success": true,
            "message": "Updated successfully"
        },
        {
            "account_id": 3,
            "success": true,
            "message": "Updated successfully"
        },
        {
            "account_id": 4,
            "success": true,
            "message": "Updated successfully"
        },
        {
            "account_id": 5,
            "success": false,
            "error": "Account not found"
        }
    ]
}
```

### 3. 批量导入用户

```rust
POST /api/v1/admin/users/batch-import
Content-Type: multipart/form-data

file: users.csv

// CSV 格式：
// email,password,name,role,quota
// user1@example.com,pass123,User One,user,1000
// user2@example.com,pass456,User Two,user,2000

Response 200:
{
    "total": 100,
    "imported": 98,
    "skipped": 1,  // 邮箱已存在
    "failed": 1,
    "results": [
        {
            "row": 1,
            "success": true,
            "user": {
                "id": 1,
                "email": "user1@example.com",
                "name": "User One"
            }
        },
        {
            "row": 50,
            "success": false,
            "error": "Invalid email format"
        }
    ],
    "errors": [
        {
            "row": 50,
            "error": "Invalid email format"
        }
    ]
}
```

### 4. 批量删除资源

```rust
POST /api/v1/admin/api-keys/batch-delete
{
    "key_ids": [1, 2, 3, 4, 5],
    "force": false  // 是否强制删除（即使有依赖）
}

Response 200:
{
    "total": 5,
    "deleted": 4,
    "failed": 1,
    "results": [
        {
            "key_id": 1,
            "success": true
        },
        // ...
        {
            "key_id": 5,
            "success": false,
            "error": "Key is in use by active request"
        }
    ]
}
```

### 5. 批量导入模型配置

```rust
POST /api/v1/admin/models/batch-import
Content-Type: application/json

{
    "models": [
        {
            "name": "gpt-4o-mini",
            "provider": "openai",
            "api_name": "gpt-4o-mini",
            "input_price": 0.15,
            "output_price": 0.6,
            "max_tokens": 16384,
            "context_window": 128000,
            "capabilities": {
                "chat": true,
                "vision": true
            }
        },
        // ... 更多模型
    ],
    "update_existing": true,  // 是否更新已存在的模型
    "stop_on_error": false
}

Response 200:
{
    "total": 50,
    "created": 30,
    "updated": 18,
    "skipped": 1,
    "failed": 1,
    "results": [...]
}
```

## 核心服务实现

### 1. BatchOperationService

```rust
// backend/src/service/batch.rs

pub struct BatchOperationService {
    db: DatabaseConnection,
    api_key_service: Arc<ApiKeyService>,
    user_service: Arc<UserService>,
    account_service: Arc<AccountService>,
    model_service: Arc<ModelRegistry>,
}

/// 批量操作结果
#[derive(Serialize)]
pub struct BatchResult<T> {
    pub total: usize,
    pub success: usize,
    pub failed: usize,
    pub results: Vec<BatchItemResult<T>>,
}

#[derive(Serialize)]
pub struct BatchItemResult<T> {
    pub index: usize,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl BatchOperationService {
    /// 批量创建 API Keys
    pub async fn batch_create_api_keys(
        &self,
        requests: Vec<CreateApiKeyRequest>,
        stop_on_error: bool,
    ) -> Result<BatchResult<ApiKey>> {
        let mut results = Vec::with_capacity(requests.len());
        let mut success_count = 0;
        let mut failed_count = 0;
        
        // 使用事务（如果 stop_on_error = true）
        let txn = if stop_on_error {
            Some(self.db.begin().await?)
        } else {
            None
        };
        
        for (index, req) in requests.into_iter().enumerate() {
            match self.api_key_service.create_with_txn(req.clone(), txn.as_ref()).await {
                Ok(key) => {
                    success_count += 1;
                    results.push(BatchItemResult {
                        index,
                        success: true,
                        data: Some(key),
                        error: None,
                    });
                }
                Err(e) => {
                    failed_count += 1;
                    results.push(BatchItemResult {
                        index,
                        success: false,
                        data: None,
                        error: Some(e.to_string()),
                    });
                    
                    if stop_on_error {
                        if let Some(txn) = txn {
                            txn.rollback().await?;
                        }
                        return Ok(BatchResult {
                            total: results.len(),
                            success: success_count,
                            failed: failed_count,
                            results,
                        });
                    }
                }
            }
        }
        
        if let Some(txn) = txn {
            txn.commit().await?;
        }
        
        Ok(BatchResult {
            total: results.len(),
            success: success_count,
            failed: failed_count,
            results,
        })
    }
    
    /// 批量更新账户（参考 sub2api）
    pub async fn batch_update_accounts(
        &self,
        account_ids: Vec<i64>,
        updates: BulkAccountUpdates,
        skip_mixed_channel_check: bool,
        stop_on_error: bool,
    ) -> Result<BatchResult<()>> {
        let mut results = Vec::with_capacity(account_ids.len());
        let mut success_count = 0;
        let mut failed_count = 0;
        
        // 预加载账户信息（用于混合渠道检查）
        let accounts = self.account_service.get_by_ids(&account_ids).await?;
        let platform_by_id: HashMap<i64, String> = accounts
            .into_iter()
            .filter_map(|a| Some((a.id, a.platform)))
            .collect();
        
        // 检查混合渠道
        if !skip_mixed_channel_check && updates.group_ids.is_some() {
            let group_ids = updates.group_ids.as_ref().unwrap();
            let has_mixed = self.check_mixed_channels(&account_ids, &platform_by_id, group_ids).await?;
            if has_mixed {
                return Err(anyhow::anyhow!("Cannot assign accounts from different platforms to the same group"));
            }
        }
        
        // 执行批量更新
        for account_id in account_ids {
            match self.account_service.update(account_id, updates.clone()).await {
                Ok(_) => {
                    success_count += 1;
                    results.push(BatchItemResult {
                        index: account_id as usize,
                        success: true,
                        data: Some(()),
                        error: None,
                    });
                }
                Err(e) => {
                    failed_count += 1;
                    results.push(BatchItemResult {
                        index: account_id as usize,
                        success: false,
                        data: None,
                        error: Some(e.to_string()),
                    });
                    
                    if stop_on_error {
                        break;
                    }
                }
            }
        }
        
        Ok(BatchResult {
            total: results.len(),
            success: success_count,
            failed: failed_count,
            results,
        })
    }
    
    /// 批量导入用户（CSV）
    pub async fn batch_import_users_csv(
        &self,
        csv_content: &str,
    ) -> Result<BatchResult<User>> {
        let mut reader = csv::Reader::from_reader(csv_content.as_bytes());
        let mut results = Vec::new();
        let mut success_count = 0;
        let mut failed_count = 0;
        
        for (index, record) in reader.deserialize().enumerate() {
            let record: UserImportRecord = match record {
                Ok(r) => r,
                Err(e) => {
                    failed_count += 1;
                    results.push(BatchItemResult {
                        index,
                        success: false,
                        data: None,
                        error: Some(format!("CSV parse error: {}", e)),
                    });
                    continue;
                }
            };
            
            match self.user_service.create(record.into()).await {
                Ok(user) => {
                    success_count += 1;
                    results.push(BatchItemResult {
                        index,
                        success: true,
                        data: Some(user),
                        error: None,
                    });
                }
                Err(e) => {
                    failed_count += 1;
                    results.push(BatchItemResult {
                        index,
                        success: false,
                        data: None,
                        error: Some(e.to_string()),
                    });
                }
            }
        }
        
        Ok(BatchResult {
            total: results.len(),
            success: success_count,
            failed: failed_count,
            results,
        })
    }
}

/// 批量更新字段（参考 sub2api）
#[derive(Debug, Deserialize)]
pub struct BulkAccountUpdates {
    pub group_ids: Option<Vec<i64>>,
    pub enabled: Option<bool>,
    pub priority: Option<i32>,
    pub max_concurrent: Option<i32>,
    pub tags: Option<Vec<String>>,
    pub rate_limit: Option<i32>,
    pub billing_rate_multiplier: Option<f64>,
}
```

### 2. Handler 实现

```rust
// backend/src/handler/batch.rs

pub async fn batch_create_api_keys(
    State(services): State<AppState>,
    Json(req): Json<BatchCreateApiKeysRequest>,
) -> Result<Json<BatchResult<ApiKey>>, ApiError> {
    // 限制批量大小
    if req.keys.len() > 100 {
        return Err(ApiError::BadRequest("Maximum 100 items per batch".into()));
    }
    
    let result = services.batch
        .batch_create_api_keys(req.keys, req.stop_on_error.unwrap_or(false))
        .await?;
    
    Ok(Json(result))
}

pub async fn batch_update_accounts(
    State(services): State<AppState>,
    Json(req): Json<BatchUpdateAccountsRequest>,
) -> Result<Json<BatchResult<()>>, ApiError> {
    // 限制批量大小
    if req.account_ids.len() > 500 {
        return Err(ApiError::BadRequest("Maximum 500 accounts per batch".into()));
    }
    
    let result = services.batch
        .batch_update_accounts(
            req.account_ids,
            req.updates,
            req.skip_mixed_channel_check.unwrap_or(false),
            req.stop_on_error.unwrap_or(true),
        )
        .await?;
    
    Ok(Json(result))
}

pub async fn batch_import_users(
    State(services): State<AppState>,
    Multipart(form): Multipart,
) -> Result<Json<BatchResult<User>>, ApiError> {
    let file = form.get_file("file").ok_or(ApiError::BadRequest("Missing file".into()))?;
    
    // 限制文件大小
    if file.size() > 10 * 1024 * 1024 {
        return Err(ApiError::BadRequest("File size exceeds 10MB".into()));
    }
    
    let csv_content = String::from_utf8(file.data)?;
    let result = services.batch.batch_import_users_csv(&csv_content).await?;
    
    Ok(Json(result))
}
```

## 限流与配额

### 1. 批量大小限制

```rust
pub const MAX_BATCH_SIZE: usize = match resource_type {
    ResourceType::ApiKey => 100,
    ResourceType::User => 500,
    ResourceType::Account => 500,
    ResourceType::Model => 100,
};
```

### 2. 速率限制

```rust
// 每个 IP 每分钟最多 10 次批量操作
pub const BATCH_RATE_LIMIT: RateLimit = RateLimit {
    requests: 10,
    window: Duration::from_secs(60),
};
```

## 测试计划

### 单元测试

```rust
#[tokio::test]
async fn test_batch_create_api_keys() {
    let service = create_test_service().await;
    
    let requests = vec![
        CreateApiKeyRequest { name: "key1".into(), ..Default::default() },
        CreateApiKeyRequest { name: "key2".into(), ..Default::default() },
    ];
    
    let result = service.batch_create_api_keys(requests, false).await.unwrap();
    
    assert_eq!(result.total, 2);
    assert_eq!(result.success, 2);
    assert_eq!(result.failed, 0);
}

#[tokio::test]
async fn test_batch_create_with_error() {
    let service = create_test_service().await;
    
    let requests = vec![
        CreateApiKeyRequest { name: "key1".into(), ..Default::default() },
        CreateApiKeyRequest { name: "".into(), ..Default::default() }, // 空名称，会失败
        CreateApiKeyRequest { name: "key3".into(), ..Default::default() },
    ];
    
    // stop_on_error = false
    let result = service.batch_create_api_keys(requests.clone(), false).await.unwrap();
    assert_eq!(result.success, 2);
    assert_eq!(result.failed, 1);
    
    // stop_on_error = true
    let result = service.batch_create_api_keys(requests, true).await.unwrap();
    assert_eq!(result.success, 1); // 第二个失败后停止
}
```

## 实施步骤

### Phase 1: 核心功能（1.5天）
1. 实现 BatchOperationService
2. 实现批量创建 API Keys
3. 实现批量更新 Accounts
4. 添加限流和验证

### Phase 2: 扩展功能（1天）
1. 实现批量导入用户（CSV）
2. 实现批量导入模型
3. 实现批量删除

### Phase 3: 测试与文档（0.5天）
1. 编写单元测试
2. 编写 API 文档
3. 添加使用示例

## 监控指标

```rust
lazy_static! {
    static ref BATCH_OPERATIONS_TOTAL: CounterVec = register_counter_vec!(
        "foxnio_batch_operations_total",
        "Total batch operations",
        &["resource_type", "operation"]
    ).unwrap();
    
    static ref BATCH_OPERATION_SIZE: HistogramVec = register_histogram_vec!(
        "foxnio_batch_operation_size",
        "Batch operation size",
        &["resource_type"]
    ).unwrap();
}
```
