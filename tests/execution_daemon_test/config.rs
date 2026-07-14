use super::*;

#[test]
fn config_load_or_create_persists_default_execution_daemon_settings() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("execution").join("config.json");

    let store = JsonExecutionConfigStore::new(&path);
    let config = store.load_or_create().unwrap();

    assert_eq!(config.poll_interval_secs, 10);
    assert_eq!(config.max_requests_per_iteration, 1);
    assert_eq!(config.auto_approval.mode, AutoApprovalMode::Manual);

    let saved = std::fs::read_to_string(&path).unwrap();
    assert!(saved.contains("\"poll_interval_secs\": 10"));
    assert!(saved.contains("\"max_requests_per_iteration\": 1"));
    assert!(saved.contains("\"mode\": \"manual\""));
}
