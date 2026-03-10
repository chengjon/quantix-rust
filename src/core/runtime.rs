use crate::core::config::{
    CLICKHOUSE_DB_ENV, CLICKHOUSE_URL_ENV, DEFAULT_CLICKHOUSE_DB, DEFAULT_CLICKHOUSE_URL,
};
use std::path::PathBuf;

pub const WATCHLIST_PATH_ENV: &str = "QUANTIX_WATCHLIST_PATH";
pub const MONITOR_DB_PATH_ENV: &str = "QUANTIX_MONITOR_DB_PATH";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClickHouseSettings {
    pub url: String,
    pub database: String,
}

impl ClickHouseSettings {
    pub fn from_env() -> Self {
        Self {
            url: std::env::var(CLICKHOUSE_URL_ENV)
                .unwrap_or_else(|_| DEFAULT_CLICKHOUSE_URL.to_string()),
            database: std::env::var(CLICKHOUSE_DB_ENV)
                .unwrap_or_else(|_| DEFAULT_CLICKHOUSE_DB.to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliRuntime {
    pub clickhouse: ClickHouseSettings,
    pub watchlist_path: PathBuf,
    pub monitor_db_path: PathBuf,
}

impl CliRuntime {
    pub fn load() -> Self {
        Self {
            clickhouse: ClickHouseSettings::from_env(),
            watchlist_path: resolve_watchlist_path(),
            monitor_db_path: resolve_monitor_db_path(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|err| err.into_inner())
    }

    struct ClickHouseEnvGuard {
        url: Option<String>,
        database: Option<String>,
        watchlist_path: Option<String>,
        monitor_db_path: Option<String>,
        home: Option<String>,
    }

    impl ClickHouseEnvGuard {
        fn capture() -> Self {
            Self {
                url: std::env::var(CLICKHOUSE_URL_ENV).ok(),
                database: std::env::var(CLICKHOUSE_DB_ENV).ok(),
                watchlist_path: std::env::var(WATCHLIST_PATH_ENV).ok(),
                monitor_db_path: std::env::var(MONITOR_DB_PATH_ENV).ok(),
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

            match &self.watchlist_path {
                Some(value) => unsafe { std::env::set_var(WATCHLIST_PATH_ENV, value) },
                None => unsafe { std::env::remove_var(WATCHLIST_PATH_ENV) },
            }

            match &self.monitor_db_path {
                Some(value) => unsafe { std::env::set_var(MONITOR_DB_PATH_ENV, value) },
                None => unsafe { std::env::remove_var(MONITOR_DB_PATH_ENV) },
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
        unsafe {
            std::env::remove_var(CLICKHOUSE_URL_ENV);
            std::env::remove_var(CLICKHOUSE_DB_ENV);
        }

        let settings = ClickHouseSettings::from_env();
        assert_eq!(settings.url, DEFAULT_CLICKHOUSE_URL);
        assert_eq!(settings.database, DEFAULT_CLICKHOUSE_DB);
    }

    #[test]
    fn test_clickhouse_settings_env_override() {
        let _lock = env_lock();
        let _guard = ClickHouseEnvGuard::capture();
        unsafe {
            std::env::set_var(CLICKHOUSE_URL_ENV, "http://example:9000");
            std::env::set_var(CLICKHOUSE_DB_ENV, "quantix_test");
        }

        let settings = ClickHouseSettings::from_env();
        assert_eq!(settings.url, "http://example:9000");
        assert_eq!(settings.database, "quantix_test");
    }

    #[test]
    fn test_cli_runtime_loads_clickhouse_settings() {
        let _lock = env_lock();
        let _guard = ClickHouseEnvGuard::capture();
        unsafe {
            std::env::set_var(CLICKHOUSE_URL_ENV, "http://runtime:8123");
            std::env::set_var(CLICKHOUSE_DB_ENV, "runtime_db");
        }

        let runtime = CliRuntime::load();
        assert_eq!(runtime.clickhouse.url, "http://runtime:8123");
        assert_eq!(runtime.clickhouse.database, "runtime_db");
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
}
