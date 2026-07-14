use super::*;
use quantix_cli::stop::{StopHistoryEventType, StopRuleUpdate};

#[tokio::test]
async fn set_rule_persists_fixed_loss_rule() {
    let service = StopService::new(FakeStopRuleStore::default());

    let rule = service
        .set_rule(
            "000001",
            Some(14.5),
            None,
            None,
            None,
            None,
            None,
            sample_time(),
        )
        .await
        .unwrap();

    assert_eq!(rule.code, "000001");
    assert_eq!(rule.stop_loss_price, Some(14.5));
    assert_eq!(rule.take_profit_price, None);
    assert_eq!(rule.trailing_pct, None);
    assert_eq!(rule.highest_price, None);
}

#[tokio::test]
async fn set_rule_persists_fixed_profit_rule() {
    let service = StopService::new(FakeStopRuleStore::default());

    let rule = service
        .set_rule(
            "000001",
            None,
            Some(18.0),
            None,
            None,
            None,
            None,
            sample_time(),
        )
        .await
        .unwrap();

    assert_eq!(rule.stop_loss_price, None);
    assert_eq!(rule.take_profit_price, Some(18.0));
    assert_eq!(rule.trailing_pct, None);
}

#[tokio::test]
async fn set_rule_persists_trailing_rule() {
    let service = StopService::new(FakeStopRuleStore::default());

    let rule = service
        .set_rule(
            "000001",
            None,
            None,
            None,
            None,
            Some(5.0),
            None,
            sample_time(),
        )
        .await
        .unwrap();

    assert_eq!(rule.stop_loss_price, None);
    assert_eq!(rule.take_profit_price, None);
    assert_eq!(rule.trailing_pct, Some(5.0));
}

#[tokio::test]
async fn set_rule_persists_percent_thresholds_and_reference_price() {
    let service = StopService::new(FakeStopRuleStore::default());

    let rule = service
        .set_rule(
            "000001",
            None,
            None,
            Some(5.0),
            Some(12.0),
            None,
            Some(15.2),
            sample_time(),
        )
        .await
        .unwrap();

    assert_eq!(rule.stop_loss_price, None);
    assert_eq!(rule.take_profit_price, None);
    assert_eq!(rule.stop_loss_pct, Some(5.0));
    assert_eq!(rule.take_profit_pct, Some(12.0));
    assert_eq!(rule.reference_price, Some(15.2));
}

#[tokio::test]
async fn set_rule_rejects_invalid_threshold_values() {
    let service = StopService::new(FakeStopRuleStore::default());

    let loss_err = service
        .set_rule(
            "000001",
            Some(0.0),
            None,
            None,
            None,
            None,
            None,
            sample_time(),
        )
        .await
        .unwrap_err();
    assert!(loss_err.to_string().contains("--loss 必须是有限正数"));

    let profit_err = service
        .set_rule(
            "000001",
            None,
            Some(-1.0),
            None,
            None,
            None,
            None,
            sample_time(),
        )
        .await
        .unwrap_err();
    assert!(profit_err.to_string().contains("--profit 必须是有限正数"));

    let trailing_err = service
        .set_rule(
            "000001",
            None,
            None,
            None,
            None,
            Some(150.0),
            None,
            sample_time(),
        )
        .await
        .unwrap_err();
    assert!(
        trailing_err
            .to_string()
            .contains("--trailing 必须在 0 到 100 之间")
    );

    let loss_pct_err = service
        .set_rule(
            "000001",
            None,
            None,
            Some(0.0),
            None,
            None,
            Some(15.2),
            sample_time(),
        )
        .await
        .unwrap_err();
    assert!(loss_pct_err.to_string().contains("--loss-pct"));

    let profit_pct_err = service
        .set_rule(
            "000001",
            None,
            None,
            None,
            Some(0.0),
            None,
            Some(15.2),
            sample_time(),
        )
        .await
        .unwrap_err();
    assert!(profit_pct_err.to_string().contains("--profit-pct"));
}

#[tokio::test]
async fn set_rule_rejects_non_finite_threshold_values() {
    let service = StopService::new(FakeStopRuleStore::default());

    let loss_err = service
        .set_rule(
            "000001",
            Some(f64::NAN),
            None,
            None,
            None,
            None,
            None,
            sample_time(),
        )
        .await
        .unwrap_err();
    assert!(loss_err.to_string().contains("--loss 必须是有限正数"));

    let profit_err = service
        .set_rule(
            "000001",
            None,
            Some(f64::INFINITY),
            None,
            None,
            None,
            None,
            sample_time(),
        )
        .await
        .unwrap_err();
    assert!(profit_err.to_string().contains("--profit 必须是有限正数"));

    let trailing_err = service
        .set_rule(
            "000001",
            None,
            None,
            None,
            None,
            Some(f64::NEG_INFINITY),
            None,
            sample_time(),
        )
        .await
        .unwrap_err();
    assert!(
        trailing_err
            .to_string()
            .contains("--trailing 必须在 0 到 100 之间")
    );
}

#[tokio::test]
async fn list_rules_returns_store_rules() {
    let store = FakeStopRuleStore::default();
    store
        .state
        .lock()
        .unwrap()
        .rules
        .extend([sample_rule("000001"), sample_rule("000002")]);
    let service = StopService::new(store);

    let rules = service.list_rules().await.unwrap();

    assert_eq!(rules.len(), 2);
    assert_eq!(rules[0].code, "000001");
    assert_eq!(rules[1].code, "000002");
}

#[tokio::test]
async fn remove_rule_deletes_rule_from_store() {
    let store = FakeStopRuleStore::default();
    store
        .state
        .lock()
        .unwrap()
        .rules
        .push(sample_rule("000001"));
    let service = StopService::new(store.clone());

    let removed = service.remove_rule("000001", sample_time()).await.unwrap();

    assert!(removed);
    assert!(store.state.lock().unwrap().rules.is_empty());
}

#[tokio::test]
async fn update_rule_patches_only_explicit_fields() {
    let store = FakeStopRuleStore::default();
    store
        .state
        .lock()
        .unwrap()
        .rules
        .push(sample_rule("000001"));
    let service = StopService::new(store);

    let updated = service
        .update_rule(
            "000001",
            StopRuleUpdate {
                stop_loss_price: None,
                take_profit_price: Some(Some(18.0)),
                stop_loss_pct: None,
                take_profit_pct: None,
                trailing_pct: None,
                reference_price: None,
            },
            sample_time(),
        )
        .await
        .unwrap();

    assert_eq!(updated.stop_loss_price, None);
    assert_eq!(updated.take_profit_price, Some(18.0));
    assert_eq!(updated.stop_loss_pct, None);
    assert_eq!(updated.reference_price, None);
}

#[tokio::test]
async fn update_rule_can_clear_thresholds_but_not_all_of_them() {
    let store = FakeStopRuleStore::default();
    store.state.lock().unwrap().rules.push(StopRule {
        code: "000001".to_string(),
        stop_loss_price: Some(14.5),
        take_profit_price: Some(18.0),
        stop_loss_pct: None,
        take_profit_pct: None,
        trailing_pct: None,
        highest_price: None,
        reference_price: None,
        last_triggered_at: None,
        created_at: sample_time(),
        updated_at: sample_time(),
    });
    let service = StopService::new(store.clone());

    let updated = service
        .update_rule(
            "000001",
            StopRuleUpdate {
                stop_loss_price: Some(None),
                take_profit_price: None,
                stop_loss_pct: None,
                take_profit_pct: None,
                trailing_pct: None,
                reference_price: None,
            },
            sample_time(),
        )
        .await
        .unwrap();

    assert_eq!(updated.stop_loss_price, None);
    assert_eq!(updated.take_profit_price, Some(18.0));

    let err = service
        .update_rule(
            "000001",
            StopRuleUpdate {
                stop_loss_price: None,
                take_profit_price: Some(None),
                stop_loss_pct: None,
                take_profit_pct: None,
                trailing_pct: None,
                reference_price: None,
            },
            sample_time(),
        )
        .await
        .unwrap_err();

    assert!(err.to_string().contains("至少需要一个条件"));
}

#[tokio::test]
async fn set_update_and_remove_append_history_events() {
    let store = FakeStopRuleStore::default();
    let service = StopService::new(store.clone());
    let set_time = sample_time();
    let update_time = set_time + chrono::Duration::minutes(1);
    let remove_time = set_time + chrono::Duration::minutes(2);

    service
        .set_rule("000001", Some(14.5), None, None, None, None, None, set_time)
        .await
        .unwrap();
    service
        .update_rule(
            "000001",
            StopRuleUpdate {
                stop_loss_price: None,
                take_profit_price: Some(Some(18.0)),
                stop_loss_pct: None,
                take_profit_pct: None,
                trailing_pct: None,
                reference_price: None,
            },
            update_time,
        )
        .await
        .unwrap();
    service.remove_rule("000001", remove_time).await.unwrap();

    let history = service.history(None, None, None, Some(10)).await.unwrap();
    let event_types: Vec<StopHistoryEventType> =
        history.into_iter().map(|event| event.event_type).collect();

    assert_eq!(
        event_types,
        vec![
            StopHistoryEventType::Remove,
            StopHistoryEventType::Update,
            StopHistoryEventType::Set,
        ]
    );
}
