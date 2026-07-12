use super::helpers::*;

#[tokio::test]
async fn kernel_success_path_persists_run_signal_order_and_events() {
    let dir = tempdir().unwrap();
    let store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let adapter = CountingAdapter::new();
    let kernel = ExecutionKernel::new(
        store.clone(),
        adapter.clone(),
        FixedRiskEvaluator {
            decision: RiskDecision::Allow,
            sync_calls: Arc::new(Mutex::new(0)),
        },
    );

    let result = kernel
        .execute_once(
            sample_run_request("run-000001-1"),
            SignalEnvelope::new(Signal::Buy),
        )
        .await
        .unwrap();

    assert_eq!(
        result,
        KernelExecutionResult {
            run_id: "run-1".to_string(),
            signal: Signal::Buy,
            order_status: Some(OrderStatus::Filled),
            client_order_id: Some("run-000001-1".to_string()),
        }
    );
    assert_eq!(adapter.submission_count(), 1);
    assert_eq!(store.count_runs().await.unwrap(), 1);
    assert_eq!(store.count_orders().await.unwrap(), 1);
    assert_eq!(
        store.list_order_events("run-000001-1").await.unwrap().len(),
        3
    );
}

#[tokio::test]
async fn kernel_partial_fill_uses_fill_delta_applier_to_create_trade_record() {
    let dir = tempdir().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
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
    let sync_calls = Arc::new(Mutex::new(0));
    let kernel = ExecutionKernel::with_fill_delta(
        runtime_store.clone(),
        CountingAdapter::partial_fill(),
        fill_applier.clone(),
        FixedRiskEvaluator {
            decision: RiskDecision::Allow,
            sync_calls: sync_calls.clone(),
        },
    );

    let result = kernel
        .execute_once(
            sample_run_request("fill-delta-direct-1"),
            SignalEnvelope::new(Signal::Buy),
        )
        .await
        .unwrap();

    assert_eq!(result.order_status, Some(OrderStatus::PartiallyFilled));
    let snapshot = trade_store.snapshot().unwrap();
    let account = snapshot.account.unwrap();
    assert_eq!(account.positions.get("000001").unwrap().volume, 50);
    assert_eq!(snapshot.trade_records.len(), 1);
    let order = runtime_store
        .find_order_by_client_order_id("fill-delta-direct-1")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(order.status, OrderStatus::PartiallyFilled);
    assert_eq!(order.filled_quantity, 50);
    let apply_results = fill_applier.results();
    assert_eq!(apply_results.len(), 1);
    assert!(apply_results[0].applied);
    assert_eq!(apply_results[0].delta_quantity, 50);
    assert!(apply_results[0].trade_record_id.is_some());
    let events = runtime_store
        .list_order_events("fill-delta-direct-1")
        .await
        .unwrap();
    assert!(
        events
            .iter()
            .any(|event| event.event_type == "partially_filled")
    );
    assert!(
        events
            .iter()
            .any(|event| event.event_type == "fill_applied")
    );
    assert_eq!(*sync_calls.lock().unwrap(), 1);
}

#[tokio::test]
async fn kernel_accepted_without_fill_skips_fill_delta_applier_and_keeps_account_unchanged() {
    let dir = tempdir().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
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
    let sync_calls = Arc::new(Mutex::new(0));
    let kernel = ExecutionKernel::with_fill_delta(
        runtime_store.clone(),
        CountingAdapter::accepted(),
        fill_applier.clone(),
        FixedRiskEvaluator {
            decision: RiskDecision::Allow,
            sync_calls: sync_calls.clone(),
        },
    );

    let result = kernel
        .execute_once(
            sample_run_request("fill-delta-direct-accepted"),
            SignalEnvelope::new(Signal::Buy),
        )
        .await
        .unwrap();

    assert_eq!(result.order_status, Some(OrderStatus::Accepted));
    let snapshot = trade_store.snapshot().unwrap();
    let account = snapshot.account.unwrap();
    assert!(account.positions.is_empty());
    assert_eq!(snapshot.trade_records.len(), 0);
    let order = runtime_store
        .find_order_by_client_order_id("fill-delta-direct-accepted")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(order.status, OrderStatus::Accepted);
    assert_eq!(order.filled_quantity, 0);
    assert!(fill_applier.results().is_empty());
    let events = runtime_store
        .list_order_events("fill-delta-direct-accepted")
        .await
        .unwrap();
    assert!(events.iter().any(|event| event.event_type == "accepted"));
    assert!(
        !events
            .iter()
            .any(|event| event.event_type == "fill_applied")
    );
    assert_eq!(*sync_calls.lock().unwrap(), 0);
}

#[tokio::test]
async fn kernel_non_final_submit_persists_adapter_identity_and_remaining_quantity() {
    let dir = tempdir().unwrap();
    let store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let kernel = ExecutionKernel::new(
        store.clone(),
        CountingAdapter::accepted(),
        FixedRiskEvaluator {
            decision: RiskDecision::Allow,
            sync_calls: Arc::new(Mutex::new(0)),
        },
    );

    let result = kernel
        .execute_once(
            sample_run_request("run-accepted-1"),
            SignalEnvelope::new(Signal::Buy),
        )
        .await
        .unwrap();

    assert_eq!(result.order_status, Some(OrderStatus::Accepted));

    let order = store
        .find_order_by_client_order_id("run-accepted-1")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(order.adapter, "counting");
    assert_eq!(order.remaining_quantity, order.requested_quantity);
}

#[tokio::test]
async fn kernel_sync_after_fill_runs_for_partial_fill_delta() {
    let dir = tempdir().unwrap();
    let store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let sync_calls = Arc::new(Mutex::new(0));
    let kernel = ExecutionKernel::new(
        store,
        CountingAdapter::partial_fill(),
        FixedRiskEvaluator {
            decision: RiskDecision::Allow,
            sync_calls: sync_calls.clone(),
        },
    );

    let result = kernel
        .execute_once(
            sample_run_request("run-partial-1"),
            SignalEnvelope::new(Signal::Buy),
        )
        .await
        .unwrap();

    assert_eq!(result.order_status, Some(OrderStatus::PartiallyFilled));
    assert_eq!(*sync_calls.lock().unwrap(), 1);
}

#[tokio::test]
async fn kernel_risk_rejection_creates_rejected_order_and_skips_adapter() {
    let dir = tempdir().unwrap();
    let store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let adapter = CountingAdapter::new();
    let kernel = ExecutionKernel::new(
        store.clone(),
        adapter.clone(),
        FixedRiskEvaluator {
            decision: RiskDecision::Reject {
                reason: "position-limit".to_string(),
            },
            sync_calls: Arc::new(Mutex::new(0)),
        },
    );

    let result = kernel
        .execute_once(
            sample_run_request("run-000001-2"),
            SignalEnvelope::new(Signal::Buy),
        )
        .await
        .unwrap();

    assert_eq!(result.order_status, Some(OrderStatus::Rejected));
    assert_eq!(adapter.submission_count(), 0);
    let order = store
        .find_order_by_client_order_id("run-000001-2")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(order.status, OrderStatus::Rejected);
}

#[tokio::test]
async fn kernel_duplicate_client_order_id_returns_stored_result_without_resubmitting() {
    let dir = tempdir().unwrap();
    let store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let adapter = CountingAdapter::new();
    let kernel = ExecutionKernel::new(
        store.clone(),
        adapter.clone(),
        FixedRiskEvaluator {
            decision: RiskDecision::Allow,
            sync_calls: Arc::new(Mutex::new(0)),
        },
    );

    let first = kernel
        .execute_once(
            sample_run_request("dup-order"),
            SignalEnvelope::new(Signal::Buy),
        )
        .await
        .unwrap();
    let second = kernel
        .execute_once(
            ExecutionRunRequest {
                run_id: "run-2".to_string(),
                bar_end: fixed_ts() + Duration::days(1),
                ..sample_run_request("dup-order")
            },
            SignalEnvelope::new(Signal::Buy),
        )
        .await
        .unwrap();

    assert_eq!(first.order_status, Some(OrderStatus::Filled));
    assert_eq!(second.order_status, Some(OrderStatus::Filled));
    assert_eq!(adapter.submission_count(), 1);
}

#[tokio::test]
async fn kernel_recover_pending_orders_returns_empty_summary() {
    let dir = tempdir().unwrap();
    let store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let adapter = CountingAdapter::new();
    let kernel = ExecutionKernel::new(
        store,
        adapter,
        FixedRiskEvaluator {
            decision: RiskDecision::Allow,
            sync_calls: Arc::new(Mutex::new(0)),
        },
    );

    let summary = kernel.recover_pending_orders().await.unwrap();

    assert_eq!(
        summary,
        RecoverySummary {
            scanned: 0,
            recovered: 0,
            unchanged: 0,
            failed: 0,
            skipped: 0,
        }
    );
}

#[tokio::test]
async fn kernel_recover_pending_orders_advances_mock_live_order() {
    let dir = tempdir().unwrap();
    let store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
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
    let sync_calls = Arc::new(Mutex::new(0));
    let adapter = MockLiveExecutionAdapter::with_state_template(
        store.clone(),
        FixedMockLiveClock,
        MockLiveOrderState {
            fill_plan: vec![
                MockLiveFillStep {
                    quantity: 50,
                    delay_secs: 0,
                },
                MockLiveFillStep {
                    quantity: 50,
                    delay_secs: 0,
                },
            ],
            ..Default::default()
        },
    );
    let kernel = ExecutionKernel::with_fill_delta(
        store.clone(),
        adapter,
        fill_applier,
        FixedRiskEvaluator {
            decision: RiskDecision::Allow,
            sync_calls: sync_calls.clone(),
        },
    );

    let run_request = ExecutionRunRequest {
        mode: "mock_live".to_string(),
        ..sample_run_request("recover-order-1")
    };
    let result = kernel
        .execute_once(run_request, SignalEnvelope::new(Signal::Buy))
        .await
        .unwrap();
    assert_eq!(result.order_status, Some(OrderStatus::Accepted));

    let summary = kernel.recover_pending_orders().await.unwrap();

    assert_eq!(summary.scanned, 1);
    assert_eq!(summary.recovered, 1);
    assert_eq!(*sync_calls.lock().unwrap(), 1);

    let order = store
        .find_order_by_client_order_id("recover-order-1")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(order.status, OrderStatus::PartiallyFilled);
    assert_eq!(order.filled_quantity, 50);
    assert_eq!(
        order.remaining_quantity,
        order.requested_quantity - order.filled_quantity
    );
    let state = store
        .get_mock_live_order_state(&order.order_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(state.last_applied_fill_id, 1);
    let snapshot = trade_store.snapshot().unwrap();
    let account = snapshot.account.unwrap();
    assert_eq!(account.positions.get("000001").unwrap().volume, 50);
    assert_eq!(snapshot.trade_records.len(), 1);

    let second_summary = kernel.recover_pending_orders().await.unwrap();
    assert_eq!(second_summary.scanned, 1);
    assert_eq!(second_summary.recovered, 1);
    assert_eq!(*sync_calls.lock().unwrap(), 2);

    let final_order = store
        .find_order_by_client_order_id("recover-order-1")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(final_order.status, OrderStatus::Filled);
    assert_eq!(final_order.filled_quantity, 100);
    let final_state = store
        .get_mock_live_order_state(&final_order.order_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(final_state.last_applied_fill_id, 2);
    let final_snapshot = trade_store.snapshot().unwrap();
    let final_account = final_snapshot.account.unwrap();
    assert_eq!(final_account.positions.get("000001").unwrap().volume, 100);
    assert_eq!(final_snapshot.trade_records.len(), 2);
}

#[tokio::test]
async fn kernel_recover_pending_orders_resolves_pending_cancel_order() {
    let dir = tempdir().unwrap();
    let store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let run = sample_run("000001", fixed_ts());
    store.insert_run(&run).await.unwrap();
    let order = OrderRecord {
        status: OrderStatus::PendingCancel,
        adapter: "mock_live".to_string(),
        ..sample_order(&run.run_id, "recover-cancel-1")
    };
    store.insert_order(&order).await.unwrap();
    store
        .insert_mock_live_order_state(
            &order.order_id,
            Some("recover-cancel-1"),
            &MockLiveOrderState {
                cancel_requested: true,
                ..Default::default()
            },
        )
        .await
        .unwrap();

    let kernel = ExecutionKernel::new(
        store.clone(),
        MockLiveExecutionAdapter::new(store.clone(), FixedMockLiveClock),
        FixedRiskEvaluator {
            decision: RiskDecision::Allow,
            sync_calls: Arc::new(Mutex::new(0)),
        },
    );

    let summary = kernel.recover_pending_orders().await.unwrap();

    assert_eq!(summary.scanned, 1);
    assert_eq!(summary.recovered, 1);

    let saved = store
        .find_order_by_client_order_id("recover-cancel-1")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved.status, OrderStatus::Canceled);
}

#[tokio::test]
async fn kernel_recover_pending_orders_marks_unknown_exhaustion_without_changing_public_status() {
    let dir = tempdir().unwrap();
    let store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let trade_store = FakePaperTradeStore::default();
    let trade_service = TradeService::new(trade_store.clone());
    trade_service
        .init_account(
            InitAccountRequest::new(Some(100000.0), None, None, None, None).unwrap(),
            fixed_ts(),
        )
        .await
        .unwrap();
    let run = sample_run("000001", fixed_ts());
    store.insert_run(&run).await.unwrap();
    let order = OrderRecord {
        status: OrderStatus::Unknown,
        adapter: "mock_live".to_string(),
        ..sample_order(&run.run_id, "recover-unknown-1")
    };
    store.insert_order(&order).await.unwrap();
    store
        .insert_mock_live_order_state(
            &order.order_id,
            Some("recover-unknown-1"),
            &MockLiveOrderState {
                unknown_retries: 3,
                fault_injection: Some(MockLiveFaultInjection {
                    mode: Some("unknown_always".to_string()),
                    delay_seconds: None,
                    rejection_reason: None,
                    timeout_seconds: None,
                }),
                ..Default::default()
            },
        )
        .await
        .unwrap();

    let kernel = ExecutionKernel::with_fill_delta(
        store.clone(),
        MockLiveExecutionAdapter::new(store.clone(), FixedMockLiveClock),
        RecordingTradeFillDeltaApplier::new(trade_store.clone()),
        FixedRiskEvaluator {
            decision: RiskDecision::Allow,
            sync_calls: Arc::new(Mutex::new(0)),
        },
    );

    let summary = kernel.recover_pending_orders().await.unwrap();

    assert_eq!(summary.scanned, 1);
    assert_eq!(summary.recovered, 1);

    let saved = store
        .find_order_by_client_order_id("recover-unknown-1")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved.status, OrderStatus::Unknown);

    let state = store
        .get_mock_live_order_state(&order.order_id)
        .await
        .unwrap()
        .unwrap();
    assert!(state.recovery_exhausted);
    assert_eq!(
        state.exhausted_reason.as_deref(),
        Some("unknown_retry_budget_exceeded")
    );

    let events = store.list_order_events(&order.order_id).await.unwrap();
    assert!(
        events
            .iter()
            .any(|event| event.event_type == "recovery_exhausted")
    );
    let snapshot = trade_store.snapshot().unwrap();
    let account = snapshot.account.unwrap();
    assert!(account.positions.is_empty());
    assert_eq!(snapshot.trade_records.len(), 0);
}
