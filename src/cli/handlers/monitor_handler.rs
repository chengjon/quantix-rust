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

mod config;
mod execution;
mod helpers;
mod notifications;
mod service;

#[allow(unused_imports)]
pub use config::*;
#[allow(unused_imports)]
pub use execution::*;
#[allow(unused_imports)]
pub use helpers::*;
#[allow(unused_imports)]
pub use notifications::*;
#[allow(unused_imports)]
pub use service::*;

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
