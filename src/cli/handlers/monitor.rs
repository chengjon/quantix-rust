use super::*;

pub async fn run_monitor_command(cmd: MonitorCommands) -> Result<()> {
    let watchlist_reader = ConfiguredMonitorWatchlistReader::new(create_watchlist_storage());
    let quote_reader = TdxMonitorQuoteReader;
    let alert_store = create_monitor_alert_store().await?;
    let service = MonitorService::new(watchlist_reader, quote_reader, alert_store.clone());
    let output = match cmd {
        MonitorCommands::Watchlist { once } => {
            let stop_store = create_stop_rule_store().await?;
            execute_monitor_command_with_stop_store(
                MonitorCommands::Watchlist { once },
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

pub async fn run_stop_command(cmd: StopCommands) -> Result<()> {
    let watchlist_storage = create_watchlist_storage();
    let service = StopService::new(create_stop_rule_store().await?);
    let output = execute_stop_command_with_service(cmd, &service, &watchlist_storage).await?;
    print_stop_command_output(&output);
    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
pub(super) enum MonitorCommandOutput {
    Watchlist {
        snapshot: MonitorWatchlistSnapshot,
        triggered_stops: Vec<TriggeredStop>,
    },
    AlertAdded(PriceAlert),
    AlertList(Vec<PriceAlert>),
    AlertRemoved {
        id: u64,
        removed: bool,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub(super) enum StopCommandOutput {
    RuleSet(StopRule),
    RuleList(Vec<StopRule>),
    RuleRemoved { code: String, removed: bool },
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

pub(super) async fn execute_monitor_command_with_service<RW, RQ, RS>(
    cmd: MonitorCommands,
    service: &MonitorService<RW, RQ, RS>,
) -> Result<MonitorCommandOutput>
where
    RW: MonitorWatchlistReader,
    RQ: MonitorQuoteReader,
    RS: MonitorAlertStore,
{
    match cmd {
        MonitorCommands::Watchlist { once } => {
            validate_monitor_watchlist_command(once)?;
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
    }
}

pub(super) async fn execute_monitor_command_with_stop_store<RW, RQ, RS, SS>(
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
        MonitorCommands::Watchlist { once } => {
            validate_monitor_watchlist_command(once)?;
            let snapshot = service.load_watchlist_snapshot().await?;
            let triggered_stops = evaluate_stop_rules_for_snapshot(&snapshot, stop_store).await?;
            Ok(MonitorCommandOutput::Watchlist {
                snapshot,
                triggered_stops,
            })
        }
        other => execute_monitor_command_with_service(other, service).await,
    }
}

async fn evaluate_stop_rules_for_snapshot<SS>(
    snapshot: &MonitorWatchlistSnapshot,
    stop_store: &SS,
) -> Result<Vec<TriggeredStop>>
where
    SS: StopRuleStore + Clone,
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
    let stop_service = StopService::new(stop_store.clone());
    let results = stop_service.evaluate_rules(&rules, &snapshot.rows, observed_at);
    let mut triggered_stops = Vec::new();

    for (original_rule, result) in rules.iter().zip(results.into_iter()) {
        if result.updated_rule != *original_rule {
            stop_store.upsert_rule(result.updated_rule.clone()).await?;
        }

        if let Some(triggered_stop) = result.triggered_stop {
            triggered_stops.push(triggered_stop);
        }
    }

    Ok(triggered_stops)
}

pub(super) async fn execute_stop_command_with_service<RS>(
    cmd: StopCommands,
    service: &StopService<RS>,
    watchlist_storage: &WatchlistStorage,
) -> Result<StopCommandOutput>
where
    RS: StopRuleStore,
{
    match cmd {
        StopCommands::Set {
            code,
            loss,
            profit,
            trailing,
        } => {
            ensure_watchlist_contains_code(watchlist_storage, &code)?;
            let rule = service
                .set_rule(&code, loss, profit, trailing, Utc::now())
                .await?;
            Ok(StopCommandOutput::RuleSet(rule))
        }
        StopCommands::List => Ok(StopCommandOutput::RuleList(service.list_rules().await?)),
        StopCommands::Remove { code } => {
            let removed = service.remove_rule(&code).await?;
            Ok(StopCommandOutput::RuleRemoved { code, removed })
        }
    }
}

fn validate_monitor_watchlist_command(once: bool) -> Result<()> {
    if once {
        Ok(())
    } else {
        Err(QuantixError::Other(
            "monitor watchlist 当前仅支持 --once".to_string(),
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

async fn create_monitor_alert_store() -> Result<SqliteMonitorAlertStore> {
    let runtime = CliRuntime::load();
    SqliteMonitorAlertStore::new(runtime.monitor_db_path).await
}

async fn create_stop_rule_store() -> Result<SqliteStopRuleStore> {
    let runtime = CliRuntime::load();
    SqliteStopRuleStore::new(runtime.monitor_db_path).await
}

pub(super) async fn persist_triggered_monitor_alerts<RS>(
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

fn print_stop_command_output(output: &StopCommandOutput) {
    match output {
        StopCommandOutput::RuleSet(rule) => {
            println!("✅ 已设置 {} 的止盈止损规则", rule.code);
        }
        StopCommandOutput::RuleList(rules) => print_stop_rules(rules),
        StopCommandOutput::RuleRemoved { code, removed } => {
            if *removed {
                println!("✅ 已移除 {} 的止盈止损规则", code);
            } else {
                println!("⚠️  未找到 {} 的止盈止损规则", code);
            }
        }
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

fn print_monitor_command_output(output: &MonitorCommandOutput) {
    match output {
        MonitorCommandOutput::Watchlist {
            snapshot,
            triggered_stops,
        } => print_monitor_watchlist_snapshot(snapshot, triggered_stops),
        MonitorCommandOutput::AlertAdded(alert) => println!(
            "✅ 已添加价格告警 #{} {} {} {:.2}",
            alert.id,
            alert.code,
            format_monitor_alert_kind(alert.kind),
            alert.target_price
        ),
        MonitorCommandOutput::AlertList(alerts) => print_monitor_alerts(alerts),
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

fn format_monitor_alert_kind(kind: PriceAlertKind) -> &'static str {
    match kind {
        PriceAlertKind::Above => "above",
        PriceAlertKind::Below => "below",
    }
}
