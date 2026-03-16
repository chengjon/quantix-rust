use std::path::PathBuf;

use quantix_cli::core::{CliRuntime, ClickHouseSettings};
use quantix_cli::monitor::systemd::MonitorUserServiceInstaller;

fn sample_runtime() -> CliRuntime {
    CliRuntime {
        clickhouse: ClickHouseSettings {
            url: "http://localhost:8123".to_string(),
            database: "quantix".to_string(),
            user: "default".to_string(),
            password: "".to_string(),
        },
        watchlist_path: PathBuf::from("/tmp/quantix/watchlist/watchlist.json"),
        trade_path: PathBuf::from("/tmp/quantix/trade/paper_trade.json"),
        risk_path: PathBuf::from("/tmp/quantix/risk/risk_state.json"),
        monitor_db_path: PathBuf::from("/tmp/quantix/monitor/alerts.db"),
        monitor_config_path: PathBuf::from("/tmp/quantix/monitor/config.json"),
    }
}

#[test]
fn monitor_systemd_render_unit_uses_current_exe_and_daemon_run() {
    let installer = MonitorUserServiceInstaller::new(
        sample_runtime(),
        PathBuf::from("/opt/quantix/bin/quantix"),
    );

    let unit = installer.render_unit();

    assert!(unit.contains("ExecStart=/opt/quantix/bin/quantix monitor daemon run"));
    assert!(unit.contains("Restart=on-failure"));
    assert!(unit.contains("RestartSec=5"));
}

#[test]
fn monitor_systemd_unit_path_targets_user_service_directory() {
    let installer = MonitorUserServiceInstaller::new(
        sample_runtime(),
        PathBuf::from("/opt/quantix/bin/quantix"),
    );

    assert_eq!(
        installer.unit_path(),
        PathBuf::from("~/.config/systemd/user/quantix-monitor.service")
    );
}

#[test]
fn monitor_systemd_command_args_target_systemctl_user_scope() {
    let installer = MonitorUserServiceInstaller::new(
        sample_runtime(),
        PathBuf::from("/opt/quantix/bin/quantix"),
    );

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
    let installer = MonitorUserServiceInstaller::new(
        sample_runtime(),
        PathBuf::from("/opt/quantix/bin/quantix"),
    );

    let unit = installer.render_unit();

    assert!(unit.contains("Environment=QUANTIX_WATCHLIST_PATH=/tmp/quantix/watchlist/watchlist.json"));
    assert!(unit.contains("Environment=QUANTIX_MONITOR_DB_PATH=/tmp/quantix/monitor/alerts.db"));
    assert!(unit.contains("Environment=QUANTIX_MONITOR_CONFIG_PATH=/tmp/quantix/monitor/config.json"));
}
