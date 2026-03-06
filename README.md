# quantix-rust

A 股量化交易 CLI 工具 - Rust 实现

与 Python quantix 项目共享数据源和数据库，提供高性能的量化分析能力。

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
│   ├── sources/         # 数据源 (TDX, AkShare, 行情采集, K线聚合)
│   ├── strategy/        # 交易策略
│   ├── sync/            # 数据同步 ETL
│   ├── tasks/           # 任务调度 (Cron, Scheduler)
│   └── tui/             # 终端 UI (可选)
├── Cargo.toml
└── README.md
```

## 开发指南

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
```

### 运行测试

```bash
# 所有测试
cargo test

# 单个模块测试
cargo test --package quantix-cli --lib analysis::backtest::tests
```

## 使用示例

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
| 序列化 | serde, serde_json |
| 时间处理 | chrono |
| 数值计算 | rust_decimal |
| HTTP 客户端 | reqwest |
| 日志 | tracing, tracing-subscriber |
| CLI | clap, dialoguer, indicatif |

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

## 链接

- [Python quantix](https://github.com/chengjon/mystocks)
- [问题反馈](https://github.com/chengjon/quantix-rust/issues)
