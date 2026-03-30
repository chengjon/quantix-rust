use std::path::PathBuf;

use quantix_cli::core::{CliRuntime, ClickHouseSettings, UpstreamMySqlSettings};
use quantix_cli::strategy::{
    JsonStrategyServiceConfigStore, StrategyServiceConfig,
    systemd::{StrategyServiceStatusSummary, StrategyUserServiceInstaller},
};
use tempfile::tempdir;

fn sample_runtime() -> CliRuntime {
    CliRuntime {
        clickhouse: ClickHouseSettings {
            url: "http://localhost:8123".to_string(),
            database: "quantix".to_string(),
            user: "default".to_string(),
            password: "".to_string(),
        },
        upstream_mysql: UpstreamMySqlSettings {
            url: "mysql://127.0.0.1:3306".to_string(),
            database: "mystocks".to_string(),
            user: "root".to_string(),
            password: "".to_string(),
        },
        watchlist_path: PathBuf::from("/tmp/quantix/watchlist/watchlist.json"),
        trade_path: PathBuf::from("/tmp/quantix/trade/paper_trade.json"),
        risk_path: PathBuf::from("/tmp/quantix/risk/risk_state.json"),
        monitor_db_path: PathBuf::from("/tmp/quantix/monitor/alerts.db"),
        monitor_config_path: PathBuf::from("/tmp/quantix/monitor/config.json"),
        strategy_config_path: PathBuf::from("/tmp/quantix/strategy/config.json"),
        strategy_runtime_db_path: PathBuf::from("/tmp/quantix/strategy/runtime.db"),
        execution_config_path: PathBuf::from("/tmp/quantix/execution/config.json"),
    }
}

fn sample_service_config() -> StrategyServiceConfig {
    StrategyServiceConfig {
        quantix_bin_path: PathBuf::from("/opt/quantix/bin/quantix"),
        environment_file_path: Some(PathBuf::from("/tmp/quantix/strategy/service.env")),
    }
}

#[test]
fn strategy_service_config_store_save_and_load_preserves_optional_env_file() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("strategy").join("service.json");
    let binary_path = dir.path().join("bin").join("quantix");
    std::fs::create_dir_all(binary_path.parent().unwrap()).unwrap();
    std::fs::write(&binary_path, "#!/bin/sh\nexit 0\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&binary_path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&binary_path, perms).unwrap();
    }

    let store = JsonStrategyServiceConfigStore::new(&path);
    let config = StrategyServiceConfig {
        quantix_bin_path: binary_path.clone(),
        environment_file_path: Some(PathBuf::from("/tmp/quantix-strategy.env")),
    };

    store.save(&config).unwrap();
    let loaded = store.load().unwrap();

    assert_eq!(loaded, config);
}

#[test]
fn strategy_service_config_validate_rejects_non_absolute_binary_path() {
    let config = StrategyServiceConfig {
        quantix_bin_path: PathBuf::from("target/debug/quantix"),
        environment_file_path: None,
    };

    let error = JsonStrategyServiceConfigStore::validate(&config).unwrap_err();
    assert!(error.to_string().contains("必须是绝对路径"));
}

#[test]
fn strategy_systemd_render_wrapper_script_runs_strategy_daemon() {
    let installer = StrategyUserServiceInstaller::new(sample_runtime(), sample_service_config());

    let script = installer.render_wrapper_script();

    assert!(script.contains("/opt/quantix/bin/quantix"));
    assert!(script.contains("strategy daemon run"));
}

#[test]
fn strategy_systemd_render_unit_includes_runtime_paths_and_env_file() {
    let installer = StrategyUserServiceInstaller::new(sample_runtime(), sample_service_config());

    let unit = installer.render_unit();

    assert!(unit.contains("ExecStart=~/.local/bin/quantix-strategy-run"));
    assert!(
        unit.contains("Environment=QUANTIX_STRATEGY_CONFIG_PATH=/tmp/quantix/strategy/config.json")
    );
    assert!(
        unit.contains(
            "Environment=QUANTIX_STRATEGY_RUNTIME_DB_PATH=/tmp/quantix/strategy/runtime.db"
        )
    );
    assert!(unit.contains("EnvironmentFile=-/tmp/quantix/strategy/service.env"));
    assert!(unit.contains("Restart=on-failure"));
}

#[test]
fn strategy_systemd_paths_target_user_scope_locations() {
    let installer = StrategyUserServiceInstaller::new(sample_runtime(), sample_service_config());

    assert_eq!(
        installer.unit_path(),
        PathBuf::from("~/.config/systemd/user/quantix-strategy.service")
    );
    assert_eq!(
        installer.wrapper_path(),
        PathBuf::from("~/.local/bin/quantix-strategy-run")
    );
}

#[test]
fn strategy_systemd_status_summary_reports_structured_fields() {
    let summary = StrategyServiceStatusSummary {
        installed: true,
        enabled: false,
        active: "inactive".to_string(),
        unit_path: PathBuf::from("~/.config/systemd/user/quantix-strategy.service"),
        wrapper_path: PathBuf::from("~/.local/bin/quantix-strategy-run"),
        quantix_bin_path: PathBuf::from("/opt/quantix/bin/quantix"),
        environment_file_path: Some(PathBuf::from("/tmp/quantix/strategy/service.env")),
        raw_status: None,
    };

    assert!(summary.installed);
    assert!(!summary.enabled);
    assert_eq!(summary.active, "inactive");
    assert_eq!(
        summary.wrapper_path,
        PathBuf::from("~/.local/bin/quantix-strategy-run")
    );
    assert_eq!(
        summary.environment_file_path,
        Some(PathBuf::from("/tmp/quantix/strategy/service.env"))
    );
}
