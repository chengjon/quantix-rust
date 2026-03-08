# Docker 部署指南

本文档介绍如何使用 Docker 部署 quantix-rust 项目。

## 前置要求

- Docker 20.10+
- Docker Compose 2.0+
- 至少 4GB 可用内存
- 至少 10GB 可用磁盘空间

## 快速开始

### 1. 克隆项目

```bash
git clone https://github.com/chengjon/quantix-rust.git
cd quantix-rust
```

### 2. 配置环境变量

```bash
cp .env.example .env
# 编辑 .env 文件，配置数据库连接等信息
```

### 3. 启动所有服务

```bash
# 启动完整栈（应用 + 数据库 + 监控）
docker-compose up -d

# 查看服务状态
docker-compose ps

# 查看日志
docker-compose logs -f quantix
```

### 4. 访问服务

- **Quantix 应用**: http://localhost:8080
- **Prometheus**: http://localhost:9090
- **Grafana**: http://localhost:3000 (admin/admin)
- **Loki**: http://localhost:3100
- **PgAdmin** (可选): http://localhost:5050

## 常用命令

### 服务管理

```bash
# 启动所有服务
docker-compose up -d

# 停止所有服务
docker-compose down

# 重启服务
docker-compose restart quantix

# 查看日志
docker-compose logs -f [service_name]

# 进入容器
docker-compose exec quantix bash
docker-compose exec postgres psql -U quantix -d quantix
docker-compose exec clickhouse clickhouse-client
```

### 数据管理

```bash
# 查看数据卷
docker volume ls | grep quantix

# 备份数据卷
docker run --rm -v quantix-postgres-data:/data -v $(pwd):/backup \
  debian:bookworm-slim tar czf /backup/postgres-backup.tar.gz -C /data .

# 恢复数据卷
docker run --rm -v quantix-postgres-data:/data -v $(pwd):/backup \
  debian:bookworm-slim tar xzf /backup/postgres-backup.tar.gz -C /data
```

### 开发模式

```bash
# 使用开发环境 Dockerfile（支持热重载）
docker-compose -f docker-compose.yml build --build-arg TARGET=dev quantix
docker-compose up -d quantix

# 查看实时日志
docker-compose logs -f quantix
```

## 服务配置

### Quantix 应用

**环境变量**:
```bash
RUST_LOG=info                    # 日志级别
POSTGRES_HOST=postgres           # PostgreSQL 主机
POSTGRES_PORT=5432               # PostgreSQL 端口
POSTGRES_DB=quantix              # 数据库名
POSTGRES_USER=quantix            # 用户名
POSTGRES_PASSWORD=quantix123     # 密码
CLICKHOUSE_URL=http://clickhouse:8123
CLICKHOUSE_DB=quantix
```

**健康检查**:
```bash
curl http://localhost:8080/health
```

### PostgreSQL

**连接信息**:
- Host: localhost:5432
- Database: quantix
- User: quantix
- Password: quantix123

**命令行连接**:
```bash
docker-compose exec postgres psql -U quantix -d quantix
```

### ClickHouse

**连接信息**:
- HTTP Port: 8123
- Native Port: 9000
- Database: quantix

**命令行连接**:
```bash
docker-compose exec clickhouse clickhouse-client
```

## 监控和日志

### Prometheus 指标

访问 http://localhost:9090 查看：
- 应用指标
- 数据库指标
- 系统资源指标

**常用查询**:
```promql
# CPU 使用率
rate(http_requests_total[5m])

# 内存使用
process_resident_memory_bytes

# 错误率
rate(http_requests_total{status=~"5.."}[5m])
```

### Grafana 仪表板

访问 http://localhost:3000

**预配置仪表板**:
- Quantix 应用监控
- PostgreSQL 性能
- ClickHouse 性能
- 系统资源

### 日志查询

使用 Loki + Promtail 查询日志：

```bash
# 查询所有日志
{job="quantix"}

# 查询错误日志
{job="quantix"} |= "ERROR"

# 查询特定时间范围
{job="quantix"} | json | line_format "{{.timestamp}} {{.level}} {{.message}}"
```

## 故障排查

### 容器启动失败

```bash
# 查看详细日志
docker-compose logs [service_name]

# 检查容器状态
docker-compose ps

# 检查资源使用
docker stats
```

### 数据库连接问题

```bash
# 检查 PostgreSQL 是否就绪
docker-compose exec postgres pg_isready -U quantix

# 检查 ClickHouse 是否就绪
docker-compose exec clickhouse clickhouse-client --query "SELECT 1"

# 查看网络连接
docker network inspect quantix-network
```

### 性能问题

```bash
# 查看容器资源使用
docker stats

# 查看数据库性能
docker-compose exec postgres psql -U quantix -d quantix -c \
  "SELECT * FROM pg_stat_activity WHERE datname = 'quantix';"

# 查看 Prometheus 指标
curl http://localhost:9090/api/v1/query?query=up
```

## 数据备份和恢复

### PostgreSQL 备份

```bash
# 创建备份
docker-compose exec postgres pg_dump -U quantix quantiz > backup.sql

# 恢复备份
docker-compose exec -T postgres psql -U quantix quantix < backup.sql
```

### ClickHouse 备份

```bash
# 使用 clickhouse-backup
docker-compose exec clickhouse clickhouse-backup create backup_name

# 恢复备份
docker-compose exec clickhouse clickhouse-backup restore backup_name
```

### 完整备份脚本

```bash
#!/bin/bash
# scripts/backup/backup-all.sh

DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="./backups/$DATE"

mkdir -p "$BACKUP_DIR"

# PostgreSQL
docker-compose exec -T postgres pg_dump -U quantix quantix > \
  "$BACKUP_DIR/postgres.sql"

# ClickHouse
docker-compose exec clickhouse clickhouse-backup create "backup_$DATE"

# 配置文件
cp -r config "$BACKUP_DIR/"
cp .env "$BACKUP_DIR/"

echo "Backup completed: $BACKUP_DIR"
```

## 生产环境部署

### 使用生产配置

```bash
# 使用生产环境 docker-compose
docker-compose -f docker-compose.prod.yml up -d
```

### 资源限制

在 `docker-compose.yml` 中配置：

```yaml
services:
  quantix:
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 4G
        reservations:
          cpus: '1'
          memory: 2G
```

### 安全建议

1. **修改默认密码**
   ```bash
   # 编辑 .env 文件
   POSTGRES_PASSWORD=your_strong_password
   ```

2. **使用 secrets 管理**
   ```bash
   echo "your_password" | docker secret create postgres_password -
   ```

3. **启用 TLS**
   ```yaml
   environment:
     - POSTGRES_TLS_ENABLED=on
     - POSTGRES_TLS_CERT_FILE=/certs/server.crt
     - POSTGRES_TLS_KEY_FILE=/certs/server.key
   ```

4. **网络隔离**
   ```yaml
   networks:
     internal:
       internal: true
     external:
       driver: bridge
   ```

## 性能优化

### PostgreSQL 优化

在 `scripts/init-postgres.sql` 中配置：

```sql
-- 增加共享缓冲区
ALTER SYSTEM SET shared_buffers = '1GB';

-- 增加工作内存
ALTER SYSTEM SET work_mem = '256MB';

-- 优化检查点
ALTER SYSTEM SET checkpoint_completion_target = 0.9;
```

### ClickHouse 优化

```xml
<!-- clickhouse-config.xml -->
<max_memory_usage>10000000000</max_memory_usage>
<max_threads>4</max_threads>
```

### 应用优化

```yaml
environment:
  - RUST_LOG=info  # 避免使用 debug 级别
  - TOKIO_WORKER_THREADS=4
```

## 更新和维护

### 更新应用

```bash
# 拉取最新镜像
docker-compose pull quantix

# 重新构建
docker-compose build quantix

# 重启服务
docker-compose up -d quantix
```

### 清理旧镜像

```bash
# 删除未使用的镜像
docker image prune -a

# 删除未使用的卷
docker volume prune

# 删除未使用的网络
docker network prune
```

## 多环境部署

### 开发环境

```bash
docker-compose -f docker-compose.yml up -d
```

### 测试环境

```bash
docker-compose -f docker-compose.yml -f docker-compose.test.yml up -d
```

### 生产环境

```bash
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d
```

## 参考资源

- [Docker 文档](https://docs.docker.com/)
- [Docker Compose 文档](https://docs.docker.com/compose/)
- [PostgreSQL Docker 镜像](https://hub.docker.com/_/postgres)
- [ClickHouse Docker 镜像](https://hub.docker.com/r/clickhouse/clickhouse-server)
- [Prometheus 文档](https://prometheus.io/docs/)
- [Grafana 文档](https://grafana.com/docs/)

## 支持

如有问题，请查看：
- [故障排查文档](TROUBLESHOOTING.md)
- [GitHub Issues](https://github.com/chengjon/quantix-rust/issues)
- [运维手册](docs/operations/)

---

**最后更新**: 2026-03-08
**作者**: MyStocks Team
