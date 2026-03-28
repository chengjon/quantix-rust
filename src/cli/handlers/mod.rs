pub(crate) use super::{
    AiCommands, AlgoCommands, AnalyzeCommands, AnomalyCommands,
    DataCommands, ExecutionBridgeCommands, ExecutionCommands, ExecutionConfigCommands,
    ExecutionDaemonCommands, MarketCommands, MonitorAlertCommands, MonitorCommands,
    MonitorConfigCommands, MonitorDaemonCommands, MonitorEventCommands, MonitorServiceCommands,
    MonitorServiceConfigCommands, FundamentalCommands, NewsCommands, NotifyCommands, RiskCommands, RiskLockCommands, RiskRuleCommands,
    ImportCommands, ScreenerCommands, SentimentCommands, StopCommands, StrategyCommands, StrategyConfigCommands,
    StrategyDaemonCommands, StrategyRequestCommands, StrategyServiceCommands,
    StrategyServiceConfigCommands, StrategySignalCommands, TaskCommands, TradeCommands,
    WatchlistCommands, WatchlistGroupCommands, WatchlistTagCommands,
};
pub(crate) use crate::ai::{DecisionEngine, LlmConfig};
pub(crate) use crate::ai::providers::OpenAICompatAdapter;
pub(crate) use crate::analysis::backtest::{BacktestConfig, BacktestEngine};
pub(crate) use crate::analysis::candle_patterns::{
    CandleInput, MarketBias, PatternConfig, ReferencePricePolicy, recognize_sequence,
};
pub(crate) use crate::analysis::polars_adapter::{PolarsCalculator, from_kline_vec};
pub(crate) use crate::bridge::client::BridgeHttpClient;
pub(crate) use crate::core::{CliRuntime, QuantixError, Result};
pub(crate) use crate::data::models::Kline;
pub(crate) use crate::db::clickhouse::ClickHouseClient;
pub(crate) use crate::execution::config::JsonExecutionConfigStore;
pub(crate) use crate::execution::daemon::{
    ExecutionDaemonIterationSummary, consume_next_pending_request_with_components,
};
pub(crate) use crate::execution::kernel::{
    ExecutionKernel, ExecutionRunRequest, FillDeltaApplier, KernelExecutionResult, RiskDecision,
    RiskEvaluator,
};
pub(crate) use crate::execution::mock_live::{MockLiveExecutionAdapter, SystemMockLiveClock};
pub(crate) use crate::execution::models::{
    ApprovalStatus, ExecutionPolicy, ExecutionRequestRecord, ExecutionRequestStatus,
    FillDeltaContext, FillDeltaResult, OrderIntent, OrderStatus, SignalStatus,
    StrategySignalRecord,
};
pub(crate) use crate::execution::paper::PaperExecutionAdapter;
pub(crate) use crate::execution::qmt_bridge::QmtBridgePreviewAdapter;
pub(crate) use crate::execution::runtime_store::StrategyRuntimeStore;
pub(crate) use crate::market::{
    BoardRankRow, BoardSortBy, BoardType, LeaderFilter, LeaderRow, MarketDataReader,
    MarketOverview, MarketSentimentSnapshot, MarketService, NorthFlowSnapshot,
};
pub(crate) use crate::monitor::storage::SqliteMonitorAlertStore;
pub(crate) use crate::monitor::{
    JsonMonitorConfigStore, JsonMonitorServiceConfigStore, MonitorAlertStore, MonitorConfig,
    MonitorEventFilter, MonitorEventRow, MonitorEventType, MonitorIterationOutput,
    MonitorQuoteReader, MonitorQuoteRow, MonitorRunMode, MonitorRunner, MonitorService,
    MonitorServiceConfig, MonitorServiceStatusSummary, MonitorUserServiceInstaller,
    MonitorWatchlistReader, MonitorWatchlistSnapshot, PriceAlert, PriceAlertKind,
};
pub(crate) use crate::risk::{
    BuyLockState, JsonRiskStore, PositionRiskRow, RiskAccountSnapshot, RiskLockStateSource,
    RiskLogEvent, RiskLogEventType, RiskRule, RiskService, RiskStatus,
};
pub(crate) use crate::screener::{
    DailyKlineLoader, PresetInvocation, RuleMatchDetail, ScreenRow, ScreenRunOptions, ScreenSortBy,
    ScreenUniverse, ScreenerService, parse_preset_invocation,
};
pub(crate) use crate::sources::TdxDayFile;
pub(crate) use crate::stop::{
    SqliteStopRuleStore, StopHistoryEvent, StopHistoryEventType, StopRule, StopRuleStore,
    StopRuleUpdate, StopService, StopStatusRow, StopTriggerKind, TriggeredStop,
};
pub(crate) use crate::strategy::daemon::StrategyBarLoadTelemetry;
pub(crate) use crate::strategy::runtime::{StrategyBarLoader, StrategyRuntime};
pub(crate) use crate::strategy::trait_def::Signal;
pub(crate) use crate::strategy::{
    FallbackStrategyBarLoader, JsonStrategyConfigStore, JsonStrategyServiceConfigStore,
    StrategyDaemonConfig, StrategyServiceConfig, StrategyServiceStatusSummary,
    StrategySignalDaemon, StrategyUserServiceInstaller,
};
pub(crate) use crate::tasks::{TaskScheduler, TaskTemplates};
pub(crate) use crate::trade::{
    CashSnapshot, InitAccountRequest, JsonPaperTradeStore, PaperTradeAccount, PaperTradeState,
    PaperTradeStore, TradeFeeRow, TradeHistoryRow, TradeOrderRequest, TradeOverview, TradePosition,
    TradePositionCurrentRow, TradeQuoteStatus, TradeRecord, TradeReportingService, TradeService,
};
pub(crate) use crate::watchlist::{
    PostgresWatchlistNameLookup, TdxWatchlistQuoteLookup, WatchlistDisplayRow,
    WatchlistHistoryEvent, WatchlistListItem, WatchlistQuoteLookup, WatchlistService,
    WatchlistStorage, WatchlistStore,
};
pub(crate) use crate::fundamental::{
    EastMoneyFundamentalProvider, FundamentalProvider,
};
pub(crate) use crate::fundamental::valuation::ValuationFetcher;
pub(crate) use crate::fundamental::earnings::EarningsFetcher;
pub(crate) use crate::fundamental::institution::InstitutionFetcher;
pub(crate) use crate::fundamental::dragon_tiger::DragonTigerFetcher;
pub(crate) use crate::market::sentiment::SentimentAggregator;
pub(crate) use crate::market::sentiment::types::SentimentTrend;
pub(crate) use async_trait::async_trait;
pub(crate) use chrono::{DateTime, NaiveDate, Utc};
pub(crate) use dialoguer::{Input, Select, theme::ColorfulTheme};
pub(crate) use indicatif::{ProgressBar, ProgressStyle};
pub(crate) use rust_decimal::Decimal;
pub(crate) use rust_decimal::prelude::ToPrimitive;
pub(crate) use rust_decimal_macros::dec;
pub(crate) use std::collections::{BTreeMap, HashMap};
pub(crate) use std::path::Path;
pub(crate) use std::str::FromStr;
pub(crate) use std::sync::Arc;
pub(crate) use std::time::Duration;

mod account;
mod algo;
mod anomaly;
mod ai;
mod fundamental;
mod import;
mod news;
mod notify;
mod risk;
mod sentiment;

pub use self::account::run_account_command;
pub use self::algo::run_algo_command;
pub use self::anomaly::run_anomaly_command;
pub use self::ai::run_ai_command;
pub use self::fundamental::run_fundamental_command;
pub use self::import::run_import_command;
pub use self::news::run_news_command;
pub use self::notify::run_notify_command;
pub use self::risk::run_risk_command;
pub use self::sentiment::run_sentiment_command;

async fn create_clickhouse_client() -> Result<ClickHouseClient> {
    let runtime = CliRuntime::load();
    ClickHouseClient::from_settings(&runtime.clickhouse).await
}

/// 初始化命令
pub async fn run_init(config_path: String) -> Result<()> {
    println!("🚀 初始化 Quantix CLI...");
    println!("📁 配置路径: {}", config_path);

    // 检查配置路径是否存在
    let path = Path::new(&config_path);
    if !path.exists() {
        println!("⚠️  警告: 配置路径不存在，将创建基本配置");
        std::fs::create_dir_all(path)
            .map_err(|e| QuantixError::Other(format!("创建配置目录失败: {}", e)))?;
    }

    // 初始化 Polars
    let _ = crate::analysis::polars_adapter::init_polars();

    println!("✅ 初始化完成！");
    println!("\n📝 下一步:");
    println!("  1. 配置数据库连接 (环境变量)");
    println!("  2. 运行 'quantix data query' 查询数据");
    println!("  3. 运行 'quantix strategy list' 查看策略");
    println!("  4. 运行 'quantix task start' 启动任务调度器");

    Ok(())
}

/// 交互式菜单（简单版）
pub async fn run_simple_menu() -> Result<()> {
    loop {
        println!("\n=== Quantix CLI 交互菜单 ===\n");

        let items = vec![
            "📊 数据同步",
            "📈 策略运行",
            "🔙 回测分析",
            "⏰ 任务管理",
            "💹 技术分析",
            "📤 数据导出",
            "❌ 退出",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .items(&items)
            .default(0)
            .interact()
            .map_err(|e| QuantixError::Other(format!("菜单选择失败: {}", e)))?;

        match selection {
            0 => run_data_sync_menu().await?,
            1 => run_strategy_menu().await?,
            2 => run_backtest_menu().await?,
            3 => run_task_menu().await?,
            4 => run_analysis_menu().await?,
            5 => run_export_menu().await?,
            6 => {
                println!("👋 再见！");
                break;
            }
            _ => {}
        }
    }
    Ok(())
}

/// TUI 菜单
pub async fn run_tui_menu() -> Result<()> {
    println!("🎨 TUI 菜单功能开发中...");
    // TODO: 实现 ratatui 菜单
    println!("💡 提示: 使用 'quantix menu' 进入简单菜单");
    Ok(())
}

/// 数据命令
pub async fn run_data_command(cmd: DataCommands) -> Result<()> {
    match cmd {
        DataCommands::Query {
            code,
            start,
            end,
            r#type,
            limit,
        } => {
            query_kline_data(code, start, end, r#type, limit).await?;
        }
        DataCommands::Export {
            code,
            format,
            output,
        } => {
            export_data(code, format, output).await?;
        }
    }
    Ok(())
}

/// 查询 K线数据
async fn query_kline_data(
    code: String,
    start: Option<String>,
    end: Option<String>,
    period_type: String,
    limit: usize,
) -> Result<()> {
    println!("📊 查询 K线数据");
    println!("  代码: {}", code);
    println!("  周期: {}", period_type);
    println!("  限制: {}", limit);

    // 解析日期
    let start_date = start
        .as_ref()
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y%m%d").ok());
    let end_date = end
        .as_ref()
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y%m%d").ok());

    // 连接 ClickHouse
    let client = create_clickhouse_client().await?;

    // 查询数据
    let klines = client
        .get_kline_data(&code, &period_type, start_date, end_date, Some(limit))
        .await?;

    if klines.is_empty() {
        println!("⚠️  未找到数据");
        return Ok(());
    }

    // 显示数据
    println!("\n📈 查询结果 (共 {} 条):", klines.len());
    println!(
        "{:<12} {:<10} {:<10} {:<10} {:<10} {:<12}",
        "日期", "开盘", "最高", "最低", "收盘", "成交量"
    );
    println!("{}", "-".repeat(70));

    for kline in klines.iter().take(20) {
        println!(
            "{:<12} {:<10.2} {:<10.2} {:<10.2} {:<10.2} {:<12}",
            kline.date, kline.open, kline.high, kline.low, kline.close, kline.volume,
        );
    }

    if klines.len() > 20 {
        println!("... (还有 {} 条)", klines.len() - 20);
    }

    Ok(())
}

/// 导出数据
async fn export_data(code: String, format: String, output: String) -> Result<()> {
    println!("📤 导出数据");
    println!("  代码: {}", code);
    println!("  格式: {}", format);
    println!("  输出: {}", output);

    // 创建输出目录
    std::fs::create_dir_all(&output)
        .map_err(|e| QuantixError::Other(format!("创建输出目录失败: {}", e)))?;

    // 连接 ClickHouse
    let client = create_clickhouse_client().await?;

    // 查询数据
    let klines = client
        .get_kline_data(
            &code,
            "1d",
            None,
            None,
            Some(10000), // 导出时使用较大的限制
        )
        .await?;

    if klines.is_empty() {
        println!("⚠️  未找到数据");
        return Ok(());
    }

    let output_path = format!("{}/{}.{}", output, code, format);
    let progress = ProgressBar::new(3);
    progress.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {msg}")
            .unwrap(),
    );

    progress.set_message("准备导出...");

    match format.as_str() {
        "csv" => {
            progress.set_message("写入 CSV...");
            progress.inc(1);

            let mut wtr = csv::Writer::from_path(&output_path)
                .map_err(|e| QuantixError::Other(format!("创建 CSV 文件失败: {}", e)))?;

            wtr.write_record(&["date", "open", "high", "low", "close", "volume"])
                .map_err(|e| QuantixError::Other(format!("写入 CSV 头失败: {}", e)))?;

            for kline in &klines {
                wtr.write_record(&[
                    kline.date.to_string(),
                    kline.open.to_string(),
                    kline.high.to_string(),
                    kline.low.to_string(),
                    kline.close.to_string(),
                    kline.volume.to_string(),
                ])
                .map_err(|e| QuantixError::Other(format!("写入 CSV 数据失败: {}", e)))?;
            }

            wtr.flush()
                .map_err(|e| QuantixError::Other(format!("刷新 CSV 失败: {}", e)))?;

            progress.inc(1);
            progress.finish_with_message("CSV 导出完成");
        }
        "parquet" => {
            progress.set_message("写入 Parquet...");
            progress.inc(1);

            // TODO: 实现 Parquet 导出
            progress.finish_with_message("Parquet 导出暂未实现");
        }
        _ => {
            return Err(QuantixError::Other(format!("不支持的格式: {}", format)));
        }
    }

    println!("✅ 数据已导出到: {}", output_path);
    Ok(())
}

/// 策略命令
pub async fn run_strategy_command(cmd: StrategyCommands) -> Result<()> {
    match cmd {
        StrategyCommands::Run { name, mode, code } => {
            run_strategy(name, mode, code).await?;
        }
        StrategyCommands::List => {
            list_strategies().await?;
        }
        StrategyCommands::Show { name } => {
            show_strategy(name).await?;
        }
        StrategyCommands::Config(subcommand) => match subcommand {
            StrategyConfigCommands::Init => {
                execute_strategy_config_init().await?;
            }
            StrategyConfigCommands::Show => {
                execute_strategy_config_show().await?;
            }
        },
        StrategyCommands::Daemon(subcommand) => match subcommand {
            StrategyDaemonCommands::Run { once } => {
                execute_strategy_daemon_run(once).await?;
            }
        },
        StrategyCommands::Signal(subcommand) => match subcommand {
            StrategySignalCommands::List {
                approval_status,
                signal_status,
                ..
            } => {
                execute_strategy_signal_list(approval_status.as_deref(), signal_status.as_deref())
                    .await?;
            }
            StrategySignalCommands::Approve {
                signal_id,
                target_mode,
                target_account,
            } => {
                execute_strategy_signal_approve(&signal_id, &target_mode, &target_account).await?;
            }
            StrategySignalCommands::Reject { signal_id, reason } => {
                execute_strategy_signal_reject(&signal_id, reason.as_deref()).await?;
            }
        },
        StrategyCommands::Request(subcommand) => match subcommand {
            StrategyRequestCommands::List {
                status,
                target_mode,
                target_account,
                limit,
                stats,
            } => {
                execute_strategy_request_list(status.as_deref(), target_mode.as_deref(), target_account.as_deref(), limit, stats).await?;
            }
            StrategyRequestCommands::Show {
                request_id,
                verbose,
            } => {
                execute_strategy_request_show(&request_id, verbose).await?;
            }
            StrategyRequestCommands::Execute { request_id } => {
                execute_strategy_request_execute(&request_id).await?;
            }
            StrategyRequestCommands::Cancel { request_id, reason } => {
                execute_strategy_request_cancel(&request_id, reason.as_deref()).await?;
            }
        },
        StrategyCommands::Service(subcommand) => match subcommand {
            StrategyServiceCommands::Install => {
                execute_strategy_service_command(StrategyServiceCommands::Install)?;
            }
            StrategyServiceCommands::Uninstall => {
                execute_strategy_service_command(StrategyServiceCommands::Uninstall)?;
            }
            StrategyServiceCommands::Start => {
                execute_strategy_service_command(StrategyServiceCommands::Start)?;
            }
            StrategyServiceCommands::Stop => {
                execute_strategy_service_command(StrategyServiceCommands::Stop)?;
            }
            StrategyServiceCommands::Status => {
                execute_strategy_service_command(StrategyServiceCommands::Status)?;
            }
            StrategyServiceCommands::Enable => {
                execute_strategy_service_command(StrategyServiceCommands::Enable)?;
            }
            StrategyServiceCommands::Disable => {
                execute_strategy_service_command(StrategyServiceCommands::Disable)?;
            }
        },
        StrategyCommands::ServiceConfig(subcommand) => match subcommand {
            StrategyServiceConfigCommands::Show => {
                let output = execute_strategy_service_config_command_with_store(
                    StrategyServiceConfigCommands::Show,
                    &JsonStrategyServiceConfigStore::with_default_path()?,
                )?;
                print_strategy_service_config_output(output)?;
            }
            StrategyServiceConfigCommands::Set {
                quantix_bin,
                env_file,
            } => {
                let output = execute_strategy_service_config_command_with_store(
                    StrategyServiceConfigCommands::Set {
                        quantix_bin,
                        env_file,
                    },
                    &JsonStrategyServiceConfigStore::with_default_path()?,
                )?;
                print_strategy_service_config_output(output)?;
            }
        },
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StrategyRunSummary {
    run_id: String,
    strategy_name: String,
    mode: String,
    symbol: String,
    signal: Signal,
    order_status: Option<OrderStatus>,
    message: String,
}

#[derive(Debug, Clone)]
struct StrategyRiskBridge<TradeStore, RiskStore> {
    trade_store: TradeStore,
    risk_service: RiskService<RiskStore>,
}

impl<TradeStore, RiskStore> StrategyRiskBridge<TradeStore, RiskStore> {
    fn new(trade_store: TradeStore, risk_service: RiskService<RiskStore>) -> Self {
        Self {
            trade_store,
            risk_service,
        }
    }
}

pub async fn run_execution_command(cmd: ExecutionCommands) -> Result<()> {
    match cmd {
        ExecutionCommands::Config(subcommand) => match subcommand {
            ExecutionConfigCommands::Init => {
                execute_execution_config_init().await?;
            }
            ExecutionConfigCommands::Show => {
                execute_execution_config_show().await?;
            }
        },
        ExecutionCommands::Daemon(subcommand) => match subcommand {
            ExecutionDaemonCommands::Run { once } => {
                execute_execution_daemon_run(once).await?;
            }
        },
        ExecutionCommands::Bridge(subcommand) => match subcommand {
            ExecutionBridgeCommands::Status => {
                execute_execution_bridge_status().await?;
            }
            ExecutionBridgeCommands::QmtPreview { request_id } => {
                execute_execution_bridge_qmt_preview(&request_id).await?;
            }
            ExecutionBridgeCommands::QmtLive { request_id, yes } => {
                execute_execution_bridge_qmt_live(&request_id, yes).await?;
            }
            ExecutionBridgeCommands::QmtQuery { order_id } => {
                execute_execution_bridge_qmt_query(&order_id).await?;
            }
            ExecutionBridgeCommands::QmtCancel { order_id } => {
                execute_execution_bridge_qmt_cancel(&order_id).await?;
            }
            ExecutionBridgeCommands::QmtAccount => {
                execute_execution_bridge_qmt_account().await?;
            }
            ExecutionBridgeCommands::QmtPositions => {
                execute_execution_bridge_qmt_positions().await?;
            }
            ExecutionBridgeCommands::QmtAsset => {
                execute_execution_bridge_qmt_asset().await?;
            }
        },
    }

    Ok(())
}

#[derive(Debug, Clone)]
struct StrategyFillDeltaBridge<TradeStore> {
    trade_service: TradeService<TradeStore>,
}

impl<TradeStore> StrategyFillDeltaBridge<TradeStore>
where
    TradeStore: PaperTradeStore,
{
    fn new(trade_store: TradeStore) -> Self {
        Self {
            trade_service: TradeService::new(trade_store),
        }
    }
}

#[async_trait]
impl<TradeStore> FillDeltaApplier for StrategyFillDeltaBridge<TradeStore>
where
    TradeStore: PaperTradeStore,
{
    async fn apply_fill_delta(&self, ctx: FillDeltaContext) -> Result<FillDeltaResult> {
        if ctx.new_filled_quantity <= ctx.old_filled_quantity {
            return Ok(FillDeltaResult {
                applied: false,
                delta_quantity: 0,
                trade_record_id: None,
            });
        }

        let fill_details = ctx.fill_details.ok_or_else(|| {
            QuantixError::Other(
                "strategy run --mode mock_live 增量成交缺少 fill_details".to_string(),
            )
        })?;

        let request = TradeOrderRequest::new(
            ctx.symbol.clone(),
            decimal_to_f64(fill_details.fill_price, "strategy run --mode mock_live")?,
            fill_details.fill_quantity,
        )
        .map_err(|err| remap_trade_request_error(err, "strategy run --mode mock_live"))?;

        let record = match ctx.side {
            crate::execution::models::OrderSide::Buy => {
                self.trade_service.buy(request, ctx.event_time).await?
            }
            crate::execution::models::OrderSide::Sell => {
                self.trade_service.sell(request, ctx.event_time).await?
            }
        };

        Ok(FillDeltaResult {
            applied: true,
            delta_quantity: fill_details.fill_quantity,
            trade_record_id: Some(record.id),
        })
    }
}

#[async_trait]
impl<TradeStore, RiskStore> RiskEvaluator for StrategyRiskBridge<TradeStore, RiskStore>
where
    TradeStore: PaperTradeStore,
    RiskStore: crate::risk::RiskStore,
{
    async fn evaluate(&self, intent: OrderIntent) -> Result<RiskDecision> {
        if intent.side == crate::execution::models::OrderSide::Sell {
            return Ok(RiskDecision::Allow);
        }

        let account = load_initialized_trade_account(&self.trade_store).await?;
        let snapshot = build_risk_account_snapshot(&account);
        let request = TradeOrderRequest::new(
            intent.symbol.clone(),
            decimal_to_f64(intent.requested_price, "strategy run --mode paper")?,
            intent.requested_quantity,
        )
        .map_err(|err| remap_trade_request_error(err, "strategy run --mode paper"))?;
        let projected_buy = build_projected_buy_impact(&account, &request);

        match self
            .risk_service
            .check_buy(&snapshot, &projected_buy, Utc::now())
            .await
        {
            Ok(()) => Ok(RiskDecision::Allow),
            Err(QuantixError::Other(reason)) => Ok(RiskDecision::Reject { reason }),
            Err(other) => Err(other),
        }
    }

    async fn sync_after_fill(&self) -> Result<()> {
        sync_risk_from_trade_store(&self.trade_store, &self.risk_service).await
    }
}

async fn execute_strategy_run_with_components<L, TS, RS>(
    name: &str,
    mode: &str,
    code: Option<String>,
    loader: L,
    trade_store: TS,
    risk_store: RS,
    runtime_store: &StrategyRuntimeStore,
) -> Result<StrategyRunSummary>
where
    L: StrategyBarLoader,
    TS: PaperTradeStore + Clone,
    RS: crate::risk::RiskStore + Clone,
{
    let risk_service = RiskService::new(risk_store.clone());
    execute_strategy_run_with_risk_service(
        name,
        mode,
        code,
        loader,
        trade_store,
        risk_service,
        runtime_store,
    )
    .await
}

async fn execute_strategy_run_with_risk_service<L, TS, RS>(
    name: &str,
    mode: &str,
    code: Option<String>,
    loader: L,
    trade_store: TS,
    risk_service: RiskService<RS>,
    runtime_store: &StrategyRuntimeStore,
) -> Result<StrategyRunSummary>
where
    L: StrategyBarLoader,
    TS: PaperTradeStore + Clone,
    RS: crate::risk::RiskStore,
{
    match mode {
        "live" => {
            return Err(QuantixError::Unsupported(
                "strategy live 模式尚未实现".to_string(),
            ));
        }
        "paper" | "mock_live" => {}
        other => {
            return Err(QuantixError::Unsupported(format!(
                "strategy {other} 模式尚未实现"
            )));
        }
    }

    if name != "ma_cross" {
        return Err(QuantixError::Other(format!("未知策略: {name}")));
    }

    let symbol = code.ok_or_else(|| {
        QuantixError::Other("strategy run --mode paper 需要显式指定 --code".to_string())
    })?;

    let account = load_initialized_trade_account(&trade_store).await?;
    let held_volume = account
        .positions
        .get(&symbol)
        .map(|position| position.volume);

    let bars = loader.load_daily_bars(&symbol, 10_000).await?;
    let latest_bar = bars.last().cloned().ok_or_else(|| {
        QuantixError::Other(format!("strategy paper 未找到 {} 的日线数据", symbol))
    })?;
    let bar_end = DateTime::<Utc>::from_naive_utc_and_offset(
        latest_bar.date.and_hms_opt(15, 0, 0).unwrap(),
        Utc,
    );

    let runtime = StrategyRuntime::new(&loader);
    let envelope = runtime.run_ma_cross_once(&symbol, 5, 10).await?;

    let risk = StrategyRiskBridge::new(trade_store.clone(), risk_service);
    let run_id = uuid::Uuid::new_v4().to_string();
    let client_order_id = format!("{run_id}_{symbol}_1");
    let run_request = ExecutionRunRequest {
        run_id: run_id.clone(),
        strategy_name: name.to_string(),
        mode: mode.to_string(),
        trigger: "once".to_string(),
        symbol: symbol.clone(),
        timeframe: "1d".to_string(),
        bar_end,
        market_price: latest_bar.close,
        held_volume,
        policy: ExecutionPolicy {
            fixed_cash_per_buy: dec!(10000),
            slippage_bps: 0,
        },
        client_order_id,
    };
    let result = match mode {
        "paper" => {
            let trade_service = TradeService::new(trade_store.clone());
            let adapter = PaperExecutionAdapter::new(trade_service);
            let kernel = ExecutionKernel::new(runtime_store.clone(), adapter, risk);
            kernel.execute_once(run_request, envelope.clone()).await?
        }
        "mock_live" => {
            let adapter = MockLiveExecutionAdapter::new(runtime_store.clone(), SystemMockLiveClock);
            let fill_delta = StrategyFillDeltaBridge::new(trade_store.clone());
            let kernel =
                ExecutionKernel::with_fill_delta(runtime_store.clone(), adapter, fill_delta, risk);
            kernel.execute_once(run_request, envelope.clone()).await?
        }
        _ => unreachable!("validated strategy mode"),
    };

    Ok(build_strategy_run_summary(
        name,
        mode,
        &symbol,
        result,
        envelope.signal,
    ))
}

fn build_strategy_run_summary(
    strategy_name: &str,
    mode: &str,
    symbol: &str,
    result: KernelExecutionResult,
    signal: Signal,
) -> StrategyRunSummary {
    let message = match result.order_status {
        Some(status) => format!(
            "signal={} order_status={}",
            signal_label(signal),
            order_status_label(status)
        ),
        None => format!("signal={} no_order", signal_label(signal)),
    };

    StrategyRunSummary {
        run_id: result.run_id,
        strategy_name: strategy_name.to_string(),
        mode: mode.to_string(),
        symbol: symbol.to_string(),
        signal,
        order_status: result.order_status,
        message,
    }
}

fn signal_label(signal: Signal) -> &'static str {
    match signal {
        Signal::Buy => "buy",
        Signal::Sell => "sell",
        Signal::Hold => "hold",
    }
}

fn order_status_label(status: OrderStatus) -> &'static str {
    status.as_str()
}

fn print_strategy_run_summary(summary: &StrategyRunSummary) {
    println!("🧾 运行 ID: {}", summary.run_id);
    println!("  策略: {}", summary.strategy_name);
    println!("  模式: {}", summary.mode);
    println!("  代码: {}", summary.symbol);
    println!("  信号: {}", signal_label(summary.signal));
    if let Some(status) = summary.order_status {
        println!("  订单状态: {}", order_status_label(status));
    }
    println!("  结果: {}", summary.message);
}

fn create_execution_config_store() -> JsonExecutionConfigStore {
    let runtime = CliRuntime::load();
    JsonExecutionConfigStore::new(runtime.execution_config_path)
}

pub fn create_bridge_client() -> Result<BridgeHttpClient> {
    let runtime = CliRuntime::load();
    BridgeHttpClient::new(runtime.bridge.base_url, runtime.bridge.api_key)
        .map_err(|err| QuantixError::Other(err.to_string()))
}

async fn execute_execution_config_init() -> Result<()> {
    let config = create_execution_config_store().load_or_create()?;
    println!("{}", serde_json::to_string_pretty(&config)?);
    Ok(())
}

async fn execute_execution_config_show() -> Result<()> {
    let config = create_execution_config_store().load_or_create()?;
    println!("{}", serde_json::to_string_pretty(&config)?);
    Ok(())
}

async fn execute_execution_daemon_run(once: bool) -> Result<()> {
    let runtime = CliRuntime::load();
    let runtime_store = StrategyRuntimeStore::new(runtime.strategy_runtime_db_path).await?;
    let config_store = JsonExecutionConfigStore::new(runtime.execution_config_path);
    let config = config_store.load_or_create()?;
    let trade_store = create_trade_store();
    let risk_store = create_risk_store();

    if once {
        let summary =
            consume_next_pending_request_with_components(&runtime_store, trade_store, risk_store)
                .await?;
        print_execution_daemon_summary(&summary);
        return Ok(());
    }

    loop {
        let summary = consume_next_pending_request_with_components(
            &runtime_store,
            trade_store.clone(),
            risk_store.clone(),
        )
        .await?;
        print_execution_daemon_summary(&summary);
        tokio::time::sleep(Duration::from_secs(config.poll_interval_secs)).await;
    }
}

async fn execute_execution_bridge_status() -> Result<()> {
    let capabilities = create_bridge_client()?.capabilities().await.map_err(|err| {
        QuantixError::Other(format!("bridge status 查询失败: {err}"))
    })?;

    println!("{}", serde_json::to_string_pretty(&serde_json::json!({
        "tdx": {
            "enabled": capabilities.tdx.enabled,
            "supports": capabilities.tdx.supports
        },
        "qmt": {
            "enabled": capabilities.qmt.enabled,
            "mode": capabilities.qmt.mode,
            "supports": capabilities.qmt.supports
        }
    }))?);
    Ok(())
}

async fn execute_execution_bridge_qmt_preview(request_id: &str) -> Result<()> {
    let runtime = CliRuntime::load();
    let runtime_store = StrategyRuntimeStore::new(runtime.strategy_runtime_db_path).await?;
    let request = runtime_store
        .get_execution_request(request_id)
        .await?
        .ok_or_else(|| QuantixError::Other(format!("request 不存在: {request_id}")))?;

    let adapter = QmtBridgePreviewAdapter::new(create_bridge_client()?);
    let preview = adapter.preview_request(&request).await?;

    println!("{}", serde_json::to_string_pretty(&serde_json::json!({
        "request_id": request_id,
        "adapter_order_id": preview.adapter_order_id,
        "latest_status": preview.latest_status.as_str(),
        "filled_quantity": preview.filled_quantity,
        "rejection_reason": preview.rejection_reason,
    }))?);
    Ok(())
}

async fn execute_execution_bridge_qmt_live(request_id: &str, skip_confirm: bool) -> Result<()> {
    let runtime = CliRuntime::load();
    let runtime_store = StrategyRuntimeStore::new(runtime.strategy_runtime_db_path).await?;
    let request = runtime_store
        .get_execution_request(request_id)
        .await?
        .ok_or_else(|| QuantixError::Other(format!("request 不存在: {request_id}")))?;

    // 从 payload_json 提取订单信息
    let snapshot = request
        .payload_json
        .get("execution_snapshot")
        .ok_or_else(|| QuantixError::Other("request 缺少 execution_snapshot".to_string()))?;
    let order_intent = snapshot
        .get("order_intent")
        .ok_or_else(|| QuantixError::Other("request 缺少 order_intent".to_string()))?;

    let symbol = snapshot
        .get("symbol")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let side = order_intent
        .get("side")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let quantity = order_intent
        .get("requested_quantity")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let price = order_intent
        .get("requested_price")
        .and_then(|v| v.as_str())
        .unwrap_or("0");
    let strategy_name = snapshot
        .get("strategy_name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    // 显示订单信息
    println!("═══════════════════════════════════════════════════════════════════");
    println!("⚠️  实盘下单确认");
    println!("═══════════════════════════════════════════════════════════════════");
    println!();
    println!("  股票代码:    {}", symbol);
    println!("  买卖方向:    {}", side);
    println!("  数量:        {} 股", quantity);
    println!("  价格:        {}", price);
    println!("  策略名称:    {}", strategy_name);
    println!();

    // 确认提示
    if !skip_confirm {
        println!("⚠️  警告: 这将提交真实订单到券商账户!");
        println!("    输入 'YES' 确认下单，其他任意键取消:");

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if input.trim() != "YES" {
            println!("已取消下单");
            return Ok(());
        }
    }

    // 构建订单请求
    let order_type = order_intent
        .get("order_type")
        .and_then(|v| v.as_str())
        .unwrap_or("limit");

    // 提交订单
    let client = create_bridge_client()?;
    let order_request = crate::bridge::models::BridgeQmtOrderRequest {
        request_id: request_id.to_string(),
        client_order_id: request.request_id.clone(),
        symbol: normalize_symbol_for_bridge(symbol),
        side: side.to_lowercase(),
        quantity,
        price: price.to_string(),
        order_type: order_type.to_string(),
        strategy_name: Some(strategy_name.to_string()),
        order_remark: Some("quantix-cli".to_string()),
        snapshot_metadata: None,
    };

    let response = client.qmt_submit_order(&order_request).await
        .map_err(|e| QuantixError::Other(e.to_string()))?;

    println!();
    println!("✓ 订单已提交");
    println!("{}", serde_json::to_string_pretty(&serde_json::json!({
        "request_id": request_id,
        "adapter_order_id": response.adapter_order_id,
        "latest_status": response.latest_status,
        "filled_quantity": response.filled_quantity,
        "avg_fill_price": response.avg_fill_price,
        "rejection_reason": response.rejection_reason,
    }))?);

    // 更新请求状态
    if response.latest_status == "rejected" {
        println!();
    } else {
        println!();
        println!("查询订单状态: quantix execution bridge qmt-query --order-id {}", response.adapter_order_id);
    }

    Ok(())
}

fn normalize_symbol_for_bridge(symbol: &str) -> String {
    if symbol.contains('.') {
        return symbol.to_string();
    }
    if symbol.starts_with('6') {
        format!("{symbol}.SH")
    } else {
        format!("{symbol}.SZ")
    }
}

async fn execute_execution_bridge_qmt_query(order_id: &str) -> Result<()> {
    let client = create_bridge_client()?;
    let response = client.qmt_query_order(order_id).await
        .map_err(|e| QuantixError::Other(e.to_string()))?;

    println!("{}", serde_json::to_string_pretty(&serde_json::json!({
        "adapter_order_id": response.adapter_order_id,
        "latest_status": response.latest_status,
        "filled_quantity": response.filled_quantity,
        "avg_fill_price": response.avg_fill_price,
    }))?);

    Ok(())
}

async fn execute_execution_bridge_qmt_cancel(order_id: &str) -> Result<()> {
    println!("⚠️  确认撤销订单: {}", order_id);
    println!("    输入 'YES' 确认撤单，其他任意键取消:");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    if input.trim() != "YES" {
        println!("已取消撤单");
        return Ok(());
    }

    let client = create_bridge_client()?;
    let response = client.qmt_cancel_order(order_id).await
        .map_err(|e| QuantixError::Other(e.to_string()))?;

    if response.success {
        println!("✓ 撤单成功: {}", response.order_id);
    } else {
        println!("✗ 撤单失败: {}", response.error_message.unwrap_or_else(|| "未知错误".to_string()));
    }

    Ok(())
}

async fn execute_execution_bridge_qmt_account() -> Result<()> {
    let client = create_bridge_client()?;
    let response = client.qmt_account_status().await
        .map_err(|e| QuantixError::Other(e.to_string()))?;

    println!("账户状态");
    println!("─────────────────────────────────────");
    println!("  适配器:      {}", response.adapter);
    println!("  模式:        {}", response.mode);
    println!("  SDK 可用:   {}", response.sdk_available);
    println!("  连接状态:    {}", if response.connected { "已连接" } else { "未连接" });
    if let Some(account) = response.account_masked {
        println!("  账户:        {}", account);
    }

    Ok(())
}

async fn execute_execution_bridge_qmt_positions() -> Result<()> {
    let client = create_bridge_client()?;
    let positions = client.qmt_positions().await
        .map_err(|e| QuantixError::Other(e.to_string()))?;

    if positions.is_empty() {
        println!("当前无持仓");
        return Ok(());
    }

    println!("持仓列表");
    println!("─────────────────────────────────────────────────────────────────────────────────");
    println!("{:<12} {:<10} {:<12} {:<12} {:<12}",
        "股票代码", "持仓", "可用", "成本价", "市值"
    );
    println!("{}", "-".repeat(76));

    for pos in positions {
        println!("{:<12} {:<10} {:<12} {:<12} {:<12}",
            pos.symbol,
            pos.volume,
            pos.available,
            pos.cost_price.as_ref().map(|s| s.as_str()).unwrap_or("-"),
            pos.market_value.as_ref().map(|s| s.as_str()).unwrap_or("-")
        );
    }

    Ok(())
}

async fn execute_execution_bridge_qmt_asset() -> Result<()> {
    let client = create_bridge_client()?;
    let asset = client.qmt_asset().await
        .map_err(|e| QuantixError::Other(e.to_string()))?;

    println!("资产信息");
    println!("─────────────────────────────────────");
    println!("  总资产:      {}", asset.total_asset);
    println!("  可用现金:    {}", asset.cash);
    println!("  持仓市值:    {}", asset.market_value);
    println!("  账户 ID:    {}", asset.account_id);

    Ok(())
}

fn print_execution_daemon_summary(summary: &ExecutionDaemonIterationSummary) {
    if summary.claimed == 0 {
        println!("execution daemon 未找到 pending request");
        return;
    }

    if summary.failed > 0 {
        println!("execution daemon consumed request status=failed");
    } else {
        println!("execution daemon consumed request status=completed");
    }
}

fn create_strategy_config_store() -> JsonStrategyConfigStore {
    let runtime = CliRuntime::load();
    JsonStrategyConfigStore::new(runtime.strategy_config_path)
}

async fn execute_strategy_config_init() -> Result<()> {
    let config = execute_strategy_config_init_to_store(&create_strategy_config_store())?;
    println!("{}", serde_json::to_string_pretty(&config)?);
    Ok(())
}

fn execute_strategy_config_init_to_store(
    store: &JsonStrategyConfigStore,
) -> Result<StrategyDaemonConfig> {
    store.load_or_create()
}

async fn execute_strategy_config_show() -> Result<()> {
    let config = execute_strategy_config_show_from_store(&create_strategy_config_store())?;
    println!("{}", serde_json::to_string_pretty(&config)?);
    Ok(())
}

fn execute_strategy_config_show_from_store(
    store: &JsonStrategyConfigStore,
) -> Result<StrategyDaemonConfig> {
    store.load_or_create()
}

async fn execute_strategy_daemon_run(once: bool) -> Result<()> {
    let runtime = CliRuntime::load();
    let runtime_store = StrategyRuntimeStore::new(runtime.strategy_runtime_db_path).await?;
    let config_store = JsonStrategyConfigStore::new(runtime.strategy_config_path);
    let loader = FallbackStrategyBarLoader::from_env_with_primary_source_id(
        ClickHouseDailyKlineLoader::new(create_clickhouse_client().await?),
        "clickhouse-storage",
    );

    if once {
        match execute_strategy_daemon_run_once_with_components(
            loader,
            &config_store,
            &runtime_store,
        )
        .await?
        {
            Some(signal) => println!("{}", format_strategy_signal_row(&signal)),
            None => println!("strategy daemon 未生成新信号"),
        }
        return Ok(());
    }

    let mut daemon = StrategySignalDaemon::new(loader, runtime_store, config_store)?;
    loop {
        daemon.run_once().await?;
        tokio::time::sleep(Duration::from_secs(daemon.check_interval_secs())).await;
    }
}

async fn execute_strategy_daemon_run_once_with_components<L>(
    loader: L,
    config_store: &JsonStrategyConfigStore,
    runtime_store: &StrategyRuntimeStore,
) -> Result<Option<StrategySignalRecord>>
where
    L: StrategyBarLoader + StrategyBarLoadTelemetry,
{
    let before_ids: Vec<String> = runtime_store
        .list_signals()
        .await?
        .into_iter()
        .map(|row| row.signal_id)
        .collect();
    let mut daemon =
        StrategySignalDaemon::new(loader, runtime_store.clone(), config_store.clone())?;
    daemon.run_once().await?;

    let latest = runtime_store
        .list_signals()
        .await?
        .into_iter()
        .find(|row| !before_ids.iter().any(|id| id == &row.signal_id));

    Ok(latest)
}

async fn execute_strategy_signal_list(
    approval_status: Option<&str>,
    signal_status: Option<&str>,
) -> Result<()> {
    let runtime = CliRuntime::load();
    let runtime_store = StrategyRuntimeStore::new(runtime.strategy_runtime_db_path).await?;
    let rows =
        execute_strategy_signal_list_with_store(&runtime_store, approval_status, signal_status)
            .await?;

    for row in rows {
        println!("{}", format_strategy_signal_row(&row));
    }

    Ok(())
}

fn format_strategy_signal_row(row: &StrategySignalRecord) -> String {
    let source_id = row
        .metadata_json
        .get("bar_source_id")
        .and_then(|value| value.as_str())
        .unwrap_or("unknown");
    let fallback = row
        .metadata_json
        .get("bar_source_fallback")
        .and_then(|value| value.as_bool())
        .map(|value| value.to_string())
        .unwrap_or_else(|| "unknown".to_string());

    format!(
        "{} {} {} {} {} {} bar_end={} source={} fallback={}",
        row.signal_id,
        row.strategy_instance_id,
        row.symbol,
        row.signal_value,
        row.signal_status.as_str(),
        row.approval_status.as_str(),
        row.bar_end.format("%Y-%m-%dT%H:%M:%SZ"),
        source_id,
        fallback
    )
}

async fn execute_strategy_signal_list_with_store(
    store: &StrategyRuntimeStore,
    approval_status: Option<&str>,
    signal_status: Option<&str>,
) -> Result<Vec<StrategySignalRecord>> {
    let approval_filter = approval_status.map(parse_approval_status).transpose()?;
    let signal_filter = signal_status.map(parse_signal_status).transpose()?;

    let rows = store.list_signals().await?;
    Ok(rows
        .into_iter()
        .filter(|row| approval_filter.is_none_or(|status| row.approval_status == status))
        .filter(|row| signal_filter.is_none_or(|status| row.signal_status == status))
        .collect())
}

async fn execute_strategy_signal_approve(
    signal_id: &str,
    target_mode: &str,
    target_account: &str,
) -> Result<()> {
    let runtime = CliRuntime::load();
    let runtime_store = StrategyRuntimeStore::new(runtime.strategy_runtime_db_path).await?;
    let request = execute_strategy_signal_approve_with_store(
        &runtime_store,
        signal_id,
        target_mode,
        target_account,
    )
    .await?;
    println!("{}", format_strategy_approval_result(&request));
    Ok(())
}

async fn execute_strategy_signal_approve_with_store(
    store: &StrategyRuntimeStore,
    signal_id: &str,
    target_mode: &str,
    target_account: &str,
) -> Result<ExecutionRequestRecord> {
    store
        .approve_signal_and_create_request(signal_id, target_mode, target_account, Some("cli"))
        .await
}

async fn execute_strategy_signal_reject(signal_id: &str, reason: Option<&str>) -> Result<()> {
    let runtime = CliRuntime::load();
    let runtime_store = StrategyRuntimeStore::new(runtime.strategy_runtime_db_path).await?;
    let signal =
        execute_strategy_signal_reject_with_store(&runtime_store, signal_id, reason).await?;
    println!("{}", format_strategy_rejection_result(&signal));
    Ok(())
}

async fn execute_strategy_signal_reject_with_store(
    store: &StrategyRuntimeStore,
    signal_id: &str,
    reason: Option<&str>,
) -> Result<StrategySignalRecord> {
    store.reject_signal(signal_id, reason).await?;
    store
        .get_signal(signal_id)
        .await?
        .ok_or_else(|| QuantixError::Other(format!("signal 不存在: {signal_id}")))
}

async fn execute_strategy_request_list(
    status: Option<&str>,
    target_mode: Option<&str>,
    target_account: Option<&str>,
    limit: usize,
    stats: bool,
) -> Result<()> {
    let runtime = CliRuntime::load();
    let runtime_store = StrategyRuntimeStore::new(runtime.strategy_runtime_db_path).await?;
    let rows = execute_strategy_request_list_with_store(&runtime_store, status).await?;

    // Apply additional filters
    let mut filtered: Vec<_> = rows
        .into_iter()
        .filter(|row| {
            let mode_match = target_mode.map_or(true, |m| row.target_mode == m);
            let account_match = target_account.map_or(true, |a| row.target_account == a);
            mode_match && account_match
        })
        .collect();

    // Show stats summary if requested
    if stats {
        let total = filtered.len();
        let pending = filtered
            .iter()
            .filter(|r| r.request_status == ExecutionRequestStatus::Pending)
            .count();
        let in_progress = filtered
            .iter()
            .filter(|r| r.request_status == ExecutionRequestStatus::InProgress)
            .count();
        let completed = filtered
            .iter()
            .filter(|r| r.request_status == ExecutionRequestStatus::Completed)
            .count();
        let failed = filtered
            .iter()
            .filter(|r| r.request_status == ExecutionRequestStatus::Failed)
            .count();
        let canceled = filtered
            .iter()
            .filter(|r| r.request_status == ExecutionRequestStatus::Canceled)
            .count();

        println!("=== Execution Request Statistics ===");
        println!("Total: {} | Pending: {} | InProgress: {} | Completed: {} | Failed: {} | Canceled: {}",
            total, pending, in_progress, completed, failed, canceled);
        println!();
    }

    // Apply limit
    filtered.truncate(limit);

    for row in filtered {
        println!("{}", format_strategy_request_row(&row));
    }

    Ok(())
}

async fn execute_strategy_request_show(request_id: &str, verbose: bool) -> Result<()> {
    let runtime = CliRuntime::load();
    let store = StrategyRuntimeStore::new(runtime.strategy_runtime_db_path).await?;

    let request = store
        .get_execution_request(request_id)
        .await?
        .ok_or_else(|| QuantixError::Other(format!("request 不存在: {request_id}")))?;

    println!("{}", format_strategy_request_detail(&request, verbose));

    // Show related order if exists
    if let Some(client_order_id) = request
        .payload_json
        .get("execution_result")
        .and_then(|r| r.get("client_order_id"))
        .and_then(|v| v.as_str())
    {
        if let Some(order) = store.find_order_by_client_order_id(client_order_id).await? {
            println!();
            println!("=== Related Order ===");
            println!("order_id: {}", order.order_id);
            println!("symbol: {}", order.symbol);
            println!("status: {}", order.status.as_str());
            println!("filled: {}/{}", order.filled_quantity, order.requested_quantity);
            if let Some(avg_price) = order.avg_fill_price {
                println!("avg_fill_price: {}", avg_price);
            }
        }
    }

    Ok(())
}

fn format_strategy_request_detail(request: &ExecutionRequestRecord, verbose: bool) -> String {
    let mut lines = vec![
        "=== Execution Request Detail ===".to_string(),
        format!("request_id: {}", request.request_id),
        format!("signal_id: {}", request.signal_id),
        format!("target_mode: {}", request.target_mode),
        format!("target_account: {}", request.target_account),
        format!("status: {}", request.request_status.as_str()),
        format!("approved_by: {}", request.approved_by.as_deref().unwrap_or("-")),
        format!("created_at: {}", request.created_at.format("%Y-%m-%dT%H:%M:%SZ")),
        format!("updated_at: {}", request.updated_at.format("%Y-%m-%dT%H:%M:%SZ")),
    ];

    // Execution snapshot summary
    if let Some(snapshot) = request.payload_json.get("execution_snapshot") {
        lines.push(String::new());
        lines.push("=== Execution Snapshot ===".to_string());
        if let Some(symbol) = snapshot.get("symbol").and_then(|v| v.as_str()) {
            lines.push(format!("symbol: {}", symbol));
        }
        if let Some(signal_value) = snapshot.get("signal_value").and_then(|v| v.as_str()) {
            lines.push(format!("signal: {}", signal_value));
        }
        if let Some(intent) = snapshot.get("order_intent") {
            if let Some(side) = intent.get("side").and_then(|v| v.as_str()) {
                lines.push(format!("side: {}", side));
            }
            if let Some(qty) = intent.get("requested_quantity").and_then(|v| v.as_i64()) {
                lines.push(format!("quantity: {}", qty));
            }
            if let Some(price) = intent.get("requested_price").and_then(|v| v.as_str()) {
                lines.push(format!("price: {}", price));
            }
        }
    }

    // Execution result
    if let Some(result) = request.payload_json.get("execution_result") {
        lines.push(String::new());
        lines.push("=== Execution Result ===".to_string());
        if let Some(run_id) = result.get("run_id").and_then(|v| v.as_str()) {
            lines.push(format!("run_id: {}", run_id));
        }
        if let Some(client_order_id) = result.get("client_order_id").and_then(|v| v.as_str()) {
            lines.push(format!("client_order_id: {}", client_order_id));
        }
        if let Some(order_status) = result.get("order_status").and_then(|v| v.as_str()) {
            lines.push(format!("order_status: {}", order_status));
        }
        if let Some(executed_at) = result.get("executed_at").and_then(|v| v.as_str()) {
            lines.push(format!("executed_at: {}", executed_at));
        }
    }

    // Execution error
    if let Some(error) = request.payload_json.get("execution_error") {
        lines.push(String::new());
        lines.push("=== Execution Error ===".to_string());
        if let Some(message) = error.get("message").and_then(|v| v.as_str()) {
            lines.push(format!("message: {}", message));
        }
        if let Some(failed_at) = error.get("failed_at").and_then(|v| v.as_str()) {
            lines.push(format!("failed_at: {}", failed_at));
        }
    }

    // Cancellation
    if let Some(cancellation) = request.payload_json.get("cancellation") {
        lines.push(String::new());
        lines.push("=== Cancellation ===".to_string());
        if let Some(reason) = cancellation.get("reason").and_then(|v| v.as_str()) {
            lines.push(format!("reason: {}", reason));
        }
        if let Some(canceled_at) = cancellation.get("canceled_at").and_then(|v| v.as_str()) {
            lines.push(format!("canceled_at: {}", canceled_at));
        }
    }

    // Full payload in verbose mode
    if verbose {
        lines.push(String::new());
        lines.push("=== Full Payload (verbose) ===".to_string());
        lines.push(
            serde_json::to_string_pretty(&request.payload_json)
                .unwrap_or_else(|_| "<serialize error>".to_string()),
        );
    }

    lines.join("\n")
}

async fn execute_strategy_request_execute(request_id: &str) -> Result<()> {
    let runtime = CliRuntime::load();
    let runtime_store = StrategyRuntimeStore::new(runtime.strategy_runtime_db_path).await?;
    let request = execute_strategy_request_execute_with_components(
        &runtime_store,
        request_id,
        create_trade_store(),
        create_risk_store(),
    )
    .await?;
    println!("{}", format_strategy_request_row(&request));
    Ok(())
}

async fn execute_strategy_request_execute_with_components<TS, RS>(
    store: &StrategyRuntimeStore,
    request_id: &str,
    trade_store: TS,
    risk_store: RS,
) -> Result<ExecutionRequestRecord>
where
    TS: PaperTradeStore + Clone,
    RS: crate::risk::RiskStore + Clone,
{
    crate::execution::daemon::execute_request_by_id_with_components(
        store,
        request_id,
        trade_store,
        risk_store,
    )
    .await
}

async fn execute_strategy_request_cancel(request_id: &str, reason: Option<&str>) -> Result<()> {
    let runtime = CliRuntime::load();
    let runtime_store = StrategyRuntimeStore::new(runtime.strategy_runtime_db_path).await?;
    let request =
        execute_strategy_request_cancel_with_store(&runtime_store, request_id, reason).await?;
    println!("{}", format_strategy_request_row(&request));
    Ok(())
}

async fn execute_strategy_request_cancel_with_store(
    store: &StrategyRuntimeStore,
    request_id: &str,
    reason: Option<&str>,
) -> Result<ExecutionRequestRecord> {
    let request = store
        .get_execution_request(request_id)
        .await?
        .ok_or_else(|| QuantixError::Other(format!("request 不存在: {request_id}")))?;
    if request.request_status != ExecutionRequestStatus::Pending {
        return Err(QuantixError::Other(format!(
            "request 不是 pending: {request_id}"
        )));
    }

    let payload_json = merge_execution_request_payload(
        &request.payload_json,
        "cancellation",
        serde_json::json!({
            "canceled_at": Utc::now().to_rfc3339(),
            "reason": reason.unwrap_or("manual cancel"),
        }),
    );
    let updated = store
        .try_cancel_execution_request(&request.request_id, payload_json, Utc::now())
        .await?;
    if !updated {
        return Err(QuantixError::Other(format!(
            "request 状态已变化: {}",
            request.request_id
        )));
    }
    store
        .get_execution_request(&request.request_id)
        .await?
        .ok_or_else(|| QuantixError::Other(format!("request 不存在: {}", request.request_id)))
}

fn format_strategy_approval_result(request: &ExecutionRequestRecord) -> String {
    format!(
        "{} signal={} target={}/{} status={}",
        request.request_id,
        request.signal_id,
        request.target_mode,
        request.target_account,
        request.request_status.as_str()
    )
}

fn format_strategy_rejection_result(signal: &StrategySignalRecord) -> String {
    let reason = signal
        .metadata_json
        .get("rejection_reason")
        .and_then(|value| value.as_str())
        .unwrap_or("-");

    format!(
        "{} signal_status={} approval_status={} reason={}",
        signal.signal_id,
        signal.signal_status.as_str(),
        signal.approval_status.as_str(),
        reason
    )
}

fn format_strategy_request_row(row: &ExecutionRequestRecord) -> String {
    let result = row
        .payload_json
        .get("execution_result")
        .and_then(|value| {
            let order_status = value.get("order_status").and_then(|item| item.as_str())?;
            let client_order_id = value
                .get("client_order_id")
                .and_then(|item| item.as_str())
                .unwrap_or("-");
            Some(format!(
                " result=order_status={} client_order_id={}",
                order_status, client_order_id
            ))
        })
        .or_else(|| {
            row.payload_json.get("execution_error").and_then(|value| {
                let message = value.get("message").and_then(|item| item.as_str())?;
                Some(format!(" result=error={message}"))
            })
        })
        .or_else(|| {
            row.payload_json.get("cancellation").and_then(|value| {
                let reason = value.get("reason").and_then(|item| item.as_str())?;
                Some(format!(" result=reason={reason}"))
            })
        })
        .unwrap_or_default();

    format!(
        "{} signal={} target={}/{} status={}{} created_at={}",
        row.request_id,
        row.signal_id,
        row.target_mode,
        row.target_account,
        row.request_status.as_str(),
        result,
        row.created_at.format("%Y-%m-%dT%H:%M:%SZ")
    )
}

async fn execute_strategy_request_list_with_store(
    store: &StrategyRuntimeStore,
    status: Option<&str>,
) -> Result<Vec<ExecutionRequestRecord>> {
    let status_filter = status.map(parse_execution_request_status).transpose()?;
    store.list_execution_requests(status_filter).await
}

fn parse_approval_status(value: &str) -> Result<ApprovalStatus> {
    ApprovalStatus::from_str(value)
        .ok_or_else(|| QuantixError::Other(format!("未知 approval_status: {value}")))
}

fn parse_signal_status(value: &str) -> Result<SignalStatus> {
    SignalStatus::from_str(value)
        .ok_or_else(|| QuantixError::Other(format!("未知 signal_status: {value}")))
}

fn parse_execution_request_status(value: &str) -> Result<ExecutionRequestStatus> {
    ExecutionRequestStatus::from_str(value)
        .ok_or_else(|| QuantixError::Other(format!("未知 request_status: {value}")))
}

fn merge_execution_request_payload(
    original: &serde_json::Value,
    key: &str,
    value: serde_json::Value,
) -> serde_json::Value {
    let mut payload = match original {
        serde_json::Value::Object(map) => serde_json::Value::Object(map.clone()),
        _ => serde_json::json!({}),
    };
    payload[key] = value;
    payload
}

fn execute_strategy_service_config_command_with_store(
    cmd: StrategyServiceConfigCommands,
    store: &JsonStrategyServiceConfigStore,
) -> Result<Option<StrategyServiceConfig>> {
    match cmd {
        StrategyServiceConfigCommands::Show => match store.load() {
            Ok(config) => Ok(Some(config)),
            Err(QuantixError::Config(_)) => Ok(None),
            Err(other) => Err(other),
        },
        StrategyServiceConfigCommands::Set {
            quantix_bin,
            env_file,
        } => {
            let config = StrategyServiceConfig {
                quantix_bin_path: quantix_bin.into(),
                environment_file_path: env_file.map(Into::into),
            };
            JsonStrategyServiceConfigStore::validate(&config)?;
            store.save(&config)?;
            Ok(Some(config))
        }
    }
}

fn print_strategy_service_config_output(config: Option<StrategyServiceConfig>) -> Result<()> {
    match config {
        Some(config) => {
            println!("{}", serde_json::to_string_pretty(&config)?);
        }
        None => {
            println!(
                "strategy service 未配置，请先运行 strategy service-config set --quantix-bin /abs/path/to/quantix"
            );
        }
    }

    Ok(())
}

trait StrategyServiceInstallerOps {
    fn install(&self) -> Result<()>;
    fn uninstall(&self) -> Result<()>;
    fn start(&self) -> Result<()>;
    fn stop(&self) -> Result<()>;
    fn enable(&self) -> Result<()>;
    fn disable(&self) -> Result<()>;
    fn status(&self) -> Result<String>;
    fn status_summary(&self) -> Result<StrategyServiceStatusSummary>;
}

impl StrategyServiceInstallerOps for StrategyUserServiceInstaller {
    fn install(&self) -> Result<()> {
        StrategyUserServiceInstaller::install(self)
    }

    fn uninstall(&self) -> Result<()> {
        StrategyUserServiceInstaller::uninstall(self)
    }

    fn start(&self) -> Result<()> {
        StrategyUserServiceInstaller::start(self)
    }

    fn stop(&self) -> Result<()> {
        StrategyUserServiceInstaller::stop(self)
    }

    fn enable(&self) -> Result<()> {
        StrategyUserServiceInstaller::enable(self)
    }

    fn disable(&self) -> Result<()> {
        StrategyUserServiceInstaller::disable(self)
    }

    fn status(&self) -> Result<String> {
        StrategyUserServiceInstaller::status(self)
    }

    fn status_summary(&self) -> Result<StrategyServiceStatusSummary> {
        StrategyUserServiceInstaller::status_summary(self)
    }
}

fn execute_strategy_service_command(cmd: StrategyServiceCommands) -> Result<()> {
    let runtime = CliRuntime::load();
    let store = JsonStrategyServiceConfigStore::with_default_path()?;
    let service_config = match store.load() {
        Ok(config) => config,
        Err(QuantixError::Config(_)) => {
            return Err(QuantixError::Other(
                "strategy service 未配置，请先运行 strategy service-config set --quantix-bin /abs/path/to/quantix".to_string(),
            ));
        }
        Err(other) => return Err(other),
    };
    let installer = StrategyUserServiceInstaller::new(runtime, service_config);
    let message = execute_strategy_service_command_with_installer(cmd, &installer)?;
    println!("{}", message);
    Ok(())
}

fn execute_strategy_service_command_with_installer<I>(
    cmd: StrategyServiceCommands,
    installer: &I,
) -> Result<String>
where
    I: StrategyServiceInstallerOps,
{
    match cmd {
        StrategyServiceCommands::Install => {
            installer.install()?;
            Ok("strategy service installed".to_string())
        }
        StrategyServiceCommands::Uninstall => {
            installer.uninstall()?;
            Ok("strategy service uninstalled".to_string())
        }
        StrategyServiceCommands::Start => {
            installer.start()?;
            Ok("strategy service started".to_string())
        }
        StrategyServiceCommands::Stop => {
            installer.stop()?;
            Ok("strategy service stopped".to_string())
        }
        StrategyServiceCommands::Enable => {
            installer.enable()?;
            Ok("strategy service enabled".to_string())
        }
        StrategyServiceCommands::Disable => {
            installer.disable()?;
            Ok("strategy service disabled".to_string())
        }
        StrategyServiceCommands::Status => installer.status(),
    }
}

/// 运行策略
async fn run_strategy(name: String, mode: String, code: Option<String>) -> Result<()> {
    println!("🎯 运行策略: {} ({})", name, mode);
    if let Some(c) = &code {
        println!("📈 股票代码: {}", c);
    }

    match name.as_str() {
        "ma_cross" => {
            if mode == "backtest" {
                run_ma_cross_backtest(code).await?;
            } else if mode == "paper" || mode == "live" {
                let runtime = CliRuntime::load();
                let runtime_store =
                    StrategyRuntimeStore::new(runtime.strategy_runtime_db_path).await?;
                let summary = execute_strategy_run_with_components(
                    &name,
                    &mode,
                    code,
                    ClickHouseDailyKlineLoader::new(create_clickhouse_client().await?),
                    create_trade_store(),
                    create_risk_store(),
                    &runtime_store,
                )
                .await?;
                print_strategy_run_summary(&summary);
            } else {
                println!("⚠️  暂不支持该运行模式");
            }
        }
        _ => {
            println!("❌ 未知策略: {}", name);
            println!("💡 可用策略: ma_cross");
        }
    }

    Ok(())
}

/// 运行 MA 交叉策略回测
async fn run_ma_cross_backtest(code: Option<String>) -> Result<()> {
    let stock_code = code.unwrap_or_else(|| "000001".to_string());

    println!("🔙 开始回测: MA 交叉策略");
    println!("  股票: {}", stock_code);
    println!("  参数: MA5, MA20");

    // 连接 ClickHouse
    let client = create_clickhouse_client().await?;

    // 获取历史数据
    let klines = client
        .get_kline_data(
            &stock_code,
            "1d",
            None,
            None,
            Some(10000), // 获取足够的数据用于回测
        )
        .await?;

    if klines.len() < 20 {
        return Err(QuantixError::Other(format!(
            "数据不足，至少需要20条，当前: {}",
            klines.len()
        )));
    }

    // 创建回测引擎
    let config = BacktestConfig {
        initial_capital: dec!(100000),
        commission_rate: dec!(0.0003),
        slippage_bps: 10, // 0.1% = 10 bps
        max_positions: 5,
        max_position_ratio: dec!(0.2),
        risk_free_rate: dec!(0.03),
    };

    let mut engine = BacktestEngine::new(config);

    // 创建策略
    let mut strategy = crate::strategy::ma_cross::MACrossStrategy::new(5, 20);

    // 转换数据格式为 HashMap
    use crate::data::models::Kline;
    use std::collections::HashMap;

    let mut data_map = HashMap::new();
    let kline_data: Vec<Kline> = klines
        .into_iter()
        .map(|k| Kline {
            code: stock_code.clone(),
            date: k.date,
            open: k.open,
            high: k.high,
            low: k.low,
            close: k.close,
            volume: k.volume,
            amount: k.amount,
            adjust_type: crate::data::models::AdjustType::None,
        })
        .collect();

    data_map.insert(stock_code.clone(), kline_data);

    // 运行回测
    let total_klines = data_map.values().map(|v| v.len()).sum::<usize>();
    let progress = ProgressBar::new(total_klines as u64);
    progress.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} K线")
            .unwrap(),
    );

    let result = engine
        .run(&mut strategy, &data_map)
        .await
        .map_err(|e| QuantixError::Other(format!("回测失败: {}", e)))?;

    progress.finish();

    // 显示结果
    println!("\n📊 回测结果:");
    println!("  总收益率: {:.2}%", result.report.total_return * dec!(100));
    println!("  夏普比率: {:.2}", result.report.sharpe_ratio);
    println!("  最大回撤: {:.2}%", result.report.max_drawdown * dec!(100));
    println!("  胜率: {:.2}%", result.report.win_rate * dec!(100));
    println!("  交易次数: {}", result.report.total_trades);

    Ok(())
}

/// 列出所有策略
async fn list_strategies() -> Result<()> {
    println!("📋 可用策略:");
    println!();
    println!("  1. ma_cross - 均线交叉策略");
    println!("     描述: MA5 上穿 MA20 买入，下穿卖出");
    println!("     运行: quantix strategy run --name ma_cross --mode backtest --code 000001");
    println!();
    println!("💡 更多策略开发中...");

    Ok(())
}

/// 显示策略详情
async fn show_strategy(name: String) -> Result<()> {
    println!("📖 策略详情: {}", name);

    match name.as_str() {
        "ma_cross" => {
            println!();
            println!("  名称: 均线交叉策略");
            println!("  原理: 当短期均线(MA5)上穿长期均线(MA20)时买入，反之卖出");
            println!("  参数:");
            println!("    - 短期周期: 5");
            println!("    - 长期周期: 20");
            println!("  适用场景: 趋势明显的市场");
            println!("  风险: 震荡市场容易频繁交易");
        }
        _ => {
            println!("❌ 未知策略: {}", name);
        }
    }

    Ok(())
}

/// 任务命令
pub async fn run_task_command(cmd: TaskCommands) -> Result<()> {
    ensure_task_command_supported_for_p0(&cmd)?;

    match cmd {
        TaskCommands::Add {
            name,
            cron,
            command,
        } => {
            add_task(name, cron, command).await?;
        }
        TaskCommands::List => {
            list_tasks().await?;
        }
        TaskCommands::Start { daemon } => {
            start_task_scheduler(daemon).await?;
        }
        TaskCommands::Stop => {
            stop_task_scheduler().await?;
        }
        TaskCommands::Status => {
            show_task_status().await?;
        }
    }
    Ok(())
}

fn ensure_task_command_supported_for_p0(cmd: &TaskCommands) -> Result<()> {
    match cmd {
        TaskCommands::Add { .. } => Err(QuantixError::Unsupported(
            "Foundation P0 仅支持预置任务模板；`task add` 暂不开放".to_string(),
        )),
        TaskCommands::Start { daemon: true } => Err(QuantixError::Unsupported(
            "Foundation P0 仅支持前台直接执行；`task start --daemon` 暂不支持".to_string(),
        )),
        _ => Ok(()),
    }
}

/// 添加任务
async fn add_task(name: String, cron: String, command: String) -> Result<()> {
    println!("⏰ 添加任务: {}", name);
    println!("  Cron: {}", cron);
    println!("  命令: {}", command);

    Err(QuantixError::Unsupported(
        "Foundation P0 仅支持预置任务模板；请使用 `quantix task list` 查看可运行任务".to_string(),
    ))
}

fn foundation_p0_task_template_descriptions() -> Vec<(String, String, String)> {
    [
        TaskTemplates::pre_market_check(),
        TaskTemplates::auction_collection(),
        TaskTemplates::market_open(),
        TaskTemplates::market_close(),
        TaskTemplates::post_market_process(),
        TaskTemplates::data_sync(),
    ]
    .into_iter()
    .map(|task| (task.name, task.command, task.cron_expr))
    .collect()
}

/// 列出所有任务
async fn list_tasks() -> Result<()> {
    println!("📋 Foundation P0 预置任务模板:");
    println!();
    println!("  预定义任务模板 (名称 | 描述 | Cron):");
    for (name, command, cron) in foundation_p0_task_template_descriptions() {
        println!("    - {} | {} | {}", name, command, cron);
    }
    println!();
    println!("💡 Foundation P0 只支持前台启动预置模板: `quantix task start`");

    Ok(())
}

/// 启动任务调度器
async fn start_task_scheduler(daemon: bool) -> Result<()> {
    if daemon {
        return Err(QuantixError::Unsupported(
            "Foundation P0 仅支持前台直接执行；后台守护模式暂不支持".to_string(),
        ));
    }

    println!("⏰ 启动任务调度器...");

    // 创建调度器
    let scheduler = TaskScheduler::new()
        .await
        .map_err(|e| QuantixError::Other(format!("创建调度器失败: {}", e)))?;

    // 添加预设任务
    if let Err(e) = scheduler.add_task(TaskTemplates::pre_market_check()).await {
        println!("⚠️  添加盘前任务失败: {}", e);
    }
    if let Err(e) = scheduler
        .add_task(TaskTemplates::auction_collection())
        .await
    {
        println!("⚠️  添加竞价任务失败: {}", e);
    }
    if let Err(e) = scheduler.add_task(TaskTemplates::market_open()).await {
        println!("⚠️  添加开盘任务失败: {}", e);
    }
    if let Err(e) = scheduler.add_task(TaskTemplates::market_close()).await {
        println!("⚠️  添加收盘任务失败: {}", e);
    }
    if let Err(e) = scheduler
        .add_task(TaskTemplates::post_market_process())
        .await
    {
        println!("⚠️  添加盘后任务失败: {}", e);
    }
    if let Err(e) = scheduler.add_task(TaskTemplates::data_sync()).await {
        println!("⚠️  添加同步任务失败: {}", e);
    }

    println!("✅ 已添加预设任务");

    // 启动调度器
    scheduler
        .start()
        .await
        .map_err(|e| QuantixError::Other(format!("启动调度器失败: {}", e)))?;

    println!("✅ 任务调度器已启动");
    println!("\n按 Ctrl+C 停止调度器");

    // 等待停止信号
    tokio::signal::ctrl_c().await?;
    println!("\n🛑 停止调度器...");
    scheduler
        .stop()
        .await
        .map_err(|e| QuantixError::Other(format!("停止调度器失败: {}", e)))?;
    println!("✅ 调度器已停止");

    Ok(())
}

/// 停止任务调度器
async fn stop_task_scheduler() -> Result<()> {
    println!("🛑 停止任务调度器...");
    println!("💡 提示: 在运行中的调度器按 Ctrl+C 停止");
    Ok(())
}

/// 显示任务状态
async fn show_task_status() -> Result<()> {
    println!("📊 Foundation P0 任务状态:");
    println!();
    println!("  状态: 仅支持当前进程内调度器");
    println!("  持久化: 暂不支持");
    println!("  后台守护: 暂不支持");
    println!();
    println!("💡 使用 `quantix task start` 以前台模式运行预置任务模板");

    Ok(())
}

/// 分析命令
pub async fn run_analyze_command(cmd: AnalyzeCommands) -> Result<()> {
    match cmd {
        AnalyzeCommands::Indicators { code, indicators } => {
            calculate_indicators(code, indicators).await?;
        }
        AnalyzeCommands::Backtest { id } => {
            show_backtest_report(id).await?;
        }
        AnalyzeCommands::CandlePattern {
            candle,
            code,
            tdx_root,
            market,
            day_file,
            start,
            end,
            r#type,
            limit,
            reference,
            previous_close,
        } => {
            analyze_candle_patterns(
                candle,
                code,
                tdx_root,
                market,
                day_file,
                start,
                end,
                r#type,
                limit,
                reference,
                previous_close,
            )
            .await?;
        }
        AnalyzeCommands::Screener(cmd) => {
            run_screener_command(cmd).await?;
        }
    }
    Ok(())
}

pub async fn run_monitor_command(cmd: MonitorCommands) -> Result<()> {
    match cmd {
        MonitorCommands::Watchlist {
            once: true,
            repeat: false,
        }
        | MonitorCommands::Alert(_) => {
            let watchlist_reader =
                ConfiguredMonitorWatchlistReader::new(create_watchlist_storage());
            let quote_reader = TdxMonitorQuoteReader;
            let alert_store = create_monitor_alert_store().await?;
            let service = MonitorService::new(watchlist_reader, quote_reader, alert_store.clone());
            let output = match cmd {
                MonitorCommands::Watchlist { once, repeat } => {
                    let stop_store = create_stop_rule_store().await?;
                    execute_monitor_command_with_stop_store(
                        MonitorCommands::Watchlist { once, repeat },
                        &service,
                        &stop_store,
                    )
                    .await?
                }
                other => execute_monitor_command_with_service(other, &service).await?,
            };

            if let MonitorCommandOutput::Watchlist {
                snapshot,
                triggered_stops: _,
            } = &output
            {
                persist_triggered_monitor_alerts(&alert_store, snapshot, Utc::now()).await?;
            }

            print_monitor_command_output(&output);
            Ok(())
        }
        MonitorCommands::Config(config_cmd) => {
            let runtime = CliRuntime::load();
            let store = JsonMonitorConfigStore::new(runtime.monitor_config_path);
            let output = execute_monitor_config_command_with_store(config_cmd, &store)?;
            print_monitor_command_output(&output);
            Ok(())
        }
        MonitorCommands::Event(event_cmd) => {
            let store = create_monitor_alert_store().await?;
            let output = execute_monitor_event_command_with_store(event_cmd, &store).await?;
            print_monitor_command_output(&output);
            Ok(())
        }
        MonitorCommands::Watchlist {
            once: false,
            repeat: true,
        } => {
            let runtime = CliRuntime::load();
            let config_store = JsonMonitorConfigStore::new(runtime.monitor_config_path);
            let runner = create_configured_monitor_runner().await?;
            run_monitor_loop(&config_store, &runner, MonitorRunMode::Foreground).await
        }
        MonitorCommands::Daemon(MonitorDaemonCommands::Run) => {
            let runtime = CliRuntime::load();
            let config_store = JsonMonitorConfigStore::new(runtime.monitor_config_path);
            let runner = create_configured_monitor_runner().await?;
            run_monitor_loop(&config_store, &runner, MonitorRunMode::Daemon).await
        }
        MonitorCommands::Service(service_cmd) => {
            let output = execute_monitor_service_command(service_cmd)?;
            print_monitor_command_output(&output);
            Ok(())
        }
        MonitorCommands::ServiceConfig(service_config_cmd) => {
            let store = JsonMonitorServiceConfigStore::with_default_path()?;
            let output =
                execute_monitor_service_config_command_with_store(service_config_cmd, &store)?;
            print_monitor_command_output(&output);
            Ok(())
        }
        MonitorCommands::Watchlist { once, repeat } => Err(QuantixError::Other(format!(
            "invalid monitor watchlist mode: once={}, repeat={}",
            once, repeat
        ))),
    }
}

pub async fn run_stop_command(cmd: StopCommands) -> Result<()> {
    let watchlist_storage = create_watchlist_storage();
    let service = StopService::new(create_stop_rule_store().await?);
    let trade_store = create_trade_store();
    let output = execute_stop_command_with_context(
        cmd,
        &service,
        &watchlist_storage,
        &TdxWatchlistQuoteLookup,
        &trade_store,
    )
    .await?;
    print_stop_command_output(&output);
    Ok(())
}

pub async fn run_trade_command(cmd: TradeCommands) -> Result<()> {
    let trade_store = create_trade_store();
    let service = TradeService::new(trade_store.clone());
    let risk_service = RiskService::new(create_risk_store());
    let output =
        execute_trade_command_with_risk(cmd, &service, &trade_store, &risk_service).await?;
    print_trade_command_output(&output);
    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
enum MonitorCommandOutput {
    Watchlist {
        snapshot: MonitorWatchlistSnapshot,
        triggered_stops: Vec<TriggeredStop>,
    },
    AutomationIteration {
        run_mode: MonitorRunMode,
        output: MonitorIterationOutput,
    },
    AlertAdded(PriceAlert),
    AlertList(Vec<PriceAlert>),
    Config(MonitorConfig),
    EventList(Vec<MonitorEventRow>),
    ServiceConfig(MonitorServiceConfig),
    ServiceStatus(MonitorServiceStatusSummary),
    ServiceMessage(String),
    AlertRemoved {
        id: u64,
        removed: bool,
    },
}

#[derive(Debug, Clone, PartialEq)]
enum StopCommandOutput {
    RuleSet(StopRule),
    RuleUpdated(StopRule),
    RuleList(Vec<StopRule>),
    StatusRows(Vec<StopStatusRow>),
    HistoryRows(Vec<StopHistoryEvent>),
    RuleRemoved { code: String, removed: bool },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TradeCommandOutput {
    AccountInitialized(PaperTradeAccount),
    AccountReset(PaperTradeAccount),
    TradeExecuted(TradeRecord),
    HistoryRows(Vec<TradeHistoryRow>),
    FeeRows(Vec<TradeFeeRow>),
    Overview(TradeOverview),
    PositionList(Vec<TradePosition>),
    PositionCurrentList(Vec<TradePositionCurrentRow>),
    Cash(CashSnapshot),
}

pub async fn run_market_command(cmd: MarketCommands) -> Result<()> {
    let output = execute_market_command_with_reader(cmd, create_clickhouse_client().await?).await?;

    match output {
        MarketCommandOutput::BoardRows(rows) => print_market_board_rows(&rows),
        MarketCommandOutput::NorthFlow(snapshot) => print_north_flow_snapshot(snapshot.as_ref()),
        MarketCommandOutput::Sentiment(snapshot) => {
            print_market_sentiment_snapshot(snapshot.as_ref())
        }
        MarketCommandOutput::Leaders(rows) => print_market_leader_rows(&rows),
        MarketCommandOutput::Overview(overview) => print_market_overview(&overview),
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
enum MarketCommandOutput {
    BoardRows(Vec<BoardRankRow>),
    NorthFlow(Option<NorthFlowSnapshot>),
    Sentiment(Option<MarketSentimentSnapshot>),
    Leaders(Vec<LeaderRow>),
    Overview(MarketOverview),
}

#[derive(Debug, Clone)]
struct ConfiguredMonitorWatchlistReader {
    storage: WatchlistStorage,
    service: WatchlistService,
}

impl ConfiguredMonitorWatchlistReader {
    fn new(storage: WatchlistStorage) -> Self {
        Self {
            storage,
            service: WatchlistService::default(),
        }
    }
}

#[async_trait]
impl MonitorWatchlistReader for ConfiguredMonitorWatchlistReader {
    async fn list_items(&self) -> Result<Vec<WatchlistListItem>> {
        let store = load_watchlist_store_for_read(&self.storage)?;
        Ok(self.service.list(&store, None, None))
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct TdxMonitorQuoteReader;

#[async_trait]
impl MonitorQuoteReader for TdxMonitorQuoteReader {
    async fn load_quotes(&self, codes: &[String]) -> Result<Vec<MonitorQuoteRow>> {
        let quote_map = TdxWatchlistQuoteLookup
            .lookup_quotes(codes)
            .await
            .unwrap_or_default();

        Ok(codes
            .iter()
            .filter_map(|code| {
                let snapshot = quote_map.get(code)?;
                Some(MonitorQuoteRow {
                    code: code.clone(),
                    group: String::new(),
                    tags: Vec::new(),
                    last_price: snapshot.latest_price.to_f64(),
                    change_pct: snapshot.price_change_pct.and_then(|value| value.to_f64()),
                    quote_time: None,
                    note: None,
                })
            })
            .collect())
    }
}

#[derive(Debug, Clone, PartialEq)]
struct MonitorAlertAddRequest {
    code: String,
    kind: PriceAlertKind,
    target_price: f64,
}

async fn execute_monitor_command_with_service<RW, RQ, RS>(
    cmd: MonitorCommands,
    service: &MonitorService<RW, RQ, RS>,
) -> Result<MonitorCommandOutput>
where
    RW: MonitorWatchlistReader,
    RQ: MonitorQuoteReader,
    RS: MonitorAlertStore,
{
    match cmd {
        MonitorCommands::Watchlist { once, repeat } => {
            validate_monitor_watchlist_command(once, repeat)?;
            Ok(MonitorCommandOutput::Watchlist {
                snapshot: service.load_watchlist_snapshot().await?,
                triggered_stops: Vec::new(),
            })
        }
        MonitorCommands::Alert(alert_cmd) => match alert_cmd {
            MonitorAlertCommands::Add { code, above, below } => {
                let request = build_monitor_alert_request(code, above, below)?;
                let alert = service
                    .add_alert(
                        &request.code,
                        request.kind,
                        request.target_price,
                        Utc::now(),
                    )
                    .await?;
                Ok(MonitorCommandOutput::AlertAdded(alert))
            }
            MonitorAlertCommands::List => Ok(MonitorCommandOutput::AlertList(
                service.list_alerts().await?,
            )),
            MonitorAlertCommands::Remove { id } => {
                let removed = service.remove_alert(monitor_alert_id_to_i64(id)?).await?;
                Ok(MonitorCommandOutput::AlertRemoved { id, removed })
            }
        },
        MonitorCommands::Config(_) => Err(QuantixError::Unsupported(
            "monitor config 尚未实现".to_string(),
        )),
        MonitorCommands::Daemon(_) => Err(QuantixError::Unsupported(
            "monitor daemon 尚未实现".to_string(),
        )),
        MonitorCommands::Service(_) => Err(QuantixError::Unsupported(
            "monitor service 尚未实现".to_string(),
        )),
        MonitorCommands::Event(_) => Err(QuantixError::Unsupported(
            "monitor event 尚未实现".to_string(),
        )),
        MonitorCommands::ServiceConfig(_) => Err(QuantixError::Unsupported(
            "monitor service-config 尚未实现".to_string(),
        )),
    }
}

async fn execute_monitor_command_with_stop_store<RW, RQ, RS, SS>(
    cmd: MonitorCommands,
    service: &MonitorService<RW, RQ, RS>,
    stop_store: &SS,
) -> Result<MonitorCommandOutput>
where
    RW: MonitorWatchlistReader,
    RQ: MonitorQuoteReader,
    RS: MonitorAlertStore,
    SS: StopRuleStore + Clone,
{
    match cmd {
        MonitorCommands::Watchlist { once, repeat } => {
            validate_monitor_watchlist_command(once, repeat)?;
            let snapshot = service.load_watchlist_snapshot().await?;
            let trade_store = create_trade_store();
            let triggered_stops =
                evaluate_stop_rules_for_snapshot(&snapshot, stop_store, &trade_store).await?;
            Ok(MonitorCommandOutput::Watchlist {
                snapshot,
                triggered_stops,
            })
        }
        other => execute_monitor_command_with_service(other, service).await,
    }
}

async fn evaluate_stop_rules_for_snapshot<SS, TS>(
    snapshot: &MonitorWatchlistSnapshot,
    stop_store: &SS,
    trade_store: &TS,
) -> Result<Vec<TriggeredStop>>
where
    SS: StopRuleStore + Clone,
    TS: PaperTradeStore,
{
    let rules = stop_store.list_rules().await?;
    if rules.is_empty() {
        return Ok(Vec::new());
    }

    let observed_at = snapshot
        .rows
        .iter()
        .filter_map(|row| row.quote_time)
        .max()
        .unwrap_or_else(Utc::now);
    let avg_cost_by_code = build_avg_cost_map_from_trade_store(trade_store).await?;
    let stop_service = StopService::new(stop_store.clone());
    let results =
        stop_service.evaluate_rules_with_anchor_map(&rules, &snapshot.rows, &avg_cost_by_code, observed_at);
    let mut triggered_stops = Vec::new();

    for (original_rule, result) in rules.iter().zip(results.into_iter()) {
        if result.updated_rule != *original_rule {
            stop_store.upsert_rule(result.updated_rule.clone()).await?;
        }

        if let Some(triggered_stop) = result.triggered_stop {
            stop_store
                .append_history(StopHistoryEvent {
                    id: uuid::Uuid::new_v4().to_string(),
                    code: triggered_stop.code.clone(),
                    event_type: StopHistoryEventType::Trigger,
                    trigger_kind: Some(match triggered_stop.kind {
                        StopTriggerKind::Loss => crate::stop::StopHistoryTriggerKind::Loss,
                        StopTriggerKind::Profit => crate::stop::StopHistoryTriggerKind::Profit,
                        StopTriggerKind::TrailingLoss => crate::stop::StopHistoryTriggerKind::Trailing,
                    }),
                    trigger_price: Some(triggered_stop.current_price),
                    anchor_price: triggered_stop.anchor_price,
                    anchor_source: triggered_stop
                        .anchor_source
                        .map(|source| source.as_str().to_string()),
                    snapshot_json: serde_json::to_value(&result.updated_rule)?,
                    created_at: triggered_stop.triggered_at.unwrap_or(observed_at),
                })
                .await?;
            triggered_stops.push(triggered_stop);
        }
    }

    Ok(triggered_stops)
}

async fn execute_stop_command_with_service<RS>(
    cmd: StopCommands,
    service: &StopService<RS>,
    watchlist_storage: &WatchlistStorage,
) -> Result<StopCommandOutput>
where
    RS: StopRuleStore,
{
    let trade_store = create_trade_store();
    execute_stop_command_with_context(
        cmd,
        service,
        watchlist_storage,
        &TdxWatchlistQuoteLookup,
        &trade_store,
    )
    .await
}

async fn execute_stop_command_with_context<RS, Q, TS>(
    cmd: StopCommands,
    service: &StopService<RS>,
    watchlist_storage: &WatchlistStorage,
    quote_lookup: &Q,
    trade_store: &TS,
) -> Result<StopCommandOutput>
where
    RS: StopRuleStore,
    Q: WatchlistQuoteLookup,
    TS: PaperTradeStore,
{
    match cmd {
        StopCommands::Set {
            code,
            loss,
            profit,
            loss_pct,
            profit_pct,
            trailing,
        } => {
            ensure_watchlist_contains_code(watchlist_storage, &code)?;
            let reference_price = if loss_pct.is_some() || profit_pct.is_some() {
                Some(resolve_stop_reference_price(&code, quote_lookup, trade_store).await?)
            } else {
                None
            };
            let rule = service
                .set_rule(
                    &code,
                    loss,
                    profit,
                    loss_pct,
                    profit_pct,
                    trailing,
                    reference_price,
                    Utc::now(),
                )
                .await?;
            Ok(StopCommandOutput::RuleSet(rule))
        }
        StopCommands::Update {
            code,
            loss,
            profit,
            loss_pct,
            profit_pct,
            trailing,
            clear_loss,
            clear_profit,
            clear_loss_pct,
            clear_profit_pct,
            clear_trailing,
        } => {
            ensure_watchlist_contains_code(watchlist_storage, &code)?;
            let existing = service
                .get_rule(&code)
                .await?
                .ok_or_else(|| QuantixError::Other(format!("stop update 未找到规则: {code}")))?;
            let needs_reference_price = (loss_pct.is_some() || profit_pct.is_some())
                && existing.reference_price.is_none();
            let reference_price = if needs_reference_price {
                Some(Some(
                    resolve_stop_reference_price(&code, quote_lookup, trade_store).await?,
                ))
            } else {
                None
            };
            let rule = service
                .update_rule(
                    &code,
                    StopRuleUpdate {
                        stop_loss_price: patch_value(loss, clear_loss),
                        take_profit_price: patch_value(profit, clear_profit),
                        stop_loss_pct: patch_value(loss_pct, clear_loss_pct),
                        take_profit_pct: patch_value(profit_pct, clear_profit_pct),
                        trailing_pct: patch_value(trailing, clear_trailing),
                        reference_price,
                    },
                    Utc::now(),
                )
                .await?;
            Ok(StopCommandOutput::RuleUpdated(rule))
        }
        StopCommands::List => Ok(StopCommandOutput::RuleList(service.list_rules().await?)),
        StopCommands::Status { code } => {
            let rules = filter_stop_rules(service.list_rules().await?, code.as_deref());
            let status_rows =
                build_stop_status_rows(service, &rules, quote_lookup, trade_store, Utc::now())
                    .await?;
            Ok(StopCommandOutput::StatusRows(status_rows))
        }
        StopCommands::History {
            code,
            limit,
            date,
            event_type,
        } => Ok(StopCommandOutput::HistoryRows(
            service
                .history(
                    code.as_deref(),
                    date.as_deref()
                        .map(|raw| parse_stop_history_date(raw))
                        .transpose()?,
                    event_type
                        .as_deref()
                        .map(parse_stop_history_event_type)
                        .transpose()?,
                    Some(limit),
                )
                .await?,
        )),
        StopCommands::Remove { code } => {
            let removed = service.remove_rule(&code, Utc::now()).await?;
            Ok(StopCommandOutput::RuleRemoved { code, removed })
        }
    }
}

async fn execute_trade_command_with_service<Store>(
    cmd: TradeCommands,
    service: &TradeService<Store>,
) -> Result<TradeCommandOutput>
where
    Store: PaperTradeStore,
{
    let reporting = TradeReportingService::new();
    match cmd {
        TradeCommands::Init {
            capital,
            commission_rate,
            commission_min,
            stamp_duty_rate,
            transfer_fee_rate,
        } => {
            let request = build_trade_init_request(
                "trade init",
                capital,
                commission_rate,
                commission_min,
                stamp_duty_rate,
                transfer_fee_rate,
            )?;
            Ok(TradeCommandOutput::AccountInitialized(
                service.init_account(request, Utc::now()).await?,
            ))
        }
        TradeCommands::Reset {
            capital,
            commission_rate,
            commission_min,
            stamp_duty_rate,
            transfer_fee_rate,
        } => {
            let request = build_trade_init_request(
                "trade reset",
                capital,
                commission_rate,
                commission_min,
                stamp_duty_rate,
                transfer_fee_rate,
            )?;
            Ok(TradeCommandOutput::AccountReset(
                service.reset_account(request, Utc::now()).await?,
            ))
        }
        TradeCommands::Buy {
            code,
            price,
            volume,
        } => {
            let request = build_trade_order_request("trade buy", code, price, volume)?;
            Ok(TradeCommandOutput::TradeExecuted(
                service.buy(request, Utc::now()).await?,
            ))
        }
        TradeCommands::Sell {
            code,
            price,
            volume,
        } => {
            let request = build_trade_order_request("trade sell", code, price, volume)?;
            Ok(TradeCommandOutput::TradeExecuted(
                service.sell(request, Utc::now()).await?,
            ))
        }
        TradeCommands::History { code, limit } => {
            let state = service.state_snapshot().await?;
            Ok(TradeCommandOutput::HistoryRows(reporting.history_rows(
                &state,
                code.as_deref(),
                limit,
            )))
        }
        TradeCommands::Fees { code, limit } => {
            let state = service.state_snapshot().await?;
            Ok(TradeCommandOutput::FeeRows(reporting.fee_rows(
                &state,
                code.as_deref(),
                limit,
            )))
        }
        TradeCommands::Overview { current: false } => {
            let state = service.state_snapshot().await?;
            Ok(TradeCommandOutput::Overview(reporting.overview(&state)))
        }
        TradeCommands::Overview { current: true } | TradeCommands::Position { current: true } => {
            Err(QuantixError::Unsupported(
                "trade current views require quote lookup".to_string(),
            ))
        }
        TradeCommands::Position { current: false } => {
            Ok(TradeCommandOutput::PositionList(service.positions().await?))
        }
        TradeCommands::Cash => Ok(TradeCommandOutput::Cash(service.cash_snapshot().await?)),
    }
}

async fn execute_trade_command_with_quote_lookup<Store, Q>(
    cmd: TradeCommands,
    service: &TradeService<Store>,
    quote_lookup: &Q,
) -> Result<TradeCommandOutput>
where
    Store: PaperTradeStore,
    Q: WatchlistQuoteLookup,
{
    let reporting = TradeReportingService::new();

    match cmd {
        TradeCommands::Overview { current: true } => {
            let state = service.state_snapshot().await?;
            let quotes = load_trade_quote_prices(&state, quote_lookup).await;
            let total_positions = state
                .account
                .as_ref()
                .map(|account| account.positions.len())
                .unwrap_or(0);
            let resolved_positions = quotes.len();

            let mut overview = reporting.overview(&state);
            overview.quote_coverage = Some((resolved_positions, total_positions));

            if total_positions == 0 {
                overview.live_position_value = Some(Decimal::ZERO);
                overview.live_total_assets = Some(overview.booked_total_assets);
            } else if resolved_positions == total_positions {
                let rows = reporting.position_rows_with_quotes(&state, &quotes);
                let live_position_value = rows
                    .iter()
                    .filter_map(|row| row.current_market_value)
                    .sum::<Decimal>();
                overview.live_position_value = Some(live_position_value);
                overview.live_total_assets = Some(overview.available_cash + live_position_value);
            }

            Ok(TradeCommandOutput::Overview(overview))
        }
        TradeCommands::Position { current: true } => {
            let state = service.state_snapshot().await?;
            let quotes = load_trade_quote_prices(&state, quote_lookup).await;
            Ok(TradeCommandOutput::PositionCurrentList(
                reporting.position_rows_with_quotes(&state, &quotes),
            ))
        }
        other => execute_trade_command_with_service(other, service).await,
    }
}

async fn execute_trade_command_with_risk<TradeStore, RiskStore>(
    cmd: TradeCommands,
    trade_service: &TradeService<TradeStore>,
    trade_store: &TradeStore,
    risk_service: &RiskService<RiskStore>,
) -> Result<TradeCommandOutput>
where
    TradeStore: PaperTradeStore,
    RiskStore: crate::risk::RiskStore,
{
    match cmd {
        TradeCommands::Init {
            capital,
            commission_rate,
            commission_min,
            stamp_duty_rate,
            transfer_fee_rate,
        } => {
            let request = build_trade_init_request(
                "trade init",
                capital,
                commission_rate,
                commission_min,
                stamp_duty_rate,
                transfer_fee_rate,
            )?;
            let account = trade_service.init_account(request, Utc::now()).await?;
            let snapshot = build_risk_account_snapshot(&account);
            risk_service
                .sync_after_trade_reset(&snapshot, Utc::now())
                .await?;
            Ok(TradeCommandOutput::AccountInitialized(account))
        }
        TradeCommands::Reset {
            capital,
            commission_rate,
            commission_min,
            stamp_duty_rate,
            transfer_fee_rate,
        } => {
            let request = build_trade_init_request(
                "trade reset",
                capital,
                commission_rate,
                commission_min,
                stamp_duty_rate,
                transfer_fee_rate,
            )?;
            let account = trade_service.reset_account(request, Utc::now()).await?;
            let snapshot = build_risk_account_snapshot(&account);
            risk_service
                .sync_after_trade_reset(&snapshot, Utc::now())
                .await?;
            Ok(TradeCommandOutput::AccountReset(account))
        }
        TradeCommands::Buy {
            code,
            price,
            volume,
        } => {
            let request = build_trade_order_request("trade buy", code, price, volume)?;
            let account = load_initialized_trade_account(trade_store).await?;
            let snapshot = build_risk_account_snapshot(&account);
            let projected_buy = build_projected_buy_impact(&account, &request);
            risk_service
                .check_buy(&snapshot, &projected_buy, Utc::now())
                .await?;

            let record = trade_service.buy(request, Utc::now()).await?;
            sync_risk_from_trade_store(trade_store, risk_service).await?;
            Ok(TradeCommandOutput::TradeExecuted(record))
        }
        TradeCommands::Sell {
            code,
            price,
            volume,
        } => {
            let request = build_trade_order_request("trade sell", code, price, volume)?;
            let record = trade_service.sell(request, Utc::now()).await?;
            sync_risk_from_trade_store(trade_store, risk_service).await?;
            Ok(TradeCommandOutput::TradeExecuted(record))
        }
        TradeCommands::Overview { current: true } | TradeCommands::Position { current: true } => {
            execute_trade_command_with_quote_lookup(cmd, trade_service, &TdxWatchlistQuoteLookup)
                .await
        }
        other => execute_trade_command_with_service(other, trade_service).await,
    }
}

fn build_trade_init_request(
    command_name: &str,
    capital: Option<f64>,
    commission_rate: Option<f64>,
    commission_min: Option<f64>,
    stamp_duty_rate: Option<f64>,
    transfer_fee_rate: Option<f64>,
) -> Result<InitAccountRequest> {
    InitAccountRequest::new(
        capital,
        commission_rate,
        commission_min,
        stamp_duty_rate,
        transfer_fee_rate,
    )
    .map_err(|err| remap_trade_request_error(err, command_name))
}

fn build_trade_order_request(
    command_name: &str,
    code: String,
    price: f64,
    volume: i64,
) -> Result<TradeOrderRequest> {
    TradeOrderRequest::new(code, price, volume)
        .map_err(|err| remap_trade_request_error(err, command_name))
}

fn decimal_to_f64(value: Decimal, command_name: &str) -> Result<f64> {
    value
        .to_f64()
        .ok_or_else(|| QuantixError::Other(format!("{command_name} 无法将价格 {value} 转换为 f64")))
}

fn remap_trade_request_error(err: QuantixError, command_name: &str) -> QuantixError {
    match err {
        QuantixError::Other(message) => QuantixError::Other(
            message
                .replace("trade init", command_name)
                .replace("trade order", command_name),
        ),
        other => other,
    }
}

fn patch_value(value: Option<f64>, clear: bool) -> Option<Option<f64>> {
    if clear {
        Some(None)
    } else {
        value.map(Some)
    }
}

fn parse_stop_history_event_type(value: &str) -> Result<StopHistoryEventType> {
    StopHistoryEventType::from_str(value)
        .ok_or_else(|| QuantixError::Other(format!("未知 stop history event_type: {value}")))
}

fn parse_stop_history_date(value: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| QuantixError::Other(format!("stop history --date 无效: {value}")))
}

fn filter_stop_rules(rules: Vec<StopRule>, code: Option<&str>) -> Vec<StopRule> {
    match code {
        Some(code) => rules.into_iter().filter(|rule| rule.code == code).collect(),
        None => rules,
    }
}

async fn build_avg_cost_map_from_trade_store<Store>(
    trade_store: &Store,
) -> Result<HashMap<String, f64>>
where
    Store: PaperTradeStore,
{
    let Some(state) = trade_store.load_state().await? else {
        return Ok(HashMap::new());
    };
    let Some(account) = state.account else {
        return Ok(HashMap::new());
    };

    Ok(account
        .positions
        .into_iter()
        .filter_map(|(code, position)| position.avg_cost.to_f64().map(|avg_cost| (code, avg_cost)))
        .collect())
}

async fn resolve_stop_reference_price<Q, TS>(
    code: &str,
    quote_lookup: &Q,
    trade_store: &TS,
) -> Result<f64>
where
    Q: WatchlistQuoteLookup,
    TS: PaperTradeStore,
{
    let quote_price = quote_lookup
        .lookup_quotes(&[code.to_string()])
        .await
        .ok()
        .and_then(|quotes| quotes.get(code).and_then(|snapshot| snapshot.latest_price.to_f64()));
    if let Some(price) = quote_price {
        return Ok(price);
    }

    let avg_cost_by_code = build_avg_cost_map_from_trade_store(trade_store).await?;
    if let Some(avg_cost) = avg_cost_by_code.get(code).copied() {
        return Ok(avg_cost);
    }

    Err(QuantixError::Other(format!(
        "stop percent 规则缺少参考价，且当前无法从行情或持仓解析 {} 的 reference_price",
        code
    )))
}

async fn build_stop_status_rows<RS, Q, TS>(
    service: &StopService<RS>,
    rules: &[StopRule],
    quote_lookup: &Q,
    trade_store: &TS,
    observed_at: DateTime<Utc>,
) -> Result<Vec<StopStatusRow>>
where
    RS: StopRuleStore,
    Q: WatchlistQuoteLookup,
    TS: PaperTradeStore,
{
    let codes: Vec<String> = rules.iter().map(|rule| rule.code.clone()).collect();
    let quote_rows = quote_lookup
        .lookup_quotes(&codes)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|(code, snapshot)| MonitorQuoteRow {
            code,
            group: String::new(),
            tags: Vec::new(),
            last_price: snapshot.latest_price.to_f64(),
            change_pct: snapshot.price_change_pct.and_then(|value| value.to_f64()),
            quote_time: None,
            note: None,
        })
        .collect::<Vec<_>>();
    let avg_cost_by_code = build_avg_cost_map_from_trade_store(trade_store).await?;
    Ok(service.status_rows(rules, &quote_rows, &avg_cost_by_code, observed_at))
}

async fn execute_market_command_with_reader<R>(
    cmd: MarketCommands,
    reader: R,
) -> Result<MarketCommandOutput>
where
    R: MarketDataReader,
{
    let service = MarketService::new(reader);

    match cmd {
        MarketCommands::Sector { top, date, sort_by } => {
            let rows = service
                .get_board_rankings(
                    BoardType::Sector,
                    parse_market_date(date.as_deref())?,
                    top,
                    parse_board_sort_by(sort_by.as_deref())?,
                )
                .await?;
            Ok(MarketCommandOutput::BoardRows(rows))
        }
        MarketCommands::Concept { top, date, sort_by } => {
            let rows = service
                .get_board_rankings(
                    BoardType::Concept,
                    parse_market_date(date.as_deref())?,
                    top,
                    parse_board_sort_by(sort_by.as_deref())?,
                )
                .await?;
            Ok(MarketCommandOutput::BoardRows(rows))
        }
        MarketCommands::North { date } => Ok(MarketCommandOutput::NorthFlow(
            service
                .get_north_flow(parse_market_date(date.as_deref())?)
                .await?,
        )),
        MarketCommands::Sentiment { date } => Ok(MarketCommandOutput::Sentiment(
            service
                .get_market_sentiment(parse_market_date(date.as_deref())?)
                .await?,
        )),
        MarketCommands::Leader {
            sector,
            concept,
            all,
            limit,
            date,
        } => {
            let filter = build_leader_filter(sector, concept, all)?;
            let rows = service
                .get_leaders(filter, limit, parse_market_date(date.as_deref())?)
                .await?;
            Ok(MarketCommandOutput::Leaders(rows))
        }
        MarketCommands::Overview { top, date } => Ok(MarketCommandOutput::Overview(
            service
                .get_overview(parse_market_date(date.as_deref())?, top)
                .await?,
        )),
    }
}

fn validate_monitor_watchlist_command(once: bool, repeat: bool) -> Result<()> {
    if once ^ repeat {
        Ok(())
    } else {
        Err(QuantixError::Other(
            "monitor watchlist 必须且只能指定 --once 或 --repeat 之一".to_string(),
        ))
    }
}

fn build_monitor_alert_request(
    code: String,
    above: Option<f64>,
    below: Option<f64>,
) -> Result<MonitorAlertAddRequest> {
    match (above, below) {
        (Some(target_price), None) => Ok(MonitorAlertAddRequest {
            code,
            kind: PriceAlertKind::Above,
            target_price,
        }),
        (None, Some(target_price)) => Ok(MonitorAlertAddRequest {
            code,
            kind: PriceAlertKind::Below,
            target_price,
        }),
        _ => Err(QuantixError::Other(
            "monitor alert add 必须且只能指定 --above 或 --below 之一".to_string(),
        )),
    }
}

fn monitor_alert_id_to_i64(id: u64) -> Result<i64> {
    i64::try_from(id).map_err(|_| QuantixError::Other(format!("告警 ID 超出支持范围: {}", id)))
}

fn parse_monitor_event_type(value: &str) -> Result<MonitorEventType> {
    match value {
        "price-alert" => Ok(MonitorEventType::PriceAlert),
        "stop-loss" => Ok(MonitorEventType::StopLoss),
        "stop-profit" => Ok(MonitorEventType::StopProfit),
        "trailing-stop" => Ok(MonitorEventType::TrailingStop),
        other => Err(QuantixError::Other(format!(
            "monitor event list 不支持的事件类型: {}",
            other
        ))),
    }
}

async fn create_monitor_alert_store() -> Result<SqliteMonitorAlertStore> {
    let runtime = CliRuntime::load();
    SqliteMonitorAlertStore::new(runtime.monitor_db_path).await
}

async fn create_stop_rule_store() -> Result<SqliteStopRuleStore> {
    let runtime = CliRuntime::load();
    SqliteStopRuleStore::new(runtime.monitor_db_path).await
}

async fn create_configured_monitor_runner() -> Result<
    MonitorRunner<
        ConfiguredMonitorWatchlistReader,
        TdxMonitorQuoteReader,
        SqliteStopRuleStore,
        JsonPaperTradeStore,
    >,
> {
    let alert_store = create_monitor_alert_store().await?;
    let stop_store = create_stop_rule_store().await?;
    let trade_store = create_trade_store();
    Ok(MonitorRunner::new(
        ConfiguredMonitorWatchlistReader::new(create_watchlist_storage()),
        TdxMonitorQuoteReader,
        alert_store,
        stop_store,
        trade_store,
    ))
}

fn execute_monitor_config_command_with_store(
    cmd: MonitorConfigCommands,
    store: &JsonMonitorConfigStore,
) -> Result<MonitorCommandOutput> {
    let mut config = store.load_or_create()?;

    match cmd {
        MonitorConfigCommands::Show => Ok(MonitorCommandOutput::Config(config)),
        MonitorConfigCommands::Set {
            interval_seconds,
            group,
            persist_events,
        } => {
            if let Some(value) = interval_seconds {
                config.interval_seconds = value.max(1);
            }
            if let Some(value) = group {
                config.watchlist_group = Some(value);
            }
            if let Some(value) = persist_events {
                config.persist_events = value;
            }

            store.save(&config)?;
            Ok(MonitorCommandOutput::Config(config))
        }
        MonitorConfigCommands::ClearGroup => {
            config.watchlist_group = None;
            store.save(&config)?;
            Ok(MonitorCommandOutput::Config(config))
        }
    }
}

async fn execute_monitor_event_command_with_store(
    cmd: MonitorEventCommands,
    store: &SqliteMonitorAlertStore,
) -> Result<MonitorCommandOutput> {
    match cmd {
        MonitorEventCommands::List {
            limit,
            code,
            event_type,
        } => Ok(MonitorCommandOutput::EventList(
            store
                .list_events(&MonitorEventFilter {
                    limit,
                    code,
                    event_type: event_type
                        .as_deref()
                        .map(parse_monitor_event_type)
                        .transpose()?,
                })
                .await?,
        )),
    }
}

fn execute_monitor_service_config_command_with_store(
    cmd: MonitorServiceConfigCommands,
    store: &JsonMonitorServiceConfigStore,
) -> Result<MonitorCommandOutput> {
    match cmd {
        MonitorServiceConfigCommands::Show => match store.load() {
            Ok(config) => Ok(MonitorCommandOutput::ServiceConfig(config)),
            Err(QuantixError::Config(_)) => Ok(MonitorCommandOutput::ServiceMessage(
                "monitor service 未配置，请先运行 monitor service-config set --quantix-bin /abs/path/to/quantix".to_string(),
            )),
            Err(other) => Err(other),
        },
        MonitorServiceConfigCommands::Set { quantix_bin } => {
            let config = MonitorServiceConfig {
                quantix_bin_path: quantix_bin.into(),
            };
            JsonMonitorServiceConfigStore::validate(&config)?;
            store.save(&config)?;
            Ok(MonitorCommandOutput::ServiceConfig(config))
        }
    }
}

trait MonitorServiceInstallerOps {
    fn install(&self) -> Result<()>;
    fn uninstall(&self) -> Result<()>;
    fn start(&self) -> Result<()>;
    fn stop(&self) -> Result<()>;
    fn enable(&self) -> Result<()>;
    fn disable(&self) -> Result<()>;
    fn status(&self) -> Result<String>;
    fn status_summary(&self) -> Result<MonitorServiceStatusSummary>;
}

impl MonitorServiceInstallerOps for MonitorUserServiceInstaller {
    fn install(&self) -> Result<()> {
        MonitorUserServiceInstaller::install(self)
    }

    fn uninstall(&self) -> Result<()> {
        MonitorUserServiceInstaller::uninstall(self)
    }

    fn start(&self) -> Result<()> {
        MonitorUserServiceInstaller::start(self)
    }

    fn stop(&self) -> Result<()> {
        MonitorUserServiceInstaller::stop(self)
    }

    fn enable(&self) -> Result<()> {
        MonitorUserServiceInstaller::enable(self)
    }

    fn disable(&self) -> Result<()> {
        MonitorUserServiceInstaller::disable(self)
    }

    fn status(&self) -> Result<String> {
        MonitorUserServiceInstaller::status(self)
    }

    fn status_summary(&self) -> Result<MonitorServiceStatusSummary> {
        MonitorUserServiceInstaller::status_summary(self)
    }
}

fn execute_monitor_service_command(cmd: MonitorServiceCommands) -> Result<MonitorCommandOutput> {
    let runtime = CliRuntime::load();
    let store = JsonMonitorServiceConfigStore::with_default_path()?;
    let service_config = match store.load() {
        Ok(config) => config,
        Err(QuantixError::Config(_)) if matches!(cmd, MonitorServiceCommands::Status) => {
            return Ok(MonitorCommandOutput::ServiceStatus(
                build_unconfigured_monitor_service_status_summary(),
            ));
        }
        Err(other) => return Err(other),
    };
    let installer = MonitorUserServiceInstaller::new(runtime, service_config);
    execute_monitor_service_command_with_installer(cmd, &installer)
}

fn execute_monitor_service_command_with_installer<I>(
    cmd: MonitorServiceCommands,
    installer: &I,
) -> Result<MonitorCommandOutput>
where
    I: MonitorServiceInstallerOps,
{
    match cmd {
        MonitorServiceCommands::Install => {
            installer.install()?;
            Ok(MonitorCommandOutput::ServiceMessage(
                "monitor service installed".to_string(),
            ))
        }
        MonitorServiceCommands::Uninstall => {
            installer.uninstall()?;
            Ok(MonitorCommandOutput::ServiceMessage(
                "monitor service uninstalled".to_string(),
            ))
        }
        MonitorServiceCommands::Start => {
            installer.start()?;
            Ok(MonitorCommandOutput::ServiceMessage(
                "monitor service started".to_string(),
            ))
        }
        MonitorServiceCommands::Stop => {
            installer.stop()?;
            Ok(MonitorCommandOutput::ServiceMessage(
                "monitor service stopped".to_string(),
            ))
        }
        MonitorServiceCommands::Status => Ok(MonitorCommandOutput::ServiceStatus(
            installer.status_summary()?,
        )),
        MonitorServiceCommands::Enable => {
            installer.enable()?;
            Ok(MonitorCommandOutput::ServiceMessage(
                "monitor service enabled".to_string(),
            ))
        }
        MonitorServiceCommands::Disable => {
            installer.disable()?;
            Ok(MonitorCommandOutput::ServiceMessage(
                "monitor service disabled".to_string(),
            ))
        }
    }
}

async fn execute_monitor_iteration_with_runner<RW, RQ, SS, TS>(
    cmd: MonitorCommands,
    config: &MonitorConfig,
    runner: &MonitorRunner<RW, RQ, SS, TS>,
    now: DateTime<Utc>,
) -> Result<MonitorCommandOutput>
where
    RW: MonitorWatchlistReader,
    RQ: MonitorQuoteReader,
    SS: StopRuleStore + Clone,
    TS: PaperTradeStore + Clone,
{
    match cmd {
        MonitorCommands::Watchlist {
            once: false,
            repeat: true,
        } => Ok(MonitorCommandOutput::AutomationIteration {
            run_mode: MonitorRunMode::Foreground,
            output: runner
                .run_once(config, MonitorRunMode::Foreground, now)
                .await?,
        }),
        MonitorCommands::Daemon(MonitorDaemonCommands::Run) => {
            Ok(MonitorCommandOutput::AutomationIteration {
                run_mode: MonitorRunMode::Daemon,
                output: runner.run_once(config, MonitorRunMode::Daemon, now).await?,
            })
        }
        other => Err(QuantixError::Unsupported(format!(
            "monitor iteration helper does not support {:?}",
            other
        ))),
    }
}

async fn run_monitor_loop<RW, RQ, SS, TS>(
    config_store: &JsonMonitorConfigStore,
    runner: &MonitorRunner<RW, RQ, SS, TS>,
    run_mode: MonitorRunMode,
) -> Result<()>
where
    RW: MonitorWatchlistReader,
    RQ: MonitorQuoteReader,
    SS: StopRuleStore + Clone,
    TS: PaperTradeStore + Clone,
{
    loop {
        let config = config_store.load_or_create()?;
        let output = runner.run_once(&config, run_mode, Utc::now()).await?;
        print_monitor_command_output(&MonitorCommandOutput::AutomationIteration {
            run_mode,
            output,
        });

        let sleep_duration = Duration::from_secs(config.interval_seconds.max(1));
        tokio::select! {
            _ = tokio::signal::ctrl_c() => break,
            _ = tokio::time::sleep(sleep_duration) => {}
        }
    }

    Ok(())
}

async fn persist_triggered_monitor_alerts<RS>(
    store: &RS,
    snapshot: &MonitorWatchlistSnapshot,
    observed_at: chrono::DateTime<Utc>,
) -> Result<()>
where
    RS: MonitorAlertStore,
{
    for alert in &snapshot.triggered_alerts {
        let triggered_at = alert.triggered_at.unwrap_or(observed_at);
        store.mark_triggered(alert.alert_id, triggered_at).await?;
    }
    Ok(())
}

fn ensure_watchlist_contains_code(storage: &WatchlistStorage, code: &str) -> Result<()> {
    let store = load_watchlist_store_for_read(storage)?;
    if store.entries.contains_key(code) {
        Ok(())
    } else {
        Err(QuantixError::Other(format!("股票不在自选池: {}", code)))
    }
}

fn build_leader_filter(
    sector: Option<String>,
    concept: Option<String>,
    all: bool,
) -> Result<LeaderFilter> {
    let mut filter_count = 0usize;
    if sector.is_some() {
        filter_count += 1;
    }
    if concept.is_some() {
        filter_count += 1;
    }
    if all {
        filter_count += 1;
    }

    if filter_count != 1 {
        return Err(QuantixError::Other(
            "leader 必须且只能指定 --sector、--concept 或 --all 之一".to_string(),
        ));
    }

    match (sector, concept, all) {
        (Some(name), None, false) => Ok(LeaderFilter::Sector(name)),
        (None, Some(name), false) => Ok(LeaderFilter::Concept(name)),
        (None, None, true) => Ok(LeaderFilter::All),
        _ => Err(QuantixError::Other(
            "leader 必须且只能指定 --sector、--concept 或 --all 之一".to_string(),
        )),
    }
}

fn parse_market_date(raw: Option<&str>) -> Result<Option<NaiveDate>> {
    raw.map(|value| {
        NaiveDate::parse_from_str(value, "%Y-%m-%d")
            .map_err(|_| QuantixError::Other(format!("无效日期格式: {}，请使用 YYYY-MM-DD", value)))
    })
    .transpose()
}

fn parse_board_sort_by(raw: Option<&str>) -> Result<BoardSortBy> {
    match raw.unwrap_or("change_pct") {
        "change" | "change_pct" => Ok(BoardSortBy::ChangePct),
        other => Err(QuantixError::Other(format!(
            "不支持的 sort_by: {}，仅支持 change 或 change_pct",
            other
        ))),
    }
}

fn print_market_board_rows(rows: &[BoardRankRow]) {
    if rows.is_empty() {
        println!("📭 没有可展示的板块数据");
        return;
    }

    println!("{:<8} {:<12} {:<16} {}", "排名", "代码", "板块", "涨跌幅");
    println!("{}", "-".repeat(56));

    for row in rows {
        println!(
            "{:<8} {:<12} {:<16} {:.2}%",
            row.rank, row.board_code, row.board_name, row.change_pct
        );
    }
}

fn print_stop_command_output(output: &StopCommandOutput) {
    match output {
        StopCommandOutput::RuleSet(rule) => {
            println!("✅ 已设置 {} 的止盈止损规则", rule.code);
        }
        StopCommandOutput::RuleUpdated(rule) => {
            println!("✅ 已更新 {} 的止盈止损规则", rule.code);
        }
        StopCommandOutput::RuleList(rules) => print_stop_rules(rules),
        StopCommandOutput::StatusRows(rows) => print_stop_status_rows(rows),
        StopCommandOutput::HistoryRows(rows) => print_stop_history_rows(rows),
        StopCommandOutput::RuleRemoved { code, removed } => {
            if *removed {
                println!("✅ 已移除 {} 的止盈止损规则", code);
            } else {
                println!("⚠️  未找到 {} 的止盈止损规则", code);
            }
        }
    }
}

fn print_stop_status_rows(rows: &[StopStatusRow]) {
    if rows.is_empty() {
        println!("📭 没有可展示的止盈止损状态");
        return;
    }

    for row in rows {
        println!(
            "{} last_price={:?} anchor_price={:?} anchor_source={} loss_threshold={:?} profit_threshold={:?} trailing_pct={:?} highest_price={:?} eval_state={}",
            row.code,
            row.last_price,
            row.anchor_price,
            row.anchor_source
                .map(|source| source.as_str())
                .unwrap_or("-"),
            row.loss_threshold,
            row.profit_threshold,
            row.trailing_pct,
            row.highest_price,
            format_stop_eval_state(row.eval_state),
        );
    }
}

fn print_stop_history_rows(rows: &[StopHistoryEvent]) {
    if rows.is_empty() {
        println!("📭 没有可展示的止盈止损历史");
        return;
    }

    for row in rows {
        println!(
            "{} type={} trigger={:?} price={:?} anchor_price={:?} anchor_source={} ts={}",
            row.code,
            row.event_type.as_str(),
            row.trigger_kind.map(|kind| kind.as_str()),
            row.trigger_price,
            row.anchor_price,
            row.anchor_source.as_deref().unwrap_or("-"),
            row.created_at.to_rfc3339(),
        );
    }
}

fn format_stop_eval_state(state: crate::stop::StopEvalState) -> &'static str {
    match state {
        crate::stop::StopEvalState::Armed => "armed",
        crate::stop::StopEvalState::Triggered => "triggered",
        crate::stop::StopEvalState::AnchorMissing => "anchor_missing",
        crate::stop::StopEvalState::QuoteMissing => "quote_missing",
    }
}

fn print_trade_command_output(output: &TradeCommandOutput) {
    match output {
        TradeCommandOutput::AccountInitialized(account) => {
            println!("✅ 已初始化模拟账户 {}", account.account_id);
            print_trade_account_summary(account);
        }
        TradeCommandOutput::AccountReset(account) => {
            println!("✅ 已重置模拟账户 {}", account.account_id);
            print_trade_account_summary(account);
        }
        TradeCommandOutput::TradeExecuted(record) => print_trade_record(record),
        TradeCommandOutput::HistoryRows(rows) => print_trade_history_rows(rows),
        TradeCommandOutput::FeeRows(rows) => print_trade_fee_rows(rows),
        TradeCommandOutput::Overview(overview) => print_trade_overview(overview),
        TradeCommandOutput::PositionList(positions) => print_trade_positions(positions),
        TradeCommandOutput::PositionCurrentList(rows) => print_trade_current_positions(rows),
        TradeCommandOutput::Cash(snapshot) => print_trade_cash(snapshot),
    }
}

fn print_trade_account_summary(account: &PaperTradeAccount) {
    println!("初始资金: {}", account.initial_capital);
    println!("可用资金: {}", account.available_cash);
    println!("佣金费率: {}", account.fee_config.commission_rate);
    println!("最低佣金: {}", account.fee_config.commission_min);
    println!("印花税率: {}", account.fee_config.stamp_duty_rate);
    println!("过户费率: {}", account.fee_config.transfer_fee_rate);
}

fn print_trade_record(record: &TradeRecord) {
    println!(
        "✅ 已{} {} {} 股 @ {}",
        format_trade_side(record),
        record.code,
        record.volume,
        record.price
    );
    println!("成交额: {}", record.amount);
    println!("总费用: {}", record.total_fee);
}

fn format_trade_side(record: &TradeRecord) -> &'static str {
    match record.side {
        crate::trade::TradeSide::Buy => "买入",
        crate::trade::TradeSide::Sell => "卖出",
    }
}

fn format_trade_side_label(side: crate::trade::TradeSide) -> &'static str {
    match side {
        crate::trade::TradeSide::Buy => "买入",
        crate::trade::TradeSide::Sell => "卖出",
    }
}

fn print_trade_positions(positions: &[TradePosition]) {
    if positions.is_empty() {
        println!("📭 暂无持仓");
        return;
    }

    println!(
        "{:<10} {:<10} {:<14} {}",
        "代码", "数量", "持仓成本", "最新成交价"
    );
    println!("{}", "-".repeat(56));

    for position in positions {
        println!(
            "{:<10} {:<10} {:<14} {}",
            position.code, position.volume, position.avg_cost, position.last_trade_price
        );
    }
}

fn print_trade_cash(snapshot: &CashSnapshot) {
    println!("初始资金: {}", snapshot.initial_capital);
    println!("可用现金: {}", snapshot.available_cash);
    println!("持仓估值: {}", snapshot.estimated_position_value);
    println!("总资产估算: {}", snapshot.estimated_total_assets);
}

fn print_trade_history_rows(rows: &[TradeHistoryRow]) {
    if rows.is_empty() {
        println!("📭 暂无成交历史");
        return;
    }

    println!(
        "{:<20} {:<10} {:<6} {:<10} {:<8} {:<12} {:<10} {}",
        "时间", "代码", "方向", "价格", "数量", "成交额", "费用", "净现金影响"
    );
    println!("{}", "-".repeat(100));

    for row in rows {
        println!(
            "{:<20} {:<10} {:<6} {:<10} {:<8} {:<12} {:<10} {}",
            row.executed_at.format("%Y-%m-%d %H:%M:%S"),
            row.code,
            format_trade_side_label(row.side),
            row.price,
            row.volume,
            row.amount,
            row.total_fee,
            row.net_cash_impact
        );
    }
}

fn print_trade_fee_rows(rows: &[TradeFeeRow]) {
    if rows.is_empty() {
        println!("📭 暂无费用明细");
        return;
    }

    println!(
        "{:<20} {:<10} {:<6} {:<10} {:<10} {:<10} {}",
        "时间", "代码", "方向", "佣金", "印花税", "过户费", "总费用"
    );
    println!("{}", "-".repeat(90));

    for row in rows {
        println!(
            "{:<20} {:<10} {:<6} {:<10} {:<10} {:<10} {}",
            row.executed_at.format("%Y-%m-%d %H:%M:%S"),
            row.code,
            format_trade_side_label(row.side),
            row.commission,
            row.stamp_duty,
            row.transfer_fee,
            row.total_fee
        );
    }
}

fn print_trade_overview(overview: &TradeOverview) {
    println!("初始资金: {}", overview.initial_capital);
    println!("可用现金: {}", overview.available_cash);
    println!("账面持仓估值: {}", overview.booked_position_value);
    println!("账面总资产: {}", overview.booked_total_assets);
    println!("成交笔数: {}", overview.trade_count);
    println!("持仓数: {}", overview.holding_count);
    println!("累计买入额: {}", overview.total_buy_amount);
    println!("累计卖出额: {}", overview.total_sell_amount);
    println!("累计费用: {}", overview.total_fee);

    if let Some((resolved, total)) = overview.quote_coverage {
        println!("实时价格覆盖: {resolved}/{total}");
    }
    if let Some(value) = overview.live_position_value {
        println!("实时持仓估值: {}", value);
    }
    if let Some(value) = overview.live_total_assets {
        println!("实时总资产: {}", value);
    }
}

fn print_trade_current_positions(rows: &[TradePositionCurrentRow]) {
    if rows.is_empty() {
        println!("📭 暂无持仓");
        return;
    }

    println!(
        "{:<10} {:<10} {:<14} {:<12} {:<12} {:<12} {:<12} {}",
        "代码", "数量", "持仓成本", "最新成交价", "当前价", "当前市值", "浮盈亏", "价格状态"
    );
    println!("{}", "-".repeat(112));

    for row in rows {
        println!(
            "{:<10} {:<10} {:<14} {:<12} {:<12} {:<12} {:<12} {}",
            row.code,
            row.volume,
            row.avg_cost,
            row.last_trade_price,
            row.current_price
                .map(|value| value.to_string())
                .unwrap_or_else(|| "-".to_string()),
            row.current_market_value
                .map(|value| value.to_string())
                .unwrap_or_else(|| "-".to_string()),
            row.unrealized_pnl
                .map(|value| value.to_string())
                .unwrap_or_else(|| "-".to_string()),
            format_trade_quote_status(row.quote_status)
        );
    }
}

fn format_trade_quote_status(status: TradeQuoteStatus) -> &'static str {
    match status {
        TradeQuoteStatus::BookOnly => "book",
        TradeQuoteStatus::Live => "live",
        TradeQuoteStatus::Missing => "missing",
    }
}

fn print_stop_rules(rules: &[StopRule]) {
    if rules.is_empty() {
        println!("📭 暂无止盈止损规则");
        return;
    }

    println!(
        "{:<10} {:<12} {:<12} {:<10} {:<12} {}",
        "代码", "止损价", "止盈价", "追踪%", "最高价", "最近触发"
    );
    println!("{}", "-".repeat(80));

    for rule in rules {
        println!(
            "{:<10} {:<12} {:<12} {:<10} {:<12} {}",
            rule.code,
            format_optional_price(rule.stop_loss_price),
            format_optional_price(rule.take_profit_price),
            format_optional_price(rule.trailing_pct),
            format_optional_price(rule.highest_price),
            format_optional_timestamp(rule.last_triggered_at),
        );
    }
}

fn format_optional_price(value: Option<f64>) -> String {
    value
        .map(|value| format!("{:.2}", value))
        .unwrap_or_else(|| "-".to_string())
}

fn format_optional_timestamp(value: Option<chrono::DateTime<Utc>>) -> String {
    value
        .map(|value| value.to_rfc3339())
        .unwrap_or_else(|| "-".to_string())
}

fn print_north_flow_snapshot(snapshot: Option<&NorthFlowSnapshot>) {
    let Some(snapshot) = snapshot else {
        println!("📭 没有可展示的北向资金数据");
        return;
    };

    println!("日期: {}", snapshot.trade_date);
    println!("沪股通: {:.2}", snapshot.sh_amount);
    println!("深股通: {:.2}", snapshot.sz_amount);
    println!("合计: {:.2}", snapshot.total_amount);
    println!("余额: {:.2}", snapshot.balance);
}

fn print_market_sentiment_snapshot(snapshot: Option<&MarketSentimentSnapshot>) {
    let Some(snapshot) = snapshot else {
        println!("📭 没有可展示的市场情绪数据");
        return;
    };

    println!("日期: {}", snapshot.trade_date);
    println!("上涨: {}", snapshot.up_count);
    println!("下跌: {}", snapshot.down_count);
    println!("涨停: {}", snapshot.limit_up_count);
    println!("跌停: {}", snapshot.limit_down_count);
    println!("封板率: {:.2}", snapshot.seal_rate);
    println!("炸板率: {:.2}", snapshot.break_rate);
    println!("连板股: {}", snapshot.consecutive_board_count);
}

fn print_market_leader_rows(rows: &[LeaderRow]) {
    if rows.is_empty() {
        println!("📭 没有可展示的龙头股数据");
        return;
    }

    println!(
        "{:<10} {:<12} {:<12} {:<12} {}",
        "代码", "名称", "行业", "概念", "涨跌幅"
    );
    println!("{}", "-".repeat(72));

    for row in rows {
        println!(
            "{:<10} {:<12} {:<12} {:<12} {:.2}%",
            row.code,
            row.name,
            row.sector_name.as_deref().unwrap_or("-"),
            row.concept_name.as_deref().unwrap_or("-"),
            row.change_pct
        );
    }
}

fn print_market_overview(overview: &MarketOverview) {
    println!("== 市场概览 ==");
    println!("行业板块: {}", overview.top_sectors.len());
    println!("概念板块: {}", overview.top_concepts.len());

    match overview.north_flow.as_ref() {
        Some(snapshot) => println!("北向资金: {:.2}", snapshot.total_amount),
        None => println!("北向资金: -"),
    }

    match overview.sentiment.as_ref() {
        Some(snapshot) => println!("涨停数: {}", snapshot.limit_up_count),
        None => println!("涨停数: -"),
    }

    if !overview.top_sectors.is_empty() {
        println!();
        println!("Top 行业:");
        print_market_board_rows(&overview.top_sectors);
    }

    if !overview.top_concepts.is_empty() {
        println!();
        println!("Top 概念:");
        print_market_board_rows(&overview.top_concepts);
    }
}

fn print_monitor_command_output(output: &MonitorCommandOutput) {
    match output {
        MonitorCommandOutput::Watchlist {
            snapshot,
            triggered_stops,
        } => print_monitor_watchlist_snapshot(snapshot, triggered_stops),
        MonitorCommandOutput::AutomationIteration {
            run_mode: _,
            output,
        } => {
            print_monitor_watchlist_snapshot(&output.snapshot, &output.triggered_stops);
            if !output.new_events.is_empty() {
                println!();
                print_monitor_events(&output.new_events);
            }
        }
        MonitorCommandOutput::AlertAdded(alert) => println!(
            "✅ 已添加价格告警 #{} {} {} {:.2}",
            alert.id,
            alert.code,
            format_monitor_alert_kind(alert.kind),
            alert.target_price
        ),
        MonitorCommandOutput::AlertList(alerts) => print_monitor_alerts(alerts),
        MonitorCommandOutput::Config(config) => {
            println!("轮询间隔(秒): {}", config.interval_seconds);
            println!(
                "分组过滤: {}",
                config.watchlist_group.as_deref().unwrap_or("-")
            );
            println!("持久化事件: {}", config.persist_events);
            println!("最大历史条数: {}", config.max_event_history);
        }
        MonitorCommandOutput::EventList(rows) => print_monitor_events(rows),
        MonitorCommandOutput::ServiceConfig(config) => {
            println!("quantix_bin_path: {}", config.quantix_bin_path.display());
        }
        MonitorCommandOutput::ServiceStatus(summary) => {
            print_monitor_service_status_summary(summary);
        }
        MonitorCommandOutput::ServiceMessage(message) => println!("{}", message),
        MonitorCommandOutput::AlertRemoved { id, removed } => {
            if *removed {
                println!("✅ 已删除价格告警 #{}", id);
            } else {
                println!("⚠️  未找到价格告警 #{}", id);
            }
        }
    }
}

fn print_monitor_watchlist_snapshot(
    snapshot: &MonitorWatchlistSnapshot,
    triggered_stops: &[TriggeredStop],
) {
    if snapshot.rows.is_empty() {
        println!("📭 自选池为空");
        return;
    }

    println!(
        "{:<10} {:<12} {:<16} {:<10} {:<10} {}",
        "代码", "分组", "标签", "最新价", "涨跌幅", "备注"
    );
    println!("{}", "-".repeat(80));

    for row in &snapshot.rows {
        println!(
            "{:<10} {:<12} {:<16} {:<10} {:<10} {}",
            row.code,
            row.group,
            format_tags(&row.tags),
            row.last_price
                .map(|value| format!("{:.2}", value))
                .unwrap_or_else(|| "-".to_string()),
            row.change_pct
                .map(|value| format!("{:.2}%", value))
                .unwrap_or_else(|| "-".to_string()),
            row.note.as_deref().unwrap_or("-")
        );
    }

    if !snapshot.triggered_alerts.is_empty() {
        println!();
        println!("== 触发告警 ==");
        for alert in &snapshot.triggered_alerts {
            println!(
                "[#{}] {} 当前价 {:.2} {} {:.2}",
                alert.alert_id,
                alert.code,
                alert.current_price,
                format_monitor_alert_kind(alert.kind),
                alert.target_price
            );
        }
    }

    if !triggered_stops.is_empty() {
        println!();
        println!("== 止盈止损 ==");
        for triggered_stop in triggered_stops {
            println!("{}", format_triggered_stop_message(triggered_stop));
        }
    }

    if !snapshot.warnings.is_empty() {
        println!();
        println!("== 警告 ==");
        for warning in &snapshot.warnings {
            println!("{}", warning);
        }
    }
}

fn format_triggered_stop_message(triggered_stop: &TriggeredStop) -> String {
    match triggered_stop.kind {
        StopTriggerKind::Loss => format!(
            "{} 当前价 {:.2} 触发 stop-loss {:.2}",
            triggered_stop.code, triggered_stop.current_price, triggered_stop.threshold_price
        ),
        StopTriggerKind::Profit => format!(
            "{} 当前价 {:.2} 触发 take-profit {:.2}",
            triggered_stop.code, triggered_stop.current_price, triggered_stop.threshold_price
        ),
        StopTriggerKind::TrailingLoss => {
            let trailing_pct = triggered_stop
                .highest_price
                .map(|highest| (1.0 - triggered_stop.threshold_price / highest) * 100.0)
                .unwrap_or_default();
            match triggered_stop.highest_price {
                Some(highest_price) => format!(
                    "{} 当前价 {:.2} 触发 trailing-stop {:.2}% (highest {:.2})",
                    triggered_stop.code, triggered_stop.current_price, trailing_pct, highest_price
                ),
                None => format!(
                    "{} 当前价 {:.2} 触发 trailing-stop {:.2}",
                    triggered_stop.code,
                    triggered_stop.current_price,
                    triggered_stop.threshold_price
                ),
            }
        }
    }
}

fn print_monitor_alerts(alerts: &[PriceAlert]) {
    if alerts.is_empty() {
        println!("📭 暂无价格告警");
        return;
    }

    println!(
        "{:<6} {:<10} {:<8} {:<12} {}",
        "ID", "代码", "类型", "目标价", "最后触发"
    );
    println!("{}", "-".repeat(64));

    for alert in alerts {
        println!(
            "{:<6} {:<10} {:<8} {:<12} {}",
            alert.id,
            alert.code,
            format_monitor_alert_kind(alert.kind),
            format!("{:.2}", alert.target_price),
            alert
                .last_triggered_at
                .map(|value| value.to_rfc3339())
                .unwrap_or_else(|| "-".to_string())
        );
    }
}

fn print_monitor_events(rows: &[MonitorEventRow]) {
    if rows.is_empty() {
        println!("📭 暂无监控事件");
        return;
    }

    println!(
        "{:<20} {:<14} {:<8} {:<8} {:<10}",
        "时间", "类型", "代码", "价格", "模式"
    );
    println!("{}", "-".repeat(72));

    for row in rows {
        println!(
            "{:<20} {:<14} {:<8} {:<8} {:<10}",
            row.event_time.format("%Y-%m-%d %H:%M:%S"),
            format_monitor_event_type(row.event_type),
            row.code,
            row.price
                .map(|value| format!("{value:.2}"))
                .unwrap_or_else(|| "-".to_string()),
            format_monitor_run_mode(row.run_mode),
        );
        println!("  {}", row.message);
    }
}

fn print_monitor_service_status_summary(summary: &MonitorServiceStatusSummary) {
    println!(
        "installed: {}",
        if summary.installed { "yes" } else { "no" }
    );
    println!("enabled: {}", if summary.enabled { "yes" } else { "no" });
    println!("active: {}", summary.active);
    println!("unit_path: {}", summary.unit_path.display());
    println!("wrapper_path: {}", summary.wrapper_path.display());
    println!("quantix_bin_path: {}", summary.quantix_bin_path.display());

    if let Some(raw_status) = &summary.raw_status {
        println!();
        print!("{}", raw_status);
    }
}

fn build_unconfigured_monitor_service_status_summary() -> MonitorServiceStatusSummary {
    MonitorServiceStatusSummary {
        installed: false,
        enabled: false,
        active: "unconfigured".to_string(),
        unit_path: std::path::PathBuf::from("~/.config/systemd/user/quantix-monitor.service"),
        wrapper_path: std::path::PathBuf::from("~/.local/bin/quantix-monitor-run"),
        quantix_bin_path: std::path::PathBuf::from("<unconfigured>"),
        raw_status: None,
    }
}

fn format_monitor_alert_kind(kind: PriceAlertKind) -> &'static str {
    match kind {
        PriceAlertKind::Above => "above",
        PriceAlertKind::Below => "below",
    }
}

fn format_monitor_event_type(event_type: MonitorEventType) -> &'static str {
    match event_type {
        MonitorEventType::PriceAlert => "price-alert",
        MonitorEventType::StopLoss => "stop-loss",
        MonitorEventType::StopProfit => "stop-profit",
        MonitorEventType::TrailingStop => "trailing-stop",
    }
}

fn format_monitor_run_mode(run_mode: MonitorRunMode) -> &'static str {
    match run_mode {
        MonitorRunMode::Foreground => "foreground",
        MonitorRunMode::Daemon => "daemon",
    }
}

async fn run_screener_command(cmd: ScreenerCommands) -> Result<()> {
    let output = match cmd {
        ScreenerCommands::PresetList => {
            execute_screener_command_with_loader(
                ScreenerCommands::PresetList,
                NullDailyKlineLoader,
                create_watchlist_storage(),
            )
            .await?
        }
        ScreenerCommands::Run { .. } => {
            let loader = ClickHouseDailyKlineLoader::new(create_clickhouse_client().await?);
            execute_screener_command_with_loader(cmd, loader, create_watchlist_storage()).await?
        }
    };

    match output {
        ScreenerCommandOutput::PresetList(presets) => print_screener_preset_list(&presets),
        ScreenerCommandOutput::Rows(rows) => print_screener_rows(&rows),
    }

    Ok(())
}

struct ClickHouseDailyKlineLoader {
    client: ClickHouseClient,
}

impl ClickHouseDailyKlineLoader {
    fn new(client: ClickHouseClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl DailyKlineLoader for ClickHouseDailyKlineLoader {
    async fn load_daily_klines(
        &self,
        code: &str,
        lookback: usize,
    ) -> Result<Vec<crate::data::models::Kline>> {
        let mut rows = self
            .client
            .get_kline_data(code, "1d", None, None, None)
            .await?;

        if rows.len() > lookback {
            rows = rows[rows.len() - lookback..].to_vec();
        }

        Ok(rows)
    }
}

#[async_trait]
impl StrategyBarLoader for ClickHouseDailyKlineLoader {
    async fn load_daily_bars(
        &self,
        code: &str,
        limit: usize,
    ) -> Result<Vec<crate::data::models::Kline>> {
        let mut rows = self
            .client
            .get_kline_data(code, "1d", None, None, None)
            .await?;
        if rows.len() > limit {
            rows = rows[rows.len() - limit..].to_vec();
        }
        Ok(rows)
    }
}

struct NullDailyKlineLoader;

#[async_trait]
impl DailyKlineLoader for NullDailyKlineLoader {
    async fn load_daily_klines(
        &self,
        _code: &str,
        _lookback: usize,
    ) -> Result<Vec<crate::data::models::Kline>> {
        Ok(Vec::new())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScreenerPresetSpec {
    name: &'static str,
    params: &'static str,
    description: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ScreenerCommandOutput {
    PresetList(Vec<ScreenerPresetSpec>),
    Rows(Vec<ScreenRow>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScreenerRunRequest {
    universe: ScreenUniverse,
    presets: Vec<PresetInvocation>,
    options: ScreenRunOptions,
}

async fn execute_screener_command_with_loader<L>(
    cmd: ScreenerCommands,
    loader: L,
    storage: WatchlistStorage,
) -> Result<ScreenerCommandOutput>
where
    L: DailyKlineLoader,
{
    match cmd {
        ScreenerCommands::PresetList => {
            Ok(ScreenerCommandOutput::PresetList(screener_preset_specs()))
        }
        ScreenerCommands::Run {
            codes,
            watchlist,
            group,
            preset,
            limit,
            sort_by,
        } => {
            let request =
                build_screener_run_request(codes, watchlist, group, preset, limit, sort_by)?;
            let service = ScreenerService::new(loader, storage);
            let rows = service
                .run(request.universe, &request.presets, request.options)
                .await?;
            Ok(ScreenerCommandOutput::Rows(rows))
        }
    }
}

fn screener_preset_specs() -> Vec<ScreenerPresetSpec> {
    vec![
        ScreenerPresetSpec {
            name: "close_above_ma",
            params: "period=<n>",
            description: "收盘价高于均线",
        },
        ScreenerPresetSpec {
            name: "close_below_ma",
            params: "period=<n>",
            description: "收盘价低于均线",
        },
        ScreenerPresetSpec {
            name: "rsi_gte",
            params: "period=<n>,value=<x>",
            description: "RSI 大于等于阈值",
        },
        ScreenerPresetSpec {
            name: "rsi_lte",
            params: "period=<n>,value=<x>",
            description: "RSI 小于等于阈值",
        },
        ScreenerPresetSpec {
            name: "volume_ratio_gte",
            params: "window=<n>,value=<x>",
            description: "量比大于等于阈值",
        },
    ]
}

fn build_screener_run_request(
    codes: Option<String>,
    watchlist: bool,
    group: Option<String>,
    preset_specs: Vec<String>,
    limit: Option<usize>,
    sort_by: Option<String>,
) -> Result<ScreenerRunRequest> {
    let universe = match (codes, watchlist) {
        (Some(_), true) => {
            return Err(QuantixError::Other(
                "--codes 与 --watchlist 不能同时使用".to_string(),
            ));
        }
        (None, false) => {
            return Err(QuantixError::Other(
                "必须指定 --codes 或 --watchlist".to_string(),
            ));
        }
        (Some(codes), false) => {
            let codes = parse_codes_csv(&codes);
            if codes.is_empty() {
                return Err(QuantixError::Other("codes 不能为空".to_string()));
            }
            if group.is_some() {
                return Err(QuantixError::Other(
                    "--group 仅可与 --watchlist 一起使用".to_string(),
                ));
            }
            ScreenUniverse::Codes(codes)
        }
        (None, true) => ScreenUniverse::Watchlist { group },
    };

    if preset_specs.is_empty() {
        return Err(QuantixError::Other("至少需要一个 --preset".to_string()));
    }

    let presets = preset_specs
        .iter()
        .map(|spec| parse_preset_invocation(spec))
        .collect::<Result<Vec<_>>>()?;

    let sort_by = match sort_by.as_deref().unwrap_or("code") {
        "code" => ScreenSortBy::Code,
        "score" => ScreenSortBy::Score,
        other => {
            return Err(QuantixError::Other(format!(
                "不支持的 sort_by: {}，仅支持 code 或 score",
                other
            )));
        }
    };

    Ok(ScreenerRunRequest {
        universe,
        presets,
        options: ScreenRunOptions { limit, sort_by },
    })
}

fn parse_codes_csv(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(|item| item.to_string())
        .collect()
}

fn print_screener_preset_list(presets: &[ScreenerPresetSpec]) {
    println!("{:<20} {:<24} {}", "Preset", "参数", "说明");
    println!("{}", "-".repeat(72));

    for preset in presets {
        println!(
            "{:<20} {:<24} {}",
            preset.name, preset.params, preset.description
        );
    }
}

fn print_screener_rows(rows: &[ScreenRow]) {
    if rows.is_empty() {
        println!("📭 没有可展示的筛选结果");
        return;
    }

    println!("{:<10} {:<8} {:<12} {}", "代码", "命中", "评分", "详情");
    println!("{}", "-".repeat(96));

    for row in rows {
        println!(
            "{:<10} {:<8} {:<12} {}",
            row.code,
            if row.matched { "yes" } else { "no" },
            row.score.round_dp(4),
            row.details
                .iter()
                .map(format_screener_rule_detail)
                .collect::<Vec<_>>()
                .join(" | "),
        );
    }
}

fn format_screener_rule_detail(detail: &RuleMatchDetail) -> String {
    let status = if detail.matched { "Y" } else { "N" };

    match (
        detail.actual_value.as_ref(),
        detail.threshold_value.as_ref(),
        detail.reason.as_deref(),
    ) {
        (_, _, Some(reason)) => format!("{}:{}({})", status, detail.preset_name, reason),
        (Some(actual), Some(threshold), None) => {
            format!(
                "{}:{} {} / {}",
                status, detail.preset_name, actual, threshold
            )
        }
        _ => format!("{}:{}", status, detail.preset_name),
    }
}

/// 自选池命令
pub async fn run_watchlist_command(cmd: WatchlistCommands) -> Result<()> {
    let storage = create_watchlist_storage();
    let service = WatchlistService::default();

    match cmd {
        WatchlistCommands::Add { code, group } => {
            let mut store = storage.load_or_create()?;
            service.add(&mut store, &code, group.as_deref(), Utc::now())?;
            storage.save(&store)?;
            println!("✅ 已添加 {} 到自选池", code);
        }
        WatchlistCommands::Remove { code } => {
            let mut store = storage.load_or_create()?;
            service.remove(&mut store, &code, Utc::now())?;
            storage.save(&store)?;
            println!("✅ 已从自选池移除 {}", code);
        }
        WatchlistCommands::List {
            group,
            tag,
            with_price,
        } => {
            let store = load_watchlist_store_for_read(&storage)?;
            let items = service.list(&store, group.as_deref(), tag.as_deref());

            if with_price {
                let resolver = crate::watchlist::WatchlistResolver::new(
                    Arc::new(PostgresWatchlistNameLookup),
                    Arc::new(TdxWatchlistQuoteLookup),
                );
                let rows = resolver.resolve_rows(&items, true).await;
                print_watchlist_rows(&rows);
            } else {
                print_basic_watchlist_items(&items);
            }
        }
        WatchlistCommands::Move { code, group } => {
            let mut store = storage.load_or_create()?;
            service.move_code(&mut store, &code, &group, Utc::now())?;
            storage.save(&store)?;
            println!("✅ 已将 {} 移动到分组 {}", code, group);
        }
        WatchlistCommands::Group(group_cmd) => match group_cmd {
            WatchlistGroupCommands::Create { name } => {
                let mut store = storage.load_or_create()?;
                service.create_group(&mut store, &name, Utc::now())?;
                storage.save(&store)?;
                println!("✅ 已创建分组 {}", name);
            }
            WatchlistGroupCommands::List => {
                let store = load_watchlist_store_for_read(&storage)?;
                print_watchlist_groups(&store);
            }
        },
        WatchlistCommands::Tag(tag_cmd) => match tag_cmd {
            WatchlistTagCommands::Add { code, tag } => {
                let mut store = storage.load_or_create()?;
                service.add_tag(&mut store, &code, &tag, Utc::now())?;
                storage.save(&store)?;
                println!("✅ 已为 {} 添加标签 {}", code, tag);
            }
            WatchlistTagCommands::Remove { code, tag } => {
                let mut store = storage.load_or_create()?;
                service.remove_tag(&mut store, &code, &tag, Utc::now())?;
                storage.save(&store)?;
                println!("✅ 已为 {} 移除标签 {}", code, tag);
            }
            WatchlistTagCommands::List { code } => {
                let store = load_watchlist_store_for_read(&storage)?;
                let entry = store
                    .entries
                    .get(&code)
                    .ok_or_else(|| QuantixError::Other(format!("股票不存在: {}", code)))?;
                print_watchlist_tags(&code, &entry.tags);
            }
        },
        WatchlistCommands::History { code, limit } => {
            let store = load_watchlist_store_for_read(&storage)?;
            let events = service.history(&store, code.as_deref(), Some(limit));
            print_watchlist_history(&events);
        }
    }

    Ok(())
}

fn create_watchlist_storage() -> WatchlistStorage {
    let runtime = CliRuntime::load();
    WatchlistStorage::new(runtime.watchlist_path)
}

fn create_trade_store() -> JsonPaperTradeStore {
    let runtime = CliRuntime::load();
    JsonPaperTradeStore::new(runtime.trade_path)
}

fn create_risk_store() -> JsonRiskStore {
    let runtime = CliRuntime::load();
    JsonRiskStore::new(runtime.risk_path)
}

async fn sync_risk_from_trade_store<TradeStore, RiskStore>(
    trade_store: &TradeStore,
    risk_service: &RiskService<RiskStore>,
) -> Result<()>
where
    TradeStore: PaperTradeStore,
    RiskStore: crate::risk::RiskStore,
{
    let account = load_initialized_trade_account(trade_store).await?;
    let snapshot = build_risk_account_snapshot(&account);
    risk_service
        .sync_after_trade_snapshot(&snapshot, Utc::now())
        .await?;
    Ok(())
}

async fn load_initialized_trade_account<Store>(trade_store: &Store) -> Result<PaperTradeAccount>
where
    Store: PaperTradeStore,
{
    trade_store
        .load_state()
        .await?
        .and_then(|state| state.account)
        .ok_or_else(|| {
            QuantixError::Other("trade account 尚未初始化，请先运行 trade init".to_string())
        })
}

async fn load_trade_quote_prices<Q>(
    state: &PaperTradeState,
    quote_lookup: &Q,
) -> BTreeMap<String, Decimal>
where
    Q: WatchlistQuoteLookup,
{
    let Some(account) = &state.account else {
        return BTreeMap::new();
    };

    let codes: Vec<String> = account.positions.keys().cloned().collect();
    quote_lookup
        .lookup_quotes(&codes)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|(code, snapshot)| (code, snapshot.latest_price))
        .collect()
}

fn build_risk_account_snapshot(account: &PaperTradeAccount) -> RiskAccountSnapshot {
    let positions: Vec<(String, rust_decimal::Decimal)> = account
        .positions
        .values()
        .map(|position| {
            (
                position.code.clone(),
                rust_decimal::Decimal::from(position.volume) * position.last_trade_price,
            )
        })
        .collect();
    let position_value = positions
        .iter()
        .fold(rust_decimal::Decimal::ZERO, |acc, (_, value)| acc + *value);

    RiskAccountSnapshot::new(
        account.account_id.clone(),
        account.available_cash + position_value,
        positions,
    )
}

fn build_projected_buy_impact(
    account: &PaperTradeAccount,
    request: &TradeOrderRequest,
) -> crate::risk::ProjectedBuyImpact {
    let current_position_value = account
        .positions
        .get(&request.code)
        .map(|position| rust_decimal::Decimal::from(position.volume) * position.last_trade_price)
        .unwrap_or(rust_decimal::Decimal::ZERO);

    crate::risk::ProjectedBuyImpact::new(
        request.code.clone(),
        current_position_value + request.price * rust_decimal::Decimal::from(request.volume),
        build_risk_account_snapshot(account).total_assets,
    )
}

fn load_watchlist_store_for_read(storage: &WatchlistStorage) -> Result<WatchlistStore> {
    Ok(storage.load()?.unwrap_or_default())
}

fn print_basic_watchlist_items(items: &[crate::watchlist::WatchlistListItem]) {
    if items.is_empty() {
        println!("📭 自选池为空");
        return;
    }

    println!("{:<10} {:<12} {}", "代码", "分组", "标签");
    println!("{}", "-".repeat(48));

    for item in items {
        println!(
            "{:<10} {:<12} {}",
            item.code,
            item.group,
            format_tags(&item.tags)
        );
    }
}

fn print_watchlist_rows(rows: &[WatchlistDisplayRow]) {
    if rows.is_empty() {
        println!("📭 自选池为空");
        return;
    }

    println!(
        "{:<10} {:<12} {:<12} {:<16} {:<12} {}",
        "代码", "名称", "分组", "标签", "最新价", "涨跌幅"
    );
    println!("{}", "-".repeat(84));

    for row in rows {
        let price = row
            .latest_price
            .map(|value| value.to_string())
            .unwrap_or_else(|| "-".to_string());
        let change_pct = row
            .price_change_pct
            .map(|value| format!("{}%", value))
            .unwrap_or_else(|| "-".to_string());

        println!(
            "{:<10} {:<12} {:<12} {:<16} {:<12} {}",
            row.code,
            row.name.as_deref().unwrap_or("-"),
            row.group,
            format_tags(&row.tags),
            price,
            change_pct
        );
    }
}

fn print_watchlist_groups(store: &WatchlistStore) {
    let mut groups: Vec<(&String, usize)> = store
        .groups
        .iter()
        .map(|(name, codes)| (name, codes.len()))
        .collect();
    groups.sort_by(|left, right| left.0.cmp(right.0));

    println!("{:<16} {}", "分组", "数量");
    println!("{}", "-".repeat(28));

    for (name, size) in groups {
        println!("{:<16} {}", name, size);
    }
}

fn print_watchlist_tags(code: &str, tags: &[String]) {
    println!("🏷️  {} 标签: {}", code, format_tags(tags));
}

fn print_watchlist_history(events: &[WatchlistHistoryEvent]) {
    if events.is_empty() {
        println!("🕘 暂无历史记录");
        return;
    }

    println!(
        "{:<22} {:<12} {:<10} {:<12} {}",
        "时间", "动作", "代码", "分组", "标签"
    );
    println!("{}", "-".repeat(72));

    for event in events {
        println!(
            "{:<22} {:<12} {:<10} {:<12} {}",
            event.ts.to_rfc3339(),
            format!("{:?}", event.action),
            event.code.as_deref().unwrap_or("-"),
            event.group.as_deref().unwrap_or("-"),
            event.tag.as_deref().unwrap_or("-")
        );
    }
}

fn format_tags(tags: &[String]) -> String {
    if tags.is_empty() {
        "-".to_string()
    } else {
        tags.join(",")
    }
}

/// 计算技术指标
async fn calculate_indicators(code: String, indicators_str: String) -> Result<()> {
    println!("💹 计算技术指标");
    println!("  代码: {}", code);
    println!("  指标: {}", indicators_str);

    // 连接 ClickHouse
    let client = create_clickhouse_client().await?;

    // 获取历史数据
    let klines = client
        .get_kline_data(&code, "1d", None, None, Some(1000))
        .await?;

    if klines.is_empty() {
        println!("⚠️  未找到数据");
        return Ok(());
    }

    // 转换为 BatchKlineData
    let batch_data = from_kline_vec(&klines);

    // 创建计算器
    let calc = PolarsCalculator::new();

    // 解析指标列表
    let indicators: Vec<&str> = indicators_str.split(',').collect();

    // 批量计算
    let results = calc.calculate_batch(&batch_data, &indicators);

    println!("\n📊 计算结果:");
    println!(
        "{:<12} {:<20} {:<15} {:<15}",
        "日期", "收盘价", "指标", "值"
    );
    println!("{}", "-".repeat(65));

    for (i, kline) in klines.iter().enumerate().take(20) {
        println!("{:<12} {:<20.2}", kline.date, kline.close,);

        for indicator in &indicators {
            if let Some(values) = results.get(*indicator) {
                if let Some(value) = values.get(i) {
                    println!(
                        "{:<12} {:<20} {:<15} {:<15}",
                        "",
                        "",
                        indicator,
                        value.map(|v| v.to_string()).unwrap_or("N/A".to_string()),
                    );
                }
            }
        }
    }

    Ok(())
}

/// 显示回测报告
async fn show_backtest_report(id: String) -> Result<()> {
    println!("📊 回测报告: {}", id);
    println!("💡 提示: 运行 'quantix strategy run' 生成新报告");
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PatternCandleRow {
    label: String,
    candle: CandleInput,
}

async fn analyze_candle_patterns(
    candle_specs: Vec<String>,
    code: Option<String>,
    tdx_root: Option<String>,
    market: Option<String>,
    day_file: Option<String>,
    start: Option<String>,
    end: Option<String>,
    period_type: String,
    limit: usize,
    reference: Option<String>,
    previous_close: bool,
) -> Result<()> {
    let rows = load_pattern_candle_rows(
        candle_specs,
        code,
        tdx_root,
        market,
        day_file,
        start,
        end,
        period_type,
        limit,
    )
    .await?;
    let candles: Vec<CandleInput> = rows.iter().map(|row| row.candle).collect();
    let policy = build_reference_policy(reference, previous_close)?;
    let references = sequence_references(&candles, &policy)?;
    let config = PatternConfig {
        epsilon: dec!(0.0001),
    };
    let patterns = recognize_sequence(&candles, &policy, &config)
        .map_err(|err| QuantixError::Other(format!("K线形态识别失败: {:?}", err)))?;

    println!("📈 K线形态识别");
    println!("  数量: {}", candles.len());
    println!(
        "  参考策略: {}",
        if previous_close {
            "前一根收盘价"
        } else {
            "显式参考价"
        }
    );

    for (idx, pattern) in patterns.iter().enumerate() {
        let row = match policy {
            ReferencePricePolicy::Explicit(_) => &rows[idx],
            ReferencePricePolicy::PreviousClose => &rows[idx + 1],
        };
        let candle = &row.candle;

        println!(
            "\n#{} {} O={} H={} L={} C={} P={}",
            idx + 1,
            row.label,
            candle.open,
            candle.high,
            candle.low,
            candle.close,
            references[idx],
        );

        match pattern.canonical_case {
            Some(case_id) => println!("  标准形态: {} {}", case_id.id(), case_id.display_name()),
            None => println!("  标准形态: 扩展形态"),
        }

        println!(
            "  偏向: {}",
            match pattern.bias {
                MarketBias::Bullish => "看多",
                MarketBias::Bearish => "看空",
                MarketBias::Neutral => "看平",
            }
        );
        println!(
            "  扩展结构: {:?} / {:?} / upper_shadow={} / lower_shadow={}",
            pattern.extended.reference_span,
            pattern.extended.body_type,
            pattern.extended.has_upper_shadow,
            pattern.extended.has_lower_shadow
        );
    }

    Ok(())
}

async fn load_pattern_candle_rows(
    candle_specs: Vec<String>,
    code: Option<String>,
    tdx_root: Option<String>,
    market: Option<String>,
    day_file: Option<String>,
    start: Option<String>,
    end: Option<String>,
    period_type: String,
    limit: usize,
) -> Result<Vec<PatternCandleRow>> {
    if !candle_specs.is_empty() {
        return parse_candle_specs(&candle_specs);
    }

    let start_date = start
        .as_ref()
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y%m%d").ok());
    let end_date = end
        .as_ref()
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y%m%d").ok());

    if let Some(day_file) = day_file {
        if period_type != "1d" {
            return Err(QuantixError::Other(
                "day 文件当前只支持 1d 周期".to_string(),
            ));
        }
        return pattern_rows_from_day_file(day_file, start_date, end_date, limit);
    }

    if let Some(tdx_root) = tdx_root {
        if period_type != "1d" {
            return Err(QuantixError::Other(
                "TDX day 根目录当前只支持 1d 周期".to_string(),
            ));
        }
        let code = code.ok_or_else(|| {
            QuantixError::Other("使用 --tdx-root 时必须同时提供 --code".to_string())
        })?;
        let day_file = resolve_tdx_day_file_path(tdx_root, &code, market.as_deref())?;
        return pattern_rows_from_day_file(day_file, start_date, end_date, limit);
    }

    let code = code.ok_or_else(|| {
        QuantixError::Other("缺少 K线输入，请提供 --candle、--day-file 或 --code".to_string())
    })?;

    let client = create_clickhouse_client().await?;
    let klines = client
        .get_kline_data(&code, &period_type, start_date, end_date, Some(limit))
        .await?;

    if klines.is_empty() {
        return Err(QuantixError::Other(format!("未找到 {} 的 K线数据", code)));
    }

    Ok(pattern_rows_from_klines(&klines))
}

fn build_reference_policy(
    reference: Option<String>,
    previous_close: bool,
) -> Result<ReferencePricePolicy> {
    if previous_close {
        return Ok(ReferencePricePolicy::PreviousClose);
    }

    let reference = reference.ok_or_else(|| {
        QuantixError::Other("缺少参考价，请提供 --reference 或 --previous-close".to_string())
    })?;

    let value = Decimal::from_str(&reference)
        .map_err(|e| QuantixError::Other(format!("参考价格式非法: {}", e)))?;

    Ok(ReferencePricePolicy::Explicit(value))
}

fn parse_candle_specs(specs: &[String]) -> Result<Vec<PatternCandleRow>> {
    specs
        .iter()
        .enumerate()
        .map(|(idx, spec)| {
            Ok(PatternCandleRow {
                label: format!("manual-{}", idx + 1),
                candle: parse_candle_spec(spec)?,
            })
        })
        .collect()
}

fn parse_candle_spec(spec: &str) -> Result<CandleInput> {
    let parts: Vec<&str> = spec.split(',').map(str::trim).collect();
    if parts.len() != 4 {
        return Err(QuantixError::Other(format!(
            "K线格式非法: {}，期望 o,h,l,c",
            spec
        )));
    }

    let parse_decimal = |value: &str| {
        Decimal::from_str(value).map_err(|e| QuantixError::Other(format!("价格格式非法: {}", e)))
    };

    Ok(CandleInput {
        open: parse_decimal(parts[0])?,
        high: parse_decimal(parts[1])?,
        low: parse_decimal(parts[2])?,
        close: parse_decimal(parts[3])?,
    })
}

fn sequence_references(
    candles: &[CandleInput],
    policy: &ReferencePricePolicy,
) -> Result<Vec<Decimal>> {
    match policy {
        ReferencePricePolicy::Explicit(value) => Ok(vec![*value; candles.len()]),
        ReferencePricePolicy::PreviousClose => {
            if candles.len() < 2 {
                return Err(QuantixError::Other(
                    "使用 --previous-close 时至少需要两根 K线".to_string(),
                ));
            }

            Ok(candles.windows(2).map(|pair| pair[0].close).collect())
        }
    }
}

fn pattern_rows_from_klines(klines: &[Kline]) -> Vec<PatternCandleRow> {
    klines
        .iter()
        .map(|kline| PatternCandleRow {
            label: kline.date.to_string(),
            candle: CandleInput {
                open: kline.open,
                high: kline.high,
                low: kline.low,
                close: kline.close,
            },
        })
        .collect()
}

fn pattern_rows_from_day_file(
    day_file: impl AsRef<Path>,
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
    limit: usize,
) -> Result<Vec<PatternCandleRow>> {
    let path = day_file.as_ref();
    let code = infer_tdx_code_from_day_file_path(path)?;
    let mut klines = TdxDayFile::to_klines(code, path, crate::data::models::AdjustType::None)?;

    if let Some(start_date) = start {
        klines.retain(|kline| kline.date >= start_date);
    }
    if let Some(end_date) = end {
        klines.retain(|kline| kline.date <= end_date);
    }
    if limit > 0 && klines.len() > limit {
        klines = klines[klines.len() - limit..].to_vec();
    }

    Ok(pattern_rows_from_klines(&klines))
}

fn infer_tdx_code_from_day_file_path(path: impl AsRef<Path>) -> Result<u32> {
    let stem = path
        .as_ref()
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| QuantixError::Other("无法从 day 文件路径解析股票代码".to_string()))?;

    let digits: String = stem.chars().filter(|ch| ch.is_ascii_digit()).collect();
    if digits.len() != 6 {
        return Err(QuantixError::Other(format!(
            "无法从 day 文件名解析 6 位股票代码: {}",
            path.as_ref().display()
        )));
    }

    digits
        .parse::<u32>()
        .map_err(|e| QuantixError::Other(format!("股票代码解析失败: {}", e)))
}

fn resolve_tdx_day_file_path(
    tdx_root: impl AsRef<Path>,
    code: &str,
    market: Option<&str>,
) -> Result<std::path::PathBuf> {
    let root = tdx_root.as_ref();

    if let Some(market) = market {
        let market = market.to_ascii_lowercase();
        let path = root
            .join("vipdoc")
            .join(&market)
            .join("lday")
            .join(format!("{}{}.day", market, code));
        if path.exists() {
            return Ok(path);
        }
        return Err(QuantixError::Other(format!(
            "未找到指定市场的 day 文件: {}",
            path.display()
        )));
    }

    let matches: Vec<std::path::PathBuf> = ["sh", "sz", "bj", "ds"]
        .iter()
        .map(|market| {
            root.join("vipdoc")
                .join(market)
                .join("lday")
                .join(format!("{}{}.day", market, code))
        })
        .filter(|path| path.exists())
        .collect();

    match matches.as_slice() {
        [single] => Ok(single.clone()),
        [] => Err(QuantixError::Other(format!(
            "未找到 {} 对应的 day 文件，请确认 --tdx-root 或补充 --market",
            code
        ))),
        many => Err(QuantixError::Other(format!(
            "代码 {} 在多个市场目录匹配到多个 day 文件: {}，请补充 --market",
            code,
            many.iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ))),
    }
}

/// 状态命令
pub async fn run_status(health: bool) -> Result<()> {
    if health {
        println!("🏥 检查数据库连接...");

        // 尝试连接 ClickHouse
        match create_clickhouse_client().await {
            Ok(_) => println!("  ✅ ClickHouse: 连接正常"),
            Err(e) => println!("  ❌ ClickHouse: 连接失败 - {}", e),
        }
    } else {
        println!("📊 Quantix CLI 状态:");
        println!();
        println!("  版本: 0.1.0");
        println!("  模式: 共享数据库模式");
        println!("  Phase: 14/14 (CLI 命令实现)");
        println!();
        println!("  📦 已完成模块:");
        println!("    ✅ 数据采集基础 (Phase 1)");
        println!("    ✅ 竞价分析 (Phase 2)");
        println!("    ✅ K线管理 (Phase 3)");
        println!("    ✅ 回测引擎 (Phase 4)");
        println!("    ✅ 任务调度 (Phase 5)");
        println!("    ✅ TDX解析 (Phase 6)");
        println!("    ✅ GBBQ存储 (Phase 7)");
        println!("    ✅ 多周期查询 (Phase 8)");
        println!("    ✅ 东方财富 (Phase 9)");
        println!("    ✅ 批量优化 (Phase 10)");
        println!("    ✅ WebSocket (Phase 11)");
        println!("    ✅ 技术指标 (Phase 12)");
        println!("    ✅ Polars适配 (Phase 13)");
        println!("    ✅ CLI命令 (Phase 14)");
        println!();
    }

    Ok(())
}

#[cfg(test)]
mod tests;

// === 菜单辅助函数 ===

/// 数据同步菜单
async fn run_data_sync_menu() -> Result<()> {
    let items = vec![
        "同步股票列表",
        "同步实时行情",
        "同步竞价数据",
        "同步K线数据",
        "返回",
    ];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&items)
        .default(0)
        .interact()
        .map_err(|e| QuantixError::Other(format!("菜单选择失败: {}", e)))?;

    match selection {
        0 => println!("📥 同步股票列表..."),
        1 => println!("📡 同步实时行情..."),
        2 => println!("💰 同步竞价数据..."),
        3 => println!("📊 同步K线数据..."),
        4 => {}
        _ => {}
    }

    Ok(())
}

/// 策略菜单
async fn run_strategy_menu() -> Result<()> {
    let items = vec!["运行策略", "查看策略列表", "返回"];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&items)
        .default(0)
        .interact()
        .map_err(|e| QuantixError::Other(format!("菜单选择失败: {}", e)))?;

    match selection {
        0 => {
            let code = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("股票代码")
                .default("000001".to_string())
                .interact()
                .map_err(|e| QuantixError::Other(format!("输入失败: {}", e)))?;

            run_ma_cross_backtest(Some(code)).await?;
        }
        1 => list_strategies().await?,
        2 => {}
        _ => {}
    }

    Ok(())
}

/// 回测菜单
async fn run_backtest_menu() -> Result<()> {
    let items = vec!["新建回测", "查看历史回测", "返回"];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&items)
        .default(0)
        .interact()
        .map_err(|e| QuantixError::Other(format!("菜单选择失败: {}", e)))?;

    match selection {
        0 => {
            let code = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("股票代码")
                .default("000001".to_string())
                .interact()
                .map_err(|e| QuantixError::Other(format!("输入失败: {}", e)))?;

            run_ma_cross_backtest(Some(code)).await?;
        }
        1 => println!("📋 历史回测列表..."),
        2 => {}
        _ => {}
    }

    Ok(())
}

/// 任务菜单
async fn run_task_menu() -> Result<()> {
    let items = vec!["查看任务列表", "启动调度器", "返回"];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&items)
        .default(0)
        .interact()
        .map_err(|e| QuantixError::Other(format!("菜单选择失败: {}", e)))?;

    match selection {
        0 => list_tasks().await?,
        1 => {
            start_task_scheduler(false).await?;
        }
        2 => {}
        _ => {}
    }

    Ok(())
}

/// 分析菜单
async fn run_analysis_menu() -> Result<()> {
    let items = vec!["计算技术指标", "返回"];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&items)
        .default(0)
        .interact()
        .map_err(|e| QuantixError::Other(format!("菜单选择失败: {}", e)))?;

    match selection {
        0 => {
            let code = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("股票代码")
                .default("000001".to_string())
                .interact()
                .map_err(|e| QuantixError::Other(format!("输入失败: {}", e)))?;

            let indicators = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("指标列表 (逗号分隔)")
                .default("ma5,ma20,rsi14".to_string())
                .interact()
                .map_err(|e| QuantixError::Other(format!("输入失败: {}", e)))?;

            calculate_indicators(code, indicators).await?;
        }
        1 => {}
        _ => {}
    }

    Ok(())
}

/// 导出菜单
async fn run_export_menu() -> Result<()> {
    let items = vec!["导出为 CSV", "导出为 Parquet", "返回"];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&items)
        .default(0)
        .interact()
        .map_err(|e| QuantixError::Other(format!("菜单选择失败: {}", e)))?;

    match selection {
        0 => {
            let code = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("股票代码")
                .default("000001".to_string())
                .interact()
                .map_err(|e| QuantixError::Other(format!("输入失败: {}", e)))?;

            export_data(code, "csv".to_string(), "./data".to_string()).await?;
        }
        1 => {
            let code = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("股票代码")
                .default("000001".to_string())
                .interact()
                .map_err(|e| QuantixError::Other(format!("输入失败: {}", e)))?;

            export_data(code, "parquet".to_string(), "./data".to_string()).await?;
        }
        2 => {}
        _ => {}
    }

    Ok(())
}

