# Phase 19: 部署与运维 - 实施计划

**制定日期**: 2026-03-08
**状态**: 📋 规划中
**预计工期**: 2-3 周

## 目标

建立完整的部署和运维基础设施，实现：
1. ✅ **Docker 容器化** - 应用和数据库的容器化部署
2. ✅ **CI/CD 增强** - 自动化构建、测试、发布流程
3. ✅ **监控告警** - 系统监控、性能指标、告警通知
4. ✅ **日志管理** - 结构化日志、日志聚合、查询分析
5. ✅ **生产就绪** - 部署脚本、健康检查、备份恢复

## 当前状态分析

### ✅ 已有基础设施

#### CI/CD 配置
- **GitHub Actions** - `.github/workflows/ci.yml`
  - ✅ 代码质量检查（rustfmt, clippy）
  - ✅ 单元测试（PostgreSQL, ClickHouse）
  - ✅ 安全审计（cargo audit）
  - ✅ 多平台构建（Linux, macOS, Windows）
  - ✅ 基准测试（Phase 18）
  - ✅ 文档生成和部署

#### 安全审计
- **GitHub Actions** - `.github/workflows/audit.yml`
  - ✅ 依赖漏洞扫描（每天凌晨2点）
  - ✅ 过期依赖检查
  - ✅ 自动创建 Issue

### ❌ 缺失的基础设施

#### 容器化
- ❌ Dockerfile（应用容器）
- ❌ docker-compose.yml（本地开发）
- ❌ Kubernetes manifests（生产环境）
- ❌ 容器镜像构建和发布

#### 监控和告警
- ❌ 应用指标导出（Prometheus）
- ❌ 健康检查端点
- ❌ 性能监控面板
- ❌ 告警规则和通知
- ❌ 日志聚合（ELK/Loki）

#### 部署和运维
- ❌ 生产环境配置管理
- ❌ 数据库迁移脚本
- ❌ 备份和恢复脚本
- ❌ 滚动更新策略
- ❌ 灾难恢复计划

## 实施计划

### 第 1 部分：Docker 容器化（第 1 周）

#### 任务 1.1：应用 Dockerfile
**文件**: `Dockerfile`

**内容**:
```dockerfile
# 多阶段构建
FROM rust:1.75-slim as builder
WORKDIR /build
COPY . .
RUN cargo build --release

# 最小运行时镜像
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    postgresql-client \
    clickhouse-client \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/quantix /usr/local/bin/
EXPOSE 8080
CMD ["quantix"]
```

**验收标准**:
- ✅ 镜像大小 < 200MB
- ✅ 支持多架构（amd64, arm64）
- ✅ 镜像包含必要的数据库客户端

#### 任务 1.2：Docker Compose 配置
**文件**: `docker-compose.yml`

**服务**:
- `quantix` - 应用容器
- `postgres` - PostgreSQL 17
- `clickhouse` - ClickHouse
- `prometheus` - 指标采集
- `grafana` - 监控面板
- `loki` - 日志聚合
- `promtail` - 日志采集

**验收标准**:
- ✅ 一键启动完整开发环境
- ✅ 服务健康检查
- ✅ 数据持久化卷
- ✅ 环境变量配置

#### 任务 1.3：Kubernetes Manifests
**目录**: `k8s/`

**文件**:
- `deployment.yaml` - 应用部署
- `service.yaml` - 服务暴露
- `configmap.yaml` - 配置管理
- `secret.yaml` - 敏感信息
- `ingress.yaml` - Ingress 路由
- `hpa.yaml` - 自动扩缩容

**验收标准**:
- ✅ 支持滚动更新
- ✅ 资源限制和请求
- ✅ 健康探针配置

### 第 2 部分：监控和告警系统（第 2 周）

#### 任务 2.1：Prometheus 指标导出
**文件**: `src/monitoring/metrics.rs`

**指标类别**:
```rust
// HTTP 指标
http_requests_total
http_request_duration_seconds

// 数据库指标
db_connections_active
db_query_duration_seconds

// 业务指标
strategy_signals_total
backtest_duration_seconds
data_freshness_seconds

// 系统指标
memory_usage_bytes
cpu_usage_percent
```

**验收标准**:
- ✅ 指标端点 `/metrics`
- ✅ 符合 Prometheus 规范
- ✅ 标签化（labelled）指标

#### 任务 2.2：健康检查端点
**文件**: `src/monitoring/health.rs`

**检查项**:
- ✅ 应用状态（alive）
- ✅ 数据库连接（PostgreSQL, ClickHouse）
- ✅ 外部服务可用性（TDX, AkShare）
- ✅ 磁盘空间
- ✅ 内存使用

**端点**:
- `GET /health` - 简单健康检查
- `GET /health/ready` - 就绪探针
- `GET /health/live` - 存活探针

**验收标准**:
- ✅ 返回 JSON 格式状态
- ✅ HTTP 200/503 状态码
- ✅ 详细的诊断信息

#### 任务 2.3：告警规则配置
**文件**: `monitoring/alerts.yml`

**告警规则**:
```yaml
# 应用层面
- 应用 down（> 1min）
- 5xx 错误率（> 5%）
- API 响应时间（> 1s）
- 数据库连接池耗尽

# 业务层面
- 数据采集延迟（> 5min）
- 策略信号异常（> 10/min）
- 回测失败率（> 10%）

# 资源层面
- CPU 使用率（> 90%）
- 内存使用率（> 90%）
- 磁盘空间（< 10%）
```

**验收标准**:
- ✅ Prometheus 告警规则
- ✅ Grafana 仪表板
- ✅ 告警通知（Email/Telegram/钉钉）

#### 任务 2.4：日志聚合
**文件**: `config/logging/tracing.toml`

**日志格式**:
```json
{
  "timestamp": "2026-03-08T10:30:00Z",
  "level": "INFO",
  "target": "quantix_cli::db",
  "message": "Database query executed",
  "span_id": "abc123",
  "trace_id": "def456",
  "fields": {
    "query": "SELECT * FROM stocks",
    "duration_ms": 23
  }
}
```

**配置**:
- ✅ 结构化日志（tracing）
- ✅ 日志级别过滤
- ✅ Promtail + Loki 集成
- ✅ Grafana 查询界面

### 第 3 部分：CI/CD 增强和自动化（第 2-3 周）

#### 任务 3.1：容器镜像构建和发布
**文件**: `.github/workflows/docker.yml`

**流程**:
```yaml
on:
  push:
    tags: ['v*']
  pull_request:
    branches: [main]

jobs:
  build-and-push:
    runs-on: ubuntu-latest
    steps:
      - Checkout code
      - Set up Docker Buildx
      - Login to GitHub Container Registry
      - Build and push (multi-arch)
      - Update deployment (main branch only)
```

**验收标准**:
- ✅ 多架构镜像（amd64, arm64）
- ✅ 自动打标签（版本号 + latest）
- ✅ 安全扫描（Trivy）
- ✅ 自动部署到测试环境

#### 任务 3.2：数据库迁移工具
**文件**: `scripts/migrate/`

**脚本**:
- `migrate.sh` - 运行迁移
- `rollback.sh` - 回滚迁移
- `status.sh` - 查看迁移状态

**迁移文件**: `migrations/`
```
001_initial_schema.up.sql
001_initial_schema.down.sql
002_add_indexes.up.sql
002_add_indexes.down.sql
...
```

**验收标准**:
- ✅ 版本化迁移
- ✅ 向前和向后兼容
- ✅ 事务支持
- ✅ 迁移历史记录

#### 任务 3.3：备份和恢复脚本
**文件**: `scripts/backup/`

**脚本**:
- `backup.sh` - 备份数据库
- `restore.sh` - 恢复数据库
- `list-backups.sh` - 列出备份
- `cleanup-backups.sh` - 清理旧备份

**备份内容**:
- PostgreSQL 数据库（pg_dump）
- ClickHouse 数据（clickhouse-backup）
- 配置文件（tar.gz）
- 日志文件（可选）

**验收标准**:
- ✅ 定时备份（cron）
- ✅ 增量备份支持
- ✅ 备份加密（GPG）
- ✅ 远程存储（S3/MinIO）
- ✅ 恢复演练文档

### 第 4 部分：生产环境准备（第 3 周）

#### 任务 4.1：生产环境配置
**文件**: `config/production.toml`

**配置项**:
```toml
[server]
host = "0.0.0.0"
port = 8080
worker_threads = 8

[database.postgresql]
pool_max_size = 20
connection_timeout = 30
idle_timeout = 600

[monitoring]
enable_metrics = true
metrics_port = 9090
enable_tracing = true

[logging]
level = "info"
format = "json"
outputs = ["stdout", "file"]
```

**验收标准**:
- ✅ 环境变量覆盖
- ✅ 密钥管理（HashiCorp Vault / AWS Secrets Manager）
- ✅ 配置验证

#### 任务 4.2：部署脚本
**文件**: `scripts/deploy/`

**脚本**:
- `deploy.sh` - 部署到生产
- `rollback.sh` - 回滚部署
- `status.sh` - 查看部署状态

**部署策略**:
- ✅ 蓝绿部署
- ✅ 金丝雀发布（可选）
- ✅ 零停机滚动更新

**验收标准**:
- ✅ 版本化部署
- ✅ 部署前健康检查
- ✅ 自动回滚机制
- ✅ 部署日志和通知

#### 任务 4.3：运维手册
**文件**: `docs/operations/`

**文档**:
- `DEPLOYMENT.md` - 部署指南
- `MONITORING.md` - 监控指南
- `TROUBLESHOOTING.md` - 故障排查
- `BACKUP_RESTORE.md` - 备份恢复
- `SCALING.md` - 扩容指南

**验收标准**:
- ✅ 详细的步骤说明
- ✅ 常见问题解决方案
- ✅ 联系方式和升级路径

## 技术栈

### 容器化
- **Docker** - 容器引擎
- **Docker Compose** - 本地开发编排
- **Kubernetes** - 生产环境编排（可选）
- **BuildKit** - 多架构构建

### 监控和告警
- **Prometheus** - 指标采集和存储
- **Grafana** - 监控面板
- **Alertmanager** - 告警路由
- **Loki** - 日志聚合
- **Promtail** - 日志采集

### CI/CD
- **GitHub Actions** - 自动化流程
- **GitHub Container Registry** - 镜像仓库
- **Trivy** - 容器安全扫描

### 备份和恢复
- **pg_dump** - PostgreSQL 备份
- **clickhouse-backup** - ClickHouse 备份
- **Rclone** - 云存储同步
- **GPG** - 备份加密

## 文件结构

```
quantix-rust/
├── Dockerfile                    # 应用容器
├── Dockerfile.dev                # 开发容器
├── docker-compose.yml            # 本地开发
├── docker-compose.prod.yml       # 生产环境
├── .dockerignore                 # Docker 排除
├── k8s/                          # Kubernetes manifests
│   ├── base/
│   ├── overlays/
│   │   ├── dev/
│   │   ├── staging/
│   │   └── production/
│   └── scripts/
├── monitoring/                   # 监控配置
│   ├── prometheus.yml
│   ├── alerts.yml
│   ├── dashboards/
│   └── grafana.ini
├── scripts/
│   ├── deploy/                   # 部署脚本
│   ├── migrate/                  # 数据库迁移
│   ├── backup/                   # 备份恢复
│   └── health-check.sh           # 健康检查
├── migrations/                   # 数据库迁移文件
├── config/
│   ├── default.toml
│   ├── development.toml
│   └── production.toml
├── src/monitoring/
│   ├── mod.rs
│   ├── metrics.rs                # Prometheus 指标
│   ├── health.rs                 # 健康检查
│   └── tracing_setup.rs          # 日志配置
└── docs/operations/              # 运维文档
    ├── DEPLOYMENT.md
    ├── MONITORING.md
    ├── TROUBLESHOOTING.md
    ├── BACKUP_RESTORE.md
    └── SCALING.md
```

## 测试计划

### 单元测试
- ✅ 健康检查逻辑
- ✅ 指标收集功能
- ✅ 配置加载和验证

### 集成测试
- ✅ Docker 容器启动
- ✅ 服务健康检查
- ✅ 数据库连接
- ✅ 日志聚合功能

### 端到端测试
- ✅ 完整部署流程
- ✅ 备份和恢复
- ✅ 滚动更新
- ✅ 告警触发和通知

## 性能指标

### 监控指标目标
- **API 响应时间**: P95 < 500ms
- **数据库查询**: P95 < 100ms
- **CPU 使用率**: < 70%
- **内存使用率**: < 80%
- **容器启动时间**: < 30s

### 可用性目标
- **系统可用性**: > 99.5%
- **数据持久性**: > 99.9%
- **恢复时间目标 (RTO)**: < 15min
- **恢复点目标 (RPO)**: < 5min

## 风险和缓解措施

### 技术风险
| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|----------|
| 容器镜像过大 | 部署慢 | 中 | 多阶段构建、alpine 基础镜像 |
| 监控数据量过大 | 存储压力 | 高 | 数据保留策略、降采样 |
| 配置漂移 | 环境不一致 | 中 | GitOps、配置版本化 |
| 依赖漏洞 | 安全问题 | 低 | 定期审计、自动更新 |

### 运维风险
| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|----------|
| 部署失败 | 服务中断 | 中 | 蓝绿部署、自动回滚 |
| 备份失败 | 数据丢失 | 低 | 定期恢复演练、多重备份 |
| 告警疲劳 | 忽略重要告警 | 中 | 告警聚合、阈值调优 |
| 文档过时 | 运维困难 | 高 | 自动化文档生成 |

## 成功标准

### 第 1 周（Docker 化）
- [x] Dockerfile 和 docker-compose.yml 创建
- [x] 本地开发环境一键启动
- [x] Kubernetes manifests 编写
- [x] 容器镜像构建和推送

### 第 2 周（监控和告警）
- [x] Prometheus 指标导出
- [x] 健康检查端点实现
- [x] Grafana 仪表板配置
- [x] 告警规则和通知配置
- [x] 日志聚合系统部署

### 第 3 周（生产就绪）
- [x] CI/CD 流水线增强
- [x] 数据库迁移工具
- [x] 备份和恢复脚本
- [x] 生产环境部署脚本
- [x] 完整的运维文档

## 后续优化

### 短期（1-2 个月）
1. 实施 GitOps（ArgoCD）
2. 添加分布式追踪（Jaeger）
3. 实施混沌工程测试
4. 优化容器镜像大小

### 中期（3-6 个月）
1. 服务网格（Istio）
2. 自动扩缩容优化
3. 多区域部署
4. 灾难恢复演练

### 长期（6-12 个月）
1. 自服务平台开发
2. A/B 测试框架
3. 灰度发布自动化
4. 全方位安全加固

## 参考资源

- [Docker Best Practices](https://docs.docker.com/develop/dev-best-practices/)
- [Kubernetes Basics](https://kubernetes.io/docs/tutorials/)
- [Prometheus Best Practices](https://prometheus.io/docs/practices/)
- [Grafana Dashboards](https://grafana.com/grafana/dashboards/)
- [Loki Documentation](https://grafana.com/docs/loki/latest/)

---

**状态**: 📋 规划完成，等待实施
**下一步**: 开始第 1 部分 - Docker 容器化
**负责人**: MyStocks Team
**最后更新**: 2026-03-08
