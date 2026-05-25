# quantix-rust 开发规范指南

> 本文档是开发规范，不作为功能状态注册表；功能状态、已设计/待实现项和可用边界以根目录 `FUNCTION_TREE.md` 的状态注册表行为准。

本文档基于 Rust 最佳实践和量化交易场景的特殊要求，为 quantix-rust 项目提供统一的开发规范。

## 📋 目录

- [核心编码规则](#核心编码规则)
- [模块化设计](#模块化设计)
- [量化交易特殊注意事项](#量化交易特殊注意事项)
- [测试规范](#测试规范)
- [性能优化指南](#性能优化指南)
- [安全与稳定性](#安全与稳定性)
- [代码质量工具](#代码质量工具)

---

## 核心编码规则

### 1. 所有权与借用

#### ✅ 必须遵守

**避免全局可变状态**
```rust
// ❌ 禁止：全局可变状态
static mut ACCOUNT: Account = Account::new();

// ✅ 推荐：使用 Arc<Mutex<T>> 共享状态
use std::sync::{Arc, Mutex};
let account = Arc::new(Mutex::new(Account::new()));

// ✅ 异步场景使用 tokio::sync::Mutex
let account = Arc::new(tokio::sync::Mutex::new(Account::new()));
```

**生命周期标注**
```rust
// 处理长生命周期资源时，明确标注生命周期
pub struct DatabaseConnection<'a> {
    pool: &'a PgPool,
}
```

**传递规则**
```rust
// 小数据（订单参数）：优先值传递
fn validate_order(params: OrderParams) -> Result<()>;

// 大数据（K线数组）：使用引用传递
fn calculate_ma(data: &[Kline], period: usize) -> Vec<f64>;
```

### 2. 错误处理

#### ✅ 统一错误类型

项目使用 `thiserror` 定义统一错误类型（`src/core/error.rs`）：

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum QuantixError {
    #[error("配置错误: {0}")]
    Config(String),

    #[error("数据库连接失败: {0}")]
    DatabaseConnection(String),

    #[error("数据源错误: {0}")]
    DataSource(String),

    #[error("数据解析错误: {0}")]
    DataParse(String),

    #[error("超时错误: {0}")]
    Timeout(String),

    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("序列化错误: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("SQLx 错误: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("HTTP 请求错误: {0}")]
    Http(#[from] reqwest::Error),

    #[error("其他错误: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, QuantixError>;
```

#### ✅ 错误传递规则

```rust
// ✅ 使用 ? 传递错误
async fn fetch_stock_data(code: &str) -> Result<StockData> {
    let url = format!("https://api.example.com/stock/{}", code);
    let resp = reqwest::get(&url).await?;  // 自动转换为 QuantixError
    Ok(resp.json().await?)
}

// ❌ 禁止：生产环境使用 unwrap/expect
fn parse_price(price_str: &str) -> Price {
    price_str.parse::<Price>().unwrap()  // 危险！
}

// ✅ 推荐：返回 Result
fn parse_price(price_str: &str) -> Result<Price> {
    price_str.parse::<Price>().map_err(|e| {
        QuantixError::DataParse(format!("价格解析失败: {}", e))
    })
}
```

#### 🚫 严格禁止

- 生产代码中使用 `unwrap()` / `expect()`
- 仅在测试代码中使用 `unwrap()`
- 仅在逻辑错误（死代码路径）时使用 `panic!`

### 3. 类型安全

#### ✅ 强类型约束

**价格和数量必须使用 Decimal**
```rust
use rust_decimal::Decimal;

pub type Price = Decimal;
pub type Volume = i64;
pub type Amount = Decimal;

// ✅ 正确：使用 Decimal 避免精度丢失
#[derive(Debug, Clone)]
pub struct Order {
    pub price: Price,
    pub volume: Volume,
}

// ❌ 错误：使用 f64/f32 可能导致精度丢失
pub struct OrderBad {
    pub price: f64,  // 危险！
}
```

**股票代码强类型**
```rust
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Symbol(String);

impl Symbol {
    pub fn new(s: &str) -> Result<Self> {
        if s.len() != 6 {
            return Err(QuantixError::DataParse(format!(
                "无效股票代码: {} (必须是6位)", s
            )));
        }
        Ok(Self(s.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

**枚举替代魔法值**
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderType {
    Limit,
    Market,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KlineInterval {
    Min1,
    Min5,
    Min15,
    Min30,
    Hour1,
    Day1,
}

// ❌ 禁止：硬编码字符串/数字
fn place_order(side: &str, type_: i32) { }  // 不清晰

// ✅ 推荐：使用枚举
fn place_order(side: OrderSide, type_: OrderType) { }
```

### 4. 内存管理

#### ✅ 性能优化原则

**预分配内存**
```rust
// ✅ 知道大小时，预分配容量
let mut klines = Vec::with_capacity(10000);
for i in 0..10000 {
    klines.push(kline);
}

// ❌ 避免：多次扩容
let mut klines = Vec::new();  // 可能扩容多次
```

**避免不必要拷贝**
```rust
// ✅ 使用引用避免拷贝
fn process_klines(klines: &[Kline]) -> Vec<Indicator> {
    klines.iter().map(|k| calculate_indicator(k)).collect()
}

// ✅ 使用 Cow 处理动态字符串
use std::borrow::Cow;

fn format_message(msg: Cow<str>) -> String {
    format!("处理消息: {}", msg)
}

// ❌ 避免：不必要的 String 克隆
fn process_klines_bad(klines: Vec<Kline>) -> Vec<Indicator> {
    klines.into_iter().map(|k| calculate_indicator(&k)).collect()
}
```

**手动释放临时大数据**
```rust
use std::mem::drop;

async fn process_large_dataset() -> Result<()> {
    let large_data = load_large_data().await?;

    // 处理数据
    let result = analyze(&large_data);

    // 手动释放，避免占用内存
    drop(large_data);

    // 继续其他操作
    save_result(result).await
}
```

---

## 模块化设计

### 1. 模块拆分原则

#### ✅ 按功能拆分模块

量化 CLI 建议拆分如下模块：

```
src/
├── cli/          # CLI 命令定义（clap）
│   ├── mod.rs
│   ├── handlers.rs    # 命令处理器
│   └── args.rs        # 命令行参数定义
├── api/          # 交易所 API 封装（可选）
│   ├── mod.rs
│   └── client.rs
├── sources/      # 数据源适配器
│   ├── mod.rs
│   ├── tdx.rs         # 通达信
│   ├── akshare.rs     # AkShare
│   └── websocket.rs   # WebSocket
├── data/         # 数据模型
│   ├── mod.rs
│   └── models.rs
├── analysis/     # 分析模块
│   ├── mod.rs
│   ├── indicators.rs  # 技术指标
│   ├── backtest.rs    # 回测引擎
│   └── portfolio.rs   # 投资组合
├── strategy/     # 策略逻辑
│   ├── mod.rs
│   ├── trait_def.rs   # 策略 trait
│   └── ma_cross.rs    # 均线交叉策略
├── db/           # 数据库客户端
│   ├── mod.rs
│   ├── clickhouse.rs
│   └── postgres.rs
├── sync/         # 数据同步 ETL
│   ├── mod.rs
│   └── etl.rs
├── tasks/        # 任务调度
│   ├── mod.rs
│   ├── cron.rs
│   └── scheduler.rs
├── core/         # 核心模块
│   ├── mod.rs
│   ├── config.rs      # 配置管理
│   ├── error.rs       # 错误处理
│   └── trading_calendar.rs
└── utils/        # 工具函数
    ├── mod.rs
    └── helpers.rs
```

### 2. 关注点分离

#### ✅ CLI 层仅处理交互

```rust
// src/cli/handlers.rs

use crate::analysis::backtest::{BacktestEngine, BacktestConfig};
use crate::strategy::ma_cross::MACrossStrategy;

pub async fn run_backtest(args: BacktestArgs) -> Result<()> {
    // CLI 层：仅处理参数解析和输出
    println!("开始回测: {} ({})", args.symbol, args.strategy);

    // 构建配置
    let config = BacktestConfig::from_args(args)?;

    // 创建引擎（核心逻辑）
    let mut engine = BacktestEngine::new(config.clone());

    // 创建策略（核心逻辑）
    let mut strategy = MACrossStrategy::new(args.short_period, args.long_period);

    // 加载数据
    let data = load_kline_data(&config.symbol).await?;

    // 运行回测（核心逻辑）
    let result = engine.run(&mut strategy, &data).await?;

    // 输出结果（CLI 层）
    print_backtest_result(&result);

    Ok(())
}
```

#### ✅ 核心逻辑封装为独立库

```rust
// src/analysis/backtest.rs

/// 回测引擎 - 可独立测试的核心逻辑
pub struct BacktestEngine {
    config: BacktestConfig,
    portfolio: Portfolio,
    calculator: PerformanceCalculator,
}

impl BacktestEngine {
    /// 创建新的回测引擎
    pub fn new(config: BacktestConfig) -> Self {
        Self {
            config,
            portfolio: Portfolio::new(),
            calculator: PerformanceCalculator::new(),
        }
    }

    /// 运行回测 - 核心业务逻辑
    pub async fn run<S>(
        &mut self,
        strategy: &mut S,
        data: &HashMap<String, Vec<Kline>>,
    ) -> Result<BacktestResult>
    where
        S: Strategy + ?Sized,
    {
        // 纯粹的业务逻辑，不涉及 CLI
        // ...
    }
}

// 可独立测试
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_backtest_engine() {
        let config = BacktestConfig::default();
        let mut engine = BacktestEngine::new(config);

        // 测试核心逻辑，无需 CLI
        assert!(engine.portfolio.is_empty());
    }
}
```

### 3. 模块间依赖原则

#### ✅ 依赖层次

```
┌─────────────────────────────┐
│      CLI 层 (cli/)         │  用户交互
│  - 参数解析                 │
│  - 输出格式化               │
└───────────┬─────────────────┘
            │ 依赖
┌───────────▼─────────────────┐
│   业务逻辑层 (strategy/)    │  策略、分析
│   - 交易策略                │
│   - 回测引擎                │
└───────────┬─────────────────┘
            │ 依赖
┌───────────▼─────────────────┐
│   数据访问层 (db/, sources/)│  存储、数据源
│   - 数据库操作              │
│   - API 调用                │
└───────────┬─────────────────┘
            │ 依赖
┌───────────▼─────────────────┐
│   核心层 (core/, data/)     │  基础设施
│   - 错误处理                │
│   - 配置管理                │
│   - 数据模型                │
└─────────────────────────────┘
```

#### ✅ 避免循环依赖

```rust
// ❌ 错误：循环依赖
// analysis/db.rs
use crate::db::Database;  // 依赖 db

// db/analysis.rs
use crate::analysis::Analyzer;  // 依赖 analysis

// ✅ 正确：通过 trait 解耦
// core/database.rs
pub trait Database {
    fn get_kline_data(&self, code: &str) -> Result<Vec<Kline>>;
}

// analysis/indicators.rs
use crate::core::Database;  // 依赖抽象 trait，不依赖具体实现

impl<T: Database> Analyzer<T> {
    pub fn calculate_ma(&self, db: &T, code: &str) -> Vec<f64> {
        let data = db.get_kline_data(code)?;
        // ...
    }
}
```

### 4. 异步 CLI 完整示例

#### ✅ 使用 clap + tokio 构建 CLI

```rust
// src/main.rs

use clap::Parser;
use quantix_cli::{cli::handlers, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into())
        )
        .init();

    // 解析命令行参数
    let args = Cli::parse();

    // 执行命令
    run_command(args).await
}

#[derive(Parser, Debug)]
#[command(name = "quantix")]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// 初始化配置
    Init {
        #[arg(short, long, default_value = "config.toml")]
        config_path: String,
    },

    /// 数据查询
    Data {
        #[command(subcommand)]
        cmd: DataCommands,
    },

    /// 运行回测
    Backtest {
        #[arg(short, long)]
        strategy: String,

        #[arg(short, long)]
        symbol: String,

        #[arg(long, default_value = "1y")]
        period: String,

        #[arg(long, default_value = "1000000")]
        initial_capital: f64,
    },

    /// 启动任务调度器
    Task {
        #[command(subcommand)]
        cmd: TaskCommands,
    },

    /// 交互式菜单
    Menu,
}

#[derive(Subcommand, Debug)]
enum DataCommands {
    Query {
        #[arg(short, long)]
        code: String,

        #[arg(short, long, default_value = "1d")]
        period: String,

        #[arg(long)]
        start: Option<String>,

        #[arg(long)]
        end: Option<String>,

        #[arg(short, long, default_value = "100")]
        limit: Option<usize>,
    },

    Export {
        #[arg(short, long)]
        code: String,

        #[arg(short, long)]
        output: String,

        #[arg(short, long, default_value = "csv")]
        format: String,
    },
}

#[derive(Subcommand, Debug)]
enum TaskCommands {
    Start {
        #[arg(short, long)]
        daemon: bool,
    },

    Stop,

    Status,
}

async fn run_command(args: Cli) -> Result<()> {
    match args.command {
        Commands::Init { config_path } => {
            handlers::run_init(config_path).await?;
        }

        Commands::Data { cmd } => {
            handlers::run_data_command(cmd).await?;
        }

        Commands::Backtest {
            strategy,
            symbol,
            period,
            initial_capital,
        } => {
            handlers::run_strategy_backtest(strategy, symbol, period, initial_capital).await?;
        }

        Commands::Task { cmd } => {
            handlers::run_task_command(cmd).await?;
        }

        Commands::Menu => {
            handlers::run_simple_menu().await?;
        }
    }

    Ok(())
}

// 优雅关闭
#[tokio::main]
async fn main() -> Result<()> {
    // 设置信号处理
    tokio::spawn(async {
        use tokio::signal::unix::{signal, SignalKind};

        let mut sigint = signal(SignalKind::interrupt()).unwrap();
        let mut sigterm = signal(SignalKind::terminate()).unwrap();

        tokio::select! {
            _ = sigint.recv() => {
                tracing::info!("收到 SIGINT 信号，正在优雅退出...");
            }
            _ = sigterm.recv() => {
                tracing::info!("收到 SIGTERM 信号，正在优雅退出...");
            }
        }

        // 清理资源
        tracing::info!("资源已清理");
        std::process::exit(0);
    });

    // 主逻辑
    main_logic().await
}
```

#### ✅ 异步命令处理器

```rust
// src/cli/handlers.rs

use crate::core::Result;
use crate::analysis::backtest::{BacktestEngine, BacktestConfig};
use crate::db::clickhouse::ClickHouseClient;
use indicatif::{ProgressBar, ProgressStyle};
use tracing::{info, error};

pub async fn run_strategy_backtest(
    strategy_name: String,
    symbol: String,
    period: String,
    initial_capital: f64,
) -> Result<()> {
    info!(
        strategy = %strategy_name,
        symbol = %symbol,
        period = %period,
        "开始回测"
    );

    // 显示进度条
    let pb = ProgressBar::new(100);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({msg})")
        .progress_chars("#>-"));

    pb.set_message("加载配置...");
    pb.inc(10);

    // 构建配置
    let config = BacktestConfig {
        symbol: symbol.clone(),
        start_date: parse_start_date(&period)?,
        end_date: chrono::Utc::now().date_naive(),
        initial_capital: rust_decimal::Decimal::from_str(&format!("{}", initial_capital))?,
        ..Default::default()
    };

    pb.set_message("连接数据库...");
    pb.inc(20);

    // 连接数据库
    let client = ClickHouseClient::new(
        &std::env::var("CLICKHOUSE_URL").unwrap_or_else(|_| "http://localhost:8123".to_string()),
        &std::env::var("CLICKHOUSE_DB").unwrap_or_else(|_| "quantix".to_string())
    ).await?;

    pb.set_message("加载数据...");
    pb.inc(30);

    // 加载数据
    let mut data_map = HashMap::new();
    let klines = client.get_kline_data(&symbol, "1d", None, Some(10000))
        .await
        .map_err(|e| {
            error!(error = %e, "加载数据失败");
            e
        })?;
    data_map.insert(symbol.clone(), klines);

    pb.set_message("运行回测...");
    pb.inc(50);

    // 创建引擎和策略
    let mut engine = BacktestEngine::new(config.clone());
    let mut strategy = create_strategy(&strategy_name)?;

    // 运行回测
    let result = tokio::spawn(async move {
        engine.run(&mut strategy, &data_map).await
    }).await??;

    pb.finish_with_message("回测完成");

    // 输出结果
    print_backtest_result(&result);

    Ok(())
}
```

### 5. 模块测试策略

#### ✅ 单元测试 + 集成测试分离

```rust
// tests/ 集成测试目录结构

tests/
├── integration/
│   ├── mod.rs
│   ├── clickhouse_tests.rs     # ClickHouse 集成测试
│   └── api_tests.rs             # API 集成测试
└── e2e/
    ├── mod.rs
    └── backtest_flow_tests.rs   # 端到端回测流程测试

// src/ 各模块的单元测试
src/
├── analysis/
│   └── indicators.rs
│       └── tests/               # 单元测试模块
│           └── indicators_tests.rs
```

#### ✅ 测试组织示例

```rust
// src/analysis/indicators.rs

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_calculate_ma() {
        // 纯函数测试，不依赖外部资源
        let data = vec![10.0, 11.0, 12.0, 13.0, 14.0];
        let result = calculate_ma(&data, 3);
        assert_eq!(result, vec![None, None, Some(dec!(11)), Some(dec!(12)), Some(dec!(13))]);
    }
}

// tests/integration/clickhouse_tests.rs

use quantix_cli::db::clickhouse::ClickHouseClient;

#[tokio::test]
async fn test_clickhouse_integration() {
    // 集成测试，依赖真实数据库
    let client = ClickHouseClient::new(
        "http://localhost:8123",
        "quantix_test"
    ).await.unwrap();

    // 测试真实数据库操作
    let result = client.get_kline_data("000001", "1d", None, Some(1))
        .await
        .unwrap();

    assert!(!result.is_empty());
}
```

---

## 量化交易特殊注意事项

### 1. 性能与效率

#### ✅ 列式存储 + 向量化计算

**必须使用 Polars 处理大数据集**（已在 Phase 13 实现）
```rust
use crate::analysis::polars_adapter::{BatchKlineData, PolarsCalculator};

// ✅ 批量计算指标
let calc = PolarsCalculator::new();
let indicators = calc.calculate_batch(&data, &["ma5", "ma20", "ma60", "rsi14"]);

// ❌ 避免：手动循环计算
for kline in &klines {
    // 单条计算效率低
}
```

#### ✅ 异步编程

**所有 I/O 操作必须异步**
```rust
// ✅ 异步数据库操作
async fn get_stock_data(code: &str) -> Result<StockData> {
    sqlx::query_as("SELECT * FROM stocks WHERE code = $1")
        .bind(code)
        .fetch_one(pool)
        .await
        .map_err(|e| QuantixError::DatabaseQuery(e.to_string()))
}

// ✅ 并发抓取多个股票数据
use futures::future::join_all;

async fn fetch_multiple_stocks(codes: &[&str]) -> Result<Vec<StockData>> {
    let futures: Vec<_> = codes.iter()
        .map(|&code| fetch_stock_data(code))
        .collect();

    join_all(futures).await
        .into_iter()
        .collect()
}
```

#### ✅ 超时控制

**所有网络请求必须设置超时**
```rust
use tokio::time::{timeout, Duration};

async fn fetch_with_timeout(url: &str) -> Result<String> {
    timeout(
        Duration::from_secs(10),
        reqwest::get(url)
    ).await
    .map_err(|_| QuantixError::Timeout("请求超时".to_string()))?
    .map_err(|e| QuantixError::Http(e))?
    .text()
    .await
    .map_err(|e| QuantixError::Http(e))
}
```

#### ✅ 线程数控制

**限制异步运行时线程数**
```rust
use tokio::runtime::Builder;

#[tokio::main]
async fn main() -> Result<()> {
    // 默认使用多线程运行时
    Ok(())
}

// 或者自定义线程数
let runtime = Builder::new_multi_thread()
    .worker_threads(4)  // 根据任务类型调整
    .enable_all()
    .build()?;
```

### 2. 安全性与稳定性

#### ✅ 参数校验

**严格校验订单参数**
```rust
#[derive(Debug, Clone)]
pub struct OrderParams {
    pub symbol: Symbol,
    pub price: Price,
    pub volume: Volume,
    pub side: OrderSide,
}

impl OrderParams {
    pub fn validate(&self) -> Result<()> {
        if self.price <= dec!(0) {
            return Err(QuantixError::Other("价格必须大于0".to_string()));
        }

        if self.volume <= 0 {
            return Err(QuantixError::Other("数量必须大于0".to_string()));
        }

        // A股最小交易单位：100股（1手）
        if self.volume % 100 != 0 {
            return Err(QuantixError::Other("数量必须是100的整数倍".to_string()));
        }

        Ok(())
    }
}
```

#### ✅ 幂等性设计

**订单操作实现幂等**
```rust
use uuid::Uuid;

pub struct OrderService {
    // 使用 UUID 防止重复提交
    submitted_orders: Arc<Mutex<HashMap<Uuid, Order>>>,
}

impl OrderService {
    pub async fn submit_order(&self, params: OrderParams) -> Result<Order> {
        let order_id = Uuid::new_v4();

        // 检查是否已提交
        {
            let orders = self.submitted_orders.lock().await;
            if orders.contains_key(&order_id) {
                return Err(QuantixError::Other("订单已提交".to_string()));
            }
        }

        // 提交订单
        let order = self.do_submit(params, order_id).await?;

        // 记录已提交订单
        let mut orders = self.submitted_orders.lock().await;
        orders.insert(order_id, order.clone());

        Ok(order)
    }
}
```

#### ✅ 信号处理

**优雅关闭资源**
```rust
use tokio::signal::unix::{signal, SignalKind};

async fn main() -> Result<()> {
    // 启动服务
    let scheduler = TaskScheduler::new().await?;

    // 监听 Ctrl+C
    let mut sigint = signal(SignalKind::interrupt())?;
    let mut sigterm = signal(SignalKind::terminate())?;

    tokio::select! {
        _ = sigint.recv() => {
            tracing::info!("收到 SIGINT 信号，正在优雅退出...");
            scheduler.stop().await?;
        }
        _ = sigterm.recv() => {
            tracing::info!("收到 SIGTERM 信号，正在优雅退出...");
            scheduler.stop().await?;
        }
    }

    Ok(())
}
```

#### ✅ 日志与监控

**使用 tracing 记录关键操作**
```rust
use tracing::{info, warn, error, debug};

// 初始化日志
tracing_subscriber::fmt()
    .with_env_filter(
        tracing_subscriber::EnvFilter::from_default_env()
            .add_directive(tracing::Level::INFO.into())
    )
    .init();

// 关键操作必须记录日志
async fn submit_order(&self, order: &Order) -> Result<()> {
    info!(
        order_id = %order.id,
        symbol = %order.symbol,
        price = %order.price,
        volume = order.volume,
        side = ?order.side,
        "提交订单"
    );

    match self.do_submit(order).await {
        Ok(_) => {
            info!(order_id = %order.id, "订单提交成功");
            Ok(())
        }
        Err(e) => {
            error!(
                order_id = %order.id,
                error = %e,
                "订单提交失败"
            );
            Err(e)
        }
    }
}
```

### 3. CLI 交互体验

#### ✅ 进度反馈

**长时间任务必须显示进度**（已在 Phase 14 实现）
```rust
use indicatif::{ProgressBar, ProgressStyle};

async fn backtest_strategy(config: &BacktestConfig) -> Result<BacktestReport> {
    let total_days = (config.end_date - config.start_date).num_days();
    let pb = ProgressBar::new(total_days as u64);

    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
        .progress_chars("#>-"));

    for date in generate_date_range(config.start_date, config.end_date) {
        // 处理数据
        process_day(date).await?;

        pb.inc(1);
    }

    pb.finish_with_message("回测完成");

    Ok(report)
}
```

#### ✅ 配置管理

**支持配置文件**（已在 Phase 1 实现）
```toml
# config.toml
[database]
clickhouse_url = "http://localhost:8123"
clickhouse_db = "quantix"

[tdx]
host = "192.168.1.100"
port = 7709

[logging]
level = "info"
```

#### ✅ 输出格式化

**支持多种输出格式**
```rust
// 表格输出（人类友好）
use prettytable::Table;

fn print_stocks_table(stocks: &[Stock]) {
    let mut table = Table::new();
    table.add_row(row!["代码", "名称", "价格", "涨跌幅"]);

    for stock in stocks {
        table.add_row(row![
            stock.code,
            stock.name,
            stock.price,
            format!("{:.2}%", stock.change_percent)
        ]);
    }

    table.printstd();
}

// JSON 输出（机器可读）
fn print_stocks_json(stocks: &[Stock]) -> Result<()> {
    let json = serde_json::to_string_pretty(stocks)?;
    println!("{}", json);
    Ok(())
}
```

### 4. 数据持久化与兼容

#### ✅ 存储选型

**根据数据量选择合适的存储**

**轻量级需求：SQLite**
```toml
# Cargo.toml
rusqlite = "0.30"
```

适用于：
- 策略配置存储
- 订单记录
- 用户偏好设置
- 本地缓存数据

```rust
use rusqlite::{Connection, Result};

fn init_sqlite_db() -> Result<Connection> {
    let conn = Connection::open("quantix.db")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS strategies (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            config TEXT NOT NULL
        )",
        [],
    )?;

    Ok(conn)
}
```

**海量行情：ClickHouse/TDengine**（已在 Phase 3 实现）
```toml
# Cargo.toml
clickhouse = "0.12"      # 已使用
taos-ws = { version = "0.5", optional = true }  # TDengine
```

适用于：
- Tick 级别数据
- 分钟/小时 K线数据
- 实时行情数据
- 历史回测数据

**存储选型对比**

| 存储引擎 | 适用场景 | 优势 | 劣势 |
|---------|---------|------|------|
| SQLite | 配置、订单、缓存 | 轻量、无需服务 | 并发性能差 |
| ClickHouse | 海量时序数据 | 列式存储、压缩比高 | 需要独立服务 |
| TDengine | 时序数据 | 专用优化、高性能 | 学习曲线陡峭 |
| PostgreSQL | 关系型数据 | 成熟稳定、功能全 | 时序性能一般 |

#### ✅ 序列化格式

**Parquet（适合行情数据，压缩比高）**
```toml
# Cargo.toml
parquet = { version = "53", features = ["async"] }
arrow = { version = "53", features = ["json"] }
```

优势：
- 列式存储，查询效率高
- 压缩比高（20:1）
- 支持并行读写
- 跨语言兼容

```rust
use arrow::array::{Float64Array, Int64Array};
use arrow::record_batch::RecordBatch;
use parquet::arrow::{ArrowWriter, ArrowReader};

// 写入 Parquet
let schema = Arc::new(Schema::new(vec![
    Field::new("timestamp", DataType::Int64, false),
    Field::new("price", DataType::Float64, false),
]));

let batch = RecordBatch::try_new(
    schema.clone(),
    vec![
        Arc::new(Int64Array::from(vec![1, 2, 3])),
        Arc::new(Float64Array::from(vec![10.5, 11.0, 11.5])),
    ],
)?;

let file = File::create("data.parquet")?;
let mut writer = ArrowWriter::try_new(file, schema, None)?;
writer.write(&batch)?;
writer.close()?;
```

**Bincode（快速序列化）**
```toml
# Cargo.toml
bincode = "1.3"
```

优势：
- 序列化/反序列化速度快
- 二进制格式，体积小
- 适合进程间通信

```rust
use bincode::{serialize, deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StrategyConfig {
    pub name: String,
    pub params: Vec<f64>,
}

// 序列化
let config = StrategyConfig {
    name: "MA-Cross".to_string(),
    params: vec![5.0, 20.0],
};
let encoded: Vec<u8> = serialize(&config)?;

// 反序列化
let decoded: StrategyConfig = deserialize(&encoded)?;
```

**JSON（人类可读）**
```toml
# Cargo.toml
serde_json = "1.0"
```

优势：
- 人类可读
- 易于调试
- 广泛支持

```rust
use serde_json::{to_string_pretty, from_str};

// 序列化
let json = to_string_pretty(&config)?;

// 反序列化
let config: StrategyConfig = from_str(&json)?;
```

#### ✅ 版本兼容性

**配置文件/数据文件添加版本号**
```toml
# config.toml
[version]
major = 1
minor = 0
patch = 0

[database]
# ... 配置内容
```

**数据结构版本控制**
```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DataFile {
    pub version: Version,
    pub data: Vec<Kline>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
}

impl Version {
    pub fn current() -> Self {
        Self { major: 1, minor: 0 }
    }

    pub fn is_compatible(&self, other: &Version) -> bool {
        // 主版本号必须相同
        self.major == other.major
    }
}

// 读取数据时检查版本
fn load_klines_data(path: &str) -> Result<Vec<Kline>> {
    let file = File::open(path)?;
    let data_file: DataFile = bincode::deserialize_from(file)?;

    if !data_file.version.is_compatible(&Version::current()) {
        return Err(QuantixError::Other(
            format!("数据版本不兼容: {:?} vs {:?}",
                data_file.version, Version::current())
        ));
    }

    Ok(data_file.data)
}
```

**数据迁移策略**
```rust
pub trait Migrate {
    type Output;

    fn migrate(self) -> Result<Self::Output>;
}

// 从 v0.9 迁移到 v1.0
impl Migrate for KlineV0_9 {
    type Output = Kline;

    fn migrate(self) -> Result<Kline> {
        Ok(Kline {
            code: self.code,
            date: self.date,
            open: self.open,
            high: self.high,
            low: self.low,
            close: self.close,
            volume: self.volume,
            amount: Some(self.amount),  // v1.0 新增字段
            adjust_type: AdjustType::None,  // v1.0 新增字段
        })
    }
}
```

---

## 测试规范

### 1. 单元测试

#### ✅ 测试覆盖率要求

**核心业务逻辑必须有单元测试**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_calculate_ma() {
        let data = vec![10.0, 11.0, 12.0, 13.0, 14.0];
        let result = calculate_ma(&data, 3);

        assert_eq!(result.len(), 5);
        assert_eq!(result[0], None);
        assert_eq!(result[1], None);
        assert_eq!(result[2], Some(dec!(11)));
    }

    #[test]
    fn test_order_validation() {
        let params = OrderParams {
            symbol: Symbol::new("000001").unwrap(),
            price: dec!(10.5),
            volume: 100,  // 正确：100的整数倍
            side: OrderSide::Buy,
        };

        assert!(params.validate().is_ok());

        let invalid_params = OrderParams {
            volume: 150,  // 错误：不是100的整数倍
            ..params.clone()
        };

        assert!(invalid_params.validate().is_err());
    }

    #[test]
    fn test_parse_price() {
        assert_eq!(parse_price("10.5"), Ok(dec!(10.5)));
        assert!(parse_price("invalid").is_err());
        assert!(parse_price("-1").is_err());  // 价格不能为负
    }
}
```

### 2. 集成测试

#### ✅ 测试真实场景

**测试数据库交互**
```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_clickhouse_connection() {
        let client = ClickHouseClient::new(
            "http://localhost:8123",
            "quantix_test"
        ).await.unwrap();

        // 测试插入
        let klines = vec![create_test_kline()];
        client.insert_kline_data(&klines).await.unwrap();

        // 测试查询
        let result = client.get_kline_data("000001", "1d", None, Some(1))
            .await
            .unwrap();

        assert!(!result.is_empty());
    }
}
```

### 3. 回测验证

#### ✅ 使用历史数据验证

**确保收益/风险计算正确**
```rust
#[tokio::test]
async fn test_backtest_performance() {
    let config = BacktestConfig {
        start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        end_date: NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
        initial_capital: dec!(1000000),
        ..Default::default()
    };

    let strategy = MACrossStrategy::new(5, 20);
    let mut engine = BacktestEngine::new(config);

    let data = load_test_data("000001").await.unwrap();
    let result = engine.run(&mut strategy, &data).await.unwrap();

    // 验证基本指标
    assert!(result.report.total_return > dec!(-1));  // 最大亏损不超过100%
    assert!(result.report.sharpe_ratio > dec!(0));    // 夏普比率应该合理
    assert!(result.report.max_drawdown <= dec!(1));   // 最大回撤不超过100%
}
```

### 4. 收口阶段门禁优先级

#### ✅ 任务进入“可收口”阶段后的强制规则

当一个任务已经满足以下特征之一时，视为进入“可收口”阶段：

- 主体实现已经完成
- 剩余问题主要集中在测试、验证、门禁、文档对齐或发布前检查
- 已经具备合并 / 提交 / 交付条件，只差运行门禁闭环

进入该阶段后，必须遵守以下要求：

- **优先完成运行门禁闭环**：先完成与当前任务直接相关的 `cargo test`、`cargo clippy`、`cargo fmt --check`、集成验证、repo hygiene、手工验收或等效门禁。
- **不得继续扩散为零散 cosmetic 微调**：在门禁未闭环前，不得继续新增与交付无关的命名清理、注释润色、输出文案微调、顺手重构、机械性告警清理或其它“顺便做一下”的改动。
- **禁止把收口阶段重新拉回发散阶段**：如果已经进入收口，只允许处理会阻塞门禁通过、影响验收结论、或直接影响交付质量的问题。
- **新增改动必须说明与门禁的直接关系**：如果确需继续修改，必须能明确回答“这项修改解决了哪个未闭环的运行门禁或验收阻塞”。

#### ✅ 允许的例外

只有在以下情况下，才允许在收口阶段继续改代码：

- 当前门禁失败，且失败原因必须通过代码修改才能修复
- 验收或运行验证暴露出真实行为缺陷，而不是表现层瑕疵
- 文档或输出文案会直接误导操作流程，导致门禁或验收结论失真

#### 🚫 明确禁止

- “顺手把这几个 warning 也清了”
- “顺手统一一下命名 / 注释 / 输出格式”
- “顺手做一点小重构，后面更干净”
- 在尚未跑完任务相关门禁前继续扩大改动面

#### ✅ 收口阶段执行顺序

1. 先冻结范围，停止继续加需求和顺手优化。
2. 跑完当前任务相关的运行门禁。
3. 只修复门禁失败项和验收阻塞项。
4. 重新验证，直到形成闭环。
5. 闭环完成后，如仍需 cosmetic 清理，必须作为后续独立任务处理。

---

## 性能优化指南

### 1. 编译优化

#### ✅ Release 配置（已配置）

```toml
[profile.release]
opt-level = 3       # 最高优化级别
lto = true          # 链接时优化
codegen-units = 1   # 减少代码生成单元
strip = true        # 去除调试符号
```

### 2. 性能分析

#### ✅ 使用 criterion 基准测试

```toml
[dev-dependencies]
criterion = "0.5"
```

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_calculate_rsi(c: &mut Criterion) {
    let data: Vec<Decimal> = (1..10000)
        .map(|i| Decimal::from(i))
        .collect();

    c.bench_function("calculate_rsi_10000", |b| {
        b.iter(|| {
            calculate_rsi(black_box(&data), 14)
        })
    });
}

criterion_group!(benches, bench_calculate_rsi);
criterion_main!(benches);
```

**运行基准测试**
```bash
cargo bench
```

### 3. 性能优化技巧

#### ✅ 避免堆分配（高频场景）

```rust
// 使用 smallvec 或 stackvec 替代 Vec
use smallvec::SmallVec;

fn process_tick_prices() {
    // 小数组在栈上分配，避免堆分配
    let prices: SmallVec<[f64; 4]> = SmallVec::new();
}
```

#### ✅ 使用迭代器惰性计算

```rust
// ✅ 推荐：使用迭代器
let results: Vec<_> = klines
    .iter()
    .filter(|k| k.volume > 1000000)
    .map(|k| calculate_indicator(k))
    .collect();

// ❌ 避免：多次中间分配
let filtered: Vec<_> = klines.iter().filter(|k| k.volume > 1000000).collect();
let results: Vec<_> = filtered.iter().map(|k| calculate_indicator(k)).collect();
```

---

## 安全与稳定性

### 1. 依赖安全

#### ✅ 定期检查依赖漏洞

```bash
# 安装 cargo-audit
cargo install cargo-audit

# 检查依赖漏洞
cargo audit
```

### 2. 资源管理

#### ✅ 确保 Drop 正确实现

```rust
pub struct DatabaseConnection {
    pool: PgPool,
}

impl Drop for DatabaseConnection {
    fn drop(&mut self) {
        // 清理资源
        tracing::info!("关闭数据库连接");
    }
}
```

### 3. 并发安全

#### ✅ 使用正确的同步原语

```rust
// 异步场景使用 tokio::sync::Mutex
use tokio::sync::Mutex as TokioMutex;

pub struct AsyncSharedState {
    data: TokioMutex<Vec<Kline>>,
}

// 同步场景使用 std::sync::Mutex
use std::sync::Mutex;

pub struct SyncSharedState {
    data: Mutex<Vec<Kline>>,
}
```

---

## 代码质量工具

### 1. 代码格式化

#### ✅ 使用 rustfmt

```bash
# 格式化代码
cargo fmt

# 检查格式
cargo fmt --check
```

**配置文件**（`.rustfmt.toml`）
```toml
max_width = 120
hard_tabs = false
tab_spaces = 4
```

### 2. 代码检查

#### ✅ 使用 clippy

```bash
# 运行 clippy
cargo clippy

# 严格模式（将警告视为错误）
cargo clippy -- -D warnings
```

### 3. 文档生成

#### ✅ 生成 API 文档

```bash
# 生成文档
cargo doc

# 生成并打开文档
cargo doc --open
```

#### ✅ 文档注释规范

```rust
/// 计算 RSI 指标
///
/// # 参数
///
/// * `data` - 价格数据
/// * `period` - RSI 周期（通常为 14）
///
/// # 返回
///
/// 返回 RSI 值的向量，前 period-1 个元素为 None
///
/// # 示例
///
/// ```
/// use quantix_cli::analysis::indicators::rsi;
/// use rust_decimal_macros::dec;
///
/// let data = vec![dec!(10), dec!(11), dec!(12)];
/// let result = rsi(&data, 2);
/// assert_eq!(result[0], None);
/// ```
///
/// # 错误处理
///
/// 如果 data.len() < period，返回全为 None 的向量
pub fn rsi(data: &[Decimal], period: usize) -> Vec<Option<Decimal>> {
    // 实现...
}
```

### 4. CI/CD 集成

#### ✅ GitHub Actions 配置

创建 `.github/workflows/ci.yml`：

```yaml
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  test:
    name: Test
    runs-on: ubuntu-latest

    services:
      postgres:
        image: postgres:17
        env:
          POSTGRES_USER: test
          POSTGRES_PASSWORD: test
          POSTGRES_DB: quantix_test
        ports:
          - 5432:5432
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

      clickhouse:
        image: clickhouse/clickhouse-server:latest
        ports:
          - 8123:8123
        options: >-
          --health-cmd "clickhouse-client --query 'SELECT 1'"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

    steps:
      - uses: actions/checkout@v3

      - uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Install Rust toolchain
        uses: actions-rs/toolchain@v1
        with:
          profile: minimal
          toolchain: stable
          components: rustfmt, clippy
          override: true

      - name: Install dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y postgresql-client

      - name: Check formatting
        run: cargo fmt --check

      - name: Run clippy
        run: cargo clippy --all-targets --all-features -- -D warnings

      - name: Run tests
        run: cargo test --all-features --verbose

      - name: Generate documentation
        run: cargo doc --no-deps --all-features

  lint:
    name: Lint
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - uses: actions-rs/toolchain@v1
        with:
          profile: minimal
          toolchain: stable
          components: rustfmt, clippy
          override: true

      - name: Check formatting
        run: cargo fmt --check

      - name: Run clippy
        run: cargo clippy --all-targets --all-features -- -D warnings

  security:
    name: Security Audit
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install cargo-audit
        run: cargo install cargo-audit

      - name: Run audit
        run: cargo audit
```

---

## 总结

### ✅ 必须遵守的规则

1. **所有权与借用**：避免全局可变状态，使用 Arc<Mutex<T>>
2. **错误处理**：统一使用 Result，禁止 unwrap/expect
3. **类型安全**：价格/数量使用 Decimal，股票代码使用强类型
4. **性能优化**：使用 Polars 批量计算，预分配内存
5. **异步编程**：所有 I/O 操作异步，设置超时
6. **参数校验**：严格校验订单参数，实现幂等性
7. **日志记录**：使用 tracing 记录关键操作
8. **测试覆盖**：核心逻辑必须有单元测试
9. **代码质量**：通过 rustfmt、clippy、cargo audit 检查

### 📊 项目当前状态

- ✅ Phase 1-14 已完成
- ✅ 已实现 Polars 适配层（Phase 13）
- ✅ 已实现 CLI 命令系统（Phase 14）
- ✅ 使用 tokio 异步运行时
- ✅ 使用 thiserror 统一错误处理
- ✅ 使用 tracing 日志记录
- ✅ 使用 clap + dialoguer + indicatif CLI 工具

### 🎯 后续改进

- [ ] 添加 GitHub Actions CI/CD
- [ ] 添加 rustfmt.toml 配置
- [ ] 添加 clippy.toml 配置
- [ ] 添加更多集成测试
- [ ] 添加性能基准测试
- [ ] 定期运行 cargo audit

---

**最后更新**: 2026-03-07
