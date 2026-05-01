Quantix 项目采用**四条独立 Workflow** 构建 CI/CD 防线，覆盖从代码提交到生产部署的完整生命周期。核心设计理念是**分层门控**：PR 提交触发轻量快路径（lint + test + security），主干推送启用完整重路径（coverage + build + bench + docs），定时任务则负责长期依赖健康度维护。这种分层策略在保障代码质量的同时，最大程度降低了 PR 反馈延迟。

## 流水线全景架构

整个 CI/CD 系统由四个 Workflow 文件组成，各司其职、触发条件互不重叠：

```mermaid
graph TB
    subgraph 触发源
        PR["Pull Request<br/>main / develop"]
        PUSH["Push<br/>main / develop"]
        TAG["Tag<br/>v*"]
        CRON["定时调度<br/>每日 02:00 / 每周日 03:00"]
        MANUAL["workflow_dispatch<br/>手动触发"]
    end

    subgraph "ci.yml — 核心质量门禁"
        LINT["Lint<br/>fmt + clippy + doc links"]
        TEST["Test<br/>unit + doc tests"]
        SEC["Security Audit<br/>cargo audit"]
        COV["Coverage<br/>tarpaulin → Codecov"]
        DEP["Dependency Outdated<br/>cargo outdated"]
        BUILD["Build<br/>Linux / macOS / Windows"]
        BENCH["Benchmark<br/>criterion → gh-pages"]
        DOCS["Documentation<br/>cargo doc → GitHub Pages"]
    end

    subgraph "docker.yml — 容器构建与发布"
        DBUILD["Docker Build<br/>amd64 + arm64"]
        DSEC["Trivy Security Scan"]
        DTEST["Deploy to Test"]
        RELEASE["GitHub Release"]
    end

    subgraph "audit.yml — 定时安全审计"
        AUDIT["Dependency Audit<br/>cargo audit --json"]
        OUTDATED["Outdated Report<br/>cargo outdated"]
    end

    subgraph "cleanup.yml — 资源回收"
        CLEANUP["Docker Image Cleanup<br/>删除 >30 天旧镜像"]
    end

    PR --> LINT & TEST & SEC
    PUSH --> LINT & TEST & SEC & COV & DEP & BUILD & BENCH & DOCS
    TAG --> DBUILD & RELEASE
    PUSH -.-> DBUILD
    CRON --> AUDIT & OUTDATED & CLEANUP
    MANUAL --> LINT & AUDIT & CLEANUP

    DBUILD --> DSEC --> DTEST
    DBUILD --> RELEASE
```

Sources: [ci.yml](.github/workflows/ci.yml#L1-L431), [docker.yml](.github/workflows/docker.yml#L1-L172), [audit.yml](.github/workflows/audit.yml#L1-L89), [cleanup.yml](.github/workflows/cleanup.yml#L1-L72)

## Workflow 一览表

| Workflow | 文件 | 触发条件 | 核心职责 | 运行频率 |
|---|---|---|---|---|
| **CI** | `ci.yml` | push / PR / schedule / 手动 | 代码质量、测试、构建、覆盖率、基准测试、文档 | 每次 push/PR + 每日 02:00 |
| **Docker** | `docker.yml` | push main / tag v* / PR main | 多平台镜像构建、安全扫描、发布 | 主干合并 + 版本发布 |
| **Security Audit** | `audit.yml` | 每日定时 / 手动 | 依赖漏洞审计、过期报告 | 每日 02:00 |
| **Cleanup** | `cleanup.yml` | 每周日定时 / 手动 | 清理过期 Docker 镜像（>30 天） | 每周日 03:00 |

Sources: [ci.yml](.github/workflows/ci.yml#L1-L11), [docker.yml](.github/workflows/docker.yml#L1-L10), [audit.yml](.github/workflows/audit.yml#L1-L8), [cleanup.yml](.github/workflows/cleanup.yml#L1-L7)

## CI 核心流水线（ci.yml）

CI 是最核心的 Workflow，包含 **8 个独立 Job**，通过触发条件和 `if` 守卫实现**快路径/重路径分流**。所有 Job 之间并行执行（无 `needs` 依赖），任何一个失败都会阻断合并。

### 触发条件与分层策略

```yaml
on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]
  workflow_dispatch:
  schedule:
    - cron: '0 2 * * *'
```

项目定义了**三条运行路径**：

| 路径 | 触发条件 | 执行的 Job | 目的 |
|---|---|---|---|
| **快路径** | Pull Request | lint, test, security | 快速反馈，阻止低质量代码进入 |
| **重路径** | Push to main/develop | 全部 Job | 完整质量保证，含覆盖率与基准 |
| **夜间路径** | 每日定时 | 全部 Job | 捕获依赖变化导致的问题 |

Sources: [ci.yml](.github/workflows/ci.yml#L1-L11)

### Job 详细解析

#### Lint — 代码格式与静态分析

Lint Job 执行三项检查：**`cargo fmt`** 强制统一代码风格，**`cargo clippy`** 以 `-D warnings` 模式将所有警告提升为错误（零容忍策略），**`cargo doc`** 检查文档中的未解析链接。文档检查通过管道捕获输出并 `grep` 检测 `unresolved link` 警告来实现，确保 `rustdoc` 注释中的交叉引用始终有效。

缓存策略上，Lint Job 将 Cargo registry、git index 和 build target 分开缓存三份，均以 `Cargo.lock` 哈希为键，确保依赖变更时缓存自动失效。

Sources: [ci.yml](.github/workflows/ci.yml#L24-L71)

#### Test — 单元测试与文档测试

Test Job 是唯一需要**外部服务依赖**的 Job。它通过 GitHub Actions `services` 启动两个容器：

- **PostgreSQL 17 Alpine**：映射 5432 端口，配置健康检查 `pg_isready`，用于数据存储相关测试
- **ClickHouse Server**：映射 8123 端口，配置 `SELECT 1` 健康检查，用于时序数据相关测试

测试执行前有显式的等待循环（最多 30 次，每次 2 秒），确保数据库完全就绪。测试命令使用 `--test-threads=1` 串行执行，避免数据库并发冲突。环境变量通过顶层 `env` 块注入 `POSTGRES_URL` 和 `CLICKHOUSE_URL`，测试代码直接读取这些变量获取连接信息。

Sources: [ci.yml](.github/workflows/ci.yml#L73-L167)

#### Coverage — 代码覆盖率（仅 main 分支）

Coverage Job 使用 **`cargo-tarpaulin`** 生成 Lcov 格式覆盖率报告，并上传至 **Codecov**。该 Job 仅在 `push` 到 `main` 分支时触发（`if: github.event_name == 'push' && github.ref == 'refs/heads/main'`），同样需要 PostgreSQL 和 ClickHouse 服务容器。超时设置为 `--timeout 120` 秒，应对复杂测试的长时间运行。

Sources: [ci.yml](.github/workflows/ci.yml#L189-L287)

#### Build — 多平台交叉编译（仅 push）

Build Job 使用 `matrix strategy` 在**三个平台**上并行构建 Release 二进制：

| 矩阵项 | OS | Target | 说明 |
|---|---|---|---|
| `ubuntu-latest` | Linux | `x86_64-unknown-linux-gnu` | 主要部署平台 |
| `macos-latest` | macOS | `x86_64-apple-darwin` | macOS 开发支持 |
| `windows-latest` | Windows | `x86_64-pc-windows-msvc` | Windows 兼容 |

构建产物通过 `upload-artifact` 上传，命名格式为 `quantix-{os}`，便于后续下载和分发。该 Job 仅在 push 到 `main` 或 `develop` 分支时触发，避免 PR 消耗多平台构建资源。

Sources: [ci.yml](.github/workflows/ci.yml#L309-L353)

#### Bench — 性能基准测试（main + 定时）

Benchmark Job 运行 `cargo bench --all-features`，使用 **Criterion** 框架生成性能数据。结果通过 `benchmark-action/github-action-benchmark` 自动推送到 `gh-pages` 分支，形成性能历史趋势图。关键配置项：

- **`alert-threshold: 150%`**：性能回归超过 150% 时触发告警
- **`fail-on-alert: true`**：告警时直接失败，阻断合并
- **`comment-on-alert: true`**：在对应 commit 上添加评论通知

Sources: [ci.yml](.github/workflows/ci.yml#L354-L393)

#### Docs — 文档发布（仅 push，main 分支部署 Pages）

Documentation Job 生成 `cargo doc` 文档，并在 push 到 `main` 分支时通过 **`peaceiris/actions-gh-pages`** 部署到 GitHub Pages，绑定自定义域名 `quantix-rust.dev`。该 Job 仅在 push 事件时运行（非 PR），避免 PR 频繁触发文档构建。

Sources: [ci.yml](.github/workflows/ci.yml#L395-L431)

### 快路径 vs 重路径执行矩阵

| Job | PR 快路径 | Push 重路径 | 定时夜间 | 运行条件 |
|---|:---:|:---:|:---:|---|
| Lint | ✅ | ✅ | ✅ | 无条件 |
| Test | ✅ | ✅ | ✅ | 无条件 |
| Security | ✅ | ✅ | ✅ | 无条件 |
| Coverage | ❌ | ✅ (仅 main) | ✅ (仅 main) | `push && ref == main` |
| Dependency Outdated | ❌ | ✅ (仅 main) | ✅ (仅 main) | `push && ref == main` |
| Build | ❌ | ✅ (main/develop) | ❌ | `push && (main \|\| develop)` |
| Benchmark | ❌ | ✅ (仅 main) | ✅ | `(push && main) \|\| schedule` |
| Documentation | ❌ | ✅ | ✅ | `push` 事件 |

Sources: [ci.yml](.github/workflows/ci.yml#L1-L431)

## Docker 构建与发布流水线（docker.yml）

Docker Workflow 管理**容器镜像的完整生命周期**：从多平台构建、安全扫描到版本发布。它采用 `matrix strategy` 实现 `linux/amd64` 和 `linux/arm64` 双平台并行构建。

### 镜像标签策略

`docker/metadata-action` 根据 Git 事件类型自动生成镜像标签：

| 事件类型 | 生成的标签 | 示例 |
|---|---|---|
| `push` 到 main | `main`, `latest` | `quantix:main`, `quantix:latest` |
| Pull Request | `pr-{number}` | `quantix:pr-42` |
| Tag `v1.2.3` | `1.2.3`, `1.2`, `1` | `quantix:1.2.3` |
| 默认分支 | `latest` | `quantix:latest` |

构建使用 **GHA 缓存**（`cache-from: type=gha`），利用 GitHub Actions 自带的缓存后端存储 Docker 层，大幅加速重复构建。构建参数注入 `BUILD_DATE`、`VCS_REF` 和 `VERSION`，实现镜像的可追溯性。

Sources: [docker.yml](.github/workflows/docker.yml#L1-L75)

### 安全扫描与部署

构建完成后，**Trivy** 漏洞扫描器对镜像进行安全审计，结果以 **SARIF 格式**上传至 GitHub Security 标签页。此步骤仅在非 PR 事件时运行（`if: github.event_name != 'pull_request'`），因为 PR 构建的镜像不会推送。

```mermaid
flowchart LR
    BUILD["Docker Build<br/>amd64 + arm64"] --> SCAN["Trivy 安全扫描"]
    BUILD --> RELEASE["GitHub Release<br/>仅 tag 触发"]
    SCAN --> DEPLOY["部署到测试环境<br/>仅 main 分支"]
```

当 main 分支推送时，安全扫描通过后触发测试环境部署（`environment: test`）；当推送 `v*` 格式 tag 时，自动创建 **GitHub Release**，包含版本号、Docker 镜像拉取命令和 CHANGELOG 链接。

Sources: [docker.yml](.github/workflows/docker.yml#L76-L172)

## 定时安全审计（audit.yml）

独立于 CI 的**每日安全审计** Workflow，在每天凌晨 02:00 自动运行。包含两个 Job：

- **Dependency Audit**：运行 `cargo audit` 和 `cargo audit --json`。审计失败时，通过 `actions/github-script` **自动创建 GitHub Issue**，标签为 `security` 和 `high-priority`，确保漏洞不被忽视
- **Outdated Dependencies**：运行 `cargo outdated --exit-code 1`，生成过期依赖报告并作为 **Artifact** 上传，供开发者查看

Sources: [audit.yml](.github/workflows/audit.yml#L1-L89)

## 资源回收（cleanup.yml）

每周日凌晨 03:00 自动清理 **GHCR（GitHub Container Registry）中超过 30 天的旧镜像版本**。通过 `gh api` 遍历所有 quantix 相关包，筛选创建时间早于 30 天的版本逐一删除。运行完成后生成 `$GITHUB_STEP_SUMMARY` 摘要，可在 Actions 页面查看清理结果。

Sources: [cleanup.yml](.github/workflows/cleanup.yml#L1-L72)

## 缓存策略

项目在所有 Job 中统一使用 **`actions/cache@v3`**，缓存键策略如下：

| 缓存范围 | 缓存路径 | 键模式 | 用途 |
|---|---|---|---|
| Lint Job（分三项） | `~/.cargo/registry` / `~/.cargo/git` / `target` | `{os}-cargo-{scope}-{lockfile_hash}` | 加速编译 |
| Test/Cov/Build Job | 合并路径 | `{os}[-{target}]-cargo-{lockfile_hash}` | 多平台独立缓存 |
| Docker 构建 | Docker 层 | GHA 内建缓存 | 多平台镜像层共享 |

所有缓存键均以 `**/Cargo.lock` 的哈希为核心，确保依赖变更时缓存自动失效。部分 Job 配置了 `restore-keys` 回退前缀，在精确匹配失败时仍能命中前一次缓存的部分内容。

Sources: [ci.yml](.github/workflows/ci.yml#L39-L55), [ci.yml](.github/workflows/ci.yml#L120-L129)

## 测试基础设施

CI 的 Test 和 Coverage Job 依赖两个**服务容器**模拟生产数据库环境：

```mermaid
graph TB
    subgraph "GitHub Actions Runner"
        TEST_CMD["cargo test<br/>--lib --all-features<br/>--test-threads=1"]
        WAIT_PG["Wait for PostgreSQL<br/>pg_isready 轮询"]
        WAIT_CH["Wait for ClickHouse<br/>SELECT 1 轮询"]
    end

    subgraph "Service Containers"
        PG["PostgreSQL 17 Alpine<br/>:5432<br/>test:test@localhost/quantix_test"]
        CH["ClickHouse Server<br/>:8123<br/>quantix_test"]
    end

    WAIT_PG --> PG
    WAIT_CH --> CH
    TEST_CMD --> PG
    TEST_CMD --> CH
```

环境变量通过 Workflow 顶层 `env` 块预定义：`POSTGRES_URL: postgresql://test:test@localhost:5432/quantix_test` 和 `CLICKHOUSE_URL: http://localhost:8123`，`CLICKHOUSE_DB: quantix_test`。测试代码直接读取这些环境变量获取连接配置。显式等待循环（最多 60 秒）确保在测试启动前两个数据库均已就绪。

Sources: [ci.yml](.github/workflows/ci.yml#L13-L20), [ci.yml](.github/workflows/ci.yml#L79-L163)

## CI 结构验证测试

项目包含一个**元测试**（meta-test）`ci_workflow_structure_test`，验证 CI Workflow 文件本身的结构完整性。该测试断言以下关键属性：

| 断言项 | 验证内容 | 设计意图 |
|---|---|---|
| `pull_request:` 存在 | PR 触发器未丢失 | 确保 PR 质量门禁 |
| `push:` 存在 | Push 触发器未丢失 | 确保主干质量门禁 |
| `coverage:` 独立 Job | 覆盖率有专属 Job | 重路径完整覆盖 |
| `dependency_outdated:` 独立 Job | 过期依赖有专属 Job | 依赖健康管理 |
| 文档检查无 PR 排除条件 | lint 中文档检查对所有事件生效 | 避免文档回归 |
| `build` Job 有 push 门控 | build 仅在 push 时运行 | 节省 PR 资源 |
| `bench` Job 有 main 门控 | bench 仅在 main 时运行 | 聚焦核心性能 |

这个测试是**CI 自身的质量保证**——防止误改 Workflow 结构导致关键检查项丢失。

Sources: [ci_workflow_structure_test.rs](tests/ci_workflow_structure_test.rs#L1-L39)

## 常见操作场景

### 场景一：PR 提交流程

开发者创建 PR 到 `main` 或 `develop` 分支时，自动触发**快路径**：Lint（格式 + Clippy + 文档链接）、Test（单元 + 文档测试）和 Security Audit 三个 Job 并行执行。所有检查通过后 PR 方可合并，任何一个失败都将阻止合并。

### 场景二：主干合并后

代码合并到 `main` 后触发**完整重路径**：除快路径的三项检查外，额外启动 Coverage（覆盖率上报 Codecov）、Build（三平台交叉编译并上传二进制）、Benchmark（性能基线比对，超 150% 回归则阻断）、Documentation（生成并部署到 `quantix-rust.dev`）、Dependency Outdated（依赖新鲜度检查）五个 Job。同时 Docker Workflow 触发双平台镜像构建，扫描后部署到测试环境。

### 场景三：版本发布

维护者在 `main` 分支打 `v*` 格式 tag 时，触发 Docker 构建和 GitHub Release 自动创建。Release 包含版本号、镜像拉取命令和 CHANGELOG 链接，自动发布到 GitHub Releases 页面。

Sources: [ci.yml](.github/workflows/ci.yml#L1-L11), [docker.yml](.github/workflows/docker.yml#L1-L172)

## 延伸阅读

- **Docker 容器化部署与生产环境配置**：Dockerfile 的多阶段构建细节和生产环境配置 → [Docker 容器化部署与生产环境配置](26-docker-rong-qi-hua-bu-shu-yu-sheng-chan-huan-jing-pei-zhi)
- **测试策略与覆盖要求**：CI 中运行的测试用例编写规范 → [测试策略与覆盖要求](29-ce-shi-ce-lue-yu-fu-gai-yao-qiu)
- **性能优化指南**：CI Benchmark 中使用的 Criterion 基准测试编写 → [性能优化指南（Polars 批量计算与 Criterion 基准测试）](30-xing-neng-you-hua-zhi-nan-polars-pi-liang-ji-suan-yu-criterion-ji-zhun-ce-shi)
- **监控告警体系**：生产环境中的 Prometheus 指标采集 → [监控告警体系与 Prometheus 指标导出](24-jian-kong-gao-jing-ti-xi-yu-prometheus-zhi-biao-dao-chu)