use super::*;

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

#[tokio::test]
async fn open_order_scanner_summary_counts_stale_and_unknown_orders() {
    let dir = tempdir().unwrap();
    let store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let run = sample_run("000001", fixed_ts());
    store.insert_run(&run).await.unwrap();

    let stale_unknown = OrderRecord {
        status: OrderStatus::Unknown,
        created_at: fixed_ts(),
        updated_at: fixed_ts(),
        last_transition_at: fixed_ts(),
        ..sample_order(&run.run_id, "reconcile-stale-unknown")
    };
    store.insert_order(&stale_unknown).await.unwrap();

    let fresh_accepted = OrderRecord {
        status: OrderStatus::Accepted,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        last_transition_at: Utc::now(),
        ..sample_order(&run.run_id, "reconcile-fresh-accepted")
    };
    store.insert_order(&fresh_accepted).await.unwrap();

    let scanner = OpenOrderScanner::with_thresholds(store, 3600, 300);
    let summary = scanner.get_open_order_summary().await.unwrap();

    assert_eq!(summary.total_open, 2);
    assert_eq!(summary.stale_count, 1);
    assert_eq!(summary.unknown_count, 1);
    assert_eq!(summary.by_status.get("unknown"), Some(&1));
    assert_eq!(summary.by_status.get("accepted"), Some(&1));
}

#[tokio::test]
async fn reconciliation_service_report_distinguishes_matched_recovered_and_failed_orders() {
    let dir = tempdir().unwrap();
    let store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let run = sample_run("000001", fixed_ts());
    store.insert_run(&run).await.unwrap();

    let matched = OrderRecord {
        status: OrderStatus::Accepted,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        last_transition_at: Utc::now(),
        ..sample_order(&run.run_id, "reconcile-matched")
    };
    store.insert_order(&matched).await.unwrap();

    let recoverable = OrderRecord {
        status: OrderStatus::Unknown,
        created_at: fixed_ts(),
        updated_at: fixed_ts(),
        last_transition_at: fixed_ts(),
        ..sample_order(&run.run_id, "reconcile-recoverable")
    };
    store.insert_order(&recoverable).await.unwrap();
    store
        .insert_mock_live_order_state(
            &recoverable.order_id,
            Some("reconcile-recoverable"),
            &MockLiveOrderState {
                fill_plan: vec![
                    MockLiveFillStep {
                        quantity: 40,
                        delay_secs: 0,
                    },
                    MockLiveFillStep {
                        quantity: 60,
                        delay_secs: 0,
                    },
                ],
                next_step_index: 1,
                simulated_fill_price: Some(dec!(12.50)),
                ..Default::default()
            },
        )
        .await
        .unwrap();

    let failed = OrderRecord {
        status: OrderStatus::Unknown,
        created_at: fixed_ts(),
        updated_at: fixed_ts(),
        last_transition_at: fixed_ts(),
        ..sample_order(&run.run_id, "reconcile-failed")
    };
    store.insert_order(&failed).await.unwrap();
    store
        .insert_mock_live_order_state(
            &failed.order_id,
            Some("reconcile-failed"),
            &MockLiveOrderState {
                recovery_exhausted: true,
                exhausted_reason: Some("unknown_retry_budget_exceeded".to_string()),
                ..Default::default()
            },
        )
        .await
        .unwrap();

    let service = ReconciliationService::new(store.clone());
    let report = service.reconcile_all().await.unwrap();

    assert_eq!(report.summary.total_open_orders, 3);
    assert_eq!(report.summary.matched_orders, 1);
    assert_eq!(report.summary.recovered_orders, 1);
    assert_eq!(report.summary.failed_orders, 1);

    let recovered_result = report
        .results
        .iter()
        .find(|result| result.client_order_id == "reconcile-recoverable")
        .unwrap();
    assert_eq!(recovered_result.action, ReconciliationAction::Recovered);

    let failed_result = report
        .results
        .iter()
        .find(|result| result.client_order_id == "reconcile-failed")
        .unwrap();
    assert_eq!(failed_result.action, ReconciliationAction::MarkedFailed);

    let saved_recovered = store
        .find_order_by_client_order_id("reconcile-recoverable")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved_recovered.status, OrderStatus::PartiallyFilled);
    assert_eq!(saved_recovered.filled_quantity, 40);
    assert_eq!(saved_recovered.remaining_quantity, 60);
    assert_eq!(saved_recovered.avg_fill_price, Some(dec!(12.50)));

    let saved_failed = store
        .find_order_by_client_order_id("reconcile-failed")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved_failed.status, OrderStatus::Rejected);
}

#[tokio::test]
async fn kernel_recover_pending_orders_applies_three_fill_deltas_without_duplication() {
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
    let kernel = ExecutionKernel::with_fill_delta(
        store.clone(),
        MockLiveExecutionAdapter::with_state_template(
            store.clone(),
            FixedMockLiveClock,
            MockLiveOrderState {
                fill_plan: vec![
                    MockLiveFillStep {
                        quantity: 20,
                        delay_secs: 0,
                    },
                    MockLiveFillStep {
                        quantity: 30,
                        delay_secs: 0,
                    },
                    MockLiveFillStep {
                        quantity: 50,
                        delay_secs: 0,
                    },
                ],
                ..Default::default()
            },
        ),
        fill_applier.clone(),
        FixedRiskEvaluator {
            decision: RiskDecision::Allow,
            sync_calls: Arc::new(Mutex::new(0)),
        },
    );

    let result = kernel
        .execute_once(
            ExecutionRunRequest {
                mode: "mock_live".to_string(),
                ..sample_run_request("recover-three-step-1")
            },
            SignalEnvelope::new(Signal::Buy),
        )
        .await
        .unwrap();
    assert_eq!(result.order_status, Some(OrderStatus::Accepted));

    let first = kernel.recover_pending_orders().await.unwrap();
    assert_eq!(first.scanned, 1);
    assert_eq!(first.recovered, 1);
    let second = kernel.recover_pending_orders().await.unwrap();
    assert_eq!(second.scanned, 1);
    assert_eq!(second.recovered, 1);
    let third = kernel.recover_pending_orders().await.unwrap();
    assert_eq!(third.scanned, 1);
    assert_eq!(third.recovered, 1);

    let order = store
        .find_order_by_client_order_id("recover-three-step-1")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(order.status, OrderStatus::Filled);
    assert_eq!(order.filled_quantity, 100);

    let snapshot = trade_store.snapshot().unwrap();
    let account = snapshot.account.unwrap();
    assert_eq!(account.positions.get("000001").unwrap().volume, 100);
    assert_eq!(snapshot.trade_records.len(), 3);
    let apply_results = fill_applier.results();
    assert_eq!(apply_results.len(), 3);
    assert_eq!(
        apply_results
            .iter()
            .map(|result| result.delta_quantity)
            .collect::<Vec<_>>(),
        vec![20, 30, 50]
    );
}

#[tokio::test]
async fn kernel_recover_pending_orders_handles_unknown_accepted_unknown_then_fill_chain() {
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
    let kernel = ExecutionKernel::with_fill_delta(
        store.clone(),
        MockLiveExecutionAdapter::with_state_template(
            store.clone(),
            FixedMockLiveClock,
            MockLiveOrderState {
                fill_plan: vec![MockLiveFillStep {
                    quantity: 100,
                    delay_secs: 0,
                }],
                fault_injection: Some(MockLiveFaultInjection {
                    mode: Some("query_script:unknown,accepted,unknown".to_string()),
                    delay_seconds: None,
                    rejection_reason: None,
                    timeout_seconds: None,
                }),
                ..Default::default()
            },
        ),
        RecordingTradeFillDeltaApplier::new(trade_store.clone()),
        FixedRiskEvaluator {
            decision: RiskDecision::Allow,
            sync_calls: Arc::new(Mutex::new(0)),
        },
    );

    let result = kernel
        .execute_once(
            ExecutionRunRequest {
                mode: "mock_live".to_string(),
                ..sample_run_request("recover-query-script-1")
            },
            SignalEnvelope::new(Signal::Buy),
        )
        .await
        .unwrap();
    assert_eq!(result.order_status, Some(OrderStatus::Accepted));

    let first = kernel.recover_pending_orders().await.unwrap();
    assert_eq!(first.recovered, 1);
    let first_order = store
        .find_order_by_client_order_id("recover-query-script-1")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(first_order.status, OrderStatus::Unknown);
    assert_eq!(first_order.filled_quantity, 0);

    let second = kernel.recover_pending_orders().await.unwrap();
    assert_eq!(second.recovered, 1);
    let second_order = store
        .find_order_by_client_order_id("recover-query-script-1")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(second_order.status, OrderStatus::Accepted);
    assert_eq!(second_order.filled_quantity, 0);

    let third = kernel.recover_pending_orders().await.unwrap();
    assert_eq!(third.recovered, 1);
    let third_order = store
        .find_order_by_client_order_id("recover-query-script-1")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(third_order.status, OrderStatus::Unknown);
    assert_eq!(third_order.filled_quantity, 0);

    let fourth = kernel.recover_pending_orders().await.unwrap();
    assert_eq!(fourth.recovered, 1);
    let final_order = store
        .find_order_by_client_order_id("recover-query-script-1")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(final_order.status, OrderStatus::Filled);
    assert_eq!(final_order.filled_quantity, 100);

    let snapshot = trade_store.snapshot().unwrap();
    let account = snapshot.account.unwrap();
    assert_eq!(account.positions.get("000001").unwrap().volume, 100);
    assert_eq!(snapshot.trade_records.len(), 1);
}

#[tokio::test]
async fn kernel_retries_after_query_fault_without_double_applying_existing_fill() {
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
    let kernel = ExecutionKernel::with_fill_delta(
        store.clone(),
        MockLiveExecutionAdapter::with_state_template(
            store.clone(),
            FixedMockLiveClock,
            MockLiveOrderState {
                fill_plan: vec![
                    MockLiveFillStep {
                        quantity: 40,
                        delay_secs: 0,
                    },
                    MockLiveFillStep {
                        quantity: 60,
                        delay_secs: 0,
                    },
                ],
                ..Default::default()
            },
        ),
        fill_applier.clone(),
        FixedRiskEvaluator {
            decision: RiskDecision::Allow,
            sync_calls: Arc::new(Mutex::new(0)),
        },
    );

    let result = kernel
        .execute_once(
            ExecutionRunRequest {
                mode: "mock_live".to_string(),
                ..sample_run_request("recover-fault-retry-1")
            },
            SignalEnvelope::new(Signal::Buy),
        )
        .await
        .unwrap();
    assert_eq!(result.order_status, Some(OrderStatus::Accepted));

    let first = kernel.recover_pending_orders().await.unwrap();
    assert_eq!(first.recovered, 1);
    let partial = store
        .find_order_by_client_order_id("recover-fault-retry-1")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(partial.status, OrderStatus::PartiallyFilled);
    assert_eq!(partial.filled_quantity, 40);

    let mut state = store
        .get_mock_live_order_state(&partial.order_id)
        .await
        .unwrap()
        .unwrap();
    state.fault_injection = Some(MockLiveFaultInjection {
        mode: Some("network_disconnect".to_string()),
        delay_seconds: None,
        rejection_reason: None,
        timeout_seconds: None,
    });
    store
        .update_mock_live_order_state(&partial.order_id, Some("recover-fault-retry-1"), &state)
        .await
        .unwrap();

    let failed = kernel.recover_pending_orders().await.unwrap();
    assert_eq!(failed.failed, 1);

    let recovered = kernel.recover_pending_orders().await.unwrap();
    assert_eq!(recovered.recovered, 1);
    let final_order = store
        .find_order_by_client_order_id("recover-fault-retry-1")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(final_order.status, OrderStatus::Filled);
    assert_eq!(final_order.filled_quantity, 100);

    let snapshot = trade_store.snapshot().unwrap();
    let account = snapshot.account.unwrap();
    assert_eq!(account.positions.get("000001").unwrap().volume, 100);
    assert_eq!(snapshot.trade_records.len(), 2);
    let apply_results = fill_applier.results();
    assert_eq!(apply_results.len(), 2);
    assert_eq!(
        apply_results
            .iter()
            .map(|result| result.delta_quantity)
            .collect::<Vec<_>>(),
        vec![40, 60]
    );
}
