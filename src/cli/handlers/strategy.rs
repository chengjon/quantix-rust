use super::*;

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

pub(super) async fn execute_strategy_run_with_components<L, TS, RS>(
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

pub(super) async fn execute_strategy_run_with_risk_service<L, TS, RS>(
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

    Ok(build_strategy_run_summary(name, mode, &symbol, result, envelope.signal))
}

pub(super) fn create_strategy_config_store() -> JsonStrategyConfigStore {
    let runtime = CliRuntime::load();
    JsonStrategyConfigStore::new(runtime.strategy_config_path)
}

pub(super) async fn execute_strategy_config_init() -> Result<()> {
    let config = execute_strategy_config_init_to_store(&create_strategy_config_store())?;
    println!("{}", serde_json::to_string_pretty(&config)?);
    Ok(())
}

pub(super) fn execute_strategy_config_init_to_store(
    store: &JsonStrategyConfigStore,
) -> Result<StrategyDaemonConfig> {
    store.load_or_create()
}

pub(super) async fn execute_strategy_config_show() -> Result<()> {
    let config = execute_strategy_config_show_from_store(&create_strategy_config_store())?;
    println!("{}", serde_json::to_string_pretty(&config)?);
    Ok(())
}

pub(super) fn execute_strategy_config_show_from_store(
    store: &JsonStrategyConfigStore,
) -> Result<StrategyDaemonConfig> {
    store.load_or_create()
}

pub(super) fn build_strategy_run_summary(
    strategy_name: &str,
    mode: &str,
    symbol: &str,
    result: KernelExecutionResult,
    signal: Signal,
) -> StrategyRunSummary {
    let message = match result.order_status {
        Some(status) => format!(
            "signal={} order_status={}",
            strategy_signal_label(signal),
            status.as_str()
        ),
        None => format!("signal={} no_order", strategy_signal_label(signal)),
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

fn strategy_signal_label(signal: Signal) -> &'static str {
    match signal {
        Signal::Buy => "buy",
        Signal::Sell => "sell",
        Signal::Hold => "hold",
    }
}

fn strategy_order_status_label(status: OrderStatus) -> &'static str {
    status.as_str()
}

pub(super) fn format_strategy_run_summary_lines(summary: &StrategyRunSummary) -> Vec<String> {
    let mut lines = vec![
        format!("🧾 运行 ID: {}", summary.run_id),
        format!("  策略: {}", summary.strategy_name),
        format!("  模式: {}", summary.mode),
        format!("  代码: {}", summary.symbol),
        format!("  信号: {}", strategy_signal_label(summary.signal)),
    ];
    if let Some(status) = summary.order_status {
        lines.push(format!(
            "  订单状态: {}",
            strategy_order_status_label(status)
        ));
    }
    lines.push(format!("  结果: {}", summary.message));
    lines
}

fn format_strategy_run_intro_lines(name: &str, mode: &str, code: Option<&str>) -> Vec<String> {
    let mut lines = vec![format!("🎯 运行策略: {} ({})", name, mode)];
    if let Some(code) = code {
        lines.push(format!("📈 股票代码: {}", code));
    }
    lines
}

pub(super) fn format_strategy_catalog_lines() -> Vec<String> {
    vec![
        "📋 可用策略:".to_string(),
        String::new(),
        "  1. ma_cross - 均线交叉策略".to_string(),
        "     描述: MA5 上穿 MA20 买入，下穿卖出".to_string(),
        "     运行: quantix strategy run --name ma_cross --mode backtest --code 000001"
            .to_string(),
        String::new(),
        "💡 更多策略开发中...".to_string(),
    ]
}

pub(super) fn format_strategy_detail_lines(name: &str) -> Vec<String> {
    let mut lines = vec![format!("📖 策略详情: {}", name)];
    match name {
        "ma_cross" => {
            lines.extend([
                String::new(),
                "  名称: 均线交叉策略".to_string(),
                "  原理: 当短期均线(MA5)上穿长期均线(MA20)时买入，反之卖出".to_string(),
                "  参数:".to_string(),
                "    - 短期周期: 5".to_string(),
                "    - 长期周期: 20".to_string(),
                "  适用场景: 趋势明显的市场".to_string(),
                "  风险: 震荡市场容易频繁交易".to_string(),
            ]);
        }
        _ => lines.push(format!("❌ 未知策略: {}", name)),
    }
    lines
}

pub(super) fn print_strategy_run_summary(summary: &StrategyRunSummary) {
    for line in format_strategy_run_summary_lines(summary) {
        println!("{}", line);
    }
}

pub(super) async fn run_strategy(name: String, mode: String, code: Option<String>) -> Result<()> {
    for line in format_strategy_run_intro_lines(&name, &mode, code.as_deref()) {
        println!("{}", line);
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

pub(super) async fn list_strategies() -> Result<()> {
    for line in format_strategy_catalog_lines() {
        println!("{}", line);
    }
    Ok(())
}

pub(super) async fn show_strategy(name: String) -> Result<()> {
    for line in format_strategy_detail_lines(&name) {
        println!("{}", line);
    }
    Ok(())
}

pub(crate) async fn run_strategy_command_impl(cmd: StrategyCommands) -> Result<()> {
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

pub(super) fn execute_strategy_service_config_command_with_store(
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

pub(super) fn print_strategy_service_config_output(
    config: Option<StrategyServiceConfig>,
) -> Result<()> {
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

pub(super) trait StrategyServiceInstallerOps {
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

pub(super) fn execute_strategy_service_command(cmd: StrategyServiceCommands) -> Result<()> {
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

pub(super) fn execute_strategy_service_command_with_installer<I>(
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

pub(super) async fn execute_strategy_daemon_run(once: bool) -> Result<()> {
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

pub(super) async fn execute_strategy_daemon_run_once_with_components<L>(
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

pub(super) fn format_strategy_signal_row(row: &StrategySignalRecord) -> String {
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

pub(super) fn format_strategy_request_detail(
    request: &ExecutionRequestRecord,
    verbose: bool,
) -> String {
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

pub(super) async fn execute_strategy_signal_list(
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

pub(super) async fn execute_strategy_signal_list_with_store(
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

pub(super) async fn execute_strategy_signal_approve(
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

pub(super) async fn execute_strategy_signal_approve_with_store(
    store: &StrategyRuntimeStore,
    signal_id: &str,
    target_mode: &str,
    target_account: &str,
) -> Result<ExecutionRequestRecord> {
    store
        .approve_signal_and_create_request(signal_id, target_mode, target_account, Some("cli"))
        .await
}

pub(super) async fn execute_strategy_signal_reject(
    signal_id: &str,
    reason: Option<&str>,
) -> Result<()> {
    let runtime = CliRuntime::load();
    let runtime_store = StrategyRuntimeStore::new(runtime.strategy_runtime_db_path).await?;
    let signal =
        execute_strategy_signal_reject_with_store(&runtime_store, signal_id, reason).await?;
    println!("{}", format_strategy_rejection_result(&signal));
    Ok(())
}

pub(super) async fn execute_strategy_signal_reject_with_store(
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

pub(super) async fn execute_strategy_request_list(
    status: Option<&str>,
    target_mode: Option<&str>,
    target_account: Option<&str>,
    limit: usize,
    stats: bool,
) -> Result<()> {
    let runtime = CliRuntime::load();
    let runtime_store = StrategyRuntimeStore::new(runtime.strategy_runtime_db_path).await?;
    let rows = execute_strategy_request_list_with_store(&runtime_store, status).await?;

    let mut filtered: Vec<_> = rows
        .into_iter()
        .filter(|row| {
            let mode_match = target_mode.map_or(true, |m| row.target_mode == m);
            let account_match = target_account.map_or(true, |a| row.target_account == a);
            mode_match && account_match
        })
        .collect();

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
        println!(
            "Total: {} | Pending: {} | InProgress: {} | Completed: {} | Failed: {} | Canceled: {}",
            total, pending, in_progress, completed, failed, canceled
        );
        println!();
    }

    filtered.truncate(limit);

    for row in filtered {
        println!("{}", format_strategy_request_row(&row));
    }

    Ok(())
}

pub(super) async fn execute_strategy_request_show(
    request_id: &str,
    verbose: bool,
) -> Result<()> {
    let runtime = CliRuntime::load();
    let store = StrategyRuntimeStore::new(runtime.strategy_runtime_db_path).await?;

    let request = store
        .get_execution_request(request_id)
        .await?
        .ok_or_else(|| QuantixError::Other(format!("request 不存在: {request_id}")))?;

    println!("{}", format_strategy_request_detail(&request, verbose));

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

pub(super) async fn execute_strategy_request_execute(request_id: &str) -> Result<()> {
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

pub(super) async fn execute_strategy_request_execute_with_components<TS>(
    store: &StrategyRuntimeStore,
    request_id: &str,
    trade_store: TS,
    risk_store: JsonRiskStore,
) -> Result<ExecutionRequestRecord>
where
    TS: PaperTradeStore + Clone,
{
    crate::execution::daemon::execute_request_by_id_with_components(
        store,
        request_id,
        trade_store,
        risk_store,
    )
    .await
}

pub(super) async fn execute_strategy_request_cancel(
    request_id: &str,
    reason: Option<&str>,
) -> Result<()> {
    let runtime = CliRuntime::load();
    let runtime_store = StrategyRuntimeStore::new(runtime.strategy_runtime_db_path).await?;
    let request =
        execute_strategy_request_cancel_with_store(&runtime_store, request_id, reason).await?;
    println!("{}", format_strategy_request_row(&request));
    Ok(())
}

pub(super) async fn execute_strategy_request_cancel_with_store(
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

pub(super) fn format_strategy_approval_result(request: &ExecutionRequestRecord) -> String {
    format!(
        "{} signal={} target={}/{} status={}",
        request.request_id,
        request.signal_id,
        request.target_mode,
        request.target_account,
        request.request_status.as_str()
    )
}

pub(super) fn format_strategy_rejection_result(signal: &StrategySignalRecord) -> String {
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

pub(super) fn format_strategy_request_row(row: &ExecutionRequestRecord) -> String {
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

pub(super) async fn execute_strategy_request_list_with_store(
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

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::StrategyRunSummary;

    #[test]
    fn format_strategy_catalog_lines_matches_current_contract() {
        assert_eq!(
            format_strategy_catalog_lines(),
            vec![
                "📋 可用策略:".to_string(),
                String::new(),
                "  1. ma_cross - 均线交叉策略".to_string(),
                "     描述: MA5 上穿 MA20 买入，下穿卖出".to_string(),
                "     运行: quantix strategy run --name ma_cross --mode backtest --code 000001"
                    .to_string(),
                String::new(),
                "💡 更多策略开发中...".to_string(),
            ]
        );
    }

    #[test]
    fn format_strategy_detail_lines_for_known_strategy_matches_current_contract() {
        assert_eq!(
            format_strategy_detail_lines("ma_cross"),
            vec![
                "📖 策略详情: ma_cross".to_string(),
                String::new(),
                "  名称: 均线交叉策略".to_string(),
                "  原理: 当短期均线(MA5)上穿长期均线(MA20)时买入，反之卖出".to_string(),
                "  参数:".to_string(),
                "    - 短期周期: 5".to_string(),
                "    - 长期周期: 20".to_string(),
                "  适用场景: 趋势明显的市场".to_string(),
                "  风险: 震荡市场容易频繁交易".to_string(),
            ]
        );
    }

    #[test]
    fn format_strategy_run_summary_lines_include_optional_order_status() {
        let summary = StrategyRunSummary {
            run_id: "run-1".to_string(),
            strategy_name: "ma_cross".to_string(),
            mode: "paper".to_string(),
            symbol: "000001".to_string(),
            signal: Signal::Buy,
            order_status: Some(OrderStatus::Filled),
            message: "signal=buy order_status=filled".to_string(),
        };

        assert_eq!(
            format_strategy_run_summary_lines(&summary),
            vec![
                "🧾 运行 ID: run-1".to_string(),
                "  策略: ma_cross".to_string(),
                "  模式: paper".to_string(),
                "  代码: 000001".to_string(),
                "  信号: buy".to_string(),
                "  订单状态: filled".to_string(),
                "  结果: signal=buy order_status=filled".to_string(),
            ]
        );
    }
}
