use crate::core::config::{
    CLICKHOUSE_DB_ENV, CLICKHOUSE_PASSWORD_ENV, CLICKHOUSE_URL_ENV, CLICKHOUSE_USER_ENV,
    DEFAULT_CLICKHOUSE_DB, DEFAULT_CLICKHOUSE_PASSWORD, DEFAULT_CLICKHOUSE_URL,
    DEFAULT_CLICKHOUSE_USER,
};
use std::path::PathBuf;

pub const WATCHLIST_PATH_ENV: &str = "QUANTIX_WATCHLIST_PATH";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClickHouseSettings {
    pub url: String,
    pub database: String,
    pub user: String,
    pub password: String,
}

impl ClickHouseSettings {
    pub fn from_env() -> Self {
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
pub struct CliRuntime {
    pub clickhouse: ClickHouseSettings,
    pub watchlist_path: PathBuf,
}

impl CliRuntime {
    pub fn load() -> Self {
        Self {
            clickhouse: ClickHouseSettings::from_env(),
            watchlist_path: resolve_watchlist_path(),
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
        user: Option<String>,
        password: Option<String>,
        watchlist_path: Option<String>,
    }

    impl ClickHouseEnvGuard {
        fn capture() -> Self {
            Self {
                url: std::env::var(CLICKHOUSE_URL_ENV).ok(),
                database: std::env::var(CLICKHOUSE_DB_ENV).ok(),
                user: std::env::var(CLICKHOUSE_USER_ENV).ok(),
                password: std::env::var(CLICKHOUSE_PASSWORD_ENV).ok(),
                watchlist_path: std::env::var(WATCHLIST_PATH_ENV).ok(),
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
        }
    }

    #[test]
    fn test_clickhouse_settings_default_values() {
        let _lock = env_lock();
        let _guard = ClickHouseEnvGuard::capture();
        unsafe {
            std::env::remove_var(CLICKHOUSE_URL_ENV);
            std::env::remove_var(CLICKHOUSE_DB_ENV);
            std::env::remove_var(CLICKHOUSE_USER_ENV);
            std::env::remove_var(CLICKHOUSE_PASSWORD_ENV);
        }

        let settings = ClickHouseSettings::from_env();
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
}
