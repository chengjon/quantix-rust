use std::time::SystemTime;

use chrono::{DateTime, FixedOffset, NaiveTime, TimeZone, Utc};
use uuid::Uuid;

use crate::core::{QuantixError, Result};
use crate::execution::models::{
    ApprovalStatus, SignalStatus, StrategyDaemonCheckpointRecord, StrategyRunRecord,
    StrategyRunStatus, StrategySignalRecord,
};
use crate::execution::runtime_store::StrategyRuntimeStore;
use crate::strategy::config::{BootstrapPolicy, JsonStrategyConfigStore, StrategyDaemonConfig};
use crate::strategy::fallback_loader::{FallbackStrategyBarLoader, StrategyBarLoadSource};
use crate::strategy::registry::StrategyRegistry;
use crate::strategy::runtime::StrategyBarLoader;

pub trait StrategyBarLoadTelemetry {
    fn last_source(&self) -> Option<StrategyBarLoadSource> {
        None
    }
}

#[derive(Debug, Clone)]
pub struct StrategySignalDaemon<L> {
    loader: L,
    store: StrategyRuntimeStore,
    config_store: JsonStrategyConfigStore,
    registry: StrategyRegistry,
    config: StrategyDaemonConfig,
    last_config_mtime: Option<SystemTime>,
}

impl<L> StrategySignalDaemon<L>
where
    L: StrategyBarLoader + StrategyBarLoadTelemetry,
{
    pub fn new(
        loader: L,
        store: StrategyRuntimeStore,
        config_store: JsonStrategyConfigStore,
    ) -> Result<Self> {
        let config = config_store.load_or_create()?;
        let last_config_mtime = std::fs::metadata(config_store.path())
            .and_then(|metadata| metadata.modified())
            .ok();

        Ok(Self {
            loader,
            store,
            config_store,
            registry: StrategyRegistry::new(),
            config,
            last_config_mtime,
        })
    }

    pub async fn run_once(&mut self) -> Result<()> {
        self.reload_config_if_changed()?;

        let active_stocks: Vec<_> = self
            .config
            .stocks
            .iter()
            .filter(|stock| stock.enabled)
            .collect();
        if active_stocks.len() != 1 {
            return Err(QuantixError::Other(
                "strategy daemon 当前要求恰好一个 enabled 股票".to_string(),
            ));
        }
        let stock = active_stocks[0];

        for strategy in stock.strategies.iter().filter(|strategy| strategy.enabled) {
            let evaluator = self.registry.build(strategy)?;
            let bars = self
                .loader
                .load_daily_bars(&stock.code, 10_000.max(evaluator.lookback_required() + 1))
                .await?;
            let source = self.loader.last_source();
            let Some(latest_bar) = bars.last() else {
                continue;
            };

            let latest_bar_end = normalize_daily_bar_end(latest_bar.date)?;
            let checkpoint = self
                .store
                .find_daemon_checkpoint(&strategy.id, &stock.code, "1d")
                .await?;

            if let Some(checkpoint) = checkpoint {
                if checkpoint.last_processed_bar >= Some(latest_bar_end) {
                    continue;
                }

                let now = Utc::now();
                let run_id = Uuid::new_v4().to_string();
                let run = StrategyRunRecord {
                    run_id: run_id.clone(),
                    strategy_name: strategy.name.clone(),
                    mode: "signal".to_string(),
                    trigger: "daemon".to_string(),
                    status: StrategyRunStatus::Success,
                    symbol: stock.code.clone(),
                    timeframe: "1d".to_string(),
                    bar_end: latest_bar_end,
                    started_at: now,
                    finished_at: Some(now),
                    metadata_json: serde_json::json!({
                        "strategy_instance_id": strategy.id,
                        "params": strategy.params,
                        "bar_source_id": source.as_ref().map(|item| item.source_id.clone()),
                        "bar_source_fallback": source.as_ref().map(|item| item.fallback_used),
                    }),
                };
                let envelope = evaluator.evaluate(&bars)?;
                let signal = StrategySignalRecord {
                    signal_id: Uuid::new_v4().to_string(),
                    strategy_instance_id: strategy.id.clone(),
                    strategy_name: strategy.name.clone(),
                    symbol: stock.code.clone(),
                    timeframe: "1d".to_string(),
                    bar_end: latest_bar_end,
                    signal_value: signal_label(envelope.signal).to_string(),
                    signal_status: SignalStatus::New,
                    approval_status: ApprovalStatus::Pending,
                    run_id: run_id.clone(),
                    metadata_json: serde_json::json!({
                        "strategy_instance_id": strategy.id,
                        "params": strategy.params,
                        "bar_source_id": source.as_ref().map(|item| item.source_id.clone()),
                        "bar_source_fallback": source.as_ref().map(|item| item.fallback_used),
                    }),
                    created_at: now,
                    updated_at: now,
                };
                let daemon_checkpoint = StrategyDaemonCheckpointRecord {
                    checkpoint_id: Uuid::new_v4().to_string(),
                    strategy_instance_id: strategy.id.clone(),
                    strategy_name: strategy.name.clone(),
                    symbol: stock.code.clone(),
                    timeframe: "1d".to_string(),
                    last_processed_bar: Some(latest_bar_end),
                    last_run_id: Some(run_id),
                    state_json: serde_json::json!({
                        "bootstrap_policy": "latest_only"
                    }),
                    updated_at: now,
                };

                self.store
                    .record_daemon_signal_run(&run, &signal, &daemon_checkpoint)
                    .await?;
            } else {
                if self.config.bootstrap_policy != BootstrapPolicy::LatestOnly {
                    return Err(QuantixError::Unsupported(
                        "strategy daemon 当前仅支持 bootstrap_policy=latest_only".to_string(),
                    ));
                }

                self.store
                    .upsert_daemon_checkpoint(&StrategyDaemonCheckpointRecord {
                        checkpoint_id: Uuid::new_v4().to_string(),
                        strategy_instance_id: strategy.id.clone(),
                        strategy_name: strategy.name.clone(),
                        symbol: stock.code.clone(),
                        timeframe: "1d".to_string(),
                        last_processed_bar: Some(latest_bar_end),
                        last_run_id: None,
                        state_json: serde_json::json!({
                            "bootstrap_policy": "latest_only"
                        }),
                        updated_at: Utc::now(),
                    })
                    .await?;
            }
        }

        Ok(())
    }

    pub fn check_interval_secs(&self) -> u64 {
        self.config.check_interval_secs
    }

    fn reload_config_if_changed(&mut self) -> Result<()> {
        let current_mtime = std::fs::metadata(self.config_store.path())
            .and_then(|metadata| metadata.modified())
            .ok();

        if current_mtime != self.last_config_mtime {
            self.config = self.config_store.load_or_create()?;
            self.last_config_mtime = current_mtime;
        }

        Ok(())
    }
}

impl<P> StrategyBarLoadTelemetry for FallbackStrategyBarLoader<P> {
    fn last_source(&self) -> Option<StrategyBarLoadSource> {
        self.last_source()
    }
}

fn normalize_daily_bar_end(date: chrono::NaiveDate) -> Result<DateTime<Utc>> {
    let shanghai = FixedOffset::east_opt(8 * 3600)
        .ok_or_else(|| QuantixError::Other("无法构造 Asia/Shanghai 时区偏移".to_string()))?;
    let local = shanghai
        .from_local_datetime(&date.and_time(NaiveTime::from_hms_opt(15, 0, 0).unwrap()))
        .single()
        .ok_or_else(|| QuantixError::Other(format!("无法规范化日线结束时间: {date}")))?;
    Ok(local.with_timezone(&Utc))
}

fn signal_label(signal: crate::strategy::trait_def::Signal) -> &'static str {
    match signal {
        crate::strategy::trait_def::Signal::Buy => "buy",
        crate::strategy::trait_def::Signal::Sell => "sell",
        crate::strategy::trait_def::Signal::Hold => "hold",
    }
}
