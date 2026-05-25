use super::strategy_helpers::{FakeLoader, fixed_ts, make_kline};
use super::*;
use rust_decimal_macros::dec;
use std::collections::HashMap;

async fn test_strategy_paper_risk_bridge_surfaces_volatility_limit_reason() {
    let dir = tempdir().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let loader = FakeLoader {
        data: HashMap::from([(
            "000001".to_string(),
            [
                dec!(10),
                dec!(9),
                dec!(8),
                dec!(7),
                dec!(6),
                dec!(5),
                dec!(4),
                dec!(3),
                dec!(2),
                dec!(1),
                dec!(1),
                dec!(1),
                dec!(1),
                dec!(1),
                dec!(12),
            ]
            .into_iter()
            .enumerate()
            .map(|(idx, close)| make_kline("000001", (idx + 1) as u32, close, 1000))
            .collect(),
        )]),
    };
    let trade_store = JsonPaperTradeStore::new(dir.path().join("paper_trade.json"));
    let trade_service = TradeService::new(trade_store.clone());
    trade_service
        .init_account(
            crate::trade::InitAccountRequest::new(Some(100000.0), None, None, None, None).unwrap(),
            fixed_ts(),
        )
        .await
        .unwrap();
    let risk_service = RiskService::with_bar_loader(
        JsonRiskStore::new(dir.path().join("risk_state.json")),
        loader.clone(),
    );
    risk_service
        .set_rule("volatility-limit", "4%", fixed_ts())
        .await
        .unwrap();

    let summary = execute_strategy_run_with_risk_service(
        "ma_cross",
        "paper",
        Some("000001".to_string()),
        loader,
        trade_store,
        risk_service,
        &runtime_store,
    )
    .await
    .unwrap();

    assert_eq!(summary.order_status, Some(OrderStatus::Rejected));

    let order = runtime_store
        .find_first_order_for_run(&summary.run_id)
        .await
        .unwrap()
        .unwrap();
    let events = runtime_store
        .list_order_events(&order.order_id)
        .await
        .unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, "risk_rejected");
    assert!(
        events[0].details_json["reason"]
            .as_str()
            .unwrap()
            .contains("volatility-limit")
    );
}

#[tokio::test]
async fn test_strategy_mock_live_risk_bridge_surfaces_volatility_limit_reason() {
    let dir = tempdir().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let loader = FakeLoader {
        data: HashMap::from([(
            "000001".to_string(),
            [
                dec!(10),
                dec!(9),
                dec!(8),
                dec!(7),
                dec!(6),
                dec!(5),
                dec!(4),
                dec!(3),
                dec!(2),
                dec!(1),
                dec!(1),
                dec!(1),
                dec!(1),
                dec!(1),
                dec!(12),
            ]
            .into_iter()
            .enumerate()
            .map(|(idx, close)| make_kline("000001", (idx + 1) as u32, close, 1000))
            .collect(),
        )]),
    };
    let trade_store = JsonPaperTradeStore::new(dir.path().join("paper_trade.json"));
    let trade_service = TradeService::new(trade_store.clone());
    trade_service
        .init_account(
            crate::trade::InitAccountRequest::new(Some(100000.0), None, None, None, None).unwrap(),
            fixed_ts(),
        )
        .await
        .unwrap();
    let risk_service = RiskService::with_bar_loader(
        JsonRiskStore::new(dir.path().join("risk_state.json")),
        loader.clone(),
    );
    risk_service
        .set_rule("volatility-limit", "4%", fixed_ts())
        .await
        .unwrap();

    let summary = execute_strategy_run_with_risk_service(
        "ma_cross",
        "mock_live",
        Some("000001".to_string()),
        loader,
        trade_store.clone(),
        risk_service,
        &runtime_store,
    )
    .await
    .unwrap();

    assert_eq!(summary.order_status, Some(OrderStatus::Rejected));

    let order = runtime_store
        .find_first_order_for_run(&summary.run_id)
        .await
        .unwrap()
        .unwrap();
    let events = runtime_store
        .list_order_events(&order.order_id)
        .await
        .unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, "risk_rejected");
    assert!(
        events[0].details_json["reason"]
            .as_str()
            .unwrap()
            .contains("volatility-limit")
    );

    let state = trade_store.load_state().await.unwrap().unwrap();
    assert!(state.account.unwrap().positions.is_empty());
}
