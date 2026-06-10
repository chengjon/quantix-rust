use super::*;

use crate::core::{CliRuntime, QuantixError, Result};
use crate::monitor::storage::SqliteMonitorAlertStore;
use crate::monitor::{
    JsonMonitorConfigStore, JsonMonitorServiceConfigStore, MonitorAlertStore, MonitorConfig,
    MonitorEventFilter, MonitorEventRow, MonitorEventType, MonitorIterationOutput,
    MonitorQuoteReader, MonitorQuoteRow, MonitorRunMode, MonitorRunner, MonitorService,
    MonitorServiceConfig, MonitorServiceStatusSummary, MonitorUserServiceInstaller,
    MonitorWatchlistReader, MonitorWatchlistSnapshot, PriceAlert, PriceAlertKind,
};
use crate::stop::{
    SqliteStopRuleStore, StopHistoryEvent, StopHistoryEventType, StopRuleStore, StopService,
    StopTriggerKind, TriggeredStop,
};
use crate::trade::{JsonPaperTradeStore, PaperTradeStore};
use crate::watchlist::{
    TdxWatchlistQuoteLookup, WatchlistListItem, WatchlistQuoteLookup, WatchlistService,
    WatchlistStorage,
};
use async_trait::async_trait;
#[cfg(test)]
use chrono::DateTime;
use chrono::Utc;
use rust_decimal::prelude::ToPrimitive;
use std::time::Duration;

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

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum MonitorCommandOutput {
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

#[derive(Debug, Clone)]
pub(crate) struct ConfiguredMonitorWatchlistReader {
    storage: WatchlistStorage,
    service: WatchlistService,
}

impl ConfiguredMonitorWatchlistReader {
    pub(crate) fn new(storage: WatchlistStorage) -> Self {
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
pub(crate) struct TdxMonitorQuoteReader;

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
pub(crate) struct MonitorAlertAddRequest {
    pub(crate) code: String,
    pub(crate) kind: PriceAlertKind,
    pub(crate) target_price: f64,
}

pub(crate) async fn execute_monitor_command_with_service<RW, RQ, RS>(
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

pub(crate) async fn execute_monitor_command_with_stop_store<RW, RQ, RS, SS>(
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

pub(crate) async fn evaluate_stop_rules_for_snapshot<SS, TS>(
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
    let results = stop_service.evaluate_rules_with_anchor_map(
        &rules,
        &snapshot.rows,
        &avg_cost_by_code,
        observed_at,
    );
    let mut triggered_stops = Vec::new();

    for (original_rule, result) in rules.iter().zip(results) {
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
                        StopTriggerKind::TrailingLoss => {
                            crate::stop::StopHistoryTriggerKind::Trailing
                        }
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

pub(crate) fn validate_monitor_watchlist_command(once: bool, repeat: bool) -> Result<()> {
    if once ^ repeat {
        Ok(())
    } else {
        Err(QuantixError::Other(
            "monitor watchlist 必须且只能指定 --once 或 --repeat 之一".to_string(),
        ))
    }
}

pub(crate) fn build_monitor_alert_request(
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

pub(crate) fn monitor_alert_id_to_i64(id: u64) -> Result<i64> {
    i64::try_from(id).map_err(|_| QuantixError::Other(format!("告警 ID 超出支持范围: {}", id)))
}

pub(crate) fn parse_monitor_event_type(value: &str) -> Result<MonitorEventType> {
    match value {
        "price-alert" => Ok(MonitorEventType::PriceAlert),
        "stop-loss" => Ok(MonitorEventType::StopLoss),
        "stop-profit" => Ok(MonitorEventType::StopProfit),
        "trailing-stop" => Ok(MonitorEventType::TrailingStop),
        other => Err(QuantixError::Unsupported(format!(
            "monitor event list 不支持的事件类型: {}",
            other
        ))),
    }
}

pub(crate) async fn create_monitor_alert_store() -> Result<SqliteMonitorAlertStore> {
    let runtime = CliRuntime::load();
    SqliteMonitorAlertStore::new(runtime.monitor_db_path).await
}

pub(crate) async fn create_stop_rule_store() -> Result<SqliteStopRuleStore> {
    let runtime = CliRuntime::load();
    SqliteStopRuleStore::new(runtime.monitor_db_path).await
}

pub(crate) async fn create_configured_monitor_runner() -> Result<
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

pub(crate) fn execute_monitor_config_command_with_store(
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
            notify,
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
            if let Some(value) = notify {
                config.notify_enabled = value;
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

pub(crate) async fn execute_monitor_event_command_with_store(
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

pub(crate) fn execute_monitor_service_config_command_with_store(
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

pub(crate) trait MonitorServiceInstallerOps {
    fn install(&self) -> Result<()>;
    fn uninstall(&self) -> Result<()>;
    fn start(&self) -> Result<()>;
    fn stop(&self) -> Result<()>;
    fn enable(&self) -> Result<()>;
    fn disable(&self) -> Result<()>;
    #[allow(dead_code)]
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

pub(crate) fn execute_monitor_service_command(
    cmd: MonitorServiceCommands,
) -> Result<MonitorCommandOutput> {
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

pub(crate) fn execute_monitor_service_command_with_installer<I>(
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

#[cfg(test)]
pub(crate) async fn execute_monitor_iteration_with_runner<RW, RQ, SS, TS>(
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

pub(crate) async fn run_monitor_loop<RW, RQ, SS, TS>(
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
        dispatch_monitor_notifications_for_output(&config, &output).await;
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

pub(crate) async fn dispatch_monitor_notifications_for_output(
    config: &MonitorConfig,
    output: &MonitorIterationOutput,
) {
    if !monitor_notifications_enabled(config) || output.new_events.is_empty() {
        return;
    }

    if let Err(err) = send_monitor_notifications(output).await {
        tracing::warn!("monitor notifications failed: {}", err);
    }
}

fn monitor_notifications_enabled(config: &MonitorConfig) -> bool {
    config.notify_enabled
        || matches!(
            std::env::var("QUANTIX_MONITOR_NOTIFY")
                .ok()
                .as_deref()
                .map(str::trim)
                .map(str::to_ascii_lowercase)
                .as_deref(),
            Some("1" | "true" | "yes" | "on")
        )
}

#[async_trait]
pub(crate) trait MonitorNotificationSender {
    async fn notify(&mut self, notification: crate::monitoring::Notification) -> Result<()>;
}

#[async_trait]
impl MonitorNotificationSender for crate::monitoring::NotificationService {
    async fn notify(&mut self, notification: crate::monitoring::Notification) -> Result<()> {
        crate::monitoring::NotificationService::notify(self, notification).await
    }
}

async fn send_monitor_notifications(output: &MonitorIterationOutput) -> Result<()> {
    let config = crate::monitoring::NotificationConfig::from_env();
    let mut service = crate::monitoring::NotificationService::new(config);
    send_monitor_notifications_with_service(output, &mut service).await
}

pub(crate) async fn send_monitor_notifications_with_service<S>(
    output: &MonitorIterationOutput,
    service: &mut S,
) -> Result<()>
where
    S: MonitorNotificationSender,
{
    for event in &output.new_events {
        let notification = crate::monitoring::Notification::new(
            format!(
                "Monitor {} {}",
                monitor_notification_event_label(event.event_type),
                event.code
            ),
            format!(
                "{}\n模式: {}\n时间: {}",
                event.message,
                monitor_notification_run_mode_label(event.run_mode),
                event.event_time.format("%Y-%m-%d %H:%M:%S")
            ),
            monitor_notification_level(event.event_type),
        )
        .with_metadata(
            "event_type",
            monitor_notification_event_key(event.event_type).to_string(),
        )
        .with_metadata("code", event.code.clone())
        .with_metadata(
            "run_mode",
            monitor_notification_run_mode_label(event.run_mode).to_string(),
        );

        service.notify(notification).await?;
    }

    Ok(())
}

fn monitor_notification_level(event_type: MonitorEventType) -> crate::monitoring::AlertLevel {
    match event_type {
        MonitorEventType::PriceAlert => crate::monitoring::AlertLevel::Warning,
        MonitorEventType::StopLoss => crate::monitoring::AlertLevel::Critical,
        MonitorEventType::StopProfit => crate::monitoring::AlertLevel::Warning,
        MonitorEventType::TrailingStop => crate::monitoring::AlertLevel::Error,
    }
}

fn monitor_notification_event_label(event_type: MonitorEventType) -> &'static str {
    match event_type {
        MonitorEventType::PriceAlert => "price alert",
        MonitorEventType::StopLoss => "stop loss",
        MonitorEventType::StopProfit => "stop profit",
        MonitorEventType::TrailingStop => "trailing stop",
    }
}

fn monitor_notification_event_key(event_type: MonitorEventType) -> &'static str {
    match event_type {
        MonitorEventType::PriceAlert => "price-alert",
        MonitorEventType::StopLoss => "stop-loss",
        MonitorEventType::StopProfit => "stop-profit",
        MonitorEventType::TrailingStop => "trailing-stop",
    }
}

fn monitor_notification_run_mode_label(run_mode: MonitorRunMode) -> &'static str {
    match run_mode {
        MonitorRunMode::Foreground => "foreground",
        MonitorRunMode::Daemon => "daemon",
    }
}

pub(crate) async fn persist_triggered_monitor_alerts<RS>(
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
