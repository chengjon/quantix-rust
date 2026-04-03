# Phase 19 生产部署指南

## 部署架构

```
                    ┌─────────────────────────────────────┐
                    │         Internet (HTTPS)            │
                    └─────────────────┬───────────────────┘
                                      │
                    ┌─────────────────▼───────────────────┐
                    │          Traefik (反向代理)          │
                    │   - SSL/TLS 终止                    │
                    │   - 负载均衡                        │
                    │   - 路由管理                        │
                    └─────────────────┬───────────────────┘
                                      │
            ┌─────────────────────────┼─────────────────────────┐
            │                         │                         │
┌───────────▼───────────┐ ┌──────────▼──────────┐ ┌───────────▼───────────┐
│   Quantix (应用)       │ │   Grafana (监控)    │ │  Prometheus (指标)    │
│   - API 服务          │ │   - 仪表板          │ │  - 数据采集          │
│   - 健康检查          │ │   - 告警            │ │  - 存储              │
└───────────┬───────────┘ └─────────────────────┘ └───────────────────────┘
            │
    ┌───────┴───────┐
    │               │
┌───▼────┐    ┌─────▼──────┐
│PostgreSQL│    │ ClickHouse │
│(关系数据)│    │(时序数据)  │
└─────────┘    └────────────┘
```

## 快速部署

### 1. 前置准备

```bash
# 安装 Docker
curl -fsSL https://get.docker.com | sh

# 安装 Docker Compose
sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" \
  -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose

# 克隆项目
git clone https://github.com/chengjon/quantix-rust.git
cd quantix-rust
```

### 2. 配置环境变量

```bash
# 创建配置文件
cp .env.example .env

# 编辑配置（必须修改密码！）
nano .env
```

**.env 必填项**:
```bash
IMAGE_NAME=ghcr.io/your-org/quantix-rust/quantix
POSTGRES_PASSWORD=your_secure_password
CLICKHOUSE_PASSWORD=your_secure_password
GRAFANA_ADMIN_PASSWORD=your_secure_password
ACME_EMAIL=your-email@example.com
QUANTIX_PUBLIC_HOST=quantix.your-domain.com
GRAFANA_PUBLIC_HOST=grafana.your-domain.com
TRAEFIK_PUBLIC_HOST=traefik.your-domain.com
```

### 3. 启动服务

```bash
# 生产环境
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d

# 查看状态
docker-compose ps

# 查看日志
docker-compose logs -f quantix
```

## 服务管理

### 启动/停止

```bash
# 启动所有服务
docker-compose up -d

# 停止所有服务
docker-compose down

# 重启单个服务
docker-compose restart quantix

# 查看状态
docker-compose ps
```

### 日志查看

```bash
# 查看所有日志
docker-compose logs -f

# 查看特定服务
docker-compose logs -f quantix

# 查看最近100行
docker-compose logs --tail=100 quantix
```

### 资源监控

```bash
# 查看资源使用
docker stats

# 查看容器详情
docker-compose exec quantix top
```

## 数据管理

### 备份

```bash
# PostgreSQL 备份
docker-compose exec -T postgres pg_dump -U quantix quantix > backup_$(date +%Y%m%d).sql

# ClickHouse 备份
docker-compose exec clickhouse clickhouse-backup create backup_$(date +%Y%m%d)
```

### 恢复

```bash
# PostgreSQL 恢复
cat backup_20260308.sql | docker-compose exec -T postgres psql -U quantix quantix

# ClickHouse 恢复
docker-compose exec clickhouse clickhouse-backup restore backup_20260308
```

### 数据卷管理

```bash
# 列出卷
docker volume ls

# 查看卷详情
docker volume inspect quantix-rust_postgres-data

# 清理未使用卷
docker volume prune
```

## 更新部署

### 滚动更新

```bash
# 1. 拉取最新镜像
docker-compose pull quantix

# 2. 重新创建容器
docker-compose up -d --no-deps quantix

# 3. 验证健康状态
docker-compose ps
curl http://localhost:8080/health
```

### 版本回滚

```bash
# 1. 指定旧版本
export VERSION=v1.0.0

# 2. 重新部署
docker-compose up -d --no-deps quantix

# 3. 验证
docker-compose logs quantix
```

## 故障排查

### 常见问题

#### 1. 容器无法启动

```bash
# 查看错误日志
docker-compose logs quantix

# 检查配置
docker-compose config

# 检查资源
docker system df
```

#### 2. 数据库连接失败

```bash
# 检查 PostgreSQL 状态
docker-compose exec postgres pg_isready -U quantix

# 检查网络
docker network inspect quantix-network

# 测试连接
docker-compose exec quantix ping postgres
```

#### 3. 内存不足

```bash
# 查看内存使用
docker stats --no-stream

# 重启服务
docker-compose restart

# 清理资源
docker system prune -a
```

### 健康检查

```bash
# 应用健康
curl http://localhost:8080/health

# 数据库健康
docker-compose exec postgres pg_isready

# ClickHouse 健康
curl http://localhost:8123/ping
```

## 性能优化

### PostgreSQL

```sql
-- 查看连接数
SELECT count(*) FROM pg_stat_activity;

-- 查看慢查询
SELECT * FROM pg_stat_statements ORDER BY total_time DESC LIMIT 10;

-- 优化建议
VACUUM ANALYZE;
```

### ClickHouse

```sql
-- 查看查询性能
SELECT * FROM system.query_log WHERE type = 'QueryFinish'
ORDER BY query_duration_ms DESC LIMIT 10;

-- 优化表
OPTIMIZE TABLE quantix.klines FINAL;
```

### Docker

```yaml
# 调整资源限制
deploy:
  resources:
    limits:
      cpus: '4'
      memory: 8G
```

## 安全加固

### 1. 网络安全

```bash
# 配置防火墙
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
sudo ufw enable

# 限制内部服务访问
# (docker-compose.prod.yml 已配置 internal 网络)
```

### 2. 密码管理

```bash
# 使用 Docker Secrets
echo "your_password" | docker secret create postgres_password -

# 更新 compose 文件
secrets:
  postgres_password:
    external: true
```

### 3. 定期更新

```bash
# 更新基础镜像
docker-compose pull

# 重建容器
docker-compose up -d

# 清理旧镜像
docker image prune -a
```

## 监控告警

### Prometheus 指标

访问 `http://localhost:9090` 查看指标。

**关键指标**:
- `http_requests_total` - HTTP 请求总数
- `http_request_duration_seconds` - 请求延迟
- `db_connections_active` - 活跃数据库连接
- `process_resident_memory_bytes` - 内存使用

### Grafana 仪表板

访问 `https://$GRAFANA_PUBLIC_HOST`。

**预配置仪表板**:
- Quantix 应用监控
- PostgreSQL 性能
- ClickHouse 性能
- 系统资源

### 告警规则

编辑 `monitoring/alerts.yml` 配置告警规则。

## 联系支持

- GitHub Issues: https://github.com/chengjon/quantix-rust/issues
- 文档: docs/guides/
- 邮件: support@example.com

---

**最后更新**: 2026-03-09
**维护者**: MyStocks Team
