use super::helpers::*;

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
async fn paper_adapter_query_returns_filled_trade_record() {
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

    let submit = adapter
        .submit_order(AdapterOrderRequest {
            client_order_id: "run_000001_query".to_string(),
            symbol: "000001".to_string(),
            side: OrderSide::Buy,
            quantity: 100,
            price: dec!(10.00),
        })
        .await
        .unwrap();
    let query = adapter.query_order(&submit.adapter_order_id).await.unwrap();

    assert_eq!(
        IMMEDIATE_FILL_ONLY,
        submit.latest_status == OrderStatus::Filled
    );
    assert_eq!(submit.latest_status, OrderStatus::Filled);
    assert_eq!(submit.filled_quantity, 100);
    assert_eq!(submit.avg_fill_price, Some(dec!(10.00)));
    assert_eq!(query.adapter_order_id, submit.adapter_order_id);
    assert_eq!(query.latest_status, OrderStatus::Filled);
    assert_eq!(query.filled_quantity, 100);
    assert_eq!(query.avg_fill_price, Some(dec!(10.00)));
}

#[tokio::test]
async fn paper_adapter_cancel_rejects_filled_trade_record_without_unsupported() {
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

    let submit = adapter
        .submit_order(AdapterOrderRequest {
            client_order_id: "run_000001_cancel".to_string(),
            symbol: "000001".to_string(),
            side: OrderSide::Buy,
            quantity: 100,
            price: dec!(10.00),
        })
        .await
        .unwrap();
    assert_eq!(submit.latest_status, OrderStatus::Filled);
    let err = adapter
        .cancel_order(&submit.adapter_order_id)
        .await
        .unwrap_err();

    assert!(matches!(err, AdapterError::Execution(message) if message.contains("已成交")));
}
