# FoxNIO Webhook 功能实现方案

## 概述

Webhook 功能允许用户订阅系统事件，并在事件发生时接收 HTTP 回调通知。这是现代 API 平台的标配功能，用于实时通知和自动化集成。

**优先级**: P0（高）
**预计工作量**: 3-5 天
**参考项目**: sub2api 未实现 webhook 功能，需独立设计

## 功能需求

### 1. 支持的事件类型

```rust
pub enum WebhookEvent {
    // 账户事件
    AccountCreated { account_id: i64 },
    AccountFailed { account_id: i64, error: String },
    AccountExpired { account_id: i64 },
    
    // API Key 事件
    ApiKeyCreated { key_id: i64 },
    ApiKeyRevoked { key_id: i64 },
    
    // 配额事件
    QuotaExhausted { user_id: i64, quota_type: String },
    QuotaWarning { user_id: i64, remaining: f64 },
    
    // 计费事件
    PaymentReceived { user_id: i64, amount: f64 },
    InvoiceGenerated { user_id: i64, invoice_id: i64 },
    
    // 系统事件
    ModelAdded { model_name: String },
    ModelDeprecated { model_name: String },
    PriceChanged { model_name: String, old_price: f64, new_price: f64 },
}
```

### 2. Webhook 配置

```rust
pub struct WebhookEndpoint {
    pub id: i64,
    pub user_id: i64,
    pub url: String,                    // 回调 URL
    pub events: Vec<WebhookEvent>,      // 订阅的事件列表
    pub secret: String,                 // 用于签名验证
    pub enabled: bool,
    pub max_retries: i32,               // 最大重试次数
    pub timeout_ms: i32,                // 请求超时时间
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

## 数据库设计

### 新增表：webhook_endpoints

```sql
CREATE TABLE webhook_endpoints (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    url VARCHAR(2048) NOT NULL,
    events JSONB NOT NULL DEFAULT '[]',
    secret VARCHAR(128) NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    max_retries INTEGER NOT NULL DEFAULT 5,
    timeout_ms INTEGER NOT NULL DEFAULT 5000,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    CONSTRAINT valid_url CHECK (url ~ '^https://'),
    CONSTRAINT valid_events CHECK (jsonb_array_length(events) > 0)
);

CREATE INDEX idx_webhook_endpoints_user ON webhook_endpoints(user_id);
CREATE INDEX idx_webhook_endpoints_enabled ON webhook_endpoints(enabled);
```

### 新增表：webhook_deliveries

```sql
CREATE TABLE webhook_deliveries (
    id BIGSERIAL PRIMARY KEY,
    endpoint_id BIGINT NOT NULL REFERENCES webhook_endpoints(id) ON DELETE CASCADE,
    event_type VARCHAR(64) NOT NULL,
    payload JSONB NOT NULL,
    status VARCHAR(32) NOT NULL,  -- pending, success, failed, retrying
    response_code INTEGER,
    response_body TEXT,
    attempts INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 5,
    next_retry_at TIMESTAMP WITH TIME ZONE,
    delivered_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    CONSTRAINT valid_status CHECK (status IN ('pending', 'success', 'failed', 'retrying'))
);

CREATE INDEX idx_webhook_deliveries_status ON webhook_deliveries(status);
CREATE INDEX idx_webhook_deliveries_next_retry ON webhook_deliveries(next_retry_at) 
    WHERE status = 'retrying';
```

## API 端点设计

### 1. Webhook 管理

```rust
// 创建 webhook
POST /api/v1/webhooks
{
    "url": "https://example.com/webhook",
    "events": ["account.failed", "quota.exhausted"],
    "secret": "webhook-secret-key",
    "max_retries": 5,
    "timeout_ms": 5000
}

// 列出 webhooks
GET /api/v1/webhooks

// 获取 webhook 详情
GET /api/v1/webhooks/:id

// 更新 webhook
PUT /api/v1/webhooks/:id

// 删除 webhook
DELETE /api/v1/webhooks/:id

// 测试 webhook
POST /api/v1/webhooks/:id/test
{
    "event_type": "account.failed",
    "payload": { "account_id": 123, "error": "Test error" }
}

// 查看 webhook 日志
GET /api/v1/webhooks/:id/deliveries?status=failed&limit=50

// 重试失败的 webhook
POST /api/v1/webhooks/:id/deliveries/:delivery_id/retry
```

## 核心服务实现

### 1. WebhookService

```rust
// backend/src/service/webhook.rs

pub struct WebhookService {
    db: DatabaseConnection,
    http_client: reqwest::Client,
    redis: RedisClient,
}

impl WebhookService {
    /// 发送 webhook
    pub async fn send_webhook(
        &self,
        event: WebhookEvent,
        payload: serde_json::Value,
    ) -> Result<()> {
        // 1. 查找订阅了该事件的所有 endpoint
        let endpoints = self.find_subscribers(&event).await?;
        
        // 2. 为每个 endpoint 创建 delivery 记录
        for endpoint in endpoints {
            let delivery = self.create_delivery(&endpoint, &event, &payload).await?;
            
            // 3. 异步发送
            tokio::spawn(self.deliver_webhook(endpoint, delivery, payload.clone()));
        }
        
        Ok(())
    }
    
    /// 发送 webhook（带重试）
    async fn deliver_webhook(
        &self,
        endpoint: WebhookEndpoint,
        mut delivery: WebhookDelivery,
        payload: serde_json::Value,
    ) -> Result<()> {
        let mut attempts = 0;
        
        while attempts < endpoint.max_retries {
            attempts += 1;
            
            match self.send_http_request(&endpoint, &payload).await {
                Ok(response) => {
                    // 成功
                    self.mark_success(&mut delivery, response).await?;
                    return Ok(());
                }
                Err(e) => {
                    // 失败，记录并计算下次重试时间
                    let next_retry = self.calculate_backoff(attempts);
                    self.mark_retry(&mut delivery, &e, next_retry).await?;
                    
                    if attempts >= endpoint.max_retries {
                        self.mark_failed(&mut delivery, &e).await?;
                        return Err(e);
                    }
                    
                    tokio::time::sleep(next_retry).await;
                }
            }
        }
        
        Ok(())
    }
    
    /// 发送 HTTP 请求
    async fn send_http_request(
        &self,
        endpoint: &WebhookEndpoint,
        payload: &serde_json::Value,
    ) -> Result<reqwest::Response> {
        // 生成签名
        let timestamp = Utc::now().timestamp();
        let signature = self.generate_signature(
            &endpoint.secret,
            timestamp,
            payload,
        );
        
        // 发送请求
        let response = self.http_client
            .post(&endpoint.url)
            .header("Content-Type", "application/json")
            .header("X-Webhook-Timestamp", timestamp)
            .header("X-Webhook-Signature", signature)
            .json(payload)
            .timeout(Duration::from_millis(endpoint.timeout_ms as u64))
            .send()
            .await?;
        
        Ok(response)
    }
    
    /// 生成签名（HMAC-SHA256）
    fn generate_signature(
        &self,
        secret: &str,
        timestamp: i64,
        payload: &serde_json::Value,
    ) -> String {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(timestamp.to_string().as_bytes());
        mac.update(payload.to_string().as_bytes());
        
        hex::encode(mac.finalize().into_bytes())
    }
    
    /// 指数退避算法
    fn calculate_backoff(&self, attempt: i32) -> Duration {
        let base_delay = 1; // 1秒
        let max_delay = 300; // 5分钟
        
        let delay = base_delay * 2u64.pow(attempt as u32 - 1);
        let delay = delay.min(max_delay);
        
        // 添加抖动（±20%）
        let jitter = (rand::random::<f64>() * 0.4 - 0.2) * delay as f64;
        let delay = (delay as f64 + jitter).max(1.0) as u64;
        
        Duration::from_secs(delay)
    }
}

/// 后台任务：处理重试队列
pub async fn process_retry_queue(service: Arc<WebhookService>) {
    loop {
        // 每分钟检查一次
        tokio::time::sleep(Duration::from_secs(60)).await;
        
        // 查找需要重试的 delivery
        if let Ok(deliveries) = service.find_pending_retries().await {
            for delivery in deliveries {
                let service = service.clone();
                tokio::spawn(async move {
                    if let Ok(endpoint) = service.get_endpoint(delivery.endpoint_id).await {
                        let _ = service.deliver_webhook(endpoint, delivery, serde_json::json!({})).await;
                    }
                });
            }
        }
    }
}
```

### 2. 事件触发点

在各个服务中集成 webhook 触发：

```rust
// backend/src/service/account.rs

impl AccountService {
    pub async fn create_account(&self, req: CreateAccountRequest) -> Result<Account> {
        let account = /* ... 创建账户 ... */;
        
        // 触发 webhook
        self.webhook_service.send_webhook(
            WebhookEvent::AccountCreated { account_id: account.id },
            serde_json::to_value(&account)?,
        ).await?;
        
        Ok(account)
    }
}
```

## 安全考虑

1. **URL 验证**
   - 仅允许 HTTPS URL
   - 禁止私有 IP 地址（127.0.0.1, 10.x.x.x, 192.168.x.x）
   - 禁止 DNS 重绑定攻击

2. **签名验证**
   - 使用 HMAC-SHA256 签名
   - 包含时间戳防重放
   - 用户可验证签名

3. **速率限制**
   - 每个 endpoint 有独立的速率限制
   - 失败端点自动降级

4. **日志审计**
   - 记录所有 webhook 发送
   - 保留历史记录 30 天

## 测试计划

### 1. 单元测试

```rust
#[tokio::test]
async fn test_webhook_signature() {
    let service = WebhookService::new(/* ... */);
    let secret = "test-secret";
    let payload = serde_json::json!({"test": "data"});
    let timestamp = 1234567890;
    
    let signature = service.generate_signature(secret, timestamp, &payload);
    assert!(signature.starts_with("hmac-sha256="));
}

#[tokio::test]
async fn test_backoff_calculation() {
    let service = WebhookService::new(/* ... */);
    
    let d1 = service.calculate_backoff(1);
    assert!(d1 >= Duration::from_secs(1) && d1 < Duration::from_secs(2));
    
    let d5 = service.calculate_backoff(5);
    assert!(d5 >= Duration::from_secs(16) && d5 <= Duration::from_secs(300));
}
```

### 2. 集成测试

```rust
#[tokio::test]
async fn test_webhook_delivery() {
    // 启动 mock server
    let mut server = mockito::Server::new();
    let mock = server.mock("POST", "/webhook")
        .with_status(200)
        .create();
    
    // 创建 webhook endpoint
    let endpoint = create_test_endpoint(&server.url()).await;
    
    // 触发事件
    service.send_webhook(WebhookEvent::AccountCreated { account_id: 1 }, json!({})).await?;
    
    // 验证调用
    mock.assert();
}
```

### 3. 性能测试

- 并发发送 1000 个 webhook
- 测试重试队列吞吐量
- 测试签名计算性能

## 实施步骤

### Phase 1: 基础功能（2天）
1. 创建数据库表
2. 实现 WebhookService 核心
3. 实现签名和验证
4. 添加 API 端点

### Phase 2: 事件集成（1天）
1. 在各服务中集成 webhook 触发
2. 实现事件类型过滤

### Phase 3: 重试机制（1天）
1. 实现重试队列
2. 实现指数退避
3. 添加后台任务

### Phase 4: 测试与文档（1天）
1. 编写单元测试和集成测试
2. 编写 API 文档
3. 编写用户指南

## 监控指标

```rust
// Prometheus metrics
lazy_static! {
    static ref WEBHOOK_SENT: Counter = register_counter!(
        "foxnio_webhook_sent_total",
        "Total number of webhooks sent"
    ).unwrap();
    
    static ref WEBHOOK_FAILED: Counter = register_counter!(
        "foxnio_webhook_failed_total",
        "Total number of failed webhook deliveries"
    ).unwrap();
    
    static ref WEBHOOK_LATENCY: Histogram = register_histogram!(
        "foxnio_webhook_latency_seconds",
        "Webhook delivery latency"
    ).unwrap();
}
```

## 参考资源

- [Stripe Webhooks](https://stripe.com/docs/webhooks)
- [GitHub Webhooks](https://docs.github.com/en/developers/webhooks-and-events/webhooks)
- [Standard Webhook Specification](https://github.com/standard-webhooks/standard-webhooks)
