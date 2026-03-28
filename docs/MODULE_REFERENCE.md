# FoxNIO 模块参考文档

## 模块概述

FoxNIO 后端采用模块化架构设计，各模块职责明确、低耦合。

---

## 代码量统计

| 模块 | 文件数 | 代码行数 | 测试数量 |
|------|--------|----------|----------|
| Migration | 24 | 2,949 | 0 |
| Entity | 31 | 2,909 | 21 |
| Service | 54 | 20,311 | 241 |
| Handler | 29 | 7,079 | 17 |
| Model | 6 | 735 | 7 |
| Config | 2 | 619 | 5 |
| Utils | 9 | 1,642 | 37 |
| 其他 | 3 | 194 | 0 |
| **总计** | **158** | **36,438** | **328** |

**注**: 测试数量包含 `#[test]` 和 `#[tokio::test]`

---

## 核心模块详解

### 1. Migration 模块 (数据库迁移)

**路径**: `backend/migration/src/`

**文件数**: 24 文件

**代码量**: 2,949 行

**功能**: 数据库表结构版本管理

**主要迁移文件**:

| 文件 | 说明 |
|------|------|
| m20240327_000001_create_users | 用户表 |
| m20240327_000002_create_accounts | 服务商账号表 |
| m20240327_000003_create_api_keys | API Key 表 |
| m20240327_000004_create_usages | 使用量表 |
| m20240327_000005_create_password_reset_tokens | 密码重置令牌表 |
| m20240327_000005_create_refresh_tokens | 刷新令牌表 |
| m20240327_000006_create_oauth_tokens | OAuth 令牌表 |
| m20240327_000007_create_audit_logs | 审计日志表 |
| m20240327_000008_create_alert_rules | 告警规则表 |
| m20240327_000009_create_alert_history | 告警历史表 |
| m20240327_000010_create_alert_channels | 告警通道表 |
| m20240328_000011_create_groups | 分组表 |
| m20240328_000012_create_model_configs | 模型配置表 |
| m20240328_000013_create_tls_fingerprint_profiles | TLS 指纹表 |
| m20240328_000014_create_announcements | 公告表 |
| m20240328_000015_create_promo_codes | 促销码表 |
| m20240328_000016_create_user_attributes | 用户属性表 |
| m20240328_000017_create_error_passthrough_rules | 错误透传规则表 |
| m20240328_000018_create_scheduled_test_plans | 定时测试计划表 |
| m20240328_000019_create_proxies | 代理配置表 |
| m20240328_000020_create_redeem_codes | 兑换码表 |
| m20240328_000021_create_quota_usage_history | 配额使用历史表 |
| m20240328_000022_create_subscriptions | 订阅表 |

**依赖关系**:
- 依赖 Sea-ORM Migration
- 被 Entity 模块依赖

---

### 2. Entity 模块 (数据库实体)

**路径**: `backend/src/entity/`

**文件数**: 31 文件

**代码量**: 2,909 行

**测试数量**: 21 个

**功能**: 数据库表对应的实体定义

**主要实体**:

| 实体文件 | 说明 | 关联表 |
|---------|------|--------|
| accounts.rs | 服务商账号 | accounts |
| api_keys.rs | API Key | api_keys |
| audit_logs.rs | 审计日志 | audit_logs |
| users.rs | 用户 | users |
| groups.rs | 分组 | groups |
| model_configs.rs | 模型配置 | model_configs |
| oauth_tokens.rs | OAuth 令牌 | oauth_tokens |
| refresh_tokens.rs | 刷新令牌 | refresh_tokens |
| subscriptions.rs | 订阅 | subscriptions |
| promo_codes.rs | 促销码 | promo_codes |
| redeem_codes.rs | 兑换码 | redeem_codes |
| announcements.rs | 公告 | announcements |
| alert_rules.rs | 告警规则 | alert_rules |
| alert_history.rs | 告警历史 | alert_history |
| alert_channels.rs | 告警通道 | alert_channels |
| usages.rs | 使用量 | usages |
| proxies.rs | 代理配置 | proxies |
| scheduled_test_plans.rs | 定时测试计划 | scheduled_test_plans |
| tls_fingerprint_profile.rs | TLS 指纹配置 | tls_fingerprint_profiles |
| encrypted_field.rs | 加密字段 | - |
| user_attribute_definitions.rs | 用户属性定义 | user_attribute_definitions |
| user_attribute_values.rs | 用户属性值 | user_attribute_values |
| account_groups.rs | 账号分组关联 | account_groups |

**特性**:
- 使用 Sea-ORM 宏定义
- 支持字段加密 (encrypted_field.rs)
- 关联关系清晰

**依赖关系**:
- 依赖 Sea-ORM
- 被 Service 模块调用

---

### 3. Service 模块 (业务服务层)

**路径**: `backend/src/service/`

**文件数**: 54 文件

**代码量**: 20,311 行

**测试数量**: 241 个

**功能**: 核心业务逻辑实现

**服务分类**:

#### 认证相关
| 服务 | 说明 |
|------|------|
| account.rs | 账号管理服务 |
| totp.rs | TOTP 两步验证 |
| password_reset.rs | 密码重置 |
| turnstile.rs | Cloudflare Turnstile 验证 |

#### API 管理
| 服务 | 说明 |
|------|------|
| api_key.rs | API Key 管理 |
| model_registry.rs | 模型注册表 |
| model_router.rs | 模型路由器 |
| credential.rs | 凭证管理 |

#### 计费相关
| 服务 | 说明 |
|------|------|
| billing.rs | 计费服务 |
| subscription.rs | 订阅管理 |
| quota.rs | 配额管理 |
| promo_code.rs | 促销码 |
| redeem_code.rs | 兑换码 |

#### OAuth 集成
| 服务 | 说明 |
|------|------|
| oauth/mod.rs | OAuth 核心 |
| oauth/github.rs | GitHub OAuth |
| oauth/google.rs | Google OAuth |
| oauth/linuxdo.rs | LinuxDo OAuth |
| oauth/antigravity.rs | Antigravity OAuth |

#### AI 服务商
| 服务 | 说明 |
|------|------|
| bedrock.rs | AWS Bedrock |
| sora.rs | OpenAI Sora |
| openai_privacy.rs | OpenAI 隐私处理 |

#### 监控告警
| 服务 | 说明 |
|------|------|
| health_scorer.rs | 健康评分 |
| qps_monitor.rs | QPS 监控 |
| realtime_monitor.rs | 实时监控 |
| scheduler.rs | 调度器 |

#### 其他服务
| 服务 | 说明 |
|------|------|
| user.rs | 用户服务 |
| group.rs | 分组服务 |
| proxy.rs | 代理服务 |
| email.rs | 邮件服务 |
| backup.rs | 备份服务 |
| audit.rs | 审计服务 |
| rate_limit.rs | 速率限制 |
| tls_fingerprint.rs | TLS 指纹 |
| concurrency.rs | 并发控制 |
| sticky_session.rs | 粘性会话 |
| batch_operations.rs | 批量操作 |

**测试覆盖**:
- api_key_test.rs
- billing_test.rs
- password_reset_test.rs
- permission_test.rs
- redemption_test.rs
- scheduler_test.rs
- subscription_test.rs
- totp_test.rs
- user_test.rs

**依赖关系**:
- 依赖 Entity 模块
- 被 Handler 模块调用
- 调用外部 API

---

### 4. Handler 模块 (HTTP 处理器)

**路径**: `backend/src/handler/`

**文件数**: 29 文件

**代码量**: 7,079 行

**测试数量**: 17 个

**功能**: HTTP 请求处理和响应

**处理器分类**:

#### 认证处理器 (handler/auth/)
| 文件 | 说明 | 端点 |
|------|------|------|
| mod.rs | 认证核心 | /api/v1/auth/* |
| password.rs | 密码相关 | /api/v1/auth/password/* |
| refresh.rs | 令牌刷新 | /api/v1/auth/refresh |
| totp.rs | TOTP | /api/v1/auth/totp/* |

#### 管理处理器
| 文件 | 说明 | 端点 |
|------|------|------|
| admin.rs | 管理核心 | /api/v1/admin/* |
| admin_accounts.rs | 账号管理 | /api/v1/admin/accounts/* |
| admin_groups.rs | 分组管理 | /api/v1/admin/groups/* |

#### 用户处理器
| 文件 | 说明 | 端点 |
|------|------|------|
| user.rs | 用户管理 | /api/v1/user/* |
| groups.rs | 用户组 | /api/v1/groups/* |
| user_groups.rs | 用户分组 | /api/v1/user-groups/* |
| user_attribute.rs | 用户属性 | /api/v1/user-attributes/* |

#### 功能处理器
| 文件 | 说明 | 端点 |
|------|------|------|
| alerts.rs | 告警管理 | /api/v1/alerts/* |
| announcement.rs | 公告管理 | /api/v1/announcements/* |
| user_announcement.rs | 用户公告 | /api/v1/user/announcements/* |
| audit.rs | 审计日志 | /api/v1/audit-logs/* |
| backup.rs | 备份管理 | /api/v1/backup/* |
| models.rs | 模型管理 | /api/v1/models/* |
| proxy.rs | 代理管理 | /api/v1/proxies/* |
| quota.rs | 配额管理 | /api/v1/quota/* |
| subscription.rs | 订阅管理 | /api/v1/subscriptions/* |
| promo_code.rs | 促销码 | /api/v1/promo-codes/* |
| redeem.rs | 兑换码 | /api/v1/redeem/* |
| scheduled_test_plan.rs | 定时测试 | /api/v1/scheduled-tests/* |
| error_passthrough_rule.rs | 错误透传 | /api/v1/error-rules/* |
| verify.rs | 验证 | /api/v1/verify/* |

#### 系统处理器
| 文件 | 说明 | 端点 |
|------|------|------|
| health.rs | 健康检查 | /health, /ready |
| metrics.rs | Prometheus 指标 | /metrics |
| dashboard.rs | 仪表板数据 | /api/v1/dashboard |

**依赖关系**:
- 依赖 Service 模块
- 被 main.rs 路由调用

---

### 5. Model 模块 (数据模型)

**路径**: `backend/src/model/`

**文件数**: 6 文件

**代码量**: 735 行

**测试数量**: 7 个

**功能**: API 请求/响应模型定义

**主要模型**:
- 请求模型 (Request)
- 响应模型 (Response)
- DTO 模型
- 验证模型

**依赖关系**:
- 依赖 Serde
- 被 Handler 和 Service 使用

---

### 6. Config 模块 (配置管理)

**路径**: `backend/src/config/`

**文件数**: 2 文件

**代码量**: 619 行

**测试数量**: 5 个

**功能**: 应用配置加载和管理

**配置项**:
- 服务器配置 (host, port)
- 数据库配置 (PostgreSQL)
- Redis 配置
- JWT 配置
- 网关配置
- 加密配置

**配置来源**:
- 环境变量
- config.yaml 文件
- .env 文件

**依赖关系**:
- 依赖 serde_yaml, dotenvy
- 被所有模块使用

---

### 7. Utils 模块 (工具函数)

**路径**: `backend/src/utils/`

**文件数**: 9 文件

**代码量**: 1,642 行

**测试数量**: 37 个

**功能**: 通用工具函数

**主要工具**:
- 加密工具 (AES-256-GCM)
- 哈希工具 (Argon2)
- JWT 工具
- 时间工具
- 字符串工具
- 验证工具

**依赖关系**:
- 独立模块
- 被所有模块使用

---

## 模块依赖关系图

```
┌─────────────┐
│   main.rs   │
└──────┬──────┘
       │
       ▼
┌─────────────┐      ┌─────────────┐
│   Handler   │─────▶│   Service   │
└──────┬──────┘      └──────┬──────┘
       │                    │
       │                    ▼
       │             ┌─────────────┐
       │             │   Entity    │
       │             └──────┬──────┘
       │                    │
       │                    ▼
       │             ┌─────────────┐
       │             │  Migration  │
       │             └─────────────┘
       │
       ▼
┌─────────────┐
│    Model    │
└─────────────┘

所有模块 ───▶ Utils, Config
```

---

## 模块开发指南

### 添加新服务

1. 在 `backend/src/service/` 创建服务文件
2. 在 `backend/src/service/mod.rs` 导出
3. 编写单元测试
4. 在 Handler 中调用

### 添加新实体

1. 创建迁移文件 `migration/src/m_*.rs`
2. 创建实体文件 `backend/src/entity/*.rs`
3. 在 `backend/src/entity/mod.rs` 导出
4. 在服务中使用

### 添加新 API

1. 创建 Handler 文件 `backend/src/handler/*.rs`
2. 定义路由
3. 创建 Service 逻辑
4. 编写测试
5. 更新 API 文档

---

## 测试策略

### 单元测试
- Service 层: 每个服务独立测试
- Utils 层: 工具函数测试
- Model 层: 模型验证测试

### 集成测试
- API 端点测试
- 数据库操作测试
- 认证流程测试

### 测试覆盖率目标
- Service: > 80%
- Handler: > 70%
- Utils: > 90%
