use super::*;
use crate::strategy::{BootstrapPolicy, ConfiguredStock, ConfiguredStrategyInstance};

fn strategy_config_store() -> (tempfile::TempDir, JsonStrategyConfigStore) {
    let dir = tempdir().unwrap();
    let store = JsonStrategyConfigStore::new(dir.path().join("strategy-config.json"));
    (dir, store)
}

fn instance(
    id: &str,
    name: &str,
    enabled: bool,
    params: serde_json::Value,
) -> ConfiguredStrategyInstance {
    ConfiguredStrategyInstance {
        id: id.to_string(),
        name: name.to_string(),
        enabled,
        params,
    }
}

fn config_with_stock(
    code: &str,
    strategies: Vec<ConfiguredStrategyInstance>,
) -> StrategyDaemonConfig {
    StrategyDaemonConfig {
        check_interval_secs: 60,
        bootstrap_policy: BootstrapPolicy::LatestOnly,
        stocks: vec![ConfiguredStock {
            code: code.to_string(),
            enabled: true,
            strategies,
        }],
    }
}

#[test]
fn strategy_create_with_store_persists_instance_and_parses_param_types() {
    let (_dir, store) = strategy_config_store();
    let params = vec![
        "fast=5".to_string(),
        "slow=20".to_string(),
        "enabled=true".to_string(),
        "label=demo".to_string(),
    ];

    let config =
        execute_strategy_create_with_store(&store, "ma-demo", "ma_cross", "600519", &params, true)
            .unwrap();

    let stock = config
        .stocks
        .iter()
        .find(|stock| stock.code == "600519")
        .expect("expected new stock entry");
    let instance = stock
        .strategies
        .iter()
        .find(|item| item.id == "ma-demo")
        .expect("expected created instance");
    assert_eq!(instance.name, "ma_cross");
    assert!(instance.enabled);
    assert_eq!(instance.params["fast"], 5);
    assert_eq!(instance.params["slow"], 20);
    assert_eq!(instance.params["enabled"], true);
    assert_eq!(instance.params["label"], "demo");

    let saved = store.load().unwrap();
    assert_eq!(saved, config);
}

#[test]
fn strategy_update_with_store_moves_instance_and_drops_empty_stock() {
    let (_dir, store) = strategy_config_store();
    store
        .save(&config_with_stock(
            "600519",
            vec![instance("ma-demo", "ma_cross", true, json!({ "fast": 5 }))],
        ))
        .unwrap();

    let updated = execute_strategy_update_with_store(
        &store,
        "ma-demo",
        None,
        Some("000001"),
        &["fast=8".to_string(), "slow=21".to_string()],
        Some(false),
    )
    .unwrap();

    assert_eq!(updated.stocks.len(), 1);
    let stock = &updated.stocks[0];
    assert_eq!(stock.code, "000001");
    assert_eq!(stock.strategies.len(), 1);
    let moved = &stock.strategies[0];
    assert_eq!(moved.id, "ma-demo");
    assert!(!moved.enabled);
    assert_eq!(moved.params, json!({ "fast": 8, "slow": 21 }));
}

#[test]
fn strategy_delete_with_store_removes_instance_and_drops_empty_stock() {
    let (_dir, store) = strategy_config_store();
    store
        .save(&config_with_stock(
            "600519",
            vec![instance("ma-demo", "ma_cross", true, json!({ "fast": 5 }))],
        ))
        .unwrap();

    let updated = execute_strategy_delete_with_store(&store, "ma-demo").unwrap();

    assert!(updated.stocks.is_empty());
    let saved = store.load().unwrap();
    assert!(saved.stocks.is_empty());
}
