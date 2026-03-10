use crate::core::config::{
    CLICKHOUSE_DB_ENV, CLICKHOUSE_URL_ENV, DEFAULT_CLICKHOUSE_DB, DEFAULT_CLICKHOUSE_URL,
};

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
}

impl CliRuntime {
    pub fn load() -> Self {
        Self {
            clickhouse: ClickHouseSettings::from_env(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
    }

    struct ClickHouseEnvGuard {
        url: Option<String>,
        database: Option<String>,
    }

    impl ClickHouseEnvGuard {
        fn capture() -> Self {
            Self {
                url: std::env::var(CLICKHOUSE_URL_ENV).ok(),
                database: std::env::var(CLICKHOUSE_DB_ENV).ok(),
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
}
