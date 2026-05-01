use super::runtime_test_support::{ClickHouseEnvGuard, env_lock};
use super::*;

use tempfile::tempdir;

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
