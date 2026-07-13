use super::monitor_helpers::{FakeStopRuleState, FakeStopRuleStore};
use super::*;
use crate::core::QuantixError;
use rust_decimal_macros::dec;

#[tokio::test]
async fn test_execute_stop_set_loss_succeeds() {
    let (_dir, storage) = stop_watchlist_storage(&["000001"]);
    let store = FakeStopRuleStore::default();
    let service = StopService::new(store.clone());

    let output = execute_stop_command_with_service(
        StopCommands::Set {
            code: "000001".to_string(),
            loss: Some(14.5),
            profit: None,
            loss_pct: None,
            profit_pct: None,
            trailing: None,
        },
        &service,
        &storage,
    )
    .await
    .unwrap();

    match output {
        StopCommandOutput::RuleSet(rule) => {
            assert_eq!(rule.code, "000001");
            assert_eq!(rule.stop_loss_price, Some(14.5));
            assert_eq!(rule.take_profit_price, None);
            assert_eq!(rule.trailing_pct, None);
        }
        other => panic!("unexpected output: {:?}", other),
    }

    assert_eq!(store.state.lock().unwrap().rules.len(), 1);
}

#[tokio::test]
async fn test_execute_stop_set_profit_succeeds() {
    let (_dir, storage) = stop_watchlist_storage(&["000001"]);
    let service = StopService::new(FakeStopRuleStore::default());

    let output = execute_stop_command_with_service(
        StopCommands::Set {
            code: "000001".to_string(),
            loss: None,
            profit: Some(18.0),
            loss_pct: None,
            profit_pct: None,
            trailing: None,
        },
        &service,
        &storage,
    )
    .await
    .unwrap();

    match output {
        StopCommandOutput::RuleSet(rule) => {
            assert_eq!(rule.take_profit_price, Some(18.0));
            assert_eq!(rule.stop_loss_price, None);
        }
        other => panic!("unexpected output: {:?}", other),
    }
}

#[tokio::test]
async fn test_execute_stop_set_trailing_succeeds() {
    let (_dir, storage) = stop_watchlist_storage(&["000001"]);
    let service = StopService::new(FakeStopRuleStore::default());

    let output = execute_stop_command_with_service(
        StopCommands::Set {
            code: "000001".to_string(),
            loss: None,
            profit: Some(18.0),
            loss_pct: None,
            profit_pct: None,
            trailing: Some(5.0),
        },
        &service,
        &storage,
    )
    .await
    .unwrap();

    match output {
        StopCommandOutput::RuleSet(rule) => {
            assert_eq!(rule.trailing_pct, Some(5.0));
            assert_eq!(rule.take_profit_price, Some(18.0));
        }
        other => panic!("unexpected output: {:?}", other),
    }
}

#[tokio::test]
async fn test_execute_stop_set_rejects_invalid_condition_combinations() {
    let (_dir, storage) = stop_watchlist_storage(&["000001"]);
    let service = StopService::new(FakeStopRuleStore::default());

    let none_err = execute_stop_command_with_service(
        StopCommands::Set {
            code: "000001".to_string(),
            loss: None,
            profit: None,
            loss_pct: None,
            profit_pct: None,
            trailing: None,
        },
        &service,
        &storage,
    )
    .await
    .unwrap_err();
    assert!(matches!(none_err, QuantixError::Other(_)));
    assert!(none_err.to_string().contains("至少需要一个条件"));

    let conflict_err = execute_stop_command_with_service(
        StopCommands::Set {
            code: "000001".to_string(),
            loss: Some(14.5),
            profit: None,
            loss_pct: None,
            profit_pct: None,
            trailing: Some(5.0),
        },
        &service,
        &storage,
    )
    .await
    .unwrap_err();
    assert!(matches!(conflict_err, QuantixError::Other(_)));
    assert!(
        conflict_err
            .to_string()
            .contains("--trailing 和 --loss/--loss-pct")
    );
}

#[tokio::test]
async fn test_execute_stop_set_rejects_codes_outside_watchlist() {
    let (_dir, storage) = stop_watchlist_storage(&["000001"]);
    let service = StopService::new(FakeStopRuleStore::default());

    let err = execute_stop_command_with_service(
        StopCommands::Set {
            code: "000002".to_string(),
            loss: Some(14.5),
            profit: None,
            loss_pct: None,
            profit_pct: None,
            trailing: None,
        },
        &service,
        &storage,
    )
    .await
    .unwrap_err();

    assert!(matches!(err, QuantixError::Other(_)));
    assert!(err.to_string().contains("不在自选池"));
}

#[tokio::test]
async fn test_execute_stop_set_overwrites_existing_rule_shape() {
    let (_dir, storage) = stop_watchlist_storage(&["000001"]);
    let store = FakeStopRuleStore {
        state: Arc::new(Mutex::new(FakeStopRuleState {
            rules: vec![StopRule {
                code: "000001".to_string(),
                stop_loss_price: Some(14.5),
                take_profit_price: Some(18.0),
                stop_loss_pct: None,
                take_profit_pct: None,
                trailing_pct: None,
                highest_price: Some(19.2),
                reference_price: None,
                last_triggered_at: Some(stop_sample_time()),
                created_at: stop_sample_time(),
                updated_at: stop_sample_time(),
            }],
            history: Vec::new(),
            removed_codes: Vec::new(),
        })),
    };
    let service = StopService::new(store.clone());

    let output = execute_stop_command_with_service(
        StopCommands::Set {
            code: "000001".to_string(),
            loss: None,
            profit: Some(21.0),
            loss_pct: None,
            profit_pct: None,
            trailing: Some(5.0),
        },
        &service,
        &storage,
    )
    .await
    .unwrap();

    match output {
        StopCommandOutput::RuleSet(rule) => {
            assert_eq!(rule.code, "000001");
            assert_eq!(rule.stop_loss_price, None);
            assert_eq!(rule.take_profit_price, Some(21.0));
            assert_eq!(rule.trailing_pct, Some(5.0));
            assert_eq!(rule.highest_price, None);
            assert_eq!(rule.last_triggered_at, None);
        }
        other => panic!("unexpected output: {:?}", other),
    }

    let state = store.state.lock().unwrap();
    assert_eq!(state.rules.len(), 1);
    assert_eq!(state.rules[0].stop_loss_price, None);
    assert_eq!(state.rules[0].take_profit_price, Some(21.0));
    assert_eq!(state.rules[0].trailing_pct, Some(5.0));
    assert_eq!(state.rules[0].highest_price, None);
    assert_eq!(state.rules[0].last_triggered_at, None);
}

#[tokio::test]
async fn test_execute_stop_set_loss_pct_resolves_reference_price_from_quote() {
    let (_dir, storage) = stop_watchlist_storage(&["000001"]);
    let store = FakeStopRuleStore::default();
    let service = StopService::new(store.clone());
    let trade_store = FakePaperTradeStore::default();
    let quote_lookup = FakeTradeQuoteLookup {
        quotes: HashMap::from([(
            "000001".to_string(),
            WatchlistQuoteSnapshot {
                latest_price: dec!(15.2),
                price_change_pct: None,
            },
        )]),
        fail: false,
    };

    let output = execute_stop_command_with_context(
        StopCommands::Set {
            code: "000001".to_string(),
            loss: None,
            profit: None,
            loss_pct: Some(5.0),
            profit_pct: None,
            trailing: None,
        },
        &service,
        &storage,
        &quote_lookup,
        &trade_store,
    )
    .await
    .unwrap();

    match output {
        StopCommandOutput::RuleSet(rule) => {
            assert_eq!(rule.stop_loss_pct, Some(5.0));
            assert_eq!(rule.reference_price, Some(15.2));
        }
        other => panic!("unexpected output: {:?}", other),
    }
}

#[tokio::test]
async fn test_execute_stop_update_applies_patch_and_clear_flags() {
    let (_dir, storage) = stop_watchlist_storage(&["000001"]);
    let store = FakeStopRuleStore {
        state: Arc::new(Mutex::new(FakeStopRuleState {
            rules: vec![StopRule {
                code: "000001".to_string(),
                stop_loss_price: Some(14.5),
                take_profit_price: Some(18.0),
                stop_loss_pct: None,
                take_profit_pct: None,
                trailing_pct: None,
                highest_price: None,
                reference_price: None,
                last_triggered_at: None,
                created_at: stop_sample_time(),
                updated_at: stop_sample_time(),
            }],
            history: Vec::new(),
            removed_codes: Vec::new(),
        })),
    };
    let service = StopService::new(store);
    let trade_store = FakePaperTradeStore::default();
    let quote_lookup = FakeTradeQuoteLookup {
        quotes: HashMap::from([(
            "000001".to_string(),
            WatchlistQuoteSnapshot {
                latest_price: dec!(15.2),
                price_change_pct: None,
            },
        )]),
        fail: false,
    };

    let output = execute_stop_command_with_context(
        StopCommands::Update {
            code: "000001".to_string(),
            loss: None,
            profit: None,
            loss_pct: None,
            profit_pct: Some(12.0),
            trailing: None,
            clear_loss: true,
            clear_profit: true,
            clear_loss_pct: false,
            clear_profit_pct: false,
            clear_trailing: false,
        },
        &service,
        &storage,
        &quote_lookup,
        &trade_store,
    )
    .await
    .unwrap();

    match output {
        StopCommandOutput::RuleUpdated(rule) => {
            assert_eq!(rule.stop_loss_price, None);
            assert_eq!(rule.take_profit_price, None);
            assert_eq!(rule.take_profit_pct, Some(12.0));
            assert_eq!(rule.reference_price, Some(15.2));
        }
        other => panic!("unexpected output: {:?}", other),
    }
}
