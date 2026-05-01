#[path = "strategy_handler/catalog.rs"]
mod catalog;
#[path = "strategy_handler/instances.rs"]
mod instances;
#[path = "strategy_handler/requests.rs"]
mod requests;
#[path = "strategy_handler/service.rs"]
mod service;

use super::*;

pub(crate) use self::catalog::*;
pub(crate) use self::instances::*;
pub(crate) use self::requests::*;
pub(crate) use self::service::*;

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
