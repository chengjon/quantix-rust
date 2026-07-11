use super::*;

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
