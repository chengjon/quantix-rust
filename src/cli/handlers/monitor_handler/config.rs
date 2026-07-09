use super::*;

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
