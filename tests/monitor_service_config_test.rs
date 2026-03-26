use std::os::unix::fs::PermissionsExt;

use quantix_cli::monitor::service_config::{JsonMonitorServiceConfigStore, MonitorServiceConfig};
use tempfile::tempdir;

#[test]
fn load_returns_error_when_service_config_is_missing() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("service.json");
    let store = JsonMonitorServiceConfigStore::new(&config_path);

    let err = store.load().unwrap_err();

    assert!(
        err.to_string().contains("service")
            || err.to_string().contains("配置")
            || err.to_string().contains("不存在"),
        "unexpected error: {err}"
    );
}

#[test]
fn save_and_reload_round_trip_service_config() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("service.json");
    let store = JsonMonitorServiceConfigStore::new(&config_path);
    let config = MonitorServiceConfig {
        quantix_bin_path: "/bin/echo".into(),
    };

    store.save(&config).unwrap();
    let reloaded = store.load().unwrap();

    assert_eq!(reloaded, config);
}

#[test]
fn validate_rejects_relative_binary_path() {
    let config = MonitorServiceConfig {
        quantix_bin_path: "relative/path/to/quantix".into(),
    };

    let err = JsonMonitorServiceConfigStore::validate(&config).unwrap_err();

    assert!(err.to_string().contains("绝对"));
}

#[test]
fn validate_rejects_missing_binary_path() {
    let config = MonitorServiceConfig {
        quantix_bin_path: "/tmp/does-not-exist-quantix".into(),
    };

    let err = JsonMonitorServiceConfigStore::validate(&config).unwrap_err();

    assert!(err.to_string().contains("不存在"));
}

#[test]
fn validate_rejects_non_executable_file() {
    let dir = tempdir().unwrap();
    let binary_path = dir.path().join("quantix");
    std::fs::write(&binary_path, "#!/bin/sh\nexit 0\n").unwrap();
    let mut perms = std::fs::metadata(&binary_path).unwrap().permissions();
    perms.set_mode(0o644);
    std::fs::set_permissions(&binary_path, perms).unwrap();

    let config = MonitorServiceConfig {
        quantix_bin_path: binary_path,
    };

    let err = JsonMonitorServiceConfigStore::validate(&config).unwrap_err();

    assert!(err.to_string().contains("可执行"));
}
