use super::settings::{
    BRIDGE_API_KEY_ENV, BRIDGE_BASE_URL_ENV, BRIDGE_BEARER_TOKEN_ENV, BRIDGE_CONTRACT_VERSION_ENV,
    BRIDGE_POLL_INTERVAL_MS_ENV, BRIDGE_POLL_TIMEOUT_MS_ENV, BRIDGE_TIMEOUT_MS_ENV,
    BridgeRuntimeSettings, CliRuntime, ClickHouseSettings, DEFAULT_BRIDGE_BASE_URL,
    DEFAULT_BRIDGE_CONTRACT_VERSION, DEFAULT_BRIDGE_POLL_INTERVAL_MS,
    DEFAULT_BRIDGE_POLL_TIMEOUT_MS, DEFAULT_BRIDGE_TIMEOUT_MS, DEFAULT_OPENSTOCK_TIMEOUT_SECS,
    EXECUTION_CONFIG_PATH_ENV, MONITOR_CONFIG_PATH_ENV, MONITOR_DB_PATH_ENV, OPENSTOCK_API_KEY_ENV,
    OPENSTOCK_BASE_URL_ENV, OpenStockSettings, RISK_PATH_ENV, STRATEGY_CONFIG_PATH_ENV,
    STRATEGY_RUNTIME_DB_PATH_ENV, TRADE_PATH_ENV, UpstreamMySqlSettings, WATCHLIST_PATH_ENV,
};
use crate::core::config::{
    CLICKHOUSE_DB_ENV, CLICKHOUSE_PASSWORD_ENV, CLICKHOUSE_URL_ENV, CLICKHOUSE_USER_ENV,
    DEFAULT_CLICKHOUSE_DB, DEFAULT_CLICKHOUSE_PASSWORD, DEFAULT_CLICKHOUSE_URL,
    DEFAULT_CLICKHOUSE_USER, DEFAULT_UPSTREAM_MYSQL_DB, DEFAULT_UPSTREAM_MYSQL_PASSWORD,
    DEFAULT_UPSTREAM_MYSQL_URL, DEFAULT_UPSTREAM_MYSQL_USER, UPSTREAM_MYSQL_DB_ENV,
    UPSTREAM_MYSQL_PASSWORD_ENV, UPSTREAM_MYSQL_URL_ENV, UPSTREAM_MYSQL_USER_ENV,
};
use std::path::PathBuf;

impl ClickHouseSettings {
    /// 从环境变量加载 ClickHouse 连接配置（URL/database/user/password），缺省时使用 DEFAULT_* 常量；过程中会先尝试加载 .env。
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

impl UpstreamMySqlSettings {
    /// 从环境变量加载上游 MySQL 连接配置，缺省时回退到 DEFAULT_* 常量；先尝试加载 .env。
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

impl CliRuntime {
    /// 聚合所有子系统的 from_env / resolve_* 结果构造 CliRuntime：涵盖 clickhouse / bridge / upstream_mysql / openstock / 各类路径（watchlist/trade/risk/monitor/strategy/execution）。
    pub fn load() -> Self {
        load_dotenv_if_present();
        Self {
            clickhouse: ClickHouseSettings::from_env(),
            bridge: BridgeRuntimeSettings::from_env(),
            upstream_mysql: UpstreamMySqlSettings::from_env(),
            openstock: OpenStockSettings::from_env(),
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
    /// 从环境变量加载 bridge 运行时配置（base_url、api_key/bearer、contract_version、各 timeout/poll 数值，以及 tdx_enabled / qmt_preview_enabled 开关）；缺省或解析失败回退 DEFAULT_*。
    pub fn from_env() -> Self {
        let api_key = optional_env(BRIDGE_API_KEY_ENV);
        Self {
            base_url: std::env::var(BRIDGE_BASE_URL_ENV)
                .unwrap_or_else(|_| DEFAULT_BRIDGE_BASE_URL.to_string()),
            api_key: api_key.clone(),
            bearer_token: optional_env(BRIDGE_BEARER_TOKEN_ENV),
            api_key_fallback: api_key,
            contract_version: std::env::var(BRIDGE_CONTRACT_VERSION_ENV)
                .ok()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| DEFAULT_BRIDGE_CONTRACT_VERSION.to_string()),
            timeout_ms: parse_u64_env(BRIDGE_TIMEOUT_MS_ENV, DEFAULT_BRIDGE_TIMEOUT_MS),
            poll_interval_ms: parse_u64_env(
                BRIDGE_POLL_INTERVAL_MS_ENV,
                DEFAULT_BRIDGE_POLL_INTERVAL_MS,
            ),
            poll_timeout_ms: parse_u64_env(
                BRIDGE_POLL_TIMEOUT_MS_ENV,
                DEFAULT_BRIDGE_POLL_TIMEOUT_MS,
            ),
            tdx_enabled: true,
            qmt_preview_enabled: true,
        }
    }
}

impl OpenStockSettings {
    /// 从环境变量加载 OpenStock 配置（base_url / api_key 可选，timeout 默认 DEFAULT_OPENSTOCK_TIMEOUT_SECS）；先尝试加载 .env。
    pub fn from_env() -> Self {
        load_dotenv_if_present();
        Self {
            base_url: optional_env(OPENSTOCK_BASE_URL_ENV),
            api_key: optional_env(OPENSTOCK_API_KEY_ENV),
            timeout_secs: DEFAULT_OPENSTOCK_TIMEOUT_SECS,
        }
    }
}

fn optional_env(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn parse_u64_env(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default)
}

fn load_dotenv_if_present() {
    if let Err(err) = dotenv::dotenv() {
        if err.not_found() {
            tracing::debug!(".env 文件不存在，跳过加载");
        } else {
            tracing::warn!(".env 加载失败: {}", err);
        }
    }
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
