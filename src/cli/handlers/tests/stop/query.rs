use super::monitor_helpers::{FakeStopRuleState, FakeStopRuleStore};
use super::*;
use rust_decimal_macros::dec;

#[tokio::test]
async fn test_execute_stop_list_returns_persisted_rules() {
    let (_dir, storage) = stop_watchlist_storage(&["000001"]);
    let store = FakeStopRuleStore {
        state: Arc::new(Mutex::new(FakeStopRuleState {
            rules: vec![stop_rule("000001")],
            history: Vec::new(),
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
            history: Vec::new(),
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

#[tokio::test]
async fn test_execute_stop_status_and_history_return_evaluated_rows() {
    let (_dir, storage) = stop_watchlist_storage(&["000001"]);
    let stop_store = FakeStopRuleStore {
        state: Arc::new(Mutex::new(FakeStopRuleState {
            rules: vec![StopRule {
                code: "000001".to_string(),
                stop_loss_price: None,
                take_profit_price: None,
                stop_loss_pct: Some(5.0),
                take_profit_pct: None,
                trailing_pct: None,
                highest_price: None,
                reference_price: Some(15.2),
                last_triggered_at: None,
                created_at: stop_sample_time(),
                updated_at: stop_sample_time(),
            }],
            history: vec![crate::stop::StopHistoryEvent {
                id: "hist-1".to_string(),
                code: "000001".to_string(),
                event_type: StopHistoryEventType::Set,
                trigger_kind: None,
                trigger_price: None,
                anchor_price: Some(15.2),
                anchor_source: Some("reference_price".to_string()),
                snapshot_json: serde_json::json!({
                    "code": "000001",
                    "stop_loss_pct": 5.0
                }),
                created_at: stop_sample_time(),
            }],
            removed_codes: Vec::new(),
        })),
    };
    let service = StopService::new(stop_store.clone());
    let trade_store = FakePaperTradeStore {
        state: Arc::new(Mutex::new(Some(PaperTradeState {
            version: 1,
            account: Some(PaperTradeAccount {
                account_id: "default".to_string(),
                initial_capital: dec!(100000),
                available_cash: dec!(80000),
                fee_config: crate::trade::FeeConfig::default(),
                positions: std::collections::BTreeMap::from([(
                    "000001".to_string(),
                    crate::trade::TradePosition {
                        code: "000001".to_string(),
                        volume: 1000,
                        avg_cost: dec!(20),
                        last_trade_price: dec!(20),
                        opened_at: stop_sample_time(),
                        updated_at: stop_sample_time(),
                    },
                )]),
                created_at: stop_sample_time(),
                updated_at: stop_sample_time(),
            }),
            trade_records: Vec::new(),
        }))),
    };
    let quote_lookup = FakeTradeQuoteLookup {
        quotes: HashMap::from([(
            "000001".to_string(),
            WatchlistQuoteSnapshot {
                latest_price: dec!(19),
                price_change_pct: None,
            },
        )]),
        fail: false,
    };

    let status_output = execute_stop_command_with_context(
        StopCommands::Status {
            code: Some("000001".to_string()),
        },
        &service,
        &storage,
        &quote_lookup,
        &trade_store,
    )
    .await
    .unwrap();

    match status_output {
        StopCommandOutput::StatusRows(rows) => {
            assert_eq!(rows.len(), 1);
            assert_eq!(
                rows[0].anchor_source,
                Some(crate::stop::StopAnchorSource::PositionCost)
            );
            assert_eq!(rows[0].loss_threshold, Some(19.0));
        }
        other => panic!("unexpected output: {:?}", other),
    }

    let history_output = execute_stop_command_with_context(
        StopCommands::History {
            code: Some("000001".to_string()),
            limit: 10,
            date: None,
            event_type: None,
        },
        &service,
        &storage,
        &quote_lookup,
        &trade_store,
    )
    .await
    .unwrap();

    match history_output {
        StopCommandOutput::HistoryRows(rows) => {
            assert!(!rows.is_empty());
        }
        other => panic!("unexpected output: {:?}", other),
    }
}
