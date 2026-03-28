# FoxNIO vs sub2api 对齐报告

**生成时间**: 2026-03-29 00:05 GMT+8  
**对比版本**: sub2api v0.1.105 vs FoxNIO latest

---

## 📊 对齐统计

| 项目 | sub2api | FoxNIO | 状态 |
|------|---------|--------|------|
| **核心服务文件** | 210 | 186 | 88.6% |
| **缺失服务** | - | 46 | - |
| **独有服务** | - | 32 | - |
| **功能对齐度** | - | - | **~85%** |

---

## ❌ 缺失的 46 个服务

### Antigravity 相关 (2)
- `antigravity_oauth` - OAuth 服务
- `antigravity_quota_scope` - 配额作用域

### API Key 相关 (2)
- `api_key_auth_cache_impl` - 缓存实现
- `api_key_auth_cache_invalidate` - 缓存失效

### Bedrock 相关 (3)
- `bedrock_request` - 请求处理
- `bedrock_signer` - 签名服务
- `bedrock_stream` - 流处理

### OpenAI 相关 (5)
- `openai_gateway_chat_completions` - Chat Completions 网关
- `openai_gateway_messages` - Messages 网关
- `openai_model_mapping` - 模型映射
- `openai_oauth` - OAuth 服务
- `openai_token_provider` - Token 提供者
- `openai_ws_v2_passthrough_adapter` - WebSocket 适配器

### Ops 运维相关 (21)
- `ops_account_availability` - 账号可用性
- `ops_advisory_lock` - 咨询锁
- `ops_alert_models` - 告警模型
- `ops_alerts` - 告警服务
- `ops_concurrency` - 并发控制
- `ops_dashboard` - 仪表盘
- `ops_dashboard_models` - 仪表盘模型
- `ops_errors` - 错误处理
- `ops_histograms` - 直方图统计
- `ops_log_runtime` - 日志运行时
- `ops_models` - 运维模型
- `ops_openai_token_stats` - Token 统计
- `ops_openai_token_stats_models` - Token 统计模型
- `ops_realtime_models` - 实时模型
- `ops_realtime_traffic_models` - 实时流量模型
- `ops_request_details` - 请求详情
- `ops_settings` - 设置
- `ops_settings_models` - 设置模型
- `ops_system_log` - 系统日志
- `ops_system_log_sink` - 日志接收
- `ops_trend_models` - 趋势模型

### Sora 相关 (5)
- `sora_account` - 账号管理
- `sora_client` - 客户端
- `sora_gateway_streaming_legacy` - 流式网关旧版
- `sora_media_sign` - 媒体签名
- `sora_models` - 模型定义

### 其他 (8)
- `data_management_grpc` - gRPC 数据管理
- `error_passthrough` - 错误透传
- `error_passthrough_runtime` - 错误透传运行时
- `scheduled_test` - 计划测试
- `scheduled_test_port` - 计划测试端口
- `usage_log_create_result` - 使用日志创建结果
- `usage_log_helpers` - 使用日志辅助

---

## ✅ FoxNIO 独有的 32 个服务

### 账号管理 (3)
- `account_quota_reset` - 配额重置
- `account_rpm` - RPM 管理
- `api_key_test` - API Key 测试

### 认证授权 (5)
- `audit` - 审计服务
- `credential` - 凭证管理
- `password_reset` - 密码重置
- `password_reset_test` - 密码重置测试
- `permission` - 权限管理

### 计费相关 (2)
- `billing_test` - 计费测试
- `redemption_test` - 兑换测试

### 网关增强 (3)
- `model_registry` - 模型注册
- `model_router` - 模型路由
- `sticky_session` - 粘性会话

### 监控相关 (3)
- `health_scorer` - 健康评分
- `qps_monitor` - QPS 监控
- `realtime_monitor` - 实时监控

### 调度器 (2)
- `scheduler` - 调度器
- `scheduler_test` - 调度器测试

### 其他 (14)
- `batch_operations` - 批量操作
- `bedrock` - Bedrock 统一服务
- `error_passthrough_rule` - 错误透传规则
- `mod` - 模块管理
- `rate_limit` - 速率限制
- `scheduled_test_plan` - 计划测试
- `sora` - Sora 统一服务
- `subscription_test` - 订阅测试
- `tls_fingerprint` - TLS 指纹
- `totp_test` - TOTP 测试
- `user_ext` - 用户扩展
- `user_group` - 用户组
- `user_test` - 用户测试
- `quota` - 配额服务

---

## 📈 对齐度分析

### 高对齐度模块 (>90%)
- ✅ 基础服务（账户、认证、API Key）
- ✅ Antigravity 核心功能
- ✅ 数据库实体
- ✅ Handler 端点

### 中等对齐度模块 (70-90%)
- ⚠️ OpenAI 网关（缺少细分服务）
- ⚠️ Sora 服务（缺少细分功能）
- ⚠️ Bedrock 服务（缺少细分）

### 低对齐度模块 (<70%)
- ❌ Ops 运维模块（缺少 21 个细分服务）
- ❌ 错误透传运行时
- ❌ 计划测试端口

---

## 🎯 优先补齐建议

### P0 - 核心功能
1. `openai_oauth` - OpenAI OAuth 支持
2. `openai_token_provider` - Token 提供者
3. `antigravity_oauth` - Antigravity OAuth

### P1 - 运维增强
1. `ops_alerts` - 告警服务
2. `ops_dashboard` - 仪表盘
3. `ops_account_availability` - 账号可用性

### P2 - 高级功能
1. `sora_models` - Sora 模型定义
2. `bedrock_stream` - Bedrock 流处理
3. `scheduled_test` - 计划测试

---

## 📝 结论

**当前对齐度**: **~85%**（核心功能）

**主要差距**:
1. Ops 运维模块细分不足（21 个服务缺失）
2. OpenAI/Sora 网关细分服务缺失
3. 部分运行时和辅助服务缺失

**建议**:
- 继续补齐 P0 核心功能达到 90% 对齐
- 逐步完善 Ops 运维模块
- 保持代码质量和编译通过
