use super::helpers::*;

#[tokio::test]
async fn fill_delta_applier_ignores_repeated_fill_id() {
    let trade_store = FakePaperTradeStore::default();
    let trade_service = TradeService::new(trade_store.clone());
    trade_service
        .init_account(
            InitAccountRequest::new(Some(100000.0), None, None, None, None).unwrap(),
            fixed_ts(),
        )
        .await
        .unwrap();
    let fill_applier = RecordingTradeFillDeltaApplier::new(trade_store.clone());

    let first = fill_applier
        .apply_fill_delta(sample_fill_delta_context("fill-delta-repeat-1", 1))
        .await
        .unwrap();
    let second = fill_applier
        .apply_fill_delta(sample_fill_delta_context("fill-delta-repeat-1", 1))
        .await
        .unwrap();

    assert!(first.applied);
    assert_eq!(first.delta_quantity, 50);
    assert!(first.trade_record_id.is_some());
    assert!(!second.applied);
    assert_eq!(second.delta_quantity, 0);
    assert!(second.trade_record_id.is_none());

    let snapshot = trade_store.snapshot().unwrap();
    let account = snapshot.account.unwrap();
    assert_eq!(account.positions.get("000001").unwrap().volume, 50);
    assert_eq!(snapshot.trade_records.len(), 1);
}

#[tokio::test]
async fn fill_apply_failure_keeps_public_order_unfilled_and_records_failure_event() {
    let dir = tempdir().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let sync_calls = Arc::new(Mutex::new(0));
    let kernel = ExecutionKernel::with_fill_delta(
        runtime_store.clone(),
        CountingAdapter::partial_fill(),
        FailingFillDeltaApplier,
        FixedRiskEvaluator {
            decision: RiskDecision::Allow,
            sync_calls: sync_calls.clone(),
        },
    );

    let err = kernel
        .execute_once(
            sample_run_request("fill-delta-direct-failed"),
            SignalEnvelope::new(Signal::Buy),
        )
        .await
        .unwrap_err();

    assert!(err.to_string().contains("fill delta apply failed"));
    let order = runtime_store
        .find_order_by_client_order_id("fill-delta-direct-failed")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(order.status, OrderStatus::PendingSubmit);
    assert_eq!(order.filled_quantity, 0);
    assert_eq!(order.remaining_quantity, order.requested_quantity);
    let events = runtime_store
        .list_order_events("fill-delta-direct-failed")
        .await
        .unwrap();
    assert!(
        events
            .iter()
            .any(|event| event.event_type == "fill_apply_failed")
    );
    assert!(
        !events
            .iter()
            .any(|event| event.event_type == "fill_applied")
    );
    assert_eq!(*sync_calls.lock().unwrap(), 0);
}
