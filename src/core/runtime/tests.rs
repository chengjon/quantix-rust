use super::*;
use crate::core::config::{
    CLICKHOUSE_DB_ENV, CLICKHOUSE_PASSWORD_ENV, CLICKHOUSE_URL_ENV, CLICKHOUSE_USER_ENV,
    DEFAULT_CLICKHOUSE_DB, DEFAULT_CLICKHOUSE_PASSWORD, DEFAULT_CLICKHOUSE_URL,
    DEFAULT_CLICKHOUSE_USER,
};
use crate::test_support::env_lock;
use std::path::PathBuf;
use tempfile::tempdir;

struct ClickHouseEnvGuard {
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
    fn capture() -> Self {
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

        match &self.bridge_base_url {
            Some(value) => unsafe { std::env::set_var(BRIDGE_BASE_URL_ENV, value) },
            None => unsafe { std::env::remove_var(BRIDGE_BASE_URL_ENV) },
        }

        match &self.bridge_api_key {
            Some(value) => unsafe { std::env::set_var(BRIDGE_API_KEY_ENV, value) },
            None => unsafe { std::env::remove_var(BRIDGE_API_KEY_ENV) },
        }

        match &self.bridge_bearer_token {
            Some(value) => unsafe { std::env::set_var(BRIDGE_BEARER_TOKEN_ENV, value) },
            None => unsafe { std::env::remove_var(BRIDGE_BEARER_TOKEN_ENV) },
        }

        match &self.bridge_contract_version {
            Some(value) => unsafe { std::env::set_var(BRIDGE_CONTRACT_VERSION_ENV, value) },
            None => unsafe { std::env::remove_var(BRIDGE_CONTRACT_VERSION_ENV) },
        }

        match &self.bridge_timeout_ms {
            Some(value) => unsafe { std::env::set_var(BRIDGE_TIMEOUT_MS_ENV, value) },
            None => unsafe { std::env::remove_var(BRIDGE_TIMEOUT_MS_ENV) },
        }

        match &self.bridge_poll_interval_ms {
            Some(value) => unsafe { std::env::set_var(BRIDGE_POLL_INTERVAL_MS_ENV, value) },
            None => unsafe { std::env::remove_var(BRIDGE_POLL_INTERVAL_MS_ENV) },
        }

        match &self.bridge_poll_timeout_ms {
            Some(value) => unsafe { std::env::set_var(BRIDGE_POLL_TIMEOUT_MS_ENV, value) },
            None => unsafe { std::env::remove_var(BRIDGE_POLL_TIMEOUT_MS_ENV) },
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

#[test]
fn test_clickhouse_settings_default_values() {
    let _lock = env_lock();
    let _guard = ClickHouseEnvGuard::capture();
    let cwd = std::env::current_dir().unwrap();
    let temp = tempdir().unwrap();
    unsafe {
        std::env::remove_var(CLICKHOUSE_URL_ENV);
        std::env::remove_var(CLICKHOUSE_DB_ENV);
        std::env::remove_var(CLICKHOUSE_USER_ENV);
        std::env::remove_var(CLICKHOUSE_PASSWORD_ENV);
    }
    std::env::set_current_dir(temp.path()).unwrap();

    let settings = ClickHouseSettings::from_env();

    std::env::set_current_dir(cwd).unwrap();
    assert_eq!(settings.url, DEFAULT_CLICKHOUSE_URL);
    assert_eq!(settings.database, DEFAULT_CLICKHOUSE_DB);
    assert_eq!(settings.user, DEFAULT_CLICKHOUSE_USER);
    assert_eq!(settings.password, DEFAULT_CLICKHOUSE_PASSWORD);
}

#[test]
fn test_clickhouse_settings_env_override() {
    let _lock = env_lock();
    let _guard = ClickHouseEnvGuard::capture();
    unsafe {
        std::env::set_var(CLICKHOUSE_URL_ENV, "http://example:9000");
        std::env::set_var(CLICKHOUSE_DB_ENV, "quantix_test");
        std::env::set_var(CLICKHOUSE_USER_ENV, "runtime_user");
        std::env::set_var(CLICKHOUSE_PASSWORD_ENV, "runtime_password");
    }

    let settings = ClickHouseSettings::from_env();
    assert_eq!(settings.url, "http://example:9000");
    assert_eq!(settings.database, "quantix_test");
    assert_eq!(settings.user, "runtime_user");
    assert_eq!(settings.password, "runtime_password");
}

#[test]
fn test_cli_runtime_loads_clickhouse_settings_from_dotenv_file() {
    let _lock = env_lock();
    let _guard = ClickHouseEnvGuard::capture();
    let cwd = std::env::current_dir().unwrap();
    let temp = tempdir().unwrap();
    let env_path = temp.path().join(".env");

    std::fs::write(
        &env_path,
        [
            "CLICKHOUSE_URL=http://192.168.123.104:8123",
            "CLICKHOUSE_DB=quantix",
            "CLICKHOUSE_USER=default",
            "CLICKHOUSE_PASSWORD=c790414J",
        ]
        .join("\n"),
    )
    .unwrap();

    unsafe {
        std::env::remove_var(CLICKHOUSE_URL_ENV);
        std::env::remove_var(CLICKHOUSE_DB_ENV);
        std::env::remove_var(CLICKHOUSE_USER_ENV);
        std::env::remove_var(CLICKHOUSE_PASSWORD_ENV);
    }
    std::env::set_current_dir(temp.path()).unwrap();

    let runtime = CliRuntime::load();

    std::env::set_current_dir(cwd).unwrap();

    assert_eq!(runtime.clickhouse.url, "http://192.168.123.104:8123");
    assert_eq!(runtime.clickhouse.database, "quantix");
    assert_eq!(runtime.clickhouse.user, "default");
    assert_eq!(runtime.clickhouse.password, "c790414J");
}

#[test]
fn test_cli_runtime_loads_clickhouse_settings() {
    let _lock = env_lock();
    let _guard = ClickHouseEnvGuard::capture();
    unsafe {
        std::env::set_var(CLICKHOUSE_URL_ENV, "http://runtime:8123");
        std::env::set_var(CLICKHOUSE_DB_ENV, "runtime_db");
        std::env::set_var(CLICKHOUSE_USER_ENV, "cli_user");
        std::env::set_var(CLICKHOUSE_PASSWORD_ENV, "cli_password");
    }

    let runtime = CliRuntime::load();
    assert_eq!(runtime.clickhouse.url, "http://runtime:8123");
    assert_eq!(runtime.clickhouse.database, "runtime_db");
    assert_eq!(runtime.clickhouse.user, "cli_user");
    assert_eq!(runtime.clickhouse.password, "cli_password");
}

#[test]
fn test_bridge_runtime_settings_default_contract_values() {
    let _lock = env_lock();
    let _guard = ClickHouseEnvGuard::capture();
    unsafe {
        std::env::remove_var(BRIDGE_BASE_URL_ENV);
        std::env::remove_var(BRIDGE_API_KEY_ENV);
        std::env::remove_var(BRIDGE_BEARER_TOKEN_ENV);
        std::env::remove_var(BRIDGE_CONTRACT_VERSION_ENV);
        std::env::remove_var(BRIDGE_TIMEOUT_MS_ENV);
        std::env::remove_var(BRIDGE_POLL_INTERVAL_MS_ENV);
        std::env::remove_var(BRIDGE_POLL_TIMEOUT_MS_ENV);
    }

    let settings = BridgeRuntimeSettings::from_env();

    assert_eq!(settings.base_url, "http://127.0.0.1:17580");
    assert_eq!(settings.api_key, None);
    assert_eq!(settings.bearer_token, None);
    assert_eq!(settings.api_key_fallback, None);
    assert_eq!(settings.contract_version, "miniqmt.v1");
    assert_eq!(settings.timeout_ms, 30_000);
    assert_eq!(settings.poll_interval_ms, 1_000);
    assert_eq!(settings.poll_timeout_ms, 30_000);
}

#[test]
fn test_bridge_runtime_settings_contract_env_override() {
    let _lock = env_lock();
    let _guard = ClickHouseEnvGuard::capture();
    unsafe {
        std::env::set_var(BRIDGE_BASE_URL_ENV, "http://bridge.internal:18080");
        std::env::set_var(BRIDGE_API_KEY_ENV, "legacy-key");
        std::env::set_var(BRIDGE_BEARER_TOKEN_ENV, "bearer-123");
        std::env::set_var(BRIDGE_CONTRACT_VERSION_ENV, "miniqmt.v1beta");
        std::env::set_var(BRIDGE_TIMEOUT_MS_ENV, "45000");
        std::env::set_var(BRIDGE_POLL_INTERVAL_MS_ENV, "1500");
        std::env::set_var(BRIDGE_POLL_TIMEOUT_MS_ENV, "90000");
    }

    let settings = BridgeRuntimeSettings::from_env();

    assert_eq!(settings.base_url, "http://bridge.internal:18080");
    assert_eq!(settings.api_key.as_deref(), Some("legacy-key"));
    assert_eq!(settings.bearer_token.as_deref(), Some("bearer-123"));
    assert_eq!(settings.api_key_fallback.as_deref(), Some("legacy-key"));
    assert_eq!(settings.contract_version, "miniqmt.v1beta");
    assert_eq!(settings.timeout_ms, 45_000);
    assert_eq!(settings.poll_interval_ms, 1_500);
    assert_eq!(settings.poll_timeout_ms, 90_000);
}

#[test]
fn test_cli_runtime_uses_watchlist_path_override() {
    let _lock = env_lock();
    let _guard = ClickHouseEnvGuard::capture();
    unsafe {
        std::env::set_var(WATCHLIST_PATH_ENV, "/tmp/quantix/watchlist.json");
    }

    let runtime = CliRuntime::load();
    assert_eq!(
        runtime.watchlist_path,
        PathBuf::from("/tmp/quantix/watchlist.json")
    );
}

#[test]
fn test_monitor_db_path_env_override() {
    let _lock = env_lock();
    let _guard = ClickHouseEnvGuard::capture();
    unsafe {
        std::env::set_var(MONITOR_DB_PATH_ENV, "/tmp/quantix/monitor/custom-alerts.db");
    }

    let runtime = CliRuntime::load();
    assert_eq!(
        runtime.monitor_db_path,
        PathBuf::from("/tmp/quantix/monitor/custom-alerts.db")
    );
}

#[test]
fn test_monitor_config_path_env_override() {
    let _lock = env_lock();
    let _guard = ClickHouseEnvGuard::capture();
    unsafe {
        std::env::set_var(
            MONITOR_CONFIG_PATH_ENV,
            "/tmp/quantix/monitor/custom-config.json",
        );
    }

    let runtime = CliRuntime::load();
    assert_eq!(
        runtime.monitor_config_path,
        PathBuf::from("/tmp/quantix/monitor/custom-config.json")
    );
}

#[test]
fn test_cli_runtime_uses_trade_path_override() {
    let _lock = env_lock();
    let _guard = ClickHouseEnvGuard::capture();
    unsafe {
        std::env::set_var(TRADE_PATH_ENV, "/tmp/quantix/trade/custom-paper-trade.json");
    }

    let runtime = CliRuntime::load();
    assert_eq!(
        runtime.trade_path,
        PathBuf::from("/tmp/quantix/trade/custom-paper-trade.json")
    );
}

#[test]
fn test_cli_runtime_uses_risk_path_override() {
    let _lock = env_lock();
    let _guard = ClickHouseEnvGuard::capture();
    unsafe {
        std::env::set_var(RISK_PATH_ENV, "/tmp/quantix/risk/custom-risk-state.json");
    }

    let runtime = CliRuntime::load();
    assert_eq!(
        runtime.risk_path,
        PathBuf::from("/tmp/quantix/risk/custom-risk-state.json")
    );
}

#[test]
fn test_trade_path_falls_back_to_home_directory() {
    let _lock = env_lock();
    let _guard = ClickHouseEnvGuard::capture();
    unsafe {
        std::env::remove_var(TRADE_PATH_ENV);
        std::env::set_var("HOME", "/tmp/quantix-home");
    }

    let runtime = CliRuntime::load();
    assert_eq!(
        runtime.trade_path,
        PathBuf::from("/tmp/quantix-home")
            .join(".quantix")
            .join("trade")
            .join("paper_trade.json")
    );
}

#[test]
fn test_risk_path_falls_back_to_home_directory() {
    let _lock = env_lock();
    let _guard = ClickHouseEnvGuard::capture();
    unsafe {
        std::env::remove_var(RISK_PATH_ENV);
        std::env::set_var("HOME", "/tmp/quantix-home");
    }

    let runtime = CliRuntime::load();
    assert_eq!(
        runtime.risk_path,
        PathBuf::from("/tmp/quantix-home")
            .join(".quantix")
            .join("risk")
            .join("risk_state.json")
    );
}

#[test]
fn test_monitor_db_path_falls_back_to_home_directory() {
    let _lock = env_lock();
    let _guard = ClickHouseEnvGuard::capture();
    unsafe {
        std::env::remove_var(MONITOR_DB_PATH_ENV);
        std::env::set_var("HOME", "/tmp/quantix-home");
    }

    let runtime = CliRuntime::load();
    assert_eq!(
        runtime.monitor_db_path,
        PathBuf::from("/tmp/quantix-home")
            .join(".quantix")
            .join("monitor")
            .join("alerts.db")
    );
}

#[test]
fn test_monitor_config_path_falls_back_to_home_directory() {
    let _lock = env_lock();
    let _guard = ClickHouseEnvGuard::capture();
    unsafe {
        std::env::remove_var(MONITOR_CONFIG_PATH_ENV);
        std::env::set_var("HOME", "/tmp/quantix-home");
    }

    let runtime = CliRuntime::load();
    assert_eq!(
        runtime.monitor_config_path,
        PathBuf::from("/tmp/quantix-home")
            .join(".quantix")
            .join("monitor")
            .join("config.json")
    );
}

#[test]
fn test_cli_runtime_uses_strategy_runtime_db_path_override() {
    let _lock = env_lock();
    let _guard = ClickHouseEnvGuard::capture();
    unsafe {
        std::env::set_var(
            STRATEGY_RUNTIME_DB_PATH_ENV,
            "/tmp/quantix/strategy/custom-runtime.db",
        );
    }

    let runtime = CliRuntime::load();
    assert_eq!(
        runtime.strategy_runtime_db_path,
        PathBuf::from("/tmp/quantix/strategy/custom-runtime.db")
    );
}

#[test]
fn test_cli_runtime_uses_strategy_config_path_override() {
    let _lock = env_lock();
    let _guard = ClickHouseEnvGuard::capture();
    unsafe {
        std::env::set_var(
            STRATEGY_CONFIG_PATH_ENV,
            "/tmp/quantix/strategy/custom-config.json",
        );
    }

    let runtime = CliRuntime::load();
    assert_eq!(
        runtime.strategy_config_path,
        PathBuf::from("/tmp/quantix/strategy/custom-config.json")
    );
}

#[test]
fn test_strategy_config_path_falls_back_to_home_directory() {
    let _lock = env_lock();
    let _guard = ClickHouseEnvGuard::capture();
    unsafe {
        std::env::remove_var(STRATEGY_CONFIG_PATH_ENV);
        std::env::set_var("HOME", "/tmp/quantix-home");
    }

    let runtime = CliRuntime::load();
    assert_eq!(
        runtime.strategy_config_path,
        PathBuf::from("/tmp/quantix-home")
            .join(".quantix")
            .join("strategy")
            .join("config.json")
    );
}

#[test]
fn test_strategy_config_path_falls_back_to_relative_path_without_home() {
    let _lock = env_lock();
    let _guard = ClickHouseEnvGuard::capture();
    unsafe {
        std::env::remove_var(STRATEGY_CONFIG_PATH_ENV);
        std::env::remove_var("HOME");
    }

    let runtime = CliRuntime::load();
    assert_eq!(
        runtime.strategy_config_path,
        PathBuf::from(".quantix")
            .join("strategy")
            .join("config.json")
    );
}

#[test]
fn test_strategy_runtime_db_path_falls_back_to_home_directory() {
    let _lock = env_lock();
    let _guard = ClickHouseEnvGuard::capture();
    unsafe {
        std::env::remove_var(STRATEGY_RUNTIME_DB_PATH_ENV);
        std::env::set_var("HOME", "/tmp/quantix-home");
    }

    let runtime = CliRuntime::load();
    assert_eq!(
        runtime.strategy_runtime_db_path,
        PathBuf::from("/tmp/quantix-home")
            .join(".quantix")
            .join("strategy")
            .join("runtime.db")
    );
}

#[test]
fn test_strategy_runtime_db_path_falls_back_to_relative_path_without_home() {
    let _lock = env_lock();
    let _guard = ClickHouseEnvGuard::capture();
    unsafe {
        std::env::remove_var(STRATEGY_RUNTIME_DB_PATH_ENV);
        std::env::remove_var("HOME");
    }

    let runtime = CliRuntime::load();
    assert_eq!(
        runtime.strategy_runtime_db_path,
        PathBuf::from(".quantix")
            .join("strategy")
            .join("runtime.db")
    );
}

#[test]
fn test_cli_runtime_uses_execution_config_path_override() {
    let _lock = env_lock();
    let _guard = ClickHouseEnvGuard::capture();
    unsafe {
        std::env::set_var(
            EXECUTION_CONFIG_PATH_ENV,
            "/tmp/quantix/execution/custom-config.json",
        );
    }

    let runtime = CliRuntime::load();
    assert_eq!(
        runtime.execution_config_path,
        PathBuf::from("/tmp/quantix/execution/custom-config.json")
    );
}

#[test]
fn test_execution_config_path_falls_back_to_home_directory() {
    let _lock = env_lock();
    let _guard = ClickHouseEnvGuard::capture();
    unsafe {
        std::env::remove_var(EXECUTION_CONFIG_PATH_ENV);
        std::env::set_var("HOME", "/tmp/quantix-home");
    }

    let runtime = CliRuntime::load();
    assert_eq!(
        runtime.execution_config_path,
        PathBuf::from("/tmp/quantix-home")
            .join(".quantix")
            .join("execution")
            .join("config.json")
    );
}

#[test]
fn test_execution_config_path_falls_back_to_relative_path_without_home() {
    let _lock = env_lock();
    let _guard = ClickHouseEnvGuard::capture();
    unsafe {
        std::env::remove_var(EXECUTION_CONFIG_PATH_ENV);
        std::env::remove_var("HOME");
    }

    let runtime = CliRuntime::load();
    assert_eq!(
        runtime.execution_config_path,
        PathBuf::from(".quantix")
            .join("execution")
            .join("config.json")
    );
}
