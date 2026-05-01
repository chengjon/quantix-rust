use super::*;
use std::sync::{Mutex, OnceLock};

pub(super) fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|err| err.into_inner())
}

pub(super) struct ClickHouseEnvGuard {
    url: Option<String>,
    database: Option<String>,
    user: Option<String>,
    password: Option<String>,
    watchlist_path: Option<String>,
    trade_path: Option<String>,
    risk_path: Option<String>,
    monitor_db_path: Option<String>,
    monitor_config_path: Option<String>,
    strategy_config_path: Option<String>,
    strategy_runtime_db_path: Option<String>,
    execution_config_path: Option<String>,
    home: Option<String>,
}

impl ClickHouseEnvGuard {
    pub(super) fn capture() -> Self {
        Self {
            url: std::env::var(CLICKHOUSE_URL_ENV).ok(),
            database: std::env::var(CLICKHOUSE_DB_ENV).ok(),
            user: std::env::var(CLICKHOUSE_USER_ENV).ok(),
            password: std::env::var(CLICKHOUSE_PASSWORD_ENV).ok(),
            watchlist_path: std::env::var(WATCHLIST_PATH_ENV).ok(),
            trade_path: std::env::var(TRADE_PATH_ENV).ok(),
            risk_path: std::env::var(RISK_PATH_ENV).ok(),
            monitor_db_path: std::env::var(MONITOR_DB_PATH_ENV).ok(),
            monitor_config_path: std::env::var(MONITOR_CONFIG_PATH_ENV).ok(),
            strategy_config_path: std::env::var(STRATEGY_CONFIG_PATH_ENV).ok(),
            strategy_runtime_db_path: std::env::var(STRATEGY_RUNTIME_DB_PATH_ENV).ok(),
            execution_config_path: std::env::var(EXECUTION_CONFIG_PATH_ENV).ok(),
            home: std::env::var("HOME").ok(),
        }
    }
}

impl Drop for ClickHouseEnvGuard {
    fn drop(&mut self) {
        match &self.url {
            Some(value) => unsafe { std::env::set_var(CLICKHOUSE_URL_ENV, value) },
            None => unsafe { std::env::remove_var(CLICKHOUSE_URL_ENV) },
        }

        match &self.database {
            Some(value) => unsafe { std::env::set_var(CLICKHOUSE_DB_ENV, value) },
            None => unsafe { std::env::remove_var(CLICKHOUSE_DB_ENV) },
        }

        match &self.user {
            Some(value) => unsafe { std::env::set_var(CLICKHOUSE_USER_ENV, value) },
            None => unsafe { std::env::remove_var(CLICKHOUSE_USER_ENV) },
        }

        match &self.password {
            Some(value) => unsafe { std::env::set_var(CLICKHOUSE_PASSWORD_ENV, value) },
            None => unsafe { std::env::remove_var(CLICKHOUSE_PASSWORD_ENV) },
        }

        match &self.watchlist_path {
            Some(value) => unsafe { std::env::set_var(WATCHLIST_PATH_ENV, value) },
            None => unsafe { std::env::remove_var(WATCHLIST_PATH_ENV) },
        }

        match &self.trade_path {
            Some(value) => unsafe { std::env::set_var(TRADE_PATH_ENV, value) },
            None => unsafe { std::env::remove_var(TRADE_PATH_ENV) },
        }

        match &self.risk_path {
            Some(value) => unsafe { std::env::set_var(RISK_PATH_ENV, value) },
            None => unsafe { std::env::remove_var(RISK_PATH_ENV) },
        }

        match &self.monitor_db_path {
            Some(value) => unsafe { std::env::set_var(MONITOR_DB_PATH_ENV, value) },
            None => unsafe { std::env::remove_var(MONITOR_DB_PATH_ENV) },
        }

        match &self.monitor_config_path {
            Some(value) => unsafe { std::env::set_var(MONITOR_CONFIG_PATH_ENV, value) },
            None => unsafe { std::env::remove_var(MONITOR_CONFIG_PATH_ENV) },
        }

        match &self.strategy_config_path {
            Some(value) => unsafe { std::env::set_var(STRATEGY_CONFIG_PATH_ENV, value) },
            None => unsafe { std::env::remove_var(STRATEGY_CONFIG_PATH_ENV) },
        }

        match &self.strategy_runtime_db_path {
            Some(value) => unsafe { std::env::set_var(STRATEGY_RUNTIME_DB_PATH_ENV, value) },
            None => unsafe { std::env::remove_var(STRATEGY_RUNTIME_DB_PATH_ENV) },
        }

        match &self.execution_config_path {
            Some(value) => unsafe { std::env::set_var(EXECUTION_CONFIG_PATH_ENV, value) },
            None => unsafe { std::env::remove_var(EXECUTION_CONFIG_PATH_ENV) },
        }

        match &self.home {
            Some(value) => unsafe { std::env::set_var("HOME", value) },
            None => unsafe { std::env::remove_var("HOME") },
        }
    }
}
