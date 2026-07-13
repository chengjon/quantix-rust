use super::*;
use rust_decimal_macros::dec;

#[tokio::test]
async fn test_execute_trade_position_returns_current_positions() {
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
            code: "600000".to_string(),
            price: 10.0,
            volume: 200,
        },
        &service,
    )
    .await
    .unwrap();

    let output =
        execute_trade_command_with_service(TradeCommands::Position { current: false }, &service)
            .await
            .unwrap();

    match output {
        TradeCommandOutput::PositionList(positions) => {
            assert_eq!(positions.len(), 1);
            assert_eq!(positions[0].code, "600000");
            assert_eq!(positions[0].volume, 200);
        }
        other => panic!("unexpected output: {:?}", other),
    }
}

#[tokio::test]
async fn test_execute_trade_cash_returns_current_snapshot() {
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

    let output = execute_trade_command_with_service(TradeCommands::Cash, &service)
        .await
        .unwrap();

    match output {
        TradeCommandOutput::Cash(snapshot) => {
            assert_eq!(snapshot.initial_capital, dec!(500000));
            assert_eq!(snapshot.available_cash, dec!(498995));
            assert_eq!(snapshot.estimated_position_value, dec!(1000));
            assert_eq!(snapshot.estimated_total_assets, dec!(499995));
        }
        other => panic!("unexpected output: {:?}", other),
    }
}

#[tokio::test]
async fn test_execute_trade_history_returns_newest_first_rows() {
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
        TradeCommands::Sell {
            code: "000001".to_string(),
            price: 12.0,
            volume: 40,
        },
        &service,
    )
    .await
    .unwrap();

    let output = execute_trade_command_with_service(
        TradeCommands::History {
            code: None,
            limit: Some(10),
        },
        &service,
    )
    .await
    .unwrap();

    match output {
        TradeCommandOutput::HistoryRows(rows) => {
            assert_eq!(rows.len(), 2);
            assert_eq!(rows[0].side, TradeSide::Sell);
            assert_eq!(rows[1].side, TradeSide::Buy);
        }
        other => panic!("unexpected output: {:?}", other),
    }
}

#[tokio::test]
async fn test_execute_trade_fees_filters_by_code() {
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

    let output = execute_trade_command_with_service(
        TradeCommands::Fees {
            code: Some("600000".to_string()),
            limit: Some(10),
        },
        &service,
    )
    .await
    .unwrap();

    match output {
        TradeCommandOutput::FeeRows(rows) => {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].code, "600000");
            assert_eq!(rows[0].transfer_fee, dec!(0.02));
        }
        other => panic!("unexpected output: {:?}", other),
    }
}

#[tokio::test]
async fn test_execute_trade_overview_returns_booked_summary() {
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

    let output =
        execute_trade_command_with_service(TradeCommands::Overview { current: false }, &service)
            .await
            .unwrap();

    match output {
        TradeCommandOutput::Overview(overview) => {
            assert_eq!(overview.initial_capital, dec!(500000));
            assert_eq!(overview.trade_count, 1);
            assert_eq!(overview.holding_count, 1);
            assert_eq!(overview.total_buy_amount, dec!(1000));
            assert_eq!(overview.total_fee, dec!(5));
        }
        other => panic!("unexpected output: {:?}", other),
    }
}

#[tokio::test]
async fn test_execute_trade_overview_before_init_returns_user_facing_error() {
    let (service, _) = trade_service();

    let err =
        execute_trade_command_with_service(TradeCommands::Overview { current: false }, &service)
            .await
            .unwrap_err();

    assert!(matches!(err, QuantixError::Other(_)));
    assert!(err.to_string().contains("尚未初始化"));
}
