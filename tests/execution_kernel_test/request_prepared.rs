use super::helpers::*;

#[tokio::test]
async fn request_prepared_execution_supports_paper_mode() {
    let dir = tempdir().unwrap();
    let trade_store = FakePaperTradeStore::default();
    let trade_service = TradeService::new(trade_store.clone());
    trade_service
        .init_account(
            InitAccountRequest::new(Some(100000.0), None, None, None, None).unwrap(),
            fixed_ts(),
        )
        .await
        .unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let kernel = ExecutionKernel::new(
        runtime_store,
        PaperExecutionAdapter::new(trade_service),
        FixedRiskEvaluator {
            decision: RiskDecision::Allow,
            sync_calls: Arc::new(Mutex::new(0)),
        },
    );

    let result = kernel
        .execute_request(sample_prepared_request("paper", "request-paper-1"))
        .await
        .unwrap();

    assert_eq!(result.order_status, Some(OrderStatus::Filled));
    assert_eq!(result.client_order_id.as_deref(), Some("request-paper-1"));
}

#[tokio::test]
async fn request_prepared_execution_supports_mock_live_mode() {
    let dir = tempdir().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let kernel = ExecutionKernel::new(
        runtime_store.clone(),
        MockLiveExecutionAdapter::new(runtime_store, FixedMockLiveClock),
        FixedRiskEvaluator {
            decision: RiskDecision::Allow,
            sync_calls: Arc::new(Mutex::new(0)),
        },
    );

    let result = kernel
        .execute_request(sample_prepared_request("mock_live", "request-mock-live-1"))
        .await
        .unwrap();

    assert_eq!(result.order_status, Some(OrderStatus::Accepted));
    assert_eq!(
        result.client_order_id.as_deref(),
        Some("request-mock-live-1")
    );
}
