pub(crate) use super::{
    AiCommands, AlgoCommands, AnalyzeCommands, AnomalyCommands, DataCommands, DataSourceCommands,
    DataSourceKind, ExecutionBridgeCommands, ExecutionCommands, ExecutionConfigCommands,
    ExecutionDaemonCommands, ExecutionQmtCommands, FundamentalCommands, ImportCommands,
    MarketCommands, MonitorAlertCommands, MonitorCommands, MonitorConfigCommands,
    MonitorDaemonCommands, MonitorEventCommands, MonitorServiceCommands,
    MonitorServiceConfigCommands, NewsCommands, NotifyCommands, PerformanceCommands, RiskCommands,
    RiskLockCommands, RiskRuleCommands, ScreenerCommands, SentimentCommands, StopCommands,
    StrategyCommands, StrategyConfigCommands, StrategyDaemonCommands, StrategyRequestCommands,
    StrategyServiceCommands, StrategyServiceConfigCommands, StrategySignalCommands,
    StrengthStockMetric, TaskCommands, TradeCommands, WatchlistCommands, WatchlistGroupCommands,
    WatchlistTagCommands,
};
pub(crate) use crate::ai::providers::OpenAICompatAdapter;
pub(crate) use crate::ai::{DecisionEngine, LlmConfig};
pub(crate) use crate::analysis::backtest::{BacktestConfig, BacktestEngine};
pub(crate) use crate::analysis::candle_patterns::{
    CandleInput, MarketBias, PatternConfig, ReferencePricePolicy, recognize_sequence,
};
pub(crate) use crate::analysis::polars_adapter::{PolarsCalculator, from_kline_vec};
pub(crate) use crate::bridge::client::BridgeHttpClient;
pub(crate) use crate::cli::commands::BacktestCommands;
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
pub(crate) use crate::execution::qmt_live_gate::{
    QMT_LIVE_BRIDGE_COMMAND, QMT_LIVE_BRIDGE_MODE_REQUIREMENT,
};
pub(crate) use crate::execution::runtime_store::StrategyRuntimeStore;
pub(crate) use crate::fundamental::dragon_tiger::DragonTigerFetcher;
pub(crate) use crate::fundamental::earnings::EarningsFetcher;
pub(crate) use crate::fundamental::institution::InstitutionFetcher;
pub(crate) use crate::fundamental::valuation::ValuationFetcher;
pub(crate) use crate::fundamental::{EastMoneyFundamentalProvider, FundamentalProvider};
pub(crate) use crate::market::sentiment::SentimentAggregator;
pub(crate) use crate::market::sentiment::types::SentimentTrend;
pub(crate) use crate::market::{
    BoardRankRow, BoardSortBy, BoardType, LeaderFilter, LeaderRow, MarketDataReader,
    MarketFoundationSummary, MarketOverview, MarketSentimentSnapshot, MarketService,
    MarketStrengthReport, NorthFlowSnapshot, StrongSectorStockRow,
    analyze_market_strength_with_reader, load_market_analysis_foundation,
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
mod ai;
mod algo;
mod analyze_handler;
mod anomaly;
mod app_shell;
mod backtest_handler;
mod data_handler;
mod execution_handler;
mod fundamental;
mod import;
mod market_handler;
mod market_output;
mod monitor_handler;
mod monitor_output;
mod news;
mod notify;
mod performance_handler;
mod risk;
mod screener_handler;
mod sentiment;
mod shared_support;
mod stop_handler;
mod stop_output;
mod strategy_handler;
mod trade_handler;
mod trade_output;
mod watchlist_handler;

pub use self::account::run_account_command;
pub use self::ai::run_ai_command;
pub use self::algo::run_algo_command;
pub(crate) use self::analyze_handler::{analyze_candle_patterns, calculate_indicators};
#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use self::analyze_handler::{
    infer_tdx_code_from_day_file_path, parse_candle_spec, pattern_rows_from_day_file,
    pattern_rows_from_klines, resolve_tdx_day_file_path, sequence_references,
};
pub use self::anomaly::run_anomaly_command;
#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use self::app_shell::{
    ensure_task_command_supported_for_p0, foundation_p0_task_template_descriptions,
};
pub use self::app_shell::{
    run_analyze_command, run_data_command, run_init, run_simple_menu, run_status, run_task_command,
    run_tui_menu,
};
pub(crate) use self::backtest_handler::{
    StoredBacktestReport, read_backtest_report, read_backtest_reports,
};
pub(crate) use self::backtest_handler::{run_backtest_command, show_backtest_report};
use self::data_handler::{
    add_data_source, export_data, import_market_fundamentals, list_data_sources, query_kline_data,
    set_default_data_source, test_data_source,
};
pub(crate) use self::execution_handler::*;
pub use self::fundamental::run_fundamental_command;
pub use self::import::run_import_command;
#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use self::market_handler::MarketCommandOutput;
#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use self::market_handler::execute_market_command_with_reader;
#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use self::market_handler::execute_market_command_with_runtime;
#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use self::market_handler::execute_market_command_with_test_payloads;
pub use self::market_handler::run_market_command;
#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use self::market_handler::{
    build_leader_filter, parse_board_sort_by, parse_market_date,
};
use self::market_output::{
    print_market_board_rows, print_market_foundation_summary, print_market_leader_rows,
    print_market_overview, print_market_sentiment_snapshot, print_market_strength_report,
    print_north_flow_snapshot,
};
pub use self::monitor_handler::run_monitor_command;
#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use self::monitor_handler::{
    ConfiguredMonitorWatchlistReader, MonitorAlertAddRequest, MonitorServiceInstallerOps,
    TdxMonitorQuoteReader, build_monitor_alert_request, create_configured_monitor_runner,
    create_monitor_alert_store, evaluate_stop_rules_for_snapshot,
    execute_monitor_command_with_service, execute_monitor_command_with_stop_store,
    execute_monitor_config_command_with_store, execute_monitor_event_command_with_store,
    execute_monitor_iteration_with_runner, execute_monitor_service_command,
    execute_monitor_service_command_with_installer,
    execute_monitor_service_config_command_with_store, monitor_alert_id_to_i64,
    parse_monitor_event_type, persist_triggered_monitor_alerts, run_monitor_loop,
    validate_monitor_watchlist_command,
};
pub(crate) use self::monitor_handler::{MonitorCommandOutput, create_stop_rule_store};
use self::monitor_output::{
    build_unconfigured_monitor_service_status_summary, print_monitor_command_output,
};
pub use self::news::run_news_command;
pub use self::notify::run_notify_command;
pub(crate) use self::performance_handler::run_performance_command;
pub use self::risk::run_risk_command;
pub(crate) use self::screener_handler::ClickHouseDailyKlineLoader;
use self::screener_handler::run_screener_command;
#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use self::screener_handler::{
    ScreenerCommandOutput, execute_screener_command_with_loader,
};
pub use self::sentiment::run_sentiment_command;
pub(crate) use self::shared_support::{
    build_avg_cost_map_from_trade_store, build_stop_status_rows, build_trade_init_request,
    build_trade_order_request, decimal_to_f64, ensure_watchlist_contains_code, filter_stop_rules,
    format_stop_eval_state, parse_stop_history_date, parse_stop_history_event_type, patch_value,
    remap_trade_request_error, resolve_stop_reference_price,
};
pub(crate) use self::stop_handler::StopCommandOutput;
pub use self::stop_handler::run_stop_command;
#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use self::stop_handler::{
    execute_stop_command_with_context, execute_stop_command_with_service,
};
use self::stop_output::print_stop_command_output;
pub(crate) use self::strategy_handler::*;
pub(crate) use self::trade_handler::TradeCommandOutput;
pub use self::trade_handler::run_trade_command;
#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use self::trade_handler::{
    execute_trade_command_with_quote_lookup, execute_trade_command_with_risk,
    execute_trade_command_with_service,
};
use self::trade_output::print_trade_command_output;
pub use self::watchlist_handler::run_watchlist_command;
pub(crate) use self::watchlist_handler::{
    create_watchlist_storage, format_tags, load_watchlist_store_for_read,
};

async fn create_clickhouse_client() -> Result<ClickHouseClient> {
    let runtime = CliRuntime::load();
    ClickHouseClient::from_settings(&runtime.clickhouse).await
}

/// 策略命令
pub async fn run_strategy_command(cmd: StrategyCommands) -> Result<()> {
    match cmd {
        StrategyCommands::Create {
            id,
            name,
            code,
            params,
            disabled,
        } => {
            execute_strategy_create(id, name, code, params, disabled).await?;
        }
        StrategyCommands::Update {
            id,
            name,
            code,
            params,
            enable,
            disable,
        } => {
            execute_strategy_update(id, name, code, params, enable, disable).await?;
        }
        StrategyCommands::Delete { id } => {
            execute_strategy_delete(id).await?;
        }
        StrategyCommands::Run { name, mode, code } => {
            run_strategy(name, mode, code).await?;
        }
        StrategyCommands::List => {
            list_strategies().await?;
        }
        StrategyCommands::Show { name, id } => {
            show_strategy_or_instance(name, id).await?;
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
                execute_strategy_request_list(
                    status.as_deref(),
                    target_mode.as_deref(),
                    target_account.as_deref(),
                    limit,
                    stats,
                )
                .await?;
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
pub(crate) struct StrategyRunSummary {
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
        ExecutionCommands::Qmt(subcommand) => match subcommand {
            ExecutionQmtCommands::Status => {
                execute_execution_bridge_status().await?;
            }
            ExecutionQmtCommands::Preview { request_id } => {
                execute_execution_bridge_qmt_preview(&request_id).await?;
            }
            ExecutionQmtCommands::Live { request_id, yes } => {
                execute_execution_bridge_qmt_live(&request_id, yes).await?;
            }
            ExecutionQmtCommands::Query { order_id } => {
                execute_execution_bridge_qmt_query(&order_id).await?;
            }
            ExecutionQmtCommands::Cancel { order_id } => {
                execute_execution_bridge_qmt_cancel(&order_id).await?;
            }
            ExecutionQmtCommands::Account => {
                execute_execution_bridge_qmt_account().await?;
            }
            ExecutionQmtCommands::Positions => {
                execute_execution_bridge_qmt_positions().await?;
            }
            ExecutionQmtCommands::Asset => {
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

#[cfg(test)]
mod tests;
