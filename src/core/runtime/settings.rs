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
pub const BRIDGE_BEARER_TOKEN_ENV: &str = "QUANTIX_BRIDGE_BEARER_TOKEN";
pub const BRIDGE_CONTRACT_VERSION_ENV: &str = "QUANTIX_BRIDGE_CONTRACT_VERSION";
pub const BRIDGE_TIMEOUT_MS_ENV: &str = "QUANTIX_BRIDGE_TIMEOUT_MS";
pub const BRIDGE_POLL_INTERVAL_MS_ENV: &str = "QUANTIX_BRIDGE_POLL_INTERVAL_MS";
pub const BRIDGE_POLL_TIMEOUT_MS_ENV: &str = "QUANTIX_BRIDGE_POLL_TIMEOUT_MS";
pub const DEFAULT_BRIDGE_BASE_URL: &str = "http://127.0.0.1:17580";
pub const DEFAULT_BRIDGE_CONTRACT_VERSION: &str = "miniqmt.v1";
pub const DEFAULT_BRIDGE_TIMEOUT_MS: u64 = 30_000;
pub const DEFAULT_BRIDGE_POLL_INTERVAL_MS: u64 = 1_000;
pub const DEFAULT_BRIDGE_POLL_TIMEOUT_MS: u64 = 30_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClickHouseSettings {
    pub url: String,
    pub database: String,
    pub user: String,
    pub password: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpstreamMySqlSettings {
    pub url: String,
    pub database: String,
    pub user: String,
    pub password: String,
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
    pub bearer_token: Option<String>,
    pub api_key_fallback: Option<String>,
    pub contract_version: String,
    pub timeout_ms: u64,
    pub poll_interval_ms: u64,
    pub poll_timeout_ms: u64,
    pub tdx_enabled: bool,
    pub qmt_preview_enabled: bool,
}
