//! 运行时测试的共享环境守卫。
//!
//! 捕获并还原所有运行时 env 变量，使测试之间互不污染。

// ClickHouse 连接常量在 `crate::config`；路径 / bridge 常量在 `crate::runtime::settings`
// 并被父模块 glob 重导出，通过 `use super::*` 带入。
use super::*;
use crate::core::config::{
    CLICKHOUSE_DB_ENV, CLICKHOUSE_PASSWORD_ENV, CLICKHOUSE_URL_ENV, CLICKHOUSE_USER_ENV,
};

pub(crate) struct ClickHouseEnvGuard {
    url: Option<String>,
    database: Option<String>,
    user: Option<String>,
    password: Option<String>,
    bridge_base_url: Option<String>,
    bridge_api_key: Option<String>,
    bridge_bearer_token: Option<String>,
    bridge_contract_version: Option<String>,
    bridge_timeout_ms: Option<String>,
    bridge_poll_interval_ms: Option<String>,
    bridge_poll_timeout_ms: Option<String>,
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
    pub(crate) fn capture() -> Self {
        Self {
            url: std::env::var(CLICKHOUSE_URL_ENV).ok(),
            database: std::env::var(CLICKHOUSE_DB_ENV).ok(),
            user: std::env::var(CLICKHOUSE_USER_ENV).ok(),
            password: std::env::var(CLICKHOUSE_PASSWORD_ENV).ok(),
            bridge_base_url: std::env::var(BRIDGE_BASE_URL_ENV).ok(),
            bridge_api_key: std::env::var(BRIDGE_API_KEY_ENV).ok(),
            bridge_bearer_token: std::env::var(BRIDGE_BEARER_TOKEN_ENV).ok(),
            bridge_contract_version: std::env::var(BRIDGE_CONTRACT_VERSION_ENV).ok(),
            bridge_timeout_ms: std::env::var(BRIDGE_TIMEOUT_MS_ENV).ok(),
            bridge_poll_interval_ms: std::env::var(BRIDGE_POLL_INTERVAL_MS_ENV).ok(),
            bridge_poll_timeout_ms: std::env::var(BRIDGE_POLL_TIMEOUT_MS_ENV).ok(),
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

fn restore(env: &str, value: &Option<String>) {
    match value {
        Some(value) => unsafe { std::env::set_var(env, value) },
        None => unsafe { std::env::remove_var(env) },
    }
}

impl Drop for ClickHouseEnvGuard {
    fn drop(&mut self) {
        restore(CLICKHOUSE_URL_ENV, &self.url);
        restore(CLICKHOUSE_DB_ENV, &self.database);
        restore(CLICKHOUSE_USER_ENV, &self.user);
        restore(CLICKHOUSE_PASSWORD_ENV, &self.password);
        restore(BRIDGE_BASE_URL_ENV, &self.bridge_base_url);
        restore(BRIDGE_API_KEY_ENV, &self.bridge_api_key);
        restore(BRIDGE_BEARER_TOKEN_ENV, &self.bridge_bearer_token);
        restore(BRIDGE_CONTRACT_VERSION_ENV, &self.bridge_contract_version);
        restore(BRIDGE_TIMEOUT_MS_ENV, &self.bridge_timeout_ms);
        restore(BRIDGE_POLL_INTERVAL_MS_ENV, &self.bridge_poll_interval_ms);
        restore(BRIDGE_POLL_TIMEOUT_MS_ENV, &self.bridge_poll_timeout_ms);
        restore(WATCHLIST_PATH_ENV, &self.watchlist_path);
        restore(TRADE_PATH_ENV, &self.trade_path);
        restore(RISK_PATH_ENV, &self.risk_path);
        restore(MONITOR_DB_PATH_ENV, &self.monitor_db_path);
        restore(MONITOR_CONFIG_PATH_ENV, &self.monitor_config_path);
        restore(STRATEGY_CONFIG_PATH_ENV, &self.strategy_config_path);
        restore(STRATEGY_RUNTIME_DB_PATH_ENV, &self.strategy_runtime_db_path);
        restore(EXECUTION_CONFIG_PATH_ENV, &self.execution_config_path);
        restore("HOME", &self.home);
    }
}
