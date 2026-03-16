use quantix_cli::monitor::config::{JsonMonitorConfigStore, MonitorConfig};
use tempfile::tempdir;

#[test]
fn load_or_create_creates_default_monitor_config_file() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("monitor").join("config.json");
    let store = JsonMonitorConfigStore::new(&config_path);

    let config = store.load_or_create().unwrap();

    assert_eq!(config, MonitorConfig::default());
    assert!(config_path.exists());
}

#[test]
fn save_and_reload_round_trips_monitor_config() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("monitor").join("config.json");
    let store = JsonMonitorConfigStore::new(&config_path);
    let expected = MonitorConfig {
        interval_seconds: 15,
        watchlist_group: Some("core".to_string()),
        persist_events: false,
        max_event_history: 250,
    };

    store.save(&expected).unwrap();
    let reloaded = store.load_or_create().unwrap();

    assert_eq!(reloaded, expected);
}

#[test]
fn load_or_create_rejects_malformed_json() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("monitor").join("config.json");
    std::fs::create_dir_all(config_path.parent().unwrap()).unwrap();
    std::fs::write(&config_path, "{not-json").unwrap();
    let store = JsonMonitorConfigStore::new(&config_path);

    let err = store.load_or_create().unwrap_err();

    assert!(
        err.to_string().contains("序列化错误")
            || err.to_string().contains("json")
            || err.to_string().contains("expected"),
        "unexpected error: {err}"
    );
}
