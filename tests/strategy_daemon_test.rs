use chrono::NaiveDate;
use quantix_cli::data::models::{AdjustType, Kline};
use quantix_cli::strategy::{
    BootstrapPolicy, JsonStrategyConfigStore, StrategyRegistry,
};
use quantix_cli::strategy::trait_def::Signal;
use rust_decimal_macros::dec;
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

fn kline(day: u32, close: i64) -> Kline {
    Kline {
        code: "000001".to_string(),
        date: NaiveDate::from_ymd_opt(2026, 3, day).unwrap(),
        open: dec!(1),
        high: dec!(1),
        low: dec!(1),
        close: rust_decimal::Decimal::from(close),
        volume: 100,
        amount: None,
        adjust_type: AdjustType::None,
    }
}

#[test]
fn strategy_registry_resolves_multiple_ma_cross_instances_with_different_params() {
    let config = JsonStrategyConfigStore::new("/tmp/unused")
        .load_or_create()
        .unwrap();
    let registry = StrategyRegistry::new();

    let fast = registry
        .build(&config.stocks[0].strategies[0])
        .unwrap();
    let slow = registry
        .build(&quantix_cli::strategy::ConfiguredStrategyInstance {
            id: "ma_fast_2_slow_3".to_string(),
            name: "ma_cross".to_string(),
            enabled: true,
            params: serde_json::json!({"fast": 2, "slow": 3}),
        })
        .unwrap();

    assert_eq!(fast.lookback_required(), 20);
    assert_eq!(slow.lookback_required(), 3);
}

#[test]
fn strategy_registry_evaluator_returns_latest_signal_envelope() {
    let registry = StrategyRegistry::new();
    let evaluator = registry
        .build(&quantix_cli::strategy::ConfiguredStrategyInstance {
            id: "ma_fast_2_slow_3".to_string(),
            name: "ma_cross".to_string(),
            enabled: true,
            params: serde_json::json!({"fast": 2, "slow": 3}),
        })
        .unwrap();

    let bars = vec![
        kline(1, 10),
        kline(2, 10),
        kline(3, 10),
        kline(4, 9),
        kline(5, 9),
        kline(6, 20),
    ];

    let envelope = evaluator.evaluate(&bars).unwrap();
    assert_eq!(envelope.signal, Signal::Buy);
}

#[test]
fn strategy_registry_rejects_unknown_strategy_names() {
    let registry = StrategyRegistry::new();
    let result = registry.build(&quantix_cli::strategy::ConfiguredStrategyInstance {
            id: "unknown-1".to_string(),
            name: "unknown_strategy".to_string(),
            enabled: true,
            params: serde_json::json!({}),
        });

    let error = match result {
        Ok(_) => panic!("expected unknown strategy to fail"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("unknown_strategy"));
}
