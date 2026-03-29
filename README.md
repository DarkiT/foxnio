# FoxNIO

AI API 网关 - 统一接入多个 AI 服务商

## 一句话说明

用 OpenAI 兼容的 API 格式，访问 OpenAI、Claude、Gemini、DeepSeek、Mistral、Cohere。

## 快速开始

```bash
# 1. 启动依赖服务
docker-compose up -d postgres redis

# 2. 配置
cp .env.example .env
# 编辑 .env，填入你的 API Key

# 3. 运行
make install && make run

# API: http://localhost:8080
# 管理后台: http://localhost:5173
```

## API 调用

```bash
curl http://localhost:8080/v1/chat/completions \
  -H "Authorization: Bearer foxnio-your-key" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-4o",
    "messages": [{"role": "user", "content": "Hello"}]
  }'
```

## 支持的服务商

FoxNIO 支持 6 大主流 AI 服务商，模型配置完全**动态管理**：

| 服务商 | 状态 | 特性 |
|--------|------|------|
| OpenAI | ✅ | 支持所有 GPT 系列模型 |
| Anthropic | ✅ | 支持 Claude 全系列 |
| Google (Gemini) | ✅ | 原生 API 支持 |
| DeepSeek | ✅ | 支持最新 V3/Coder |
| Mistral | ✅ | 支持 Mistral 全系列 |
| Cohere | ✅ | 支持 Command 系列 |

### 动态模型管理

- **数据库持久化**: 模型配置存储在数据库中，无需重启即可更新
- **热加载**: 通过 API 实时更新模型配置
- **自动同步**: 支持从各服务商自动同步最新模型（计划中）
- **模型路由**: 支持别名映射、自动降级、负载均衡

**查看当前模型列表**:
```bash
curl http://localhost:8080/v1/models \
  -H "Authorization: Bearer foxnio-your-key"
```

**管理模型配置** (需要管理员权限):
```bash
# 列出所有模型
curl http://localhost:8080/api/v1/admin/models \
  -H "Authorization: Bearer <admin-token>"

# 导入默认模型
curl -X POST http://localhost:8080/api/v1/admin/models/import-defaults \
  -H "Authorization: Bearer <admin-token>"

# 热加载模型配置
curl -X POST http://localhost:8080/api/v1/admin/models/reload \
  -H "Authorization: Bearer <admin-token>"
```

详见 [动态模型配置文档](backend/docs/DYNAMIC_MODEL_IMPLEMENTATION.md)

## 核心功能

- **统一 API** - OpenAI 兼容格式，切换服务商无需改代码
- **智能调度** - 自动选择最优账户，故障自动转移
- **用量计费** - 按请求计费，配额管理
- **流式响应** - 支持 SSE/WebSocket 流式输出
- **管理后台** - 用户、账户、账单管理

## 必需配置

```bash
# .env
DATABASE_URL=postgres://foxnio:password@localhost:5432/foxnio
REDIS_URL=redis://localhost:6379
JWT_SECRET=随机字符串

# 至少配置一个 AI 服务商
OPENAI_API_KEY=sk-xxx
# 或
ANTHROPIC_API_KEY=sk-ant-xxx
```

## 文档

- [API 参考](docs/API_REFERENCE.md)
- [部署指南](docs/DEPLOYMENT.md)
- [开发指南](docs/DEVELOPMENT.md)

## 许可证

MIT
