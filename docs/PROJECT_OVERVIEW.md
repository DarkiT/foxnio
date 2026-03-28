# FoxNIO 项目概述

## 项目介绍

FoxNIO 是一个高性能的 AI API 网关，使用 Rust (Axum) 和 SvelteKit 构建而成。作为 AI 服务提供商（OpenAI、Anthropic、Google 等）的统一接入层，提供智能路由、故障转移、配额管理和监控告警等核心功能。

**核心定位**：企业级 AI API 网关，支持多服务商、智能路由、高可用架构。

## 技术栈

### 后端
- **框架**: Rust 1.75+ / Axum 0.7
- **数据库**: PostgreSQL 16 + Sea-ORM 0.12
- **缓存**: Redis 7
- **认证**: JWT + Argon2 密码哈希
- **HTTP**: HTTP/2 + WebSocket 支持
- **监控**: Prometheus + 自定义健康检查

### 前端
- **框架**: SvelteKit + TypeScript
- **UI**: TailwindCSS + DaisyUI
- **状态**: Svelte Stores
- **构建**: Vite

### 基础设施
- **容器化**: Docker + Docker Compose
- **反向代理**: Nginx
- **CI/CD**: GitHub Actions
- **监控**: Grafana + Prometheus

## 核心特性

### 🚀 AI 网关功能
- **OpenAI 兼容 API** - 完全兼容 OpenAI API 规范
- **多服务商支持** - OpenAI、Anthropic、Google、DeepSeek、Mistral、Cohere
- **智能路由** - 模型别名解析、自动降级、粘性会话
- **故障转移** - 自动检测失败、指数退避重试
- **响应流式** - 支持 SSE 流式响应

### 🔐 认证与授权
- **JWT 认证** - 安全市令牌认证机制
- **API Key 管理** - 创建、管理、撤销 API Key
- **两步验证** - TOTP 2FA 支持
- **OAuth 集成** - 支持 GitHub、Google、LinuxDo 等平台
- **角色权限** - 基于角色的访问控制 (RBAC)

### 📊 监控与告警
- **Prometheus 指标** - 内置指标收集
- **健康检查** - PostgreSQL、Redis、磁盘、内存监控
- **告警系统** - 多通道告警（邮件、Slack、钉钉、飞书）
- **审计日志** - 完整的操作历史记录
- **实时监控** - WebSocket 实时状态推送

### 💰 计费与配额
- **订阅管理** - 多级订阅计划
- **配额控制** - 按用户/组设置配额
- **使用统计** - 详细的使用量追踪
- **促销码** - 促销码和兑换码系统
- **账单管理** - 自动化账单生成

### 🛡️ 安全特性
- **数据加密** - AES-256-GCM 敏感数据加密
- **速率限制** - 基于 Redis 的分布式限流
- **TLS 指纹** - 客户端指纹识别
- **审计追踪** - 完整操作日志

## 架构设计

### 系统架构

```
┌────────────────────────────────────────────────────────────────────┐
│                         FoxNIO 架构                                 │
├────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌─────────────┐      ┌─────────────┐      ┌─────────────┐        │
│  │  Frontend   │      │   Gateway   │      │   Backend   │        │
│  │ (SvelteKit) │─────▶│   (Axum)    │─────▶│  Services   │        │
│  │  Port:5173  │      │  Port:8080  │      │             │        │
│  └─────────────┘      └─────────────┘      └─────────────┘        │
│        │                     │                     │               │
│        │                     ▼                     ▼               │
│        │         ┌────────────────────────────────────┐           │
│        │         │      PostgreSQL + Redis            │           │
│        │         └────────────────────────────────────┘           │
│        │                                                              │
│        │                     ▼                                        │
│        │         ┌────────────────────────────────────┐             │
│        └────────▶│    External AI Providers           │             │
│                  │  OpenAI | Anthropic | Google ...   │             │
│                  └────────────────────────────────────┘             │
│                                                                     │
└────────────────────────────────────────────────────────────────────┘
```

### 模块架构

```
Backend 模块结构
├── gateway/        # AI 网关核心
│   ├── proxy       # 请求代理
│   ├── router      # 智能路由
│   ├── failover    # 故障转移
│   └── middleware  # 中间件
├── service/        # 业务服务层
│   ├── account     # 账号管理
│   ├── api_key     # API Key 管理
│   ├── billing     # 计费服务
│   ├── quota       # 配额管理
│   ├── oauth       # OAuth 集成
│   └── ...
├── handler/        # HTTP 处理器
│   ├── auth        # 认证端点
│   ├── admin       # 管理端点
│   ├── user        # 用户端点
│   └── ...
├── entity/         # 数据库实体
├── alert/          # 告警系统
├── health/         # 健康检查
├── metrics/        # Prometheus 指标
└── config/         # 配置管理
```

## 快速开始

### 环境要求

- Rust 1.75+
- Node.js 20+
- Docker & Docker Compose
- PostgreSQL 16
- Redis 7

### Docker 部署（推荐）

```bash
# 克隆项目
git clone https://github.com/your-org/foxnio.git
cd foxnio

# 启动所有服务
make docker-up

# 访问
# API: http://localhost:8080
# 前端: http://localhost:5173
```

### 手动部署

```bash
# 1. 安装依赖
make install

# 2. 配置环境变量
cp .env.example .env
# 编辑 .env 文件

# 3. 启动数据库
make db-up

# 4. 运行数据库迁移
make migrate

# 5. 启动后端
make run-backend

# 6. 启动前端（新终端）
make run-frontend
```

### 验证安装

```bash
# 健康检查
curl http://localhost:8080/health

# 注册用户
curl -X POST http://localhost:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"user@example.com","password":"Password123"}'

# 登录
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"user@example.com","password":"Password123"}'
```

## 开发状态

- ✅ 核心网关功能
- ✅ 用户认证系统
- ✅ API Key 管理
- ✅ 配额管理
- ✅ 订阅系统
- ✅ 告警系统
- ✅ 审计日志
- ✅ OAuth 集成
- ✅ 两步验证
- ✅ 健康检查
- ✅ Prometheus 指标
- ✅ 数据加密
- ✅ WebSocket 支持

## 性能指标

- **并发连接**: 支持 10000+ 并发连接
- **响应延迟**: P99 < 100ms
- **吞吐量**: 10000+ req/s
- **内存占用**: < 200MB (空闲)
- **启动时间**: < 5s

## 许可证

MIT License - 详见 [LICENSE](../LICENSE) 文件
