# FoxNIO 开发文档

## 开发环境搭建

### 前置要求

- **Rust**: 1.75+
- **Node.js**: 20+
- **PostgreSQL**: 16+
- **Redis**: 7+
- **Git**: 2.40+
- **Make**: 任意版本

### 安装依赖

#### Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 验证安装
rustc --version
cargo --version
```

#### Node.js

```bash
# 使用 nvm
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 20
nvm use 20

# 或使用包管理器
# Ubuntu/Debian
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt-get install -y nodejs

# macOS
brew install node@20
```

#### PostgreSQL

```bash
# Ubuntu/Debian
sudo apt-get install postgresql postgresql-contrib

# macOS
brew install postgresql@16

# 启动服务
sudo systemctl start postgresql  # Linux
brew services start postgresql   # macOS
```

#### Redis

```bash
# Ubuntu/Debian
sudo apt-get install redis-server

# macOS
brew install redis

# 启动服务
sudo systemctl start redis  # Linux
brew services start redis   # macOS
```

### 克隆项目

```bash
git clone https://github.com/your-org/foxnio.git
cd foxnio
```

### 配置环境

```bash
# 复制环境变量模板
cp .env.example .env

# 编辑配置
vim .env
```

**必要配置**:
```env
DATABASE_URL=postgres://foxnio:password@localhost:5432/foxnio
REDIS_URL=redis://localhost:6379/0
JWT_SECRET=dev-secret-key-for-testing-only
FOXNIO_MASTER_KEY=dev-master-key-32-bytes-12345678
RUST_LOG=foxnio=debug,tower_http=debug
```

### 初始化数据库

```bash
# 创建数据库
sudo -u postgres createdb foxnio

# 运行迁移
make migrate

# 或手动运行
cd backend
cargo run --bin migrate
```

### 安装项目依赖

```bash
# 后端依赖 (Rust 会自动处理)
cd backend
cargo build

# 前端依赖
cd frontend
npm install
```

---

## 项目结构

```
foxnio/
├── backend/                    # Rust 后端
│   ├── src/
│   │   ├── gateway/           # AI 网关核心
│   │   ├── service/           # 业务服务层
│   │   ├── handler/           # HTTP 处理器
│   │   ├── entity/            # 数据库实体
│   │   ├── model/             # 数据模型
│   │   ├── config/            # 配置管理
│   │   ├── utils/             # 工具函数
│   │   ├── alert/             # 告警系统
│   │   ├── health/            # 健康检查
│   │   ├── metrics/           # Prometheus 指标
│   │   ├── db/                # 数据库连接
│   │   ├── main.rs            # 入口文件
│   │   ├── lib.rs             # 库定义
│   │   └── state.rs           # 应用状态
│   ├── migration/             # 数据库迁移
│   ├── tests/                 # 集成测试
│   ├── Cargo.toml             # Rust 依赖
│   └── .env                   # 环境变量
├── frontend/                  # SvelteKit 前端
│   ├── src/
│   │   ├── routes/           # 页面路由
│   │   ├── components/       # UI 组件
│   │   ├── stores/           # 状态管理
│   │   ├── lib/              # 工具库
│   │   └── app.html          # HTML 模板
│   ├── static/               # 静态资源
│   ├── package.json          # Node 依赖
│   └── vite.config.ts        # Vite 配置
├── docs/                      # 文档
│   ├── PROJECT_OVERVIEW.md   # 项目概述
│   ├── API_REFERENCE.md      # API 参考
│   ├── MODULE_REFERENCE.md   # 模块参考
│   ├── DATABASE_SCHEMA.md    # 数据库架构
│   ├── DEPLOYMENT.md         # 部署文档
│   ├── DEVELOPMENT.md        # 开发文档
│   └── FINAL_DELIVERY.md     # 最终交付
├── scripts/                   # 脚本
├── Makefile                   # Make 命令
├── docker-compose.yml         # Docker 编排
├── nginx.conf                 # Nginx 配置
└── README.md                  # 项目说明
```

---

## 开发工作流

### 启动开发服务器

**后端**:
```bash
# 使用 Make
make run-backend

# 或手动
cd backend
cargo run
```

**前端**:
```bash
# 使用 Make
make run-frontend

# 或手动
cd frontend
npm run dev
```

**同时启动**:
```bash
make dev
```

### 热重载

**后端** (使用 cargo-watch):
```bash
cargo install cargo-watch
cargo watch -x run
```

**前端** (Vite 内置):
```bash
npm run dev
```

---

## 代码规范

### Rust 代码规范

**格式化**:
```bash
# 安装 rustfmt
rustup component add rustfmt

# 格式化代码
cargo fmt

# 检查格式
cargo fmt -- --check
```

**Linter**:
```bash
# 安装 clippy
rustup component add clippy

# 运行检查
cargo clippy

# 修复警告
cargo clippy --fix
```

**命名规范**:
- 函数/变量: `snake_case`
- 类型/Trait: `PascalCase`
- 常量: `SCREAMING_SNAKE_CASE`
- 模块: `snake_case`

**注释规范**:
```rust
/// 文档注释 (支持 Markdown)
/// 
/// # Examples
/// ```
/// let x = foo();
/// ```
pub fn foo() -> i32 {
    42
}

// 普通注释
// TODO: 需要优化性能
// FIXME: 已知问题
// NOTE: 重要说明
```

**错误处理**:
```rust
// 使用 Result<T, E>
pub fn create_user(email: &str) -> Result<User, ApiError> {
    // 使用 ? 操作符
    validate_email(email)?;
    
    let user = User::new(email);
    Ok(user)
}

// 自定义错误类型
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Invalid email: {0}")]
    InvalidEmail(String),
    
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}
```

### TypeScript 代码规范

**格式化** (Prettier):
```bash
# 安装
npm install -D prettier

# 格式化
npm run format

# 检查
npm run format:check
```

**Linter** (ESLint):
```bash
# 运行检查
npm run lint

# 修复问题
npm run lint:fix
```

**命名规范**:
- 变量/函数: `camelCase`
- 类/接口/类型: `PascalCase`
- 常量: `SCREAMING_SNAKE_CASE`
- 文件: `kebab-case`

---

## 测试指南

### 单元测试

**Rust 测试**:
```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_create_user

# 运行测试并显示输出
cargo test -- --nocapture

# 运行特定模块的测试
cargo test --lib service::user
```

**测试示例**:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_user_creation() {
        let user = User::new("test@example.com");
        assert_eq!(user.email, "test@example.com");
    }
    
    #[tokio::test]
    async fn test_async_operation() {
        let result = async_function().await;
        assert!(result.is_ok());
    }
}
```

### 集成测试

```bash
# 运行集成测试
cargo test --test '*'

# 运行特定集成测试
cargo test --test api_test
```

**集成测试示例** (`tests/api_test.rs`):
```rust
use axum_test::TestServer;

#[tokio::test]
async fn test_health_endpoint() {
    let app = create_test_app().await;
    let server = TestServer::new(app).unwrap();
    
    let response = server.get("/health").await;
    
    assert_eq!(response.status_code(), StatusCode::OK);
    assert!(response.json::<HealthResponse>().status == "healthy");
}
```

### 测试覆盖率

```bash
# 安装 tarpaulin
cargo install cargo-tarpaulin

# 生成覆盖率报告
cargo tarpaulin --out Html --output-dir coverage

# 查看
open coverage/tarpaulin-report.html
```

### 前端测试

```bash
# 运行测试
npm run test

# 监听模式
npm run test:watch

# 覆盖率
npm run test:coverage
```

---

## Git 工作流

### 分支策略

```
main           # 主分支 (生产环境)
  └── develop  # 开发分支
       ├── feature/user-auth     # 功能分支
       ├── feature/billing       # 功能分支
       └── bugfix/api-key-issue  # 修复分支
```

### 提交规范

使用 [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: 添加新功能
fix: 修复 bug
docs: 文档更新
style: 代码格式调整
refactor: 重构代码
perf: 性能优化
test: 测试相关
chore: 构建/工具相关
```

**示例**:
```bash
feat: 添加用户两步验证功能
fix: 修复 API Key 验证逻辑错误
docs: 更新部署文档
refactor: 重构服务层代码结构
```

### Pull Request 流程

1. **创建功能分支**
```bash
git checkout develop
git pull
git checkout -b feature/new-feature
```

2. **开发并提交**
```bash
git add .
git commit -m "feat: 添加新功能"
git push origin feature/new-feature
```

3. **创建 Pull Request**
- 在 GitHub 上创建 PR
- 填写 PR 模板
- 关联相关 Issue

4. **代码审查**
- 等待 CI 通过
- 响应审查意见
- 修改代码

5. **合并**
- Squash and merge
- 删除功能分支

---

## 调试技巧

### 后端调试

**日志调试**:
```rust
use tracing::{debug, info, warn, error};

debug!("调试信息: {:?}", data);
info!("处理请求: {}", request_id);
warn!("警告: {}", warning_msg);
error!("错误: {:?}", error);
```

**设置日志级别**:
```bash
RUST_LOG=foxnio=debug cargo run
```

**调试器** (rust-gdb):
```bash
rust-gdb ./target/debug/foxnio
```

### 数据库调试

**查看 SQL 查询**:
```bash
# 启用 SQL 日志
RUST_LOG=sqlx::query=debug cargo run
```

**使用 pgAdmin**:
```bash
docker run -p 5050:80 \
  -e PGADMIN_DEFAULT_EMAIL=admin@localhost \
  -e PGADMIN_DEFAULT_PASSWORD=admin \
  dpage/pgadmin4
```

### Redis 调试

```bash
# 监控 Redis 命令
redis-cli monitor

# 查看所有 keys
redis-cli keys "foxnio:*"
```

---

## 性能优化

### 性能分析

**使用 perf**:
```bash
# 记录性能数据
perf record -g ./target/release/foxnio

# 分析
perf report
```

**火焰图**:
```bash
# 安装 flamegraph
cargo install flamegraph

# 生成火焰图
cargo flamegraph
```

### 常见优化点

1. **数据库查询优化**
- 使用索引
- 避免 N+1 查询
- 批量操作

2. **缓存优化**
- Redis 缓存热点数据
- LRU 缓存策略

3. **异步优化**
- 使用 async/await
- 避免阻塞操作

---

## 贡献指南

### 贡献流程

1. Fork 项目
2. 创建功能分支
3. 编写代码和测试
4. 提交 Pull Request
5. 等待代码审查
6. 合并到主分支

### 代码审查清单

- [ ] 代码符合规范
- [ ] 测试通过
- [ ] 文档已更新
- [ ] 无安全漏洞
- [ ] 性能无明显下降
- [ ] Commit 消息清晰

### 文档贡献

- 修正拼写错误
- 补充缺失文档
- 改进示例代码
- 翻译文档

---

## 发布流程

### 版本号规范

使用语义化版本: `MAJOR.MINOR.PATCH`

- `MAJOR`: 不兼容的 API 变更
- `MINOR`: 向后兼容的功能新增
- `PATCH`: 向后兼容的问题修复

### 发布步骤

1. 更新版本号
2. 更新 CHANGELOG.md
3. 运行完整测试
4. 创建 Git Tag
5. 构建发布版本
6. 部署到生产环境

```bash
# 创建 tag
git tag -a v1.0.0 -m "Release v1.0.0"
git push origin v1.0.0

# 构建发布版本
cargo build --release
```
