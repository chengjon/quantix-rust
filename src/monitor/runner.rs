use std::collections::HashSet;

use chrono::{DateTime, Utc};

use crate::core::Result;
use crate::monitor::{
    MonitorAlertStore, MonitorConfig, MonitorEventRow, MonitorEventType, MonitorQuoteReader,
    MonitorRunMode, MonitorService, MonitorWatchlistReader, NewMonitorEvent,
    SqliteMonitorAlertStore,
};
use crate::stop::{StopRuleStore, StopService, StopTriggerKind, TriggeredStop};

#[derive(Debug, Clone, PartialEq)]
pub struct MonitorIterationOutput {
    pub snapshot: crate::monitor::MonitorWatchlistSnapshot,
    pub triggered_stops: Vec<TriggeredStop>,
    pub new_events: Vec<MonitorEventRow>,
}

#[derive(Debug, Clone)]
pub struct MonitorRunner<RW, RQ, SS> {
    monitor_service: MonitorService<RW, RQ, SqliteMonitorAlertStore>,
    alert_store: SqliteMonitorAlertStore,
    stop_store: SS,
}

impl<RW, RQ, SS> MonitorRunner<RW, RQ, SS>
where
    RW: MonitorWatchlistReader,
    RQ: MonitorQuoteReader,
    SS: StopRuleStore + Clone,
{
    pub fn new(
        watchlist_reader: RW,
        quote_reader: RQ,
        alert_store: SqliteMonitorAlertStore,
        stop_store: SS,
    ) -> Self {
        let monitor_service =
            MonitorService::new(watchlist_reader, quote_reader, alert_store.clone());
        Self {
            monitor_service,
            alert_store,
            stop_store,
        }
    }

    pub async fn run_once(
        &self,
        config: &MonitorConfig,
        run_mode: MonitorRunMode,
        now: DateTime<Utc>,
    ) -> Result<MonitorIterationOutput> {
        let mut snapshot = self.monitor_service.load_watchlist_snapshot().await?;
        if let Some(group) = &config.watchlist_group {
            snapshot.rows.retain(|row| row.group == *group);
            let allowed_codes = snapshot
                .rows
                .iter()
                .map(|row| row.code.as_str())
                .collect::<HashSet<_>>();
            snapshot
                .triggered_alerts
                .retain(|alert| allowed_codes.contains(alert.code.as_str()));
        }

        let observed_at = snapshot
            .rows
            .iter()
            .filter_map(|row| row.quote_time)
            .max()
            .unwrap_or(now);

        let mut new_events = self
            .persist_alert_events(&snapshot, config.max_event_history, run_mode, observed_at)
            .await?;
        let triggered_stops = self
            .evaluate_stop_rules(
                &snapshot,
                config.max_event_history,
                run_mode,
                observed_at,
                &mut new_events,
            )
            .await?;

        Ok(MonitorIterationOutput {
            snapshot,
            triggered_stops,
            new_events,
        })
    }

    async fn persist_alert_events(
        &self,
        snapshot: &crate::monitor::MonitorWatchlistSnapshot,
        max_event_history: usize,
        run_mode: MonitorRunMode,
        observed_at: DateTime<Utc>,
    ) -> Result<Vec<MonitorEventRow>> {
        let mut new_events = Vec::new();
        let allowed_codes = snapshot
            .rows
            .iter()
            .map(|row| row.code.as_str())
            .collect::<HashSet<_>>();
        let triggered_ids = snapshot
            .triggered_alerts
            .iter()
            .map(|alert| alert.alert_id)
            .collect::<HashSet<_>>();

        for alert in self.alert_store.list_alerts().await? {
            if !allowed_codes.is_empty() && !allowed_codes.contains(alert.code.as_str()) {
                continue;
            }

            let source_key = format!("price_alert:{}", alert.id);
            if let Some(triggered) = snapshot
                .triggered_alerts
                .iter()
                .find(|candidate| candidate.alert_id == alert.id)
            {
                let triggered_at = triggered.triggered_at.unwrap_or(observed_at);
                self.alert_store
                    .mark_triggered(alert.id, triggered_at)
                    .await?;
                let event = NewMonitorEvent {
                    event_time: triggered_at,
                    event_type: MonitorEventType::PriceAlert,
                    code: triggered.code.clone(),
                    price: Some(triggered.current_price),
                    message: format!(
                        "{} crossed {:?} {:.2}",
                        triggered.code, triggered.kind, triggered.target_price
                    ),
                    source_type: "price_alert".to_string(),
                    source_key: source_key.clone(),
                    observed_at: Some(triggered_at),
                    run_mode,
                };

                if self
                    .alert_store
                    .record_event_edge(
                        "price_alert",
                        &source_key,
                        true,
                        Some(event.clone()),
                        max_event_history,
                    )
                    .await?
                {
                    new_events.push(event_row_from_new_event(0, &event));
                }
            } else if !triggered_ids.contains(&alert.id) {
                self.alert_store
                    .record_event_edge("price_alert", &source_key, false, None, max_event_history)
                    .await?;
            }
        }

        Ok(new_events)
    }

    async fn evaluate_stop_rules(
        &self,
        snapshot: &crate::monitor::MonitorWatchlistSnapshot,
        max_event_history: usize,
        run_mode: MonitorRunMode,
        observed_at: DateTime<Utc>,
        new_events: &mut Vec<MonitorEventRow>,
    ) -> Result<Vec<TriggeredStop>> {
        let rules = self.stop_store.list_rules().await?;
        if rules.is_empty() {
            return Ok(Vec::new());
        }

        let stop_service = StopService::new(self.stop_store.clone());
        let results = stop_service.evaluate_rules(&rules, &snapshot.rows, observed_at);
        let mut triggered_stops = Vec::new();

        for (original_rule, result) in rules.iter().zip(results.into_iter()) {
            if result.updated_rule != *original_rule {
                self.stop_store
                    .upsert_rule(result.updated_rule.clone())
                    .await?;
            }

            let source_key = format!("stop_rule:{}", original_rule.code);
            if let Some(triggered_stop) = result.triggered_stop {
                let event = NewMonitorEvent {
                    event_time: triggered_stop.triggered_at.unwrap_or(observed_at),
                    event_type: stop_event_type(triggered_stop.kind),
                    code: triggered_stop.code.clone(),
                    price: Some(triggered_stop.current_price),
                    message: format!(
                        "{} hit {:?} {:.2}",
                        triggered_stop.code, triggered_stop.kind, triggered_stop.threshold_price
                    ),
                    source_type: "stop_rule".to_string(),
                    source_key: source_key.clone(),
                    observed_at: triggered_stop.triggered_at.or(Some(observed_at)),
                    run_mode,
                };

                if self
                    .alert_store
                    .record_event_edge(
                        "stop_rule",
                        &source_key,
                        true,
                        Some(event.clone()),
                        max_event_history,
                    )
                    .await?
                {
                    new_events.push(event_row_from_new_event(0, &event));
                }

                triggered_stops.push(triggered_stop);
            } else {
                self.alert_store
                    .record_event_edge("stop_rule", &source_key, false, None, max_event_history)
                    .await?;
            }
        }

        Ok(triggered_stops)
    }
}

fn stop_event_type(kind: StopTriggerKind) -> MonitorEventType {
    match kind {
        StopTriggerKind::Loss => MonitorEventType::StopLoss,
        StopTriggerKind::Profit => MonitorEventType::StopProfit,
        StopTriggerKind::TrailingLoss => MonitorEventType::TrailingStop,
    }
}

fn event_row_from_new_event(id: i64, event: &NewMonitorEvent) -> MonitorEventRow {
    MonitorEventRow {
        id,
        event_time: event.event_time,
        event_type: event.event_type,
        code: event.code.clone(),
        price: event.price,
        message: event.message.clone(),
        source_type: event.source_type.clone(),
        source_key: event.source_key.clone(),
        observed_at: event.observed_at,
        run_mode: event.run_mode,
    }
}
