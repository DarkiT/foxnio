# FoxNIO 数据库架构文档

## 数据库概述

FoxNIO 使用 PostgreSQL 16 作为主数据库，采用 Sea-ORM 进行对象关系映射。

**数据库连接**:
```
postgres://foxnio:password@localhost:5432/foxnio
```

---

## 数据表列表

| 序号 | 表名 | 说明 | 迁移文件 |
|------|------|------|----------|
| 1 | users | 用户表 | m20240327_000001 |
| 2 | accounts | 服务商账号表 | m20240327_000002 |
| 3 | api_keys | API Key 表 | m20240327_000003 |
| 4 | usages | 使用量表 | m20240327_000004 |
| 5 | password_reset_tokens | 密码重置令牌表 | m20240327_000005 |
| 6 | refresh_tokens | 刷新令牌表 | m20240327_000005 |
| 7 | oauth_tokens | OAuth 令牌表 | m20240327_000006 |
| 8 | audit_logs | 审计日志表 | m20240327_000007 |
| 9 | alert_rules | 告警规则表 | m20240327_000008 |
| 10 | alert_history | 告警历史表 | m20240327_000009 |
| 11 | alert_channels | 告警通道表 | m20240327_000010 |
| 12 | groups | 分组表 | m20240328_000011 |
| 13 | model_configs | 模型配置表 | m20240328_000012 |
| 14 | tls_fingerprint_profiles | TLS 指纹配置表 | m20240328_000013 |
| 15 | announcements | 公告表 | m20240328_000014 |
| 16 | promo_codes | 促销码表 | m20240328_000015 |
| 17 | user_attribute_definitions | 用户属性定义表 | m20240328_000016 |
| 18 | user_attribute_values | 用户属性值表 | m20240328_000016 |
| 19 | error_passthrough_rules | 错误透传规则表 | m20240328_000017 |
| 20 | scheduled_test_plans | 定时测试计划表 | m20240328_000018 |
| 21 | scheduled_test_results | 定时测试结果表 | m20240328_000018 |
| 22 | proxies | 代理配置表 | m20240328_000019 |
| 23 | redeem_codes | 兑换码表 | m20240328_000020 |
| 24 | quota_usage_history | 配额使用历史表 | m20240328_000021 |
| 25 | subscriptions | 订阅表 | m20240328_000022 |
| 26 | account_groups | 账号分组关联表 | - |
| 27 | promo_code_usages | 促销码使用记录表 | - |
| 28 | announcement_reads | 公告阅读记录表 | - |

---

## 核心表结构

### 1. users (用户表)

**用途**: 存储用户账户信息

| 字段名 | 类型 | 说明 | 约束 |
|--------|------|------|------|
| id | BIGINT | 主键 | PRIMARY KEY, AUTO INCREMENT |
| email | VARCHAR(255) | 邮箱 | UNIQUE, NOT NULL |
| password_hash | VARCHAR(255) | 密码哈希 | NOT NULL |
| name | VARCHAR(100) | 用户名 | |
| avatar_url | VARCHAR(500) | 头像 URL | |
| is_active | BOOLEAN | 是否激活 | DEFAULT true |
| is_admin | BOOLEAN | 是否管理员 | DEFAULT false |
| totp_secret | VARCHAR(100) | TOTP 密钥 | |
| totp_enabled | BOOLEAN | TOTP 是否启用 | DEFAULT false |
| quota_limit | BIGINT | 配额限制 | DEFAULT 0 |
| quota_used | BIGINT | 已用配额 | DEFAULT 0 |
| created_at | TIMESTAMP | 创建时间 | DEFAULT NOW() |
| updated_at | TIMESTAMP | 更新时间 | DEFAULT NOW() |

**索引**:
- `idx_users_email` ON (email)
- `idx_users_active` ON (is_active)

---

### 2. accounts (服务商账号表)

**用途**: 存储 AI 服务商账号信息

| 字段名 | 类型 | 说明 | 约束 |
|--------|------|------|------|
| id | BIGINT | 主键 | PRIMARY KEY |
| name | VARCHAR(100) | 账号名称 | NOT NULL |
| provider | VARCHAR(50) | 服务商 | NOT NULL |
| api_key | TEXT | API Key (加密) | |
| api_endpoint | VARCHAR(255) | API 端点 | |
| models | JSONB | 支持的模型列表 | |
| priority | INT | 优先级 | DEFAULT 0 |
| weight | INT | 权重 | DEFAULT 100 |
| is_active | BOOLEAN | 是否启用 | DEFAULT true |
| health_status | VARCHAR(20) | 健康状态 | DEFAULT 'unknown' |
| last_check_at | TIMESTAMP | 最后检查时间 | |
| error_count | INT | 错误计数 | DEFAULT 0 |
| created_at | TIMESTAMP | 创建时间 | |
| updated_at | TIMESTAMP | 更新时间 | |

**支持的 provider**:
- `openai` - OpenAI
- `anthropic` - Anthropic
- `google` - Google AI
- `deepseek` - DeepSeek
- `mistral` - Mistral
- `cohere` - Cohere
- `bedrock` - AWS Bedrock

**索引**:
- `idx_accounts_provider` ON (provider)
- `idx_accounts_active` ON (is_active)

---

### 3. api_keys (API Key 表)

**用途**: 管理用户的 API Key

| 字段名 | 类型 | 说明 | 约束 |
|--------|------|------|------|
| id | BIGINT | 主键 | PRIMARY KEY |
| user_id | BIGINT | 用户 ID | FOREIGN KEY |
| name | VARCHAR(100) | Key 名称 | NOT NULL |
| key_hash | VARCHAR(255) | Key 哈希 | NOT NULL, UNIQUE |
| key_prefix | VARCHAR(20) | Key 前缀 | NOT NULL |
| permissions | JSONB | 权限列表 | |
| rate_limit | INT | 速率限制 | |
| quota_limit | BIGINT | 配额限制 | |
| is_active | BOOLEAN | 是否启用 | DEFAULT true |
| last_used_at | TIMESTAMP | 最后使用时间 | |
| expires_at | TIMESTAMP | 过期时间 | |
| created_at | TIMESTAMP | 创建时间 | |

**索引**:
- `idx_api_keys_user` ON (user_id)
- `idx_api_keys_prefix` ON (key_prefix)

---

### 4. usages (使用量表)

**用途**: 记录 API 使用量

| 字段名 | 类型 | 说明 | 约束 |
|--------|------|------|------|
| id | BIGINT | 主键 | PRIMARY KEY |
| user_id | BIGINT | 用户 ID | FOREIGN KEY |
| api_key_id | BIGINT | API Key ID | FOREIGN KEY |
| model | VARCHAR(100) | 模型名 | NOT NULL |
| provider | VARCHAR(50) | 服务商 | NOT NULL |
| request_tokens | INT | 请求 tokens | |
| response_tokens | INT | 响应 tokens | |
| total_tokens | INT | 总 tokens | |
| cost | DECIMAL(10,4) | 费用 | |
| latency_ms | INT | 延迟 (ms) | |
| status_code | INT | HTTP 状态码 | |
| is_success | BOOLEAN | 是否成功 | |
| created_at | TIMESTAMP | 创建时间 | |

**索引**:
- `idx_usages_user` ON (user_id, created_at)
- `idx_usages_model` ON (model)
- `idx_usages_date` ON (created_at)

---

### 5. subscriptions (订阅表)

**用途**: 用户订阅管理

| 字段名 | 类型 | 说明 | 约束 |
|--------|------|------|------|
| id | BIGINT | 主键 | PRIMARY KEY |
| user_id | BIGINT | 用户 ID | FOREIGN KEY |
| plan_name | VARCHAR(50) | 计划名称 | NOT NULL |
| quota_limit | BIGINT | 配额限制 | |
| price | DECIMAL(10,2) | 价格 | |
| billing_cycle | VARCHAR(20) | 计费周期 | |
| status | VARCHAR(20) | 状态 | DEFAULT 'active' |
| started_at | TIMESTAMP | 开始时间 | |
| expires_at | TIMESTAMP | 过期时间 | |
| created_at | TIMESTAMP | 创建时间 | |

**索引**:
- `idx_subscriptions_user` ON (user_id)
- `idx_subscriptions_status` ON (status)

---

### 6. groups (分组表)

**用途**: 资源分组管理

| 字段名 | 类型 | 说明 | 约束 |
|--------|------|------|------|
| id | BIGINT | 主键 | PRIMARY KEY |
| name | VARCHAR(100) | 分组名称 | NOT NULL |
| description | TEXT | 描述 | |
| type | VARCHAR(20) | 分组类型 | NOT NULL |
| config | JSONB | 配置 | |
| created_at | TIMESTAMP | 创建时间 | |
| updated_at | TIMESTAMP | 更新时间 | |

**分组类型**:
- `account` - 账号分组
- `user` - 用户分组
- `model` - 模型分组

---

### 7. model_configs (模型配置表)

**用途**: 模型路由配置

| 字段名 | 类型 | 说明 | 约束 |
|--------|------|------|------|
| id | BIGINT | 主键 | PRIMARY KEY |
| alias | VARCHAR(100) | 模型别名 | NOT NULL, UNIQUE |
| target_model | VARCHAR(100) | 目标模型 | NOT NULL |
| provider | VARCHAR(50) | 服务商 | NOT NULL |
| account_id | BIGINT | 账号 ID | FOREIGN KEY |
| priority | INT | 优先级 | DEFAULT 0 |
| is_active | BOOLEAN | 是否启用 | DEFAULT true |
| config | JSONB | 额外配置 | |
| created_at | TIMESTAMP | 创建时间 | |

**示例**:
```json
{
  "alias": "gpt-4",
  "target_model": "gpt-4-turbo",
  "provider": "openai",
  "account_id": 1,
  "priority": 100,
  "is_active": true
}
```

---

### 8. audit_logs (审计日志表)

**用途**: 操作审计追踪

| 字段名 | 类型 | 说明 | 约束 |
|--------|------|------|------|
| id | BIGINT | 主键 | PRIMARY KEY |
| user_id | BIGINT | 用户 ID | |
| action | VARCHAR(50) | 操作类型 | NOT NULL |
| resource_type | VARCHAR(50) | 资源类型 | |
| resource_id | VARCHAR(100) | 资源 ID | |
| ip_address | VARCHAR(45) | IP 地址 | |
| user_agent | VARCHAR(500) | User Agent | |
| request_data | JSONB | 请求数据 | |
| response_status | INT | 响应状态码 | |
| created_at | TIMESTAMP | 创建时间 | |

**操作类型**:
- `login` - 登录
- `logout` - 登出
- `api_key.create` - 创建 API Key
- `api_key.delete` - 删除 API Key
- `user.update` - 更新用户
- `admin.user.delete` - 管理员删除用户

**索引**:
- `idx_audit_logs_user` ON (user_id)
- `idx_audit_logs_action` ON (action)
- `idx_audit_logs_date` ON (created_at)

---

### 9. alert_rules (告警规则表)

**用途**: 告警规则配置

| 字段名 | 类型 | 说明 | 约束 |
|--------|------|------|------|
| id | BIGINT | 主键 | PRIMARY KEY |
| name | VARCHAR(100) | 规则名称 | NOT NULL |
| metric | VARCHAR(100) | 监控指标 | NOT NULL |
| condition | VARCHAR(50) | 条件 | NOT NULL |
| threshold | DECIMAL | 阈值 | NOT NULL |
| duration_seconds | INT | 持续时间 | |
| channels | JSONB | 告警通道 | |
| is_active | BOOLEAN | 是否启用 | DEFAULT true |
| created_at | TIMESTAMP | 创建时间 | |

**监控指标**:
- `error_rate` - 错误率
- `latency_p99` - P99 延迟
- `quota_usage` - 配额使用率
- `account_health` - 账号健康度

---

### 10. oauth_tokens (OAuth 令牌表)

**用途**: OAuth 认证令牌存储

| 字段名 | 类型 | 说明 | 约束 |
|--------|------|------|------|
| id | BIGINT | 主键 | PRIMARY KEY |
| user_id | BIGINT | 用户 ID | FOREIGN KEY |
| provider | VARCHAR(50) | OAuth 提供商 | NOT NULL |
| access_token | TEXT | 访问令牌 | |
| refresh_token | TEXT | 刷新令牌 | |
| expires_at | TIMESTAMP | 过期时间 | |
| created_at | TIMESTAMP | 创建时间 | |

**支持的 OAuth 提供商**:
- `github`
- `google`
- `linuxdo`
- `antigravity`

---

## 关联表

### account_groups (账号分组关联)

| 字段名 | 类型 | 说明 |
|--------|------|------|
| account_id | BIGINT | 账号 ID |
| group_id | BIGINT | 分组 ID |

### promo_code_usages (促销码使用记录)

| 字段名 | 类型 | 说明 |
|--------|------|------|
| id | BIGINT | 主键 |
| promo_code_id | BIGINT | 促销码 ID |
| user_id | BIGINT | 用户 ID |
| used_at | TIMESTAMP | 使用时间 |

---

## 数据库优化

### 索引策略

1. **主键索引**: 所有表都有自增主键
2. **外键索引**: 所有外键字段
3. **查询索引**: 高频查询字段
4. **组合索引**: 多字段查询

### 分区策略

对于大数据量表采用分区：

**usages 表** - 按时间分区
```sql
-- 月度分区
CREATE TABLE usages_2024_03 PARTITION OF usages
  FOR VALUES FROM ('2024-03-01') TO ('2024-04-01');
```

**audit_logs 表** - 按时间分区
```sql
-- 季度分区
CREATE TABLE audit_logs_2024_q1 PARTITION OF audit_logs
  FOR VALUES FROM ('2024-01-01') TO ('2024-04-01');
```

### 数据归档

- `usages` - 保留 90 天，之后归档
- `audit_logs` - 保留 1 年，之后归档
- `alert_history` - 保留 30 天

---

## Redis 缓存结构

### Key 命名规范

```
foxnio:{resource}:{id}:{field}
```

### 常用缓存

| Key | 类型 | TTL | 说明 |
|-----|------|-----|------|
| `foxnio:user:{id}` | Hash | 1h | 用户信息 |
| `foxnio:api_key:{prefix}` | Hash | 10m | API Key 信息 |
| `foxnio:quota:{user_id}` | String | 1d | 配额缓存 |
| `foxnio:rate_limit:{key}` | String | 1m | 速率限制 |
| `foxnio:session:{token}` | Hash | 24h | 会话信息 |

---

## 备份策略

### 全量备份
- 每天凌晨 2:00
- 保留 7 天

### 增量备份
- 每小时
- 保留 24 小时

### 备份命令
```bash
# 全量备份
pg_dump foxnio > backup_$(date +%Y%m%d).sql

# 恢复
psql foxnio < backup_20240328.sql
```
