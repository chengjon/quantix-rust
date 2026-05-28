# 部署指南

本文档介绍如何部署 quantix-rust 到生产环境。

## 前置要求

### 系统要求

| 项目 | 最低配置 | 推荐配置 |
|------|----------|----------|
| CPU | 2 核 | 4 核 |
| 内存 | 4 GB | 8 GB |
| 磁盘 | 20 GB | 50 GB SSD |
| 操作系统 | Ubuntu 20.04+ | Ubuntu 22.04 LTS |

### 软件要求

- Docker 20.10+
- Docker Compose 2.0+
- Git
- curl, wget

## 环境准备

### 1. 安装 Docker

```bash
# Ubuntu/Debian
curl -fsSL https://get.docker.com | sh
sudo usermod -aG docker $USER

# 重新登录以应用组权限
```

### 2. 安装 Docker Compose

```bash
sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose
```

### 3. 克隆项目

```bash
git clone https://github.com/chengjon/quantix-rust.git
cd quantix-rust
```

## 配置

### 1. 创建环境变量文件

```bash
cp .env.example .env
```

### 2. 编辑环境变量

```bash
# .env 文件内容
# 镜像配置
IMAGE_NAME=ghcr.io/your-org/quantix-rust/quantix

# 数据库密码（生产环境必须修改！）
POSTGRES_PASSWORD=your_secure_postgres_password
CLICKHOUSE_PASSWORD=your_secure_clickhouse_password

# Grafana 配置
GRAFANA_ADMIN_USER=admin
GRAFANA_ADMIN_PASSWORD=your_secure_grafana_password

# Traefik 配置
ACME_EMAIL=your-email@example.com
QUANTIX_PUBLIC_HOST=quantix.your-domain.com
GRAFANA_PUBLIC_HOST=grafana.your-domain.com
TRAEFIK_PUBLIC_HOST=traefik.your-domain.com
TRAEFIK_AUTH_USERS=admin:$$apr1$$xxx...  # htpasswd 生成

# 应用版本
VERSION=latest  # 或指定版本号如 v1.0.0
```

### 3. 生成 Traefik 认证

```bash
# 安装 htpasswd 工具
sudo apt-get install apache2-utils

# 生成密码
htpasswd -nb admin your_password
# 输出: admin:$apr1$xxx...
# 复制到 .env 文件的 TRAEFIK_AUTH_USERS（注意转义 $ 为 $$）
```

## 部署

### 开发环境

```bash
# 启动所有服务
docker-compose up -d

# 查看日志
docker-compose logs -f quantix

# 检查状态
docker-compose ps
```

### 生产环境

```bash
# 使用生产配置启动
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d

# 或者使用部署脚本
./scripts/deploy/deploy.sh --environment production --tag v1.0.0
```

### 验证部署

```bash
# 检查服务状态
docker-compose ps

# 检查健康状态
curl http://localhost:8080/health

# 检查数据库连接
docker-compose exec postgres pg_isready -U quantix
docker-compose exec clickhouse clickhouse-client --query "SELECT 1"
```

## 服务访问

| 服务 | URL | 说明 |
|------|-----|------|
| Quantix API | https://$QUANTIX_PUBLIC_HOST | 主应用 API |
| Grafana | https://$GRAFANA_PUBLIC_HOST | 监控面板 |
| Prometheus | http://localhost:9090 | 指标采集（内部） |
| Traefik | https://$TRAEFIK_PUBLIC_HOST | 反向代理仪表板 |

## 更新部署

### 滚动更新

```bash
# 拉取最新镜像
docker-compose pull quantix

# 重新创建容器（零停机）
docker-compose up -d --no-deps --build quantix

# 或使用部署脚本
./scripts/deploy/deploy.sh --environment production --tag v1.1.0
```

### 数据库迁移

```bash
# 检查迁移状态
./scripts/migrate/status.sh

# 执行迁移
./scripts/migrate/migrate.sh

# 回滚（如需要）
./scripts/migrate/rollback.sh
```

## 备份和恢复

### 自动备份

系统配置了自动备份 cron 任务：

```bash
# 查看备份
./scripts/backup/list-backups.sh

# 手动备份
./scripts/backup/backup.sh
```

### 手动备份

```bash
# PostgreSQL
docker-compose exec -T postgres pg_dump -U quantix quantix > backup_$(date +%Y%m%d).sql

# ClickHouse
docker-compose exec clickhouse clickhouse-backup create backup_$(date +%Y%m%d)
```

### 恢复

```bash
# PostgreSQL
cat backup_20260308.sql | docker-compose exec -T postgres psql -U quantix quantix

# ClickHouse
docker-compose exec clickhouse clickhouse-backup restore backup_20260308
```

## 监控和日志

### 查看日志

```bash
# 应用日志
docker-compose logs -f quantix

# 所有服务日志
docker-compose logs -f

# 特定时间段
docker-compose logs --since 1h quantix
```

### Grafana 仪表板

1. 访问 https://$GRAFANA_PUBLIC_HOST
2. 登录（admin/your_password）
3. 导入预配置的仪表板

### Prometheus 指标

```bash
# 查看应用指标
curl http://localhost:8080/metrics

# Prometheus 查询
curl 'http://localhost:9090/api/v1/query?query=up'
```

## 故障排查

### 服务无法启动

```bash
# 查看详细日志
docker-compose logs quantix

# 检查配置
docker-compose config

# 检查资源
docker stats
```

### 数据库连接问题

```bash
# 检查 PostgreSQL
docker-compose exec postgres psql -U quantix -d quantix -c "SELECT 1"

# 检查 ClickHouse
docker-compose exec clickhouse clickhouse-client --query "SELECT 1"

# 检查网络
docker network inspect quantix-network
```

### 性能问题

```bash
# 查看资源使用
docker stats

# 查看慢查询（PostgreSQL）
docker-compose exec postgres psql -U quantix -d quantix -c \
  "SELECT * FROM pg_stat_statements ORDER BY total_time DESC LIMIT 10;"

# 查看内存使用
docker-compose exec clickhouse clickhouse-client --query \
  "SELECT * FROM system.asynchronous_metrics WHERE metric LIKE '%Memory%'"
```

## 安全加固

### 1. 修改默认密码

编辑 `.env` 文件，修改所有密码。

### 2. 配置防火墙

```bash
# 只开放必要端口
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
sudo ufw enable
```

### 3. 启用 TLS

生产配置已自动配置 Let's Encrypt 证书。

### 4. 定期更新

```bash
# 更新镜像
docker-compose pull

# 重建容器
docker-compose up -d
```

## 扩容

### 垂直扩容

编辑 `docker-compose.prod.yml` 中的资源限制：

```yaml
deploy:
  resources:
    limits:
      cpus: '4'
      memory: 8G
```

### 水平扩容

对于无状态服务：

```bash
docker-compose up -d --scale quantix=3
```

注意：需要配置负载均衡和会话共享。

## 灾难恢复

### 恢复流程

1. **停止服务**
   ```bash
   docker-compose down
   ```

2. **恢复数据卷**
   ```bash
   # 从备份恢复
   ./scripts/backup/restore.sh backup_20260308
   ```

3. **重启服务**
   ```bash
   docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d
   ```

4. **验证**
   ```bash
   curl http://localhost:8080/health
   ```

## 联系支持

- **文档**: `docs/operations/`
- **问题**: https://github.com/chengjon/quantix-rust/issues
- **邮件**: 替换为实际支持邮箱

---

**最后更新**: 2026-03-08
**维护者**: MyStocks Team
