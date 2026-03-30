use std::path::PathBuf;

use quantix_cli::core::{
    BridgeRuntimeSettings, CliRuntime, ClickHouseSettings, UpstreamMySqlSettings,
};
use quantix_cli::monitor::{
    MonitorServiceConfig,
    systemd::{MonitorServiceStatusSummary, MonitorUserServiceInstaller},
};

fn sample_runtime() -> CliRuntime {
    CliRuntime {
        clickhouse: ClickHouseSettings {
            url: "http://localhost:8123".to_string(),
            database: "quantix".to_string(),
            user: "default".to_string(),
            password: "".to_string(),
        },
        bridge: BridgeRuntimeSettings {
            base_url: "http://localhost:8080".to_string(),
            api_key: None,
            tdx_enabled: false,
            qmt_preview_enabled: false,
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

fn sample_service_config() -> MonitorServiceConfig {
    MonitorServiceConfig {
        quantix_bin_path: PathBuf::from("/opt/quantix/bin/quantix"),
    }
}

#[test]
fn monitor_systemd_render_unit_uses_wrapper_script_instead_of_current_exe() {
    let installer = MonitorUserServiceInstaller::new(sample_runtime(), sample_service_config());

    let unit = installer.render_unit();

    assert!(unit.contains("ExecStart=~/.local/bin/quantix-monitor-run"));
    assert!(unit.contains("Restart=on-failure"));
    assert!(unit.contains("RestartSec=5"));
}

#[test]
fn monitor_systemd_unit_path_targets_user_service_directory() {
    let installer = MonitorUserServiceInstaller::new(sample_runtime(), sample_service_config());

    assert_eq!(
        installer.unit_path(),
        PathBuf::from("~/.config/systemd/user/quantix-monitor.service")
    );
}

#[test]
fn monitor_systemd_wrapper_path_targets_local_bin() {
    let installer = MonitorUserServiceInstaller::new(sample_runtime(), sample_service_config());

    assert_eq!(
        installer.wrapper_path(),
        PathBuf::from("~/.local/bin/quantix-monitor-run")
    );
}

#[test]
fn monitor_systemd_render_wrapper_script_runs_configured_quantix_binary() {
    let installer = MonitorUserServiceInstaller::new(sample_runtime(), sample_service_config());

    let script = installer.render_wrapper_script();

    assert!(script.contains("/opt/quantix/bin/quantix"));
    assert!(script.contains("monitor daemon run"));
}

#[test]
fn monitor_systemd_command_args_target_systemctl_user_scope() {
    let installer = MonitorUserServiceInstaller::new(sample_runtime(), sample_service_config());

    assert_eq!(
        installer.systemctl_args("start"),
        vec!["--user", "start", "quantix-monitor.service"]
    );
    assert_eq!(
        installer.systemctl_args("daemon-reload"),
        vec!["--user", "daemon-reload"]
    );
}

#[test]
fn monitor_systemd_render_unit_includes_environment_lines_for_runtime_paths() {
    let installer = MonitorUserServiceInstaller::new(sample_runtime(), sample_service_config());

    let unit = installer.render_unit();

    assert!(
        unit.contains("Environment=QUANTIX_WATCHLIST_PATH=/tmp/quantix/watchlist/watchlist.json")
    );
    assert!(unit.contains("Environment=QUANTIX_MONITOR_DB_PATH=/tmp/quantix/monitor/alerts.db"));
    assert!(
        unit.contains("Environment=QUANTIX_MONITOR_CONFIG_PATH=/tmp/quantix/monitor/config.json")
    );
}

#[test]
fn monitor_systemd_status_summary_reports_structured_fields() {
    let summary = MonitorServiceStatusSummary {
        installed: true,
        enabled: false,
        active: "inactive".to_string(),
        unit_path: PathBuf::from("~/.config/systemd/user/quantix-monitor.service"),
        wrapper_path: PathBuf::from("~/.local/bin/quantix-monitor-run"),
        quantix_bin_path: PathBuf::from("/opt/quantix/bin/quantix"),
        raw_status: None,
    };

    assert!(summary.installed);
    assert!(!summary.enabled);
    assert_eq!(summary.active, "inactive");
    assert_eq!(
        summary.wrapper_path,
        PathBuf::from("~/.local/bin/quantix-monitor-run")
    );
}
