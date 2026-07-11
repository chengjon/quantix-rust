//! Top-level clap dispatch root: `Cli` struct + `Commands` enum +
//! the `run()` matcher that routes each variant to its handler.

use crate::cli::handlers;
use crate::core::Result;
use clap::Parser;

use super::{
    AccountCommands, AiCommands, AlgoCommands, AnalyzeCommands, AnomalyCommands, BacktestCommands,
    DataCommands, ExecutionCommands, FactorCommands, FundamentalCommands, ImportCommands,
    MarketCommands, MonitorCommands, NewsCommands, NotifyCommands, PerformanceCommands,
    RiskCommands, SafetyCommands, SentimentCommands, StopCommands, StrategyCommands, TaskCommands,
    TradeCommands, WatchlistCommands,
};

/// quantix CLI 顶层解析结构：command 为命令族根枚举。由 clap 解析子命令并路由到 handlers。
#[derive(Parser, Debug)]
#[command(name = "quantix")]
#[command(about = "A股量化交易 CLI 工具", long_about = None)]
#[command(version = "0.1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// 顶层命令枚举：Init 配置初始化、Data 数据源与导入、Market 行情、Strategy 策略、Execution 执行、Risk 风控、Monitor 监控、Analyze 分析、Backtest 回测、Factor 因子、Info 信息聚合、Performance 绩效、Safety 安全开关、Watchlist 自选、Stop 停止服务等。
#[derive(clap::Subcommand, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum Commands {
    /// 初始化配置和数据库
    Init {
        /// 配置文件路径（指向原 quantix 项目）
        #[arg(short, long, default_value = "../config")]
        config_path: String,
    },

    /// 交互式菜单
    Menu {
        /// 启用 TUI 界面
        #[arg(long)]
        tui: bool,
    },

    /// 数据命令
    #[command(subcommand)]
    Data(DataCommands),

    /// 策略命令
    #[command(subcommand)]
    Strategy(StrategyCommands),

    /// 任务命令（实验性，Foundation P0）
    #[command(subcommand)]
    Task(TaskCommands),

    /// 分析命令
    #[command(subcommand)]
    Analyze(AnalyzeCommands),

    /// 回测命令
    #[command(subcommand)]
    Backtest(BacktestCommands),

    /// 绩效命令
    #[command(subcommand)]
    Performance(PerformanceCommands),

    /// Factor research commands
    #[command(subcommand)]
    Factor(FactorCommands),

    /// 监控命令
    #[command(subcommand)]
    Monitor(MonitorCommands),

    /// 止盈止损命令
    #[command(subcommand)]
    Stop(StopCommands),

    /// 自选池命令
    #[command(subcommand)]
    Watchlist(WatchlistCommands),

    /// 市场分析命令
    #[command(subcommand)]
    Market(MarketCommands),

    /// 模拟交易命令
    #[command(subcommand)]
    Trade(TradeCommands),

    /// 风险管理命令
    #[command(subcommand)]
    Risk(RiskCommands),

    /// 系统安全控制命令
    #[command(subcommand)]
    Safety(SafetyCommands),

    /// 执行自动化命令
    #[command(subcommand)]
    Execution(ExecutionCommands),

    /// 异常检测命令 (Isolation Forest)
    #[command(subcommand)]
    Anomaly(AnomalyCommands),

    /// 算法交易命令 (TWAP/VWAP)
    #[command(subcommand)]
    Algo(AlgoCommands),

    /// 账户管理命令
    #[command(subcommand)]
    Account(AccountCommands),

    /// 通知命令
    #[command(subcommand)]
    Notify(NotifyCommands),

    /// AI 决策命令
    #[command(subcommand)]
    Ai(AiCommands),

    /// 新闻搜索命令
    #[command(subcommand)]
    News(NewsCommands),

    /// 基本面数据命令
    #[command(subcommand)]
    Fundamental(FundamentalCommands),

    /// 舆情分析命令
    #[command(subcommand)]
    Sentiment(SentimentCommands),

    /// 智能导入命令
    #[command(subcommand)]
    Import(ImportCommands),

    /// 系统状态
    Status {
        /// 检查数据库连接
        #[arg(long)]
        health: bool,
    },
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        match self.command {
            Commands::Init { config_path } => {
                handlers::run_init(config_path).await?;
            }
            Commands::Menu { tui } => {
                if tui {
                    handlers::run_tui_menu().await?;
                } else {
                    handlers::run_simple_menu().await?;
                }
            }
            Commands::Data(cmd) => {
                handlers::run_data_command(cmd).await?;
            }
            Commands::Strategy(cmd) => {
                handlers::run_strategy_command(cmd).await?;
            }
            Commands::Task(cmd) => {
                handlers::run_task_command(cmd).await?;
            }
            Commands::Analyze(cmd) => {
                handlers::run_analyze_command(cmd).await?;
            }
            Commands::Backtest(cmd) => {
                handlers::run_backtest_command(cmd).await?;
            }
            Commands::Performance(cmd) => {
                handlers::run_performance_command(cmd)?;
            }
            Commands::Factor(cmd) => {
                handlers::run_factor_command(cmd).await?;
            }
            Commands::Monitor(cmd) => {
                handlers::run_monitor_command(cmd).await?;
            }
            Commands::Stop(cmd) => {
                handlers::run_stop_command(cmd).await?;
            }
            Commands::Watchlist(cmd) => {
                handlers::run_watchlist_command(cmd).await?;
            }
            Commands::Market(cmd) => {
                handlers::run_market_command(cmd).await?;
            }
            Commands::Trade(cmd) => {
                handlers::run_trade_command(cmd).await?;
            }
            Commands::Risk(cmd) => {
                handlers::run_risk_command(cmd).await?;
            }
            Commands::Safety(cmd) => {
                handlers::run_safety_command(cmd)?;
            }
            Commands::Execution(cmd) => {
                handlers::run_execution_command(cmd).await?;
            }
            Commands::Anomaly(cmd) => {
                handlers::run_anomaly_command(cmd).await?;
            }
            Commands::Algo(cmd) => {
                handlers::run_algo_command(cmd).await?;
            }
            Commands::Account(cmd) => {
                handlers::run_account_command(cmd).await?;
            }
            Commands::Notify(cmd) => {
                handlers::run_notify_command(cmd).await?;
            }
            Commands::Ai(cmd) => {
                handlers::run_ai_command(cmd).await?;
            }
            Commands::News(cmd) => {
                handlers::run_news_command(cmd).await?;
            }
            Commands::Fundamental(cmd) => {
                handlers::run_fundamental_command(cmd).await?;
            }
            Commands::Sentiment(cmd) => {
                handlers::run_sentiment_command(cmd).await?;
            }
            Commands::Import(cmd) => {
                handlers::run_import_command(cmd).await?;
            }
            Commands::Status { health } => {
                handlers::run_status(health).await?;
            }
        }
        Ok(())
    }
}
