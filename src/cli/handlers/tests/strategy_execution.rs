use super::strategy_helpers::{FakeLoader, fixed_ts, make_kline, sample_run, sample_signal};
use super::*;
use crate::bridge::models::{
    BridgeCapabilitiesResponse, BridgeCapabilitySection, BridgeQmtCapabilitySection,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;

#[tokio::test]
async fn test_strategy_paper_requires_explicit_code() {
    let dir = tempdir().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();

    let err = execute_strategy_run_with_components(
        "ma_cross",
        "paper",
        None,
        FakeLoader::default(),
        JsonPaperTradeStore::new(dir.path().join("paper_trade.json")),
        JsonRiskStore::new(dir.path().join("risk_state.json")),
        &runtime_store,
    )
    .await
    .unwrap_err();

    assert!(err.to_string().contains("--code"));
    assert!(err.to_string().contains("--mode paper"));
}

#[tokio::test]
async fn test_strategy_mock_live_requires_explicit_code() {
    let dir = tempdir().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();

    let err = execute_strategy_run_with_components(
        "ma_cross",
        "mock_live",
        None,
        FakeLoader::default(),
        JsonPaperTradeStore::new(dir.path().join("paper_trade.json")),
        JsonRiskStore::new(dir.path().join("risk_state.json")),
        &runtime_store,
    )
    .await
    .unwrap_err();

    assert!(err.to_string().contains("--code"));
    assert!(err.to_string().contains("--mode mock_live"));
}

#[tokio::test]
async fn test_strategy_paper_requires_initialized_account() {
    let dir = tempdir().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let loader = FakeLoader {
        data: HashMap::from([(
            "000001".to_string(),
            (1..=30)
                .map(|day| make_kline("000001", day, dec!(10) + Decimal::from(day), 1000))
                .collect(),
        )]),
    };

    let err = execute_strategy_run_with_components(
        "ma_cross",
        "paper",
        Some("000001".to_string()),
        loader,
        JsonPaperTradeStore::new(dir.path().join("paper_trade.json")),
        JsonRiskStore::new(dir.path().join("risk_state.json")),
        &runtime_store,
    )
    .await
    .unwrap_err();

    assert!(err.to_string().contains("trade init"));
}

#[tokio::test]
async fn test_strategy_live_remains_unsupported() {
    let dir = tempdir().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();

    let err = execute_strategy_run_with_components(
        "ma_cross",
        "live",
        Some("000001".to_string()),
        FakeLoader::default(),
        JsonPaperTradeStore::new(dir.path().join("paper_trade.json")),
        JsonRiskStore::new(dir.path().join("risk_state.json")),
        &runtime_store,
    )
    .await
    .unwrap_err();

    assert!(matches!(err, QuantixError::Unsupported(_)));
    let message = err.to_string();
    assert!(message.contains("live 模式尚未实现"));
    assert!(message.contains("qmt_live"));
    assert!(message.contains("execution bridge qmt-live"));
    assert!(message.contains("qmt.mode=live"));
}

#[tokio::test]
async fn test_strategy_mock_live_returns_non_final_status() {
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

    let summary = execute_strategy_run_with_components(
        "ma_cross",
        "mock_live",
        Some("000001".to_string()),
        loader,
        trade_store,
        JsonRiskStore::new(dir.path().join("risk_state.json")),
        &runtime_store,
    )
    .await
    .unwrap();

    assert_eq!(summary.mode, "mock_live");
    assert_eq!(summary.order_status, Some(OrderStatus::Accepted));
    assert!(summary.message.contains("order_status=accepted"));
}
