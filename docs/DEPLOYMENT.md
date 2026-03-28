# FoxNIO 部署文档

## 环境要求

### 硬件要求

| 配置项 | 最低要求 | 推荐配置 |
|--------|----------|----------|
| CPU | 2 核 | 4+ 核 |
| 内存 | 4 GB | 8+ GB |
| 存储 | 20 GB | 100+ GB SSD |
| 网络 | 10 Mbps | 100+ Mbps |

### 软件要求

| 软件 | 版本要求 |
|------|----------|
| Rust | 1.75+ |
| Node.js | 20+ |
| PostgreSQL | 16+ |
| Redis | 7+ |
| Docker | 24+ |
| Docker Compose | 2.20+ |

---

## 部署方式

### 方式一: Docker Compose (推荐)

#### 1. 克隆项目

```bash
git clone https://github.com/your-org/foxnio.git
cd foxnio
```

#### 2. 配置环境变量

```bash
cp .env.example .env
vim .env
```

**必要配置项**:
```env
# 数据库
DATABASE_URL=postgres://foxnio:password@postgres:5432/foxnio

# Redis
REDIS_URL=redis://:password@redis:6379/0

# JWT
JWT_SECRET=your-secret-key-here
JWT_EXPIRE_HOURS=24

# 加密密钥
FOXNIO_MASTER_KEY=your-master-key-32-bytes

# 邮件服务
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USER=your-email@gmail.com
SMTP_PASS=your-app-password
```

#### 3. 启动服务

```bash
# 启动所有服务
docker-compose up -d

# 查看日志
docker-compose logs -f

# 查看服务状态
docker-compose ps
```

#### 4. 初始化数据库

```bash
# 运行迁移
docker-compose exec backend make migrate

# 创建管理员用户
docker-compose exec backend make create-admin
```

#### 5. 验证部署

```bash
# 健康检查
curl http://localhost:8080/health

# 访问前端
open http://localhost:5173
```

---

### 方式二: 手动部署

#### 1. 安装依赖

**Rust**:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

**Node.js**:
```bash
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt-get install -y nodejs
```

**PostgreSQL**:
```bash
sudo apt-get install postgresql postgresql-contrib
sudo systemctl start postgresql
sudo systemctl enable postgresql
```

**Redis**:
```bash
sudo apt-get install redis-server
sudo systemctl start redis
sudo systemctl enable redis
```

#### 2. 编译后端

```bash
cd foxnio/backend
cargo build --release
```

#### 3. 编译前端

```bash
cd foxnio/frontend
npm install
npm run build
```

#### 4. 配置数据库

```bash
# 创建数据库
sudo -u postgres createdb foxnio

# 创建用户
sudo -u postgres psql -c "CREATE USER foxnio WITH PASSWORD 'password';"
sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE foxnio TO foxnio;"

# 运行迁移
cd backend
cargo run --bin migrate
```

#### 5. 启动服务

**后端**:
```bash
cd backend
./target/release/foxnio
```

**前端** (使用 Nginx):
```nginx
server {
    listen 80;
    server_name your-domain.com;

    location / {
        root /path/to/foxnio/frontend/build;
        try_files $uri $uri/ /index.html;
    }

    location /api/ {
        proxy_pass http://localhost:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

---

## 配置说明

### 后端配置 (config.yaml)

```yaml
server:
  host: "0.0.0.0"
  port: 8080

database:
  host: "localhost"
  port: 5432
  user: "foxnio"
  password: "password"
  dbname: "foxnio"
  max_connections: 20

redis:
  host: "localhost"
  port: 6379
  password: "password"
  db: 0

jwt:
  secret: "your-jwt-secret"
  expire_hours: 24

gateway:
  api_key_prefix: "foxnio-"
  rate_multiplier: 1.0
  default_timeout: 60

email:
  smtp_host: "smtp.gmail.com"
  smtp_port: 587
  smtp_user: "your-email@gmail.com"
  smtp_pass: "your-app-password"
  from_address: "noreply@foxnio.com"

monitoring:
  prometheus_enabled: true
  prometheus_port: 9090
```

### 环境变量

| 变量名 | 说明 | 默认值 |
|--------|------|--------|
| FOXNIO_MASTER_KEY | 主加密密钥 | - |
| DATABASE_URL | 数据库连接串 | - |
| REDIS_URL | Redis 连接串 | - |
| JWT_SECRET | JWT 密钥 | - |
| RUST_LOG | 日志级别 | info |

---

## 生产环境部署

### 1. 使用 HTTPS

**Let's Encrypt**:
```bash
sudo apt-get install certbot python3-certbot-nginx
sudo certbot --nginx -d your-domain.com
```

**Nginx 配置**:
```nginx
server {
    listen 443 ssl http2;
    server_name your-domain.com;

    ssl_certificate /etc/letsencrypt/live/your-domain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/your-domain.com/privkey.pem;

    # SSL 优化
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;
    ssl_prefer_server_ciphers on;

    location / {
        root /var/www/foxnio/frontend;
        try_files $uri $uri/ /index.html;
    }

    location /api/ {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}

server {
    listen 80;
    server_name your-domain.com;
    return 301 https://$server_name$request_uri;
}
```

### 2. 数据库优化

**PostgreSQL 配置** (`/etc/postgresql/16/main/postgresql.conf`):
```conf
# 连接
max_connections = 200

# 内存
shared_buffers = 256MB
effective_cache_size = 768MB
work_mem = 4MB
maintenance_work_mem = 64MB

# WAL
wal_buffers = 16MB

# 查询优化
random_page_cost = 1.1
effective_io_concurrency = 200

# 日志
logging_collector = on
log_directory = 'pg_log'
log_filename = 'postgresql-%Y-%m-%d.log'
```

### 3. Redis 优化

**Redis 配置** (`/etc/redis/redis.conf`):
```conf
# 内存
maxmemory 2gb
maxmemory-policy allkeys-lru

# 持久化
appendonly yes
appendfsync everysec

# 性能
save 900 1
save 300 10
save 60 10000
```

### 4. 系统优化

**系统限制** (`/etc/security/limits.conf`):
```
* soft nofile 65535
* hard nofile 65535
* soft nproc 65535
* hard nproc 65535
```

**内核参数** (`/etc/sysctl.conf`):
```
net.core.somaxconn = 65535
net.ipv4.tcp_max_syn_backlog = 65535
net.ipv4.ip_local_port_range = 1024 65535
net.ipv4.tcp_tw_reuse = 1
```

---

## 监控与告警

### Prometheus 配置

```yaml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'foxnio'
    static_configs:
      - targets: ['localhost:8080']
```

### Grafana Dashboard

导入 `grafana-dashboard.json` 到 Grafana。

监控指标:
- 请求 QPS
- 响应延迟 (P50, P90, P99)
- 错误率
- 数据库连接数
- Redis 内存使用
- CPU/内存使用率

### 告警规则

**Prometheus 告警规则** (`alerts.yml`):
```yaml
groups:
  - name: foxnio
    rules:
      - alert: HighErrorRate
        expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.05
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: High error rate detected

      - alert: HighLatency
        expr: histogram_quantile(0.99, rate(http_request_duration_seconds_bucket[5m])) > 1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: High latency detected
```

---

## 备份与恢复

### 数据库备份

**自动备份脚本** (`/usr/local/bin/backup-foxnio.sh`):
```bash
#!/bin/bash
DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="/backup/foxnio"

mkdir -p $BACKUP_DIR

# PostgreSQL 备份
pg_dump foxnio | gzip > $BACKUP_DIR/foxnio_$DATE.sql.gz

# Redis 备份
redis-cli BGSAVE
cp /var/lib/redis/dump.rdb $BACKUP_DIR/redis_$DATE.rdb

# 清理旧备份 (保留 7 天)
find $BACKUP_DIR -type f -mtime +7 -delete
```

**Cron 定时任务**:
```cron
0 2 * * * /usr/local/bin/backup-foxnio.sh >> /var/log/foxnio-backup.log 2>&1
```

### 数据恢复

```bash
# PostgreSQL 恢复
gunzip -c foxnio_20240328.sql.gz | psql foxnio

# Redis 恢复
systemctl stop redis
cp redis_20240328.rdb /var/lib/redis/dump.rdb
systemctl start redis
```

---

## 故障排查

### 常见问题

#### 1. 无法连接数据库

```bash
# 检查 PostgreSQL 状态
systemctl status postgresql

# 检查连接
psql -h localhost -U foxnio -d foxnio

# 查看日志
tail -f /var/log/postgresql/postgresql-16-main.log
```

#### 2. Redis 连接失败

```bash
# 检查 Redis 状态
systemctl status redis

# 测试连接
redis-cli ping

# 查看日志
tail -f /var/log/redis/redis-server.log
```

#### 3. 服务启动失败

```bash
# 查看后端日志
journalctl -u foxnio -f

# 检查端口占用
netstat -tlnp | grep 8080

# 检查配置
./target/release/foxnio --check-config
```

---

## 运维指南

### 日常运维

**查看服务状态**:
```bash
docker-compose ps
docker-compose logs -f --tail=100 backend
```

**重启服务**:
```bash
docker-compose restart backend
```

**更新部署**:
```bash
git pull
docker-compose build
docker-compose up -d
```

### 性能调优

**数据库慢查询**:
```sql
SELECT query, calls, total_time, mean_time
FROM pg_stat_statements
ORDER BY mean_time DESC
LIMIT 10;
```

**Redis 内存分析**:
```bash
redis-cli info memory
redis-cli memory usage foxnio:user:1
```

### 安全加固

1. 定期更新系统和依赖
2. 使用防火墙限制访问
3. 启用 TLS 1.3
4. 配置 fail2ban
5. 定期备份数据

---

## 升级指南

### 版本升级

```bash
# 备份数据
make backup

# 拉取新版本
git fetch --tags
git checkout vx.x.x

# 运行迁移
make migrate

# 重启服务
docker-compose up -d
```

### 回滚

```bash
# 回滚到上一版本
git checkout HEAD~1

# 恢复数据
make restore BACKUP=backup_20240328.sql
```
