use super::*;
use crate::core::QuantixError;
use rust_decimal_macros::dec;
use std::collections::HashMap;

#[tokio::test]
async fn test_execute_trade_position_current_uses_live_quotes_when_available() {
    let (service, _) = trade_service();

    execute_trade_command_with_service(
        TradeCommands::Init {
            capital: None,
            commission_rate: None,
            commission_min: None,
            stamp_duty_rate: None,
            transfer_fee_rate: None,
        },
        &service,
    )
    .await
    .unwrap();
    execute_trade_command_with_service(
        TradeCommands::Buy {
            code: "000001".to_string(),
            price: 10.0,
            volume: 100,
        },
        &service,
    )
    .await
    .unwrap();

    let output = execute_trade_command_with_quote_lookup(
        TradeCommands::Position { current: true },
        &service,
        &FakeTradeQuoteLookup {
            quotes: HashMap::from([(
                "000001".to_string(),
                WatchlistQuoteSnapshot {
                    latest_price: dec!(12),
                    price_change_pct: Some(dec!(5)),
                },
            )]),
            fail: false,
        },
    )
    .await
    .unwrap();

    match output {
        TradeCommandOutput::PositionCurrentList(rows) => {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].current_price, Some(dec!(12)));
            assert_eq!(rows[0].quote_status, crate::trade::TradeQuoteStatus::Live);
        }
        other => panic!("unexpected output: {:?}", other),
    }
}

#[tokio::test]
async fn test_execute_trade_position_current_degrades_when_quotes_are_partial() {
    let (service, _) = trade_service();

    execute_trade_command_with_service(
        TradeCommands::Init {
            capital: None,
            commission_rate: None,
            commission_min: None,
            stamp_duty_rate: None,
            transfer_fee_rate: None,
        },
        &service,
    )
    .await
    .unwrap();
    execute_trade_command_with_service(
        TradeCommands::Buy {
            code: "000001".to_string(),
            price: 10.0,
            volume: 100,
        },
        &service,
    )
    .await
    .unwrap();
    execute_trade_command_with_service(
        TradeCommands::Buy {
            code: "600000".to_string(),
            price: 20.0,
            volume: 100,
        },
        &service,
    )
    .await
    .unwrap();

    let output = execute_trade_command_with_quote_lookup(
        TradeCommands::Position { current: true },
        &service,
        &FakeTradeQuoteLookup {
            quotes: HashMap::from([(
                "000001".to_string(),
                WatchlistQuoteSnapshot {
                    latest_price: dec!(12),
                    price_change_pct: Some(dec!(5)),
                },
            )]),
            fail: false,
        },
    )
    .await
    .unwrap();

    match output {
        TradeCommandOutput::PositionCurrentList(rows) => {
            let missing = rows.iter().find(|row| row.code == "600000").unwrap();
            assert_eq!(missing.current_price, None);
            assert_eq!(
                missing.quote_status,
                crate::trade::TradeQuoteStatus::Missing
            );
        }
        other => panic!("unexpected output: {:?}", other),
    }
}

#[tokio::test]
async fn test_execute_trade_overview_current_uses_live_totals_when_quotes_are_complete() {
    let (service, _) = trade_service();

    execute_trade_command_with_service(
        TradeCommands::Init {
            capital: Some(500000.0),
            commission_rate: None,
            commission_min: None,
            stamp_duty_rate: None,
            transfer_fee_rate: None,
        },
        &service,
    )
    .await
    .unwrap();
    execute_trade_command_with_service(
        TradeCommands::Buy {
            code: "000001".to_string(),
            price: 10.0,
            volume: 100,
        },
        &service,
    )
    .await
    .unwrap();

    let output = execute_trade_command_with_quote_lookup(
        TradeCommands::Overview { current: true },
        &service,
        &FakeTradeQuoteLookup {
            quotes: HashMap::from([(
                "000001".to_string(),
                WatchlistQuoteSnapshot {
                    latest_price: dec!(12),
                    price_change_pct: Some(dec!(5)),
                },
            )]),
            fail: false,
        },
    )
    .await
    .unwrap();

    match output {
        TradeCommandOutput::Overview(overview) => {
            assert_eq!(overview.live_position_value, Some(dec!(1200)));
            assert_eq!(overview.live_total_assets, Some(dec!(500195)));
            assert_eq!(overview.quote_coverage, Some((1, 1)));
        }
        other => panic!("unexpected output: {:?}", other),
    }
}

#[tokio::test]
async fn test_execute_trade_overview_current_withholds_live_totals_on_partial_quotes() {
    let (service, _) = trade_service();

    execute_trade_command_with_service(
        TradeCommands::Init {
            capital: Some(500000.0),
            commission_rate: None,
            commission_min: None,
            stamp_duty_rate: None,
            transfer_fee_rate: None,
        },
        &service,
    )
    .await
    .unwrap();
    execute_trade_command_with_service(
        TradeCommands::Buy {
            code: "000001".to_string(),
            price: 10.0,
            volume: 100,
        },
        &service,
    )
    .await
    .unwrap();
    execute_trade_command_with_service(
        TradeCommands::Buy {
            code: "600000".to_string(),
            price: 20.0,
            volume: 100,
        },
        &service,
    )
    .await
    .unwrap();

    let output = execute_trade_command_with_quote_lookup(
        TradeCommands::Overview { current: true },
        &service,
        &FakeTradeQuoteLookup {
            quotes: HashMap::from([(
                "000001".to_string(),
                WatchlistQuoteSnapshot {
                    latest_price: dec!(12),
                    price_change_pct: Some(dec!(5)),
                },
            )]),
            fail: false,
        },
    )
    .await
    .unwrap();

    match output {
        TradeCommandOutput::Overview(overview) => {
            assert_eq!(overview.live_position_value, None);
            assert_eq!(overview.live_total_assets, None);
            assert_eq!(overview.quote_coverage, Some((1, 2)));
        }
        other => panic!("unexpected output: {:?}", other),
    }
}

#[tokio::test]
async fn test_execute_trade_overview_current_degrades_gracefully_on_quote_failure() {
    let (service, _) = trade_service();

    execute_trade_command_with_service(
        TradeCommands::Init {
            capital: Some(500000.0),
            commission_rate: None,
            commission_min: None,
            stamp_duty_rate: None,
            transfer_fee_rate: None,
        },
        &service,
    )
    .await
    .unwrap();
    execute_trade_command_with_service(
        TradeCommands::Buy {
            code: "000001".to_string(),
            price: 10.0,
            volume: 100,
        },
        &service,
    )
    .await
    .unwrap();

    let output = execute_trade_command_with_quote_lookup(
        TradeCommands::Overview { current: true },
        &service,
        &FakeTradeQuoteLookup {
            quotes: HashMap::new(),
            fail: true,
        },
    )
    .await
    .unwrap();

    match output {
        TradeCommandOutput::Overview(overview) => {
            assert_eq!(overview.live_position_value, None);
            assert_eq!(overview.live_total_assets, None);
            assert_eq!(overview.quote_coverage, Some((0, 1)));
        }
        other => panic!("unexpected output: {:?}", other),
    }
}

#[tokio::test]
async fn test_execute_trade_buy_before_init_returns_user_facing_error() {
    let (service, _) = trade_service();

    let err = execute_trade_command_with_service(
        TradeCommands::Buy {
            code: "000001".to_string(),
            price: 15.0,
            volume: 100,
        },
        &service,
    )
    .await
    .unwrap_err();

    assert!(matches!(err, QuantixError::Other(_)));
    assert!(err.to_string().contains("尚未初始化"));
}

#[tokio::test]
async fn test_execute_trade_sell_before_init_returns_user_facing_error() {
    let (service, _) = trade_service();

    let err = execute_trade_command_with_service(
        TradeCommands::Sell {
            code: "000001".to_string(),
            price: 15.0,
            volume: 100,
        },
        &service,
    )
    .await
    .unwrap_err();

    assert!(matches!(err, QuantixError::Other(_)));
    assert!(err.to_string().contains("尚未初始化"));
}

#[tokio::test]
async fn test_execute_trade_buy_rejects_invalid_price_or_volume() {
    let (service, _) = trade_service();

    execute_trade_command_with_service(
        TradeCommands::Init {
            capital: None,
            commission_rate: None,
            commission_min: None,
            stamp_duty_rate: None,
            transfer_fee_rate: None,
        },
        &service,
    )
    .await
    .unwrap();

    let price_err = execute_trade_command_with_service(
        TradeCommands::Buy {
            code: "000001".to_string(),
            price: 0.0,
            volume: 100,
        },
        &service,
    )
    .await
    .unwrap_err();
    assert!(matches!(price_err, QuantixError::Other(_)));
    assert!(price_err.to_string().contains("--price"));

    let volume_err = execute_trade_command_with_service(
        TradeCommands::Buy {
            code: "000001".to_string(),
            price: 10.0,
            volume: 0,
        },
        &service,
    )
    .await
    .unwrap_err();
    assert!(matches!(volume_err, QuantixError::Other(_)));
    assert!(volume_err.to_string().contains("--volume"));
}

#[tokio::test]
async fn test_execute_trade_sell_rejects_unheld_code_or_excess_volume() {
    let (service, _) = trade_service();

    execute_trade_command_with_service(
        TradeCommands::Init {
            capital: None,
            commission_rate: None,
            commission_min: None,
            stamp_duty_rate: None,
            transfer_fee_rate: None,
        },
        &service,
    )
    .await
    .unwrap();

    let missing_err = execute_trade_command_with_service(
        TradeCommands::Sell {
            code: "000001".to_string(),
            price: 10.0,
            volume: 100,
        },
        &service,
    )
    .await
    .unwrap_err();
    assert!(matches!(missing_err, QuantixError::Other(_)));
    assert!(missing_err.to_string().contains("未持有"));

    execute_trade_command_with_service(
        TradeCommands::Buy {
            code: "000001".to_string(),
            price: 10.0,
            volume: 100,
        },
        &service,
    )
    .await
    .unwrap();

    let excess_err = execute_trade_command_with_service(
        TradeCommands::Sell {
            code: "000001".to_string(),
            price: 10.0,
            volume: 200,
        },
        &service,
    )
    .await
    .unwrap_err();
    assert!(matches!(excess_err, QuantixError::Other(_)));
    assert!(excess_err.to_string().contains("可卖数量不足"));
}
