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

| 服务商 | 模型示例 |
|--------|----------|
| OpenAI | gpt-4o, gpt-4-turbo, gpt-3.5-turbo |
| Anthropic | claude-3.5-sonnet, claude-3-opus |
| Google | gemini-1.5-pro, gemini-1.5-flash |
| DeepSeek | deepseek-chat, deepseek-coder |
| Mistral | mistral-large, mistral-medium |
| Cohere | command-r, command-r-plus |

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
