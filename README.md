# FoxNIO - AI API Gateway

<div align="center">

🦊 **FoxNIO** - 高性能 AI API 网关

优雅 · 专业 · 克制

[![CI/CD](https://github.com/your-org/foxnio/workflows/FoxNIO%20CI/CD/badge.svg)](https://github.com/your-org/foxnio/actions)
[![Backend Coverage](https://codecov.io/gh/your-org/foxnio/branch/main/graph/badge.svg?flag=backend)](https://codecov.io/gh/your-org/foxnio)
[![Frontend Coverage](https://codecov.io/gh/your-org/foxnio/branch/main/graph/badge.svg?flag=frontend)](https://codecov.io/gh/your-org/foxnio)
[![Security Audit](https://img.shields.io/badge/security-audit-green.svg)](https://github.com/your-org/foxnio/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

[English](#english) | [中文文档](#中文文档)

</div>

---

## English

### Overview

FoxNIO is a high-performance AI API gateway built with Rust and SvelteKit. It serves as a unified entry point for AI service providers (OpenAI, Anthropic, Google, etc.), offering intelligent routing, failover, quota management, and monitoring capabilities.

### Features

#### 🚀 Core Features

- **OpenAI Compatible API** - Drop-in replacement for OpenAI API endpoints
- **Multi-Provider Support** - OpenAI, Anthropic Claude, Google Gemini, DeepSeek, Mistral, Cohere
- **Intelligent Routing** - Model alias resolution and automatic fallback
- **High Availability** - Automatic failover with exponential backoff

#### 🔐 Security & Auth

- **JWT Authentication** - Secure token-based authentication
- **API Key Management** - Create, manage, and revoke API keys
- **Two-Factor Auth** - TOTP-based 2FA support
- **OAuth Integration** - GitHub, Google, LinuxDo, Antigravity
- **Role-Based Access Control** - Fine-grained permission system

#### 📊 Monitoring & Alerting

- **Prometheus Metrics** - Built-in metrics collection
- **Health Checks** - Comprehensive health monitoring
- **Alert System** - Multi-channel alerting (Email, Slack, DingTalk, Feishu)
- **Audit Logging** - Complete operation history

#### ⚡ Performance

- **HTTP/2 Support** - Modern protocol for better performance
- **Response Compression** - Automatic gzip/brotli compression
- **Connection Pooling** - Efficient connection management
- **Redis Caching** - Fast rate limiting and session storage

#### 💰 Billing & Quota

- **Subscription Management** - Multiple subscription plans
- **Quota Control** - Per-user/group quota settings
- **Usage Statistics** - Detailed usage tracking
- **Promo Codes** - Promo code and redemption code system

### Architecture

```
┌────────────────────────────────────────────────────────────────────┐
│                         FoxNIO Architecture                         │
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

### Quick Start

#### Prerequisites

- Rust 1.75+
- Node.js 20+
- Docker & Docker Compose
- PostgreSQL 16
- Redis 7

#### Using Docker (Recommended)

```bash
# Clone the repository
git clone https://github.com/your-org/foxnio.git
cd foxnio

# Start services
docker-compose up -d

# The API will be available at http://localhost:8080
# The frontend will be available at http://localhost:5173
```

#### Manual Setup

```bash
# 1. Install dependencies
make install

# 2. Configure environment
cp .env.example .env
# Edit .env with your settings

# 3. Start database
make db-up

# 4. Run migrations
make migrate

# 5. Start the server
make run
```

### API Usage

#### OpenAI Compatible Endpoints

```bash
# Chat Completions
curl http://localhost:8080/v1/chat/completions \
  -H "Authorization: Bearer foxnio-your-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-4",
    "messages": [{"role": "user", "content": "Hello!"}]
  }'

# List Models
curl http://localhost:8080/v1/models \
  -H "Authorization: Bearer foxnio-your-api-key"
```

#### User Registration & Login

```bash
# Register
curl -X POST http://localhost:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com", "password": "Password123"}'

# Login
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com", "password": "Password123"}'
```

### Supported Models

| Provider | Models |
|----------|--------|
| OpenAI | gpt-4-turbo, gpt-4o, gpt-4, gpt-3.5-turbo |
| Anthropic | claude-3-opus, claude-3.5-sonnet, claude-3-haiku |
| Google | gemini-1.5-pro, gemini-1.5-flash |
| DeepSeek | deepseek-chat, deepseek-coder |
| Mistral | mistral-large, mistral-medium, mistral-small |
| Cohere | command-r-plus, command-r |

### Documentation

- [Project Overview](docs/PROJECT_OVERVIEW.md) - Introduction, tech stack, architecture
- [API Reference](docs/API_REFERENCE.md) - Complete API documentation
- [Module Reference](docs/MODULE_REFERENCE.md) - Module details and statistics
- [Database Schema](docs/DATABASE_SCHEMA.md) - Database structure
- [Deployment Guide](docs/DEPLOYMENT.md) - Deployment and operations
- [Development Guide](docs/DEVELOPMENT.md) - Development workflow and conventions

### Development

```bash
# Run in development mode
make dev

# Run tests
make test

# Run with coverage
make coverage

# Lint code
make lint
```

### Contributing

We welcome all contributions!

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing`)
5. Create a Pull Request

See [Development Guide](docs/DEVELOPMENT.md) for details.

### License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 中文文档

### 项目概述

FoxNIO 是一个使用 Rust 和 SvelteKit 构建的高性能 AI API 网关。它作为 AI 服务提供商（OpenAI、Anthropic、Google 等）的统一接入层，提供智能路由、故障转移、配额管理和监控告警等核心功能。

### 核心特性

#### 🚀 核心功能

- **OpenAI 兼容 API** - 可直接替换 OpenAI API 端点
- **多服务商支持** - OpenAI、Anthropic Claude、Google Gemini、DeepSeek、Mistral、Cohere
- **智能路由** - 模型别名解析和自动降级
- **高可用性** - 自动故障转移和指数退避重试

#### 🔐 安全与认证

- **JWT 认证** - 安全的令牌认证机制
- **API Key 管理** - 创建、管理和撤销 API Key
- **两步验证** - 基于 TOTP 的 2FA 支持
- **OAuth 集成** - GitHub、Google、LinuxDo、Antigravity
- **角色权限控制** - 细粒度的权限系统

#### 📊 监控与告警

- **Prometheus 指标** - 内置指标收集
- **健康检查** - 全面的健康监控
- **告警系统** - 多通道告警（邮件、Slack、钉钉、飞书）
- **审计日志** - 完整的操作历史记录

#### ⚡ 性能优化

- **HTTP/2 支持** - 现代协议，更优性能
- **响应压缩** - 自动 gzip/brotli 压缩
- **连接池** - 高效的连接管理
- **Redis 缓存** - 快速限流和会话存储

#### 💰 计费与配额

- **订阅管理** - 多级订阅计划
- **配额控制** - 按用户/组设置配额
- **使用统计** - 详细的使用量追踪
- **促销码** - 促销码和兑换码系统

### 快速开始

#### 环境要求

- Rust 1.75+
- Node.js 20+
- Docker & Docker Compose
- PostgreSQL 16
- Redis 7

#### Docker 部署（推荐）

```bash
# 克隆项目
git clone https://github.com/your-org/foxnio.git
cd foxnio

# 启动服务
docker-compose up -d

# API 地址：http://localhost:8080
# 前端地址：http://localhost:5173
```

#### 手动部署

```bash
# 1. 安装依赖
make install

# 2. 配置环境变量
cp .env.example .env
# 编辑 .env 文件

# 3. 启动数据库
make db-up

# 4. 运行迁移
make migrate

# 5. 启动服务
make run
```

### 支持的模型

| 服务商 | 模型 |
|--------|------|
| OpenAI | gpt-4-turbo, gpt-4o, gpt-4, gpt-3.5-turbo |
| Anthropic | claude-3-opus, claude-3.5-sonnet, claude-3-haiku |
| Google | gemini-1.5-pro, gemini-1.5-flash |
| DeepSeek | deepseek-chat, deepseek-coder |
| Mistral | mistral-large, mistral-medium, mistral-small |
| Cohere | command-r-plus, command-r |

### 文档目录

- [项目概述](docs/PROJECT_OVERVIEW.md) - 项目介绍、技术栈、架构设计
- [API 参考](docs/API_REFERENCE.md) - 完整的 API 文档
- [模块参考](docs/MODULE_REFERENCE.md) - 模块详情和统计
- [数据库架构](docs/DATABASE_SCHEMA.md) - 数据库结构说明
- [部署指南](docs/DEPLOYMENT.md) - 部署和运维
- [开发指南](docs/DEVELOPMENT.md) - 开发流程和规范

### 项目统计

| 项目 | 数量 |
|------|------|
| 后端代码行数 | 36,438+ |
| 测试用例 | 328+ |
| API 端点 | 49 |
| 数据库表 | 28 |
| 支持的服务商 | 6+ |

### 开发指南

```bash
# 开发模式运行
make dev

# 运行测试
make test

# 生成覆盖率报告
make coverage

# 代码检查
make lint
```

### 贡献指南

我们欢迎所有形式的贡献！

1. Fork 项目
2. 创建功能分支 (`git checkout -b feature/amazing`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing`)
5. 创建 Pull Request

详见 [开发指南](docs/DEVELOPMENT.md)。

### 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件

### 致谢

感谢所有贡献者和以下开源项目：

- [Axum](https://github.com/tokio-rs/axum) - Web 框架
- [Sea-ORM](https://github.com/SeaQL/sea-orm) - 数据库 ORM
- [SvelteKit](https://kit.svelte.dev/) - 前端框架
- [TailwindCSS](https://tailwindcss.com/) - CSS 框架

---

<div align="center">

Made with ❤️ by FoxNIO Team

**[文档](docs/)** | **[API 参考](docs/API_REFERENCE.md)** | **[部署指南](docs/DEPLOYMENT.md)**

</div>
