use crate::core::config::{
    CLICKHOUSE_DB_ENV, CLICKHOUSE_PASSWORD_ENV, CLICKHOUSE_URL_ENV, CLICKHOUSE_USER_ENV,
    DEFAULT_CLICKHOUSE_DB, DEFAULT_CLICKHOUSE_PASSWORD, DEFAULT_CLICKHOUSE_URL,
    DEFAULT_CLICKHOUSE_USER, DEFAULT_UPSTREAM_MYSQL_DB, DEFAULT_UPSTREAM_MYSQL_PASSWORD,
    DEFAULT_UPSTREAM_MYSQL_URL, DEFAULT_UPSTREAM_MYSQL_USER, UPSTREAM_MYSQL_DB_ENV,
    UPSTREAM_MYSQL_PASSWORD_ENV, UPSTREAM_MYSQL_URL_ENV, UPSTREAM_MYSQL_USER_ENV,
};
use std::path::PathBuf;

pub const WATCHLIST_PATH_ENV: &str = "QUANTIX_WATCHLIST_PATH";
pub const TRADE_PATH_ENV: &str = "QUANTIX_TRADE_PATH";
pub const RISK_PATH_ENV: &str = "QUANTIX_RISK_PATH";
pub const MONITOR_DB_PATH_ENV: &str = "QUANTIX_MONITOR_DB_PATH";
pub const MONITOR_CONFIG_PATH_ENV: &str = "QUANTIX_MONITOR_CONFIG_PATH";
pub const STRATEGY_CONFIG_PATH_ENV: &str = "QUANTIX_STRATEGY_CONFIG_PATH";
pub const STRATEGY_RUNTIME_DB_PATH_ENV: &str = "QUANTIX_STRATEGY_RUNTIME_DB_PATH";
pub const EXECUTION_CONFIG_PATH_ENV: &str = "QUANTIX_EXECUTION_CONFIG_PATH";
pub const BRIDGE_BASE_URL_ENV: &str = "QUANTIX_BRIDGE_BASE_URL";
pub const BRIDGE_API_KEY_ENV: &str = "QUANTIX_BRIDGE_API_KEY";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClickHouseSettings {
    pub url: String,
    pub database: String,
    pub user: String,
    pub password: String,
}

impl ClickHouseSettings {
    pub fn from_env() -> Self {
        load_dotenv_if_present();
        Self {
            url: std::env::var(CLICKHOUSE_URL_ENV)
                .unwrap_or_else(|_| DEFAULT_CLICKHOUSE_URL.to_string()),
            database: std::env::var(CLICKHOUSE_DB_ENV)
                .unwrap_or_else(|_| DEFAULT_CLICKHOUSE_DB.to_string()),
            user: std::env::var(CLICKHOUSE_USER_ENV)
                .unwrap_or_else(|_| DEFAULT_CLICKHOUSE_USER.to_string()),
            password: std::env::var(CLICKHOUSE_PASSWORD_ENV)
                .unwrap_or_else(|_| DEFAULT_CLICKHOUSE_PASSWORD.to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpstreamMySqlSettings {
    pub url: String,
    pub database: String,
    pub user: String,
    pub password: String,
}

impl UpstreamMySqlSettings {
    pub fn from_env() -> Self {
        load_dotenv_if_present();
        Self {
            url: std::env::var(UPSTREAM_MYSQL_URL_ENV)
                .unwrap_or_else(|_| DEFAULT_UPSTREAM_MYSQL_URL.to_string()),
            database: std::env::var(UPSTREAM_MYSQL_DB_ENV)
                .unwrap_or_else(|_| DEFAULT_UPSTREAM_MYSQL_DB.to_string()),
            user: std::env::var(UPSTREAM_MYSQL_USER_ENV)
                .unwrap_or_else(|_| DEFAULT_UPSTREAM_MYSQL_USER.to_string()),
            password: std::env::var(UPSTREAM_MYSQL_PASSWORD_ENV)
                .unwrap_or_else(|_| DEFAULT_UPSTREAM_MYSQL_PASSWORD.to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliRuntime {
    pub clickhouse: ClickHouseSettings,
    pub bridge: BridgeRuntimeSettings,
    pub upstream_mysql: UpstreamMySqlSettings,
    pub watchlist_path: PathBuf,
    pub trade_path: PathBuf,
    pub risk_path: PathBuf,
    pub monitor_db_path: PathBuf,
    pub monitor_config_path: PathBuf,
    pub strategy_config_path: PathBuf,
    pub strategy_runtime_db_path: PathBuf,
    pub execution_config_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeRuntimeSettings {
    pub base_url: String,
    pub api_key: Option<String>,
    pub tdx_enabled: bool,
    pub qmt_preview_enabled: bool,
}

impl CliRuntime {
    pub fn load() -> Self {
        load_dotenv_if_present();
        Self {
            clickhouse: ClickHouseSettings::from_env(),
            bridge: BridgeRuntimeSettings::from_env(),
            upstream_mysql: UpstreamMySqlSettings::from_env(),
            watchlist_path: resolve_watchlist_path(),
            trade_path: resolve_trade_path(),
            risk_path: resolve_risk_path(),
            monitor_db_path: resolve_monitor_db_path(),
            monitor_config_path: resolve_monitor_config_path(),
            strategy_config_path: resolve_strategy_config_path(),
            strategy_runtime_db_path: resolve_strategy_runtime_db_path(),
            execution_config_path: resolve_execution_config_path(),
        }
    }
}

impl BridgeRuntimeSettings {
    pub fn from_env() -> Self {
        Self {
            base_url: std::env::var(BRIDGE_BASE_URL_ENV)
                .unwrap_or_else(|_| "http://127.0.0.1:17580".to_string()),
            api_key: std::env::var(BRIDGE_API_KEY_ENV).ok(),
            tdx_enabled: true,
            qmt_preview_enabled: true,
        }
    }
}

fn load_dotenv_if_present() {
    let _ = dotenv::dotenv();
}

fn resolve_watchlist_path() -> PathBuf {
    if let Some(path) = std::env::var_os(WATCHLIST_PATH_ENV) {
        return PathBuf::from(path);
    }

    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home)
            .join(".quantix")
            .join("watchlist")
            .join("watchlist.json");
    }

    PathBuf::from(".quantix")
        .join("watchlist")
        .join("watchlist.json")
}

fn resolve_monitor_db_path() -> PathBuf {
    if let Some(path) = std::env::var_os(MONITOR_DB_PATH_ENV) {
        return PathBuf::from(path);
    }

    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home)
            .join(".quantix")
            .join("monitor")
            .join("alerts.db");
    }

    PathBuf::from(".quantix").join("monitor").join("alerts.db")
}

fn resolve_monitor_config_path() -> PathBuf {
    if let Some(path) = std::env::var_os(MONITOR_CONFIG_PATH_ENV) {
        return PathBuf::from(path);
    }

    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home)
            .join(".quantix")
            .join("monitor")
            .join("config.json");
    }

    PathBuf::from(".quantix")
        .join("monitor")
        .join("config.json")
}

fn resolve_trade_path() -> PathBuf {
    if let Some(path) = std::env::var_os(TRADE_PATH_ENV) {
        return PathBuf::from(path);
    }

    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home)
            .join(".quantix")
            .join("trade")
            .join("paper_trade.json");
    }

    PathBuf::from(".quantix")
        .join("trade")
        .join("paper_trade.json")
}

fn resolve_risk_path() -> PathBuf {
    if let Some(path) = std::env::var_os(RISK_PATH_ENV) {
        return PathBuf::from(path);
    }

    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home)
            .join(".quantix")
            .join("risk")
            .join("risk_state.json");
    }

    PathBuf::from(".quantix")
        .join("risk")
        .join("risk_state.json")
}

fn resolve_strategy_config_path() -> PathBuf {
    if let Some(path) = std::env::var_os(STRATEGY_CONFIG_PATH_ENV) {
        return PathBuf::from(path);
    }

    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home)
            .join(".quantix")
            .join("strategy")
            .join("config.json");
    }

    PathBuf::from(".quantix")
        .join("strategy")
        .join("config.json")
}

fn resolve_strategy_runtime_db_path() -> PathBuf {
    if let Some(path) = std::env::var_os(STRATEGY_RUNTIME_DB_PATH_ENV) {
        return PathBuf::from(path);
    }

    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home)
            .join(".quantix")
            .join("strategy")
            .join("runtime.db");
    }

    PathBuf::from(".quantix")
        .join("strategy")
        .join("runtime.db")
}

fn resolve_execution_config_path() -> PathBuf {
    if let Some(path) = std::env::var_os(EXECUTION_CONFIG_PATH_ENV) {
        return PathBuf::from(path);
    }

    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home)
            .join(".quantix")
            .join("execution")
            .join("config.json");
    }

    PathBuf::from(".quantix")
        .join("execution")
        .join("config.json")
}

#[cfg(test)]
#[path = "runtime/runtime_config_test.rs"]
mod runtime_config_test;
#[cfg(test)]
#[path = "runtime/runtime_paths_test.rs"]
mod runtime_paths_test;
#[cfg(test)]
#[path = "runtime/runtime_test_support.rs"]
mod runtime_test_support;
