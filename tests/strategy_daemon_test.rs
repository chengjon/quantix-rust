use quantix_cli::strategy::{BootstrapPolicy, JsonStrategyConfigStore};
use tempfile::tempdir;

#[test]
fn strategy_config_store_load_or_create_persists_default_latest_only_config() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("strategy").join("config.json");

    let store = JsonStrategyConfigStore::new(&path);
    let config = store.load_or_create().unwrap();

    assert_eq!(config.check_interval_secs, 60);
    assert_eq!(config.bootstrap_policy, BootstrapPolicy::LatestOnly);
    assert_eq!(config.stocks.len(), 1);
    assert_eq!(config.stocks[0].code, "000001");
    assert!(config.stocks[0].enabled);
    assert_eq!(config.stocks[0].strategies.len(), 1);
    assert_eq!(config.stocks[0].strategies[0].id, "ma_fast_5_slow_20");
    assert_eq!(config.stocks[0].strategies[0].name, "ma_cross");
    assert!(config.stocks[0].strategies[0].enabled);
    assert_eq!(config.stocks[0].strategies[0].params["fast"], 5);
    assert_eq!(config.stocks[0].strategies[0].params["slow"], 20);

    let saved = std::fs::read_to_string(&path).unwrap();
    assert!(saved.contains("\"bootstrap_policy\": \"latest_only\""));
}
