use super::monitor_helpers::{FakeStopRuleState, FakeStopRuleStore};
use super::*;

use crate::core::{CliRuntime, QuantixError, Result};
use crate::monitor::{
    JsonMonitorConfigStore, JsonMonitorServiceConfigStore, MonitorAlertStore, MonitorConfig,
    MonitorEventFilter, MonitorEventRow, MonitorEventType, MonitorIterationOutput,
    MonitorQuoteReader, MonitorQuoteRow, MonitorRunMode, MonitorRunner, MonitorService,
    MonitorServiceConfig, MonitorServiceStatusSummary, MonitorUserServiceInstaller,
    MonitorWatchlistReader, MonitorWatchlistSnapshot, PriceAlert, PriceAlertKind,
};
use crate::watchlist::{
    PostgresWatchlistNameLookup, TdxWatchlistQuoteLookup, WatchlistDisplayRow,
    WatchlistHistoryEvent, WatchlistListItem, WatchlistQuoteLookup, WatchlistService,
    WatchlistStorage, WatchlistStore,
};
use async_trait::async_trait;
use rust_decimal_macros::dec;

fn stop_sample_time() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 11, 12, 0, 0).unwrap()
}

pub(super) fn stop_rule(code: &str) -> StopRule {
    StopRule {
        code: code.to_string(),
        stop_loss_price: Some(14.5),
        take_profit_price: None,
        stop_loss_pct: None,
        take_profit_pct: None,
        trailing_pct: None,
        highest_price: None,
        reference_price: None,
        last_triggered_at: None,
        created_at: stop_sample_time(),
        updated_at: stop_sample_time(),
    }
}

fn stop_watchlist_storage(codes: &[&str]) -> (tempfile::TempDir, WatchlistStorage) {
    let dir = tempfile::tempdir().unwrap();
    let storage = WatchlistStorage::new(dir.path().join("watchlist.json"));
    let service = WatchlistService::default();
    let mut store = storage.load_or_create().unwrap();
    for code in codes {
        service.add(&mut store, code, None, Utc::now()).unwrap();
    }
    storage.save(&store).unwrap();
    (dir, storage)
}

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
