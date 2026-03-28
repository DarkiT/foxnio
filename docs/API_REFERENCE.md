# FoxNIO API 参考文档

## API 概述

FoxNIO 提供两类 API：
1. **OpenAI 兼容 API** - 完全兼容 OpenAI API 规范
2. **管理 API** - FoxNIO 管理功能 API

**Base URL**: `http://localhost:8080`

## 认证方式

### Bearer Token (JWT)
```http
Authorization: Bearer <jwt_token>
```

### API Key
```http
Authorization: Bearer foxnio-<api_key>
```

---

## OpenAI 兼容 API

### Chat Completions

**POST** `/v1/chat/completions`

创建聊天补全请求。

**请求头**:
```http
Authorization: Bearer foxnio-your-api-key
Content-Type: application/json
```

**请求体**:
```json
{
  "model": "gpt-4",
  "messages": [
    {"role": "user", "content": "Hello!"}
  ],
  "temperature": 0.7,
  "max_tokens": 1000,
  "stream": false
}
```

**响应**:
```json
{
  "id": "chatcmpl-xxx",
  "object": "chat.completion",
  "created": 1711557800,
  "model": "gpt-4",
  "choices": [{
    "index": 0,
    "message": {
      "role": "assistant",
      "content": "Hello! How can I help you?"
    },
    "finish_reason": "stop"
  }],
  "usage": {
    "prompt_tokens": 10,
    "completion_tokens": 20,
    "total_tokens": 30
  }
}
```

### Models

**GET** `/v1/models`

列出可用模型。

**响应**:
```json
{
  "object": "list",
  "data": [
    {"id": "gpt-4", "object": "model", "owned_by": "openai"},
    {"id": "claude-3-opus", "object": "model", "owned_by": "anthropic"}
  ]
}
```

---

## 认证 API

### 用户注册

**POST** `/api/v1/auth/register`

**请求体**:
```json
{
  "email": "user@example.com",
  "password": "Password123"
}
```

**响应**:
```json
{
  "success": true,
  "message": "User registered successfully"
}
```

### 用户登录

**POST** `/api/v1/auth/login`

**请求体**:
```json
{
  "email": "user@example.com",
  "password": "Password123"
}
```

**响应**:
```json
{
  "access_token": "eyJ...",
  "refresh_token": "eyJ...",
  "token_type": "Bearer",
  "expires_in": 3600
}
```

### 刷新令牌

**POST** `/api/v1/auth/refresh`

**请求体**:
```json
{
  "refresh_token": "eyJ..."
}
```

### 登出

**POST** `/api/v1/auth/logout`

**请求头**: `Authorization: Bearer <token>`

### TOTP 两步验证

**POST** `/api/v1/auth/totp/setup`

设置两步验证。

**POST** `/api/v1/auth/totp/verify`

验证 TOTP 代码。

### 密码重置

**POST** `/api/v1/auth/password/reset-request`

请求密码重置邮件。

**POST** `/api/v1/auth/password/reset`

重置密码。

---

## 用户 API

### 获取当前用户

**GET** `/api/v1/user/me`

**响应**:
```json
{
  "id": 1,
  "email": "user@example.com",
  "created_at": "2024-03-27T00:00:00Z",
  "quota": 10000
}
```

### 更新用户信息

**PATCH** `/api/v1/user/me`

**请求体**:
```json
{
  "email": "newemail@example.com"
}
```

### 修改密码

**POST** `/api/v1/user/change-password`

**请求体**:
```json
{
  "old_password": "OldPass123",
  "new_password": "NewPass456"
}
```

---

## API Key 管理

### 列出 API Keys

**GET** `/api/v1/user/api-keys`

**响应**:
```json
{
  "keys": [
    {
      "id": 1,
      "name": "Production Key",
      "prefix": "foxnio-abc...",
      "created_at": "2024-03-27T00:00:00Z",
      "last_used": "2024-03-28T00:00:00Z"
    }
  ]
}
```

### 创建 API Key

**POST** `/api/v1/user/api-keys`

**请求体**:
```json
{
  "name": "My API Key",
  "permissions": ["chat", "models"]
}
```

**响应**:
```json
{
  "id": 1,
  "name": "My API Key",
  "key": "foxnio-xxxxxxxxxxxx",
  "created_at": "2024-03-27T00:00:00Z"
}
```

### 删除 API Key

**DELETE** `/api/v1/user/api-keys/:id`

---

## 配额管理

### 查看配额

**GET** `/api/v1/quota`

**响应**:
```json
{
  "total": 10000,
  "used": 1500,
  "remaining": 8500,
  "reset_at": "2024-04-01T00:00:00Z"
}
```

### 查看使用历史

**GET** `/api/v1/quota/history`

**查询参数**:
- `start_date`: 开始日期
- `end_date`: 结束日期
- `page`: 页码
- `page_size`: 每页数量

---

## 订阅管理

### 列出订阅计划

**GET** `/api/v1/subscriptions/plans`

### 当前订阅

**GET** `/api/v1/subscriptions/current`

### 订阅计划

**POST** `/api/v1/subscriptions/subscribe`

**请求体**:
```json
{
  "plan_id": 1,
  "promo_code": "SAVE20"
}
```

### 取消订阅

**POST** `/api/v1/subscriptions/cancel`

---

## 促销码与兑换码

### 使用促销码

**POST** `/api/v1/redeem/promo`

**请求体**:
```json
{
  "code": "SAVE20"
}
```

### 兑换充值码

**POST** `/api/v1/redeem/code`

**请求体**:
```json
{
  "code": "REDEEM-XXXX-XXXX"
}
```

---

## OAuth 认证

### 发起 OAuth 登录

**GET** `/api/v1/oauth/:provider`

支持的 provider:
- `github`
- `google`
- `linuxdo`
- `antigravity`

### OAuth 回调

**GET** `/api/v1/oauth/:provider/callback`

---

## 管理员 API

### 用户管理

**GET** `/api/v1/admin/users`

列出所有用户。

**GET** `/api/v1/admin/users/:id`

获取用户详情。

**PATCH** `/api/v1/admin/users/:id`

更新用户信息。

**DELETE** `/api/v1/admin/users/:id`

删除用户。

### 账号管理

**GET** `/api/v1/admin/accounts`

列出 AI 服务商账号。

**POST** `/api/v1/admin/accounts`

添加服务商账号。

**PATCH** `/api/v1/admin/accounts/:id`

更新账号信息。

**DELETE** `/api/v1/admin/accounts/:id`

删除账号。

### 模型配置

**GET** `/api/v1/admin/models`

列出模型配置。

**POST** `/api/v1/admin/models`

添加模型配置。

**PATCH** `/api/v1/admin/models/:id`

更新模型配置。

**DELETE** `/api/v1/admin/models/:id`

删除模型配置。

### 分组管理

**GET** `/api/v1/admin/groups`

列出所有分组。

**POST** `/api/v1/admin/groups`

创建分组。

**PATCH** `/api/v1/admin/groups/:id`

更新分组。

**DELETE** `/api/v1/admin/groups/:id`

删除分组。

### 告警规则

**GET** `/api/v1/admin/alerts/rules`

列出告警规则。

**POST** `/api/v1/admin/alerts/rules`

创建告警规则。

**PATCH** `/api/v1/admin/alerts/rules/:id`

更新告警规则。

**DELETE** `/api/v1/admin/alerts/rules/:id`

删除告警规则。

### 公告管理

**GET** `/api/v1/admin/announcements`

列出公告。

**POST** `/api/v1/admin/announcements`

创建公告。

**PATCH** `/api/v1/admin/announcements/:id`

更新公告。

**DELETE** `/api/v1/admin/announcements/:id`

删除公告。

### 审计日志

**GET** `/api/v1/admin/audit-logs`

查询审计日志。

**查询参数**:
- `user_id`: 用户 ID
- `action`: 操作类型
- `start_time`: 开始时间
- `end_time`: 结束时间
- `page`: 页码
- `page_size`: 每页数量

### 备份管理

**GET** `/api/v1/admin/backup`

获取备份状态。

**POST** `/api/v1/admin/backup`

创建备份。

**POST** `/api/v1/admin/backup/restore`

恢复备份。

---

## 系统监控 API

### 健康检查

**GET** `/health`

**响应**:
```json
{
  "status": "healthy",
  "checks": {
    "database": "ok",
    "redis": "ok",
    "disk": "ok",
    "memory": "ok"
  }
}
```

### 就绪检查

**GET** `/ready`

### Prometheus 指标

**GET** `/metrics`

返回 Prometheus 格式的指标数据。

### Dashboard 数据

**GET** `/api/v1/dashboard`

获取仪表板统计数据。

**响应**:
```json
{
  "total_requests": 10000,
  "active_users": 150,
  "quota_usage": {
    "total": 100000,
    "used": 25000
  },
  "top_models": [
    {"model": "gpt-4", "requests": 5000},
    {"model": "claude-3-opus", "requests": 3000}
  ]
}
```

---

## 用户组 API

### 列出用户组

**GET** `/api/v1/user-groups`

### 创建用户组

**POST** `/api/v1/user-groups`

### 加入用户组

**POST** `/api/v1/user-groups/:id/join`

### 离开用户组

**POST** `/api/v1/user-groups/:id/leave`

---

## 用户属性 API

### 获取用户属性

**GET** `/api/v1/user-attributes`

### 设置用户属性

**POST** `/api/v1/user-attributes`

---

## 错误响应

所有错误响应格式：

```json
{
  "error": {
    "code": "INVALID_REQUEST",
    "message": "Invalid request parameters",
    "details": {}
  }
}
```

### 常见错误码

| 状态码 | 错误码 | 说明 |
|--------|--------|------|
| 400 | INVALID_REQUEST | 请求参数无效 |
| 401 | UNAUTHORIZED | 未认证 |
| 403 | FORBIDDEN | 权限不足 |
| 404 | NOT_FOUND | 资源不存在 |
| 409 | CONFLICT | 资源冲突 |
| 429 | RATE_LIMITED | 请求频率超限 |
| 500 | INTERNAL_ERROR | 服务器内部错误 |

---

## 速率限制

- **默认限制**: 100 请求/分钟
- **API Key 限制**: 可自定义
- **响应头**:
  - `X-RateLimit-Limit`: 限制值
  - `X-RateLimit-Remaining`: 剩余次数
  - `X-RateLimit-Reset`: 重置时间

---

## WebSocket API

### 连接

```
ws://localhost:8080/ws
```

### 认证

```json
{
  "type": "auth",
  "token": "your-jwt-token"
}
```

### 订阅事件

```json
{
  "type": "subscribe",
  "channel": "monitoring"
}
```

### 接收消息

```json
{
  "type": "event",
  "channel": "monitoring",
  "data": {
    "event": "request_completed",
    "payload": {...}
  }
}
```
