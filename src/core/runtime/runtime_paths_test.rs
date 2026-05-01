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
