use super::*;
use super::support::{
    FakeStopRuleState, FakeStopRuleStore, stop_rule, stop_sample_time, stop_watchlist_storage,
};

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
            trailing: Some(5.0),
        },
        &service,
        &storage,
    )
    .await
    .unwrap_err();
    assert!(matches!(conflict_err, QuantixError::Other(_)));
    assert!(conflict_err.to_string().contains("--loss 和 --trailing"));
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
                trailing_pct: None,
                highest_price: Some(19.2),
                last_triggered_at: Some(stop_sample_time()),
                created_at: stop_sample_time(),
                updated_at: stop_sample_time(),
            }],
            removed_codes: Vec::new(),
        })),
    };
    let service = StopService::new(store.clone());

    let output = execute_stop_command_with_service(
        StopCommands::Set {
            code: "000001".to_string(),
            loss: None,
            profit: Some(21.0),
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
async fn test_execute_stop_list_returns_persisted_rules() {
    let (_dir, storage) = stop_watchlist_storage(&["000001"]);
    let store = FakeStopRuleStore {
        state: Arc::new(Mutex::new(FakeStopRuleState {
            rules: vec![stop_rule("000001")],
            removed_codes: Vec::new(),
        })),
    };
    let service = StopService::new(store);

    let output = execute_stop_command_with_service(StopCommands::List, &service, &storage)
        .await
        .unwrap();

    match output {
        StopCommandOutput::RuleList(rules) => {
            assert_eq!(rules.len(), 1);
            assert_eq!(rules[0].code, "000001");
        }
        other => panic!("unexpected output: {:?}", other),
    }
}

#[tokio::test]
async fn test_execute_stop_remove_succeeds() {
    let (_dir, storage) = stop_watchlist_storage(&["000001"]);
    let store = FakeStopRuleStore {
        state: Arc::new(Mutex::new(FakeStopRuleState {
            rules: vec![stop_rule("000001")],
            removed_codes: Vec::new(),
        })),
    };
    let service = StopService::new(store.clone());

    let output = execute_stop_command_with_service(
        StopCommands::Remove {
            code: "000001".to_string(),
        },
        &service,
        &storage,
    )
    .await
    .unwrap();

    match output {
        StopCommandOutput::RuleRemoved { code, removed } => {
            assert_eq!(code, "000001");
            assert!(removed);
        }
        other => panic!("unexpected output: {:?}", other),
    }

    let state = store.state.lock().unwrap();
    assert!(state.rules.is_empty());
    assert_eq!(state.removed_codes, vec!["000001".to_string()]);
}
