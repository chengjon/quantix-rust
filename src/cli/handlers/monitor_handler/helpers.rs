use super::*;

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

pub(super) fn monitor_notifications_enabled(config: &MonitorConfig) -> bool {
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

pub(super) fn monitor_notification_level(
    event_type: MonitorEventType,
) -> crate::monitoring::AlertLevel {
    match event_type {
        MonitorEventType::PriceAlert => crate::monitoring::AlertLevel::Warning,
        MonitorEventType::StopLoss => crate::monitoring::AlertLevel::Critical,
        MonitorEventType::StopProfit => crate::monitoring::AlertLevel::Warning,
        MonitorEventType::TrailingStop => crate::monitoring::AlertLevel::Error,
    }
}

pub(super) fn monitor_notification_event_label(event_type: MonitorEventType) -> &'static str {
    match event_type {
        MonitorEventType::PriceAlert => "price alert",
        MonitorEventType::StopLoss => "stop loss",
        MonitorEventType::StopProfit => "stop profit",
        MonitorEventType::TrailingStop => "trailing stop",
    }
}

pub(super) fn monitor_notification_event_key(event_type: MonitorEventType) -> &'static str {
    match event_type {
        MonitorEventType::PriceAlert => "price-alert",
        MonitorEventType::StopLoss => "stop-loss",
        MonitorEventType::StopProfit => "stop-profit",
        MonitorEventType::TrailingStop => "trailing-stop",
    }
}

pub(super) fn monitor_notification_run_mode_label(run_mode: MonitorRunMode) -> &'static str {
    match run_mode {
        MonitorRunMode::Foreground => "foreground",
        MonitorRunMode::Daemon => "daemon",
    }
}
