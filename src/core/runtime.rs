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

impl CliRuntime {
    pub fn load() -> Self {
        load_dotenv_if_present();
        Self {
            clickhouse: ClickHouseSettings::from_env(),
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
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};
    use tempfile::tempdir;

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|err| err.into_inner())
    }

    struct ClickHouseEnvGuard {
        url: Option<String>,
        database: Option<String>,
        user: Option<String>,
        password: Option<String>,
        upstream_mysql_url: Option<String>,
        upstream_mysql_database: Option<String>,
        upstream_mysql_user: Option<String>,
        upstream_mysql_password: Option<String>,
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
                upstream_mysql_url: std::env::var(UPSTREAM_MYSQL_URL_ENV).ok(),
                upstream_mysql_database: std::env::var(UPSTREAM_MYSQL_DB_ENV).ok(),
                upstream_mysql_user: std::env::var(UPSTREAM_MYSQL_USER_ENV).ok(),
                upstream_mysql_password: std::env::var(UPSTREAM_MYSQL_PASSWORD_ENV).ok(),
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

            match &self.upstream_mysql_url {
                Some(value) => unsafe { std::env::set_var(UPSTREAM_MYSQL_URL_ENV, value) },
                None => unsafe { std::env::remove_var(UPSTREAM_MYSQL_URL_ENV) },
            }

            match &self.upstream_mysql_database {
                Some(value) => unsafe { std::env::set_var(UPSTREAM_MYSQL_DB_ENV, value) },
                None => unsafe { std::env::remove_var(UPSTREAM_MYSQL_DB_ENV) },
            }

            match &self.upstream_mysql_user {
                Some(value) => unsafe { std::env::set_var(UPSTREAM_MYSQL_USER_ENV, value) },
                None => unsafe { std::env::remove_var(UPSTREAM_MYSQL_USER_ENV) },
            }

            match &self.upstream_mysql_password {
                Some(value) => unsafe { std::env::set_var(UPSTREAM_MYSQL_PASSWORD_ENV, value) },
                None => unsafe { std::env::remove_var(UPSTREAM_MYSQL_PASSWORD_ENV) },
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
    fn test_cli_runtime_loads_upstream_mysql_settings() {
        let _lock = env_lock();
        let _guard = ClickHouseEnvGuard::capture();
        unsafe {
            std::env::set_var(UPSTREAM_MYSQL_URL_ENV, "mysql://192.168.123.104:3306");
            std::env::set_var(UPSTREAM_MYSQL_DB_ENV, "mystocks");
            std::env::set_var(UPSTREAM_MYSQL_USER_ENV, "root");
            std::env::set_var(UPSTREAM_MYSQL_PASSWORD_ENV, "secret");
        }

        let runtime = CliRuntime::load();

        assert_eq!(runtime.upstream_mysql.url, "mysql://192.168.123.104:3306");
        assert_eq!(runtime.upstream_mysql.database, "mystocks");
        assert_eq!(runtime.upstream_mysql.user, "root");
        assert_eq!(runtime.upstream_mysql.password, "secret");
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
}
