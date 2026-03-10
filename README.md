# quantix-rust

A 股量化交易 CLI 工具 - Rust 实现

与 Python quantix 项目共享数据源和数据库，提供高性能的量化分析能力。

## Foundation P0 工作约束

- 仓库内本地 worktree 放在 `.worktrees/`，全文检索和批量扫描应排除该目录，避免重复命中。
- 本地分析产物和工具目录如 `.gitnexus/`、`target/` 应视为噪音目录，并通过 `.ignore` 排除。
- Foundation P0 的任务能力只支持直接运行 CLI 前台进程，不假设 daemon、常驻调度服务或任务持久化已经可用。

## 功能特性

### 已完成模块

#### Phase 1: 数据采集基础 ✅
- **数据源适配器**
  - TDX (通达信) - 实时行情数据
  - AkShare - 财务数据、历史数据
  - Quote Collector - 多股票实时采集
  - Auction Collector - 竞价数据采集

#### Phase 2: 竞价分析模块 ✅
- **竞价分析器** (`src/analysis/auction.rs`)
  - 抢筹强度评分 (涨幅40% + 买盘占比30% + 成交量30%)
  - 封单金额计算
  - 板块统计 (上海主板/科创板/深圳主板/创业板)
  - 推荐买入筛选

#### Phase 3: K线数据管理与同步 ✅
- **K线聚合器** (`src/sources/kline_aggregator.rs`)
  - 实时聚合 Tick 数据为 1m/5m/15m/30m/60m/1d K线
  - 使用 tokio channels 替代 Redis Stream
  - 自动窗口对齐和过期清理
- **数据同步模块** (`src/sync/etl.rs`)
  - PostgreSQL/TDengine → ClickHouse 数据桥接
  - 支持日线和分钟线同步
  - 定时同步任务调度
- **ClickHouse 数据库** (`src/db/clickhouse.rs`)
  - MergeTree 引擎，月度分区
  - 4张核心表: stock_info, stock_realtime_quotes, kline_data, limit_up_events

#### Phase 4: 回测引擎 ✅
- **投资组合管理** (`src/analysis/portfolio.rs`)
  - 持仓管理、买入卖出逻辑
  - 手续费计算、滑点模拟
- **性能指标计算** (`src/analysis/performance.rs`)
  - 夏普比率、索提诺比率、最大回撤
  - 胜率、盈亏比、卡玛比率
- **回测引擎** (`src/analysis/backtest.rs`)
  - 事件驱动回测框架
  - 策略 trait 接口集成
  - 完整的回测报告生成

#### Phase 5: 任务调度 ✅
- **Cron 解析器** (`src/tasks/cron.rs`)
  - 支持 */N 步长语法
  - 支持 1-5 范围语法
  - 支持列表语法 1,2,3
- **任务调度器** (`src/tasks/scheduler.rs`)
  - 集成 tokio-cron-scheduler
  - 动态添加/删除任务
  - 预定义任务模板 (盘前/竞价/开盘/收盘/盘后/数据同步)

#### Phase 6: TDX 文件解析与复权 ✅
- **Day 文件解析器** (`src/sources/tdx_file.rs`)
  - 通达信 day 文件解析 (32字节/记录)
  - GBBQ 股本变迁文件解析 (29字节/记录)
  - 复权因子计算算法 (基于涨跌幅连续计算)
  - 前复权/后复权应用
  - 批量导入器

#### Phase 7: GBBQ 数据存储 ✅
- **数据模型** (`src/data/models.rs`)
  - GbbqEvent: 除权除息事件
  - CapitalChange: 股本变更摘要
- **ClickHouse 存储** (`src/db/clickhouse.rs`)
  - gbbq_events 表 (按月分区)
  - 事件插入/查询接口
  - 最新除权事件查询

#### Phase 8: 多周期 K线查询 ✅
- **K线查询接口** (`src/db/clickhouse.rs`)
  - get_kline_data(): 查询指定周期 K线
  - insert_kline_data(): 插入 K线数据
  - insert_kline_data_batch(): 批量插入
  - get_daily_from_minute(): 分钟线聚合日线
- **支持的周期**: 1m, 5m, 15m, 30m, 60m, 1d

#### Phase 9: 东方财富数据采集 ✅
- **EastMoney 数据源** (`src/sources/eastmoney.rs`)
  - 股票列表获取 (支持板块分类: HS300, ZZ500, SZ50, KCB50, BZ50)
  - 实时行情查询
  - 资金流向数据
  - 财务数据获取
  - HTTP 客户端集成

#### Phase 10: ClickHouse 批量导入优化 ✅
- **批量插入优化** (`src/db/clickhouse.rs`)
  - 使用 clickhouse crate 的 insert API
  - 可配置批次大小 (默认 1000)
  - 支持 async_insert 选项提升性能
  - 新增 insert_stock_quotes_batch 方法
  - 优化的 insert_gbbq_events_batch 和 insert_kline_data_batch

#### Phase 11: WebSocket 实时行情订阅 ✅
- **WebSocket 客户端** (`src/sources/websocket.rs`)
  - tokio-tungstenite WebSocket 连接
  - 连接状态管理 (Disconnected/Connecting/Connected/Reconnecting)
  - 订阅/取消订阅方法
  - 心跳保活机制 (可配置间隔)
  - 自动重连机制 (可配置最大次数)
  - 使用 tokio::select! 实现并发消息处理

#### Phase 12: 技术指标增强 ✅
- **技术指标** (`src/analysis/indicators.rs`)
  - SMA/EMA/WMA (简单/指数/加权移动平均)
  - RSI (相对强弱指标)
  - MACD (指数平滑异同移动平均线)
  - KDJ (随机指标)
  - Bollinger Bands (布林带)
  - ATR (平均真实波幅)
  - OBV (能量潮)
  - CCI (顺势指标)
  - Williams %R (威廉指标)

#### Phase 13: Polars 统一数据管理层 ✅
- **Polars 适配器** (`src/analysis/polars_adapter.rs`)
  - 全局线程配置 (`init_polars`)
  - 批量K线数据结构 (`BatchKlineData`)
  - Polars 指标计算器 (`PolarsCalculator`)
    - 移动平均线 (MA) 使用 `RollingOptionsFixedWindow`
    - 批量计算多个指标 (`calculate_batch`)
  - 多股票批量处理 (`MultiStockData`)
  - K线数据转换 (`from_kline_vec`)
- **Polars 0.43 API 适配**
  - `PlSmallStr` 用于 Series 名称
  - `rolling_mean()` 使用固定窗口选项
  - `cast()` 使用 `&DataType` 参数
  - Series 迭代使用 `.iter().map(|av| av.extract::<f64>())`

#### Phase 14: CLI 命令实现 ✅
- **完整命令处理器** (`src/cli/handlers.rs`)
  - `init` - 初始化配置和数据库
  - `menu` - 交互式菜单 (简单版 + TUI 规划)
  - `data query` - 查询历史K线数据
  - `data export` - 导出数据为 CSV/Parquet
  - `strategy run` - 运行策略回测
  - `strategy list/show` - 策略管理
  - `task start/stop/status` - 任务调度器管理
  - `analyze indicators` - 技术指标计算
  - `status --health` - 数据库健康检查
- **交互式菜单系统**
  - 数据同步菜单
  - 策略运行菜单
  - 回测分析菜单
  - 任务管理菜单
  - 技术分析菜单
  - 数据导出菜单
- **CSV 数据导出** - 使用 csv crate 实现
- **进度条支持** - indicatif 进度显示

#### Phase 21: 自选池管理 ✅
- **自选池命令** (`src/cli/mod.rs`, `src/cli/handlers.rs`)
  - `watchlist add/remove/list/move` - 基础自选池维护
  - `watchlist group create/list` - 分组管理
  - `watchlist tag add/remove/list` - 标签管理
  - `watchlist history` - 本地操作历史
  - `watchlist list --with-price` - 最佳努力价格展示
- **JSON 持久化** (`src/watchlist/storage.rs`)
  - 默认路径 `~/.quantix/watchlist/watchlist.json`
  - 可通过 `QUANTIX_WATCHLIST_PATH` 覆盖
- **P0 约束**
  - 价格展示使用前台最佳努力查询
  - 行情不可用时降级为空价格，不影响 `watchlist list` 返回

#### Phase 15: 具体策略实现 ✅
- **MA Cross 策略** (`src/strategy/ma_cross.rs`)
  - 完整实现 MA 金叉死叉逻辑
  - 可配置短期和长期均线周期
  - 自动持仓状态管理
  - ✅ 4个单元测试通过
- **Mean Reversion 策略** (`src/strategy/mean_reversion.rs`)
  - 基于 RSI 和布林带的均值回归
  - 可配置超买超卖阈值
  - 双重条件确认信号
  - ✅ 4个单元测试通过
- **Momentum 策略** (`src/strategy/momentum.rs`)
  - 基于 MACD 的动量跟踪
  - 可配置快慢线和信号线周期
  - 支持背离检测（预留）
  - ✅ 3个单元测试通过
- **Breakout 策略** (`src/strategy/breakout.rs`)
  - 价格突破 + 成交量确认
  - ATR 动态止损止盈
  - 支持做多和做空
  - ✅ 1个单元测试通过
- **Grid Trading 策略** (`src/strategy/grid.rs`)
  - 震荡市场网格交易
  - ATR 动态价格区间
  - 自动网格订单管理
  - 支持动态调整
  - ✅ 3个单元测试通过
- **测试工具模块** (`src/strategy/test_utils.rs`)
  - 可配置的测试数据生成器
  - 消除所有硬编码参数
  - 支持多种价格趋势生成
  - ✅ 4个单元测试通过
- **所有策略完全可配置** - 无硬编码参数
- **统一 Strategy trait** - 标准化接口
- **测试覆盖率** - 90% (19/21个函数)

#### Phase 16: 实时监控系统 ✅
- **信号监控** (`src/monitoring/signal_monitor.rs`)
  - 实时追踪策略交易信号
  - 信号统计分析（买入/卖出/观望计数）
  - 策略级别和股票级别统计
  - 信号频率统计（每分钟）
  - ✅ 10个单元测试通过
- **持仓监控** (`src/monitoring/position_monitor.rs`)
  - 实时持仓状态追踪
  - 持仓变化检测（新增/加仓/减仓/平仓/价格更新）
  - 持仓快照和盈亏计算
  - 持仓比例告警检查
  - ✅ 11个单元测试通过
- **性能监控** (`src/monitoring/performance_monitor.rs`)
  - 权益历史追踪
  - 实时性能指标计算（收益率、回撤、夏普比率等）
  - 回撤状态分级（Normal/Caution/Warning/Critical）
  - 交易盈亏记录和统计
  - ✅ 10个单元测试通过
- **告警系统** (`src/monitoring/alert.rs`)
  - 阈值告警机制
  - 多级告警（Info/Warning/Error/Critical）
  - 冷却时间机制
  - 告警历史和确认功能
  - 预定义阈值构建器
  - ✅ 15个单元测试通过
- **所有模块完全可配置** - 零硬编码
- **测试覆盖率** - 85% (46/46个测试通过)

#### Phase 17: 数据导入导出增强 ✅
- **数据导出器** (`src/io/exporter.rs`)
  - 多格式导出支持 (CSV, JSON, Parquet)
  - 可配置输出参数（精度、表头、日期格式）
  - Arrow/Parquet 列式存储集成
  - 导出结果跟踪（文件大小、记录数、执行时间）
  - ✅ 5个单元测试通过
- **数据导入器** (`src/io/importer.rs`)
  - 多格式导入支持 (CSV, JSON, Parquet)
  - 错误处理和无效行跳过
  - 性能指标（导入时长、跳过/错误计数）
  - 可选数据验证集成
  - ✅ 4个单元测试通过
- **数据验证器** (`src/io/validation.rs`)
  - 全面数据验证（价格、成交量、日期）
  - 价格逻辑验证（high ≥ low, close in range）
  - 数据质量评分（0-100分制）
  - 批量验证支持
  - ✅ 7个单元测试通过
- **批处理处理器** (`src/io/batch.rs`)
  - 大数据集内存优化批处理
  - 信号量限制并发控制
  - 实时进度条（indicatif）
  - 流式处理支持超大文件
  - ✅ 5个单元测试通过
- **所有配置完全可定制** - 无硬编码参数
- **测试覆盖率** - 100% (21/21个io模块测试通过)

#### Phase 18: 性能测试与优化 ✅
- **基准测试框架** (`benches/bench_main.rs`)
  - Criterion 集成，42个测试用例全部通过
  - 技术指标、导入导出、验证、批处理基准
  - 多规模测试（100-1M 条记录）
  - 自动化基线对比和回归检测
  - ✅ 性能基线已建立
- **性能基线数据** (`docs/reports/PHASE18_BENCHMARK_RESULTS.md`)
  - 技术指标: SMA 1.54ms, MACD 5.57ms (10K条)
  - 数据导出: CSV 679K 记录/秒, JSON 593K 记录/秒
  - 性能计算: 总收益率 18.8M 次/秒
  - 批处理: 9.80-23.4M 记录/秒
- **性能优化工具** (`src/core/performance_utils.rs`)
  - PerfTimer 性能计时器
  - MemoryTracker 内存跟踪器
  - 优化建议生成器（批次大小、并行化、预分配）
  - 性能分析器（自动优化建议）
  - ✅ 3个单元测试通过
- **基准测试脚本** (`scripts/dev/run_benchmarks.sh`)
  - 一键运行、基线保存、性能对比
  - Flamegraph/DHAT 集成支持
  - HTML 报告生成
- **性能优化指南** (`docs/guides/PERFORMANCE_OPTIMIZATION.md`)
  - 基准测试解读、性能剖析工具使用
  - 优化策略（批量、并行、内存、算法）
  - CI/CD 集成、常见问题诊断
- **所有配置完全可定制** - 无硬编码参数
- **完整文档** - 使用指南、优化策略、工具集成
- **零错误完成** - 修复所有溢出问题，42/42 测试通过

#### Phase 19: 部署与运维 ✅
- **Docker 容器化**
  - `Dockerfile` - 生产环境多阶段构建（<200MB）
  - `Dockerfile.dev` - 开发环境热重载
  - `docker-compose.yml` - 完整服务栈（7个服务）
  - `docker-compose.prod.yml` - 生产环境配置
  - 非 root 用户运行、健康检查集成
- **CI/CD 增强** (`.github/workflows/`)
  - `docker.yml` - 多架构镜像构建（amd64/arm64）
  - `cleanup.yml` - 定期清理旧镜像
  - GitHub Container Registry 集成
  - Trivy 安全扫描
  - 自动部署和 GitHub Release
- **部署脚本** (`scripts/deploy/`)
  - 多环境支持（dev/staging/production）
  - 健康检查、模拟运行模式
  - 彩色输出和日志
- **监控配置** (`monitoring/`)
  - `prometheus.yml` - 指标采集配置
  - `alerts.yml` - 20+ 告警规则
  - `loki.yml` + `promtail.yml` - 日志聚合
- **数据库初始化**
  - `scripts/init-postgres.sql` - PostgreSQL 扩展和优化
  - `scripts/init-clickhouse.sql` - ClickHouse 表结构
- **完整文档**
  - `docs/guides/DOCKER_GUIDE.md` - Docker 部署指南
  - `docs/guides/PRODUCTION_DEPLOYMENT.md` - 生产部署指南
- **服务栈**: Quantix + PostgreSQL 17 + ClickHouse + Prometheus + Grafana + Loki + Promtail + Traefik

#### Phase 20: Zellij 集成和 CLI 增强 ✅
- **Zellij 配置** (`config/zellij/`)
  - `config.kdl` - 主配置文件（Catppuccin Mocha 主题）
  - 快捷键绑定（Alt+h/j/k/l 移动焦点，Alt+s/v 分割窗格）
  - quantix 专用快捷键（Alt+m 菜单，Alt+q 状态）
- **预设布局** (`config/zellij/layouts/`)
  - `main.kdl` - 主工作区（CLI + 状态监控 + 日志）
  - `monitor.kdl` - 监控工作区（4窗格实时监控）
  - `backtest.kdl` - 回测工作区（控制台 + 结果 + 日志）
  - `dev.kdl` - 开发工作区（代码 + 构建 + Git）
- **启动脚本** (`scripts/zellij/`)
  - `start-session.sh` - 会话启动（自动检测现有会话）
  - `install.sh` - 一键安装（支持 cargo/包管理器）
  - `status-collector.sh` - 状态采集（CPU/内存/数据库/任务）
- **完整文档** (`docs/guides/ZELLIJ_GUIDE.md`)
  - 安装指南、快捷键参考、布局说明
  - 常见问题、最佳实践
- **Rust 原生终端复用器** - 与项目技术栈一致

## 项目结构

```
quantix-rust/
├── src/
│   ├── analysis/        # 分析模块
│   │   ├── auction.rs       # 竞价分析
│   │   ├── backtest.rs      # 回测引擎
│   │   ├── indicators.rs    # 技术指标
│   │   ├── portfolio.rs     # 投资组合管理
│   │   └── performance.rs   # 性能计算
│   ├── cli/             # CLI 命令行
│   ├── core/            # 核心模块 (配置、错误、交易日历)
│   ├── data/            # 数据模型
│   ├── db/              # 数据库客户端 (PostgreSQL, TDengine, ClickHouse)
│   ├── io/              # 数据导入导出 (Phase 17)
│   ├── monitoring/      # 实时监控系统 (Phase 16)
│   ├── sources/         # 数据源 (TDX, AkShare, 行情采集, K线聚合)
│   ├── strategy/        # 交易策略
│   ├── sync/            # 数据同步 ETL
│   ├── tasks/           # 任务调度 (Cron, Scheduler)
│   └── tui/             # 终端 UI (可选)
├── Cargo.toml
└── README.md
```

## 开发指南

### 📚 开发规范

**重要**: 所有贡献者和开发者必须阅读 [开发规范指南](docs/standards/DEVELOPMENT_GUIDELINES.md)

该规范文档包含：
- ✅ 核心编码规则（所有权、错误处理、类型安全）
- ✅ 量化交易特殊注意事项（性能优化、安全性、CLI交互）
- ✅ 测试规范（单元测试、集成测试、回测验证）
- ✅ 性能优化指南（编译优化、基准测试）
- ✅ 安全与稳定性（依赖安全、资源管理、并发安全）
- ✅ 代码质量工具（rustfmt、clippy、CI/CD）

### 代码质量检查

```bash
# 格式化代码
cargo fmt

# 检查代码质量
cargo clippy -- -D warnings

# 运行所有测试
cargo test --all-features

# 检查依赖漏洞
cargo install cargo-audit
cargo audit
```

### 环境要求
- Rust 1.70+
- PostgreSQL 17+ (可选)
- TDengine 3.3+ (可选)
- ClickHouse (推荐用于 OLAP 分析)

### 构建运行

```bash
# 克隆项目
git clone https://github.com/chengjon/quantix-rust.git
cd quantix-rust

# 开发运行
cargo run -- --help

# 构建
cargo build --release
```

### 配置

通过环境变量配置数据库连接：

```bash
# PostgreSQL
export POSTGRES_URL="postgresql://localhost:5432/quantix"

# ClickHouse
export CLICKHOUSE_URL="http://localhost:8123"
export CLICKHOUSE_DB="quantix"

# TDX 数据源
export TDX_HOST="192.168.1.100"
export TDX_PORT=7709

# 自选池 JSON 存储路径（可选）
export QUANTIX_WATCHLIST_PATH="$HOME/.quantix/watchlist/watchlist.json"
```

### 运行测试

```bash
# 所有测试
cargo test --all-features

# 单个模块测试
cargo test --package quantix-cli --lib analysis::backtest::tests

# 运行文档测试
cargo test --doc

# 运行基准测试
cargo bench
```

## 使用示例

### 自选池 CLI

```bash
# 创建分组
quantix watchlist group create --name core

# 添加股票并打标签
quantix watchlist add --code 000001 --group core
quantix watchlist tag add --code 000001 --tag bank

# 查看列表与历史
quantix watchlist list --group core
quantix watchlist list --with-price
quantix watchlist history --code 000001 --limit 20
```

### 回测示例

```rust
use quantix_cli::analysis::backtest::{BacktestEngine, BacktestConfig};
use quantix_cli::strategy::ma_cross::MACrossStrategy;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = BacktestConfig::default();
    let mut engine = BacktestEngine::new(config);

    let mut strategy = MACrossStrategy::new(5, 20); // MA5, MA20

    // 加载历史数据
    let data = load_kline_data("000001").await?;

    // 运行回测
    let result = engine.run(&mut strategy, &data).await?;

    println!("总收益率: {}%", result.report.total_return * 100);
    println!("夏普比率: {}", result.report.sharpe_ratio);

    Ok(())
}
```

### 任务调度示例

```rust
use quantix_cli::tasks::{TaskScheduler, TaskTemplates};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let scheduler = TaskScheduler::new().await?;

    // 添加预设任务
    scheduler.add_task(TaskTemplates::market_open()).await?;
    scheduler.add_task(TaskTemplates::market_close()).await?;

    // 启动调度器
    scheduler.start().await?;

    // 保持运行
    tokio::signal::ctrl_c().await?;
    scheduler.stop().await?;

    Ok(())
}
```

## 技术栈

| 类别 | 技术 |
|------|------|
| 语言 | Rust 2024 Edition |
| 异步运行时 | tokio 1.35 |
| 数据库 | PostgreSQL, TDengine, ClickHouse |
| DataFrame | Polars 0.43 (批量数据处理) |
| 序列化 | serde, serde_json |
| 时间处理 | chrono |
| 数值计算 | rust_decimal |
| HTTP 客户端 | reqwest |
| WebSocket | tokio-tungstenite |
| 日志 | tracing, tracing-subscriber |
| CLI | clap, dialoguer, indicatif |
| 终端复用 | Zellij (Rust 原生) |
| 容器化 | Docker, Docker Compose |
| 监控 | Prometheus, Grafana, Loki |

## 项目进度

**✅ 全部 20 个阶段已完成！**

| Phase | 模块 | 状态 |
|-------|------|------|
| 1-5 | 数据采集、竞价分析、K线管理、回测引擎、任务调度 | ✅ |
| 6-10 | TDX解析、GBBQ存储、多周期查询、东财采集、ClickHouse优化 | ✅ |
| 11-15 | WebSocket、技术指标、Polars、CLI命令、策略实现 | ✅ |
| 16-18 | 实时监控、导入导出、性能测试与优化 | ✅ |
| 19-20 | 部署与运维（Docker/CI-CD）、Zellij集成 | ✅ |

## 与 Python quantix 的关系

```
┌─────────────────────────────────────────────────────────┐
│                     Python quantix                      │
│  (数据采集、存储、Web API、完整业务逻辑)                 │
└─────────────────────────────────────────────────────────┘
                          ↕ 共享数据库
┌─────────────────────────────────────────────────────────┐
│                      quantix-rust                       │
│  (高性能回测、实时分析、任务调度)                         │
└─────────────────────────────────────────────────────────┘
```

- **共享数据源**: PostgreSQL, TDengine
- **共享数据格式**: 一致的数据模型和序列化格式
- **互补定位**:
  - Python: 快速开发、完整业务功能
  - Rust: 高性能计算、低延迟分析

## 许可证

MIT License

## 作者

MyStocks Team

## 文档

- [开发规范指南](docs/standards/DEVELOPMENT_GUIDELINES.md) - 必读！
- [用户手册](docs/USER_MANUAL.md)
- [Python quantix](https://github.com/chengjon/mystocks)
- [问题反馈](https://github.com/chengjon/quantix-rust/issues)

## CI/CD

项目使用 GitHub Actions 进行持续集成：
- ✅ 代码格式检查（rustfmt）
- ✅ 代码质量检查（clippy）
- ✅ 单元测试和集成测试
- ✅ 安全审计（cargo audit）
- ✅ 依赖版本检查
- ✅ 多平台构建（Linux/macOS/Windows）
- ✅ 文档生成和部署
