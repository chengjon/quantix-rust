use super::*;

#[tokio::test]
async fn strategy_runtime_returns_latest_signal_for_ma_cross() {
    let runtime = StrategyRuntime::new(FakeBarLoader {
        bars: create_ma_cross_fixture(),
    });

    let envelope = runtime.run_ma_cross_once("000001", 5, 10).await.unwrap();

    assert!(matches!(
        envelope.signal,
        Signal::Buy | Signal::Sell | Signal::Hold
    ));
}

#[tokio::test]
async fn paper_adapter_buy_submission_returns_filled_and_updates_account() {
    let store = FakePaperTradeStore::default();
    let service = TradeService::new(store.clone());
    service
        .init_account(
            InitAccountRequest::new(Some(100000.0), None, None, None, None).unwrap(),
            chrono::Utc::now(),
        )
        .await
        .unwrap();
    let adapter = PaperExecutionAdapter::new(service);

    let result = adapter
        .submit_order(AdapterOrderRequest {
            client_order_id: "run_000001_1".to_string(),
            symbol: "000001".to_string(),
            side: OrderSide::Buy,
            quantity: 100,
            price: dec!(10.00),
        })
        .await
        .unwrap();

    assert_eq!(
        result.latest_status,
        quantix_cli::execution::models::OrderStatus::Filled
    );
    assert_eq!(result.filled_quantity, 100);
    assert_eq!(result.avg_fill_price, Some(dec!(10.00)));

    let account = store.snapshot().unwrap().account.unwrap();
    assert!(account.positions.contains_key("000001"));
}

#[tokio::test]
async fn paper_adapter_sell_submission_returns_filled() {
    let store = FakePaperTradeStore::default();
    let service = TradeService::new(store.clone());
    let now = chrono::Utc::now();
    service
        .init_account(
            InitAccountRequest::new(Some(100000.0), None, None, None, None).unwrap(),
            now,
        )
        .await
        .unwrap();
    service
        .buy(
            quantix_cli::trade::TradeOrderRequest::new("000001", 10.0, 100).unwrap(),
            now,
        )
        .await
        .unwrap();
    let adapter = PaperExecutionAdapter::new(service);

    let result = adapter
        .submit_order(AdapterOrderRequest {
            client_order_id: "run_000001_2".to_string(),
            symbol: "000001".to_string(),
            side: OrderSide::Sell,
            quantity: 100,
            price: dec!(11.00),
        })
        .await
        .unwrap();

    assert_eq!(
        result.latest_status,
        quantix_cli::execution::models::OrderStatus::Filled
    );
    assert_eq!(result.filled_quantity, 100);
    assert_eq!(result.avg_fill_price, Some(dec!(11.00)));
    assert!(
        store
            .snapshot()
            .unwrap()
            .account
            .unwrap()
            .positions
            .is_empty()
    );
}

#[tokio::test]
async fn paper_adapter_cancel_returns_unsupported() {
    let store = FakePaperTradeStore::default();
    let service = TradeService::new(store);
    let adapter = PaperExecutionAdapter::new(service);

    let err = adapter.cancel_order("paper-order-1").await.unwrap_err();

    assert!(matches!(err, AdapterError::Unsupported(_)));
}

#[tokio::test]
async fn paper_adapter_query_returns_unsupported() {
    let store = FakePaperTradeStore::default();
    let service = TradeService::new(store);
    let adapter = PaperExecutionAdapter::new(service);

    let err = adapter.query_order("paper-order-1").await.unwrap_err();

    assert!(matches!(err, AdapterError::Unsupported(_)));
}
