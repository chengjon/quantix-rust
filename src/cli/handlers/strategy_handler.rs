#[path = "strategy_handler/catalog.rs"]
mod catalog;
#[path = "strategy_handler/instances.rs"]
mod instances;
#[path = "strategy_handler/requests.rs"]
mod requests;
#[path = "strategy_handler/service.rs"]
mod service;

use super::*;
use crate::safety::{
    JsonKillSwitchStore, format_execution_kill_switch_block_message,
    load_blocking_kill_switch_state,
};

pub(crate) use self::catalog::*;
pub(crate) use self::instances::*;
pub(crate) use self::requests::*;
pub(crate) use self::service::*;

use crate::core::signal::Signal;
use crate::core::{CliRuntime, QuantixError, Result};
use crate::execution::daemon::ExecutionDaemonIterationSummary;
use crate::execution::kernel::{
    ExecutionKernel, ExecutionRunRequest, FillDeltaApplier, KernelExecutionResult, RiskDecision,
    RiskEvaluator,
};
use crate::execution::mock_live::{MockLiveExecutionAdapter, SystemMockLiveClock};
use crate::execution::models::{
    ExecutionPolicy, FillDeltaContext, FillDeltaResult, OrderIntent, OrderStatus,
    StrategySignalRecord,
};
use crate::execution::paper::PaperExecutionAdapter;
use crate::execution::qmt_live_gate::{QMT_LIVE_BRIDGE_COMMAND, QMT_LIVE_BRIDGE_MODE_REQUIREMENT};
use crate::execution::runtime_store::StrategyRuntimeStore;
use crate::risk::RiskService;
use crate::strategy::daemon::StrategyBarLoadTelemetry;
use crate::strategy::runtime::{StrategyBarLoader, StrategyRuntime};
use crate::strategy::{
    FallbackStrategyBarLoader, JsonStrategyConfigStore, JsonStrategyServiceConfigStore,
    StrategyDaemonConfig, StrategyServiceConfig, StrategyServiceStatusSummary,
    StrategySignalDaemon, StrategyUserServiceInstaller,
};
use crate::trade::{
    CashSnapshot, InitAccountRequest, JsonPaperTradeStore, PaperTradeAccount, PaperTradeState,
    PaperTradeStore, TradeFeeRow, TradeHistoryRow, TradeOrderRequest, TradeOverview, TradePosition,
    TradePositionCurrentRow, TradeQuoteStatus, TradeRecord, TradeReportingService, TradeService,
};
use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal_macros::dec;
use std::time::Duration;

/// 策略命令
pub(crate) async fn execute_strategy_command(cmd: StrategyCommands) -> Result<()> {
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
                strategy_instance,
                strategy,
                code,
                approval_status,
                signal_status,
                limit,
            } => {
                execute_strategy_signal_list(
                    strategy_instance.as_deref(),
                    strategy.as_deref(),
                    code.as_deref(),
                    approval_status.as_deref(),
                    signal_status.as_deref(),
                    limit,
                )
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
    pub(crate) run_id: String,
    pub(crate) strategy_name: String,
    pub(crate) mode: String,
    pub(crate) symbol: String,
    pub(crate) signal: Signal,
    pub(crate) order_status: Option<OrderStatus>,
    pub(crate) message: String,
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

pub(crate) async fn execute_strategy_run_with_components<L, TS, RS>(
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

pub(crate) async fn execute_strategy_run_with_risk_service<L, TS, RS>(
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
    let kill_switch_store = JsonKillSwitchStore::with_default_path()?;
    execute_strategy_run_with_risk_service_and_kill_switch(
        name,
        mode,
        code,
        loader,
        trade_store,
        risk_service,
        runtime_store,
        &kill_switch_store,
    )
    .await
}

pub(crate) async fn execute_strategy_run_with_risk_service_and_kill_switch<L, TS, RS>(
    name: &str,
    mode: &str,
    code: Option<String>,
    loader: L,
    trade_store: TS,
    risk_service: RiskService<RS>,
    runtime_store: &StrategyRuntimeStore,
    kill_switch_store: &JsonKillSwitchStore,
) -> Result<StrategyRunSummary>
where
    L: StrategyBarLoader,
    TS: PaperTradeStore + Clone,
    RS: crate::risk::RiskStore,
{
    match mode {
        "live" => {
            return Err(QuantixError::Unsupported(format!(
                "strategy live 模式尚未实现；如需真实 QMT 提交，请改走 qmt_live request / {QMT_LIVE_BRIDGE_COMMAND} 路径，并确保 {QMT_LIVE_BRIDGE_MODE_REQUIREMENT}"
            )));
        }
        "paper" | "mock_live" => {}
        other => {
            return Err(QuantixError::Unsupported(format!(
                "strategy {other} 模式尚未实现"
            )));
        }
    }

    guard_strategy_run_kill_switch(kill_switch_store, mode)?;

    if name != "ma_cross" {
        return Err(QuantixError::Other(format!("未知策略: {name}")));
    }

    let symbol = code.ok_or_else(|| {
        QuantixError::Other(format!("strategy run --mode {mode} 需要显式指定 --code"))
    })?;

    let account = load_initialized_trade_account(&trade_store).await?;
    let held_volume = account
        .positions
        .get(&symbol)
        .map(|position| position.volume);

    let bars = loader.load_daily_bars(&symbol, 10_000).await?;
    let latest_bar = bars.last().cloned().ok_or_else(|| {
        QuantixError::Other(format!("strategy {mode} 未找到 {} 的日线数据", symbol))
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

fn guard_strategy_run_kill_switch(
    kill_switch_store: &JsonKillSwitchStore,
    mode: &str,
) -> Result<()> {
    let Some(state) = load_blocking_kill_switch_state(kill_switch_store, mode)? else {
        return Ok(());
    };

    Err(QuantixError::Other(
        format_execution_kill_switch_block_message(mode, &state),
    ))
}

pub(crate) fn build_strategy_run_summary(
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

pub(crate) fn signal_label(signal: Signal) -> &'static str {
    match signal {
        Signal::Buy => "buy",
        Signal::Sell => "sell",
        Signal::Hold => "hold",
    }
}

pub(crate) fn order_status_label(status: OrderStatus) -> &'static str {
    status.as_str()
}

pub(crate) fn print_strategy_run_summary(summary: &StrategyRunSummary) {
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

pub(crate) fn create_strategy_config_store() -> JsonStrategyConfigStore {
    let runtime = CliRuntime::load();
    JsonStrategyConfigStore::new(runtime.strategy_config_path)
}

pub(crate) async fn execute_strategy_config_init() -> Result<()> {
    let config = execute_strategy_config_init_to_store(&create_strategy_config_store())?;
    println!("{}", serde_json::to_string_pretty(&config)?);
    Ok(())
}

pub(crate) fn execute_strategy_config_init_to_store(
    store: &JsonStrategyConfigStore,
) -> Result<StrategyDaemonConfig> {
    store.load_or_create()
}

pub(crate) async fn execute_strategy_config_show() -> Result<()> {
    let config = execute_strategy_config_show_from_store(&create_strategy_config_store())?;
    println!("{}", serde_json::to_string_pretty(&config)?);
    Ok(())
}

pub(crate) fn execute_strategy_config_show_from_store(
    store: &JsonStrategyConfigStore,
) -> Result<StrategyDaemonConfig> {
    store.load_or_create()
}

pub(crate) async fn execute_strategy_daemon_run(once: bool) -> Result<()> {
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

pub(crate) async fn execute_strategy_daemon_run_once_with_components<L>(
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

pub(crate) fn is_non_terminal_order_status(order_status: &str) -> bool {
    matches!(
        order_status,
        "pending_submit"
            | "submitted"
            | "accepted"
            | "partially_filled"
            | "pending_cancel"
            | "unknown"
    )
}
