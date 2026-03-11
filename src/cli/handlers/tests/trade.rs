use super::*;
use super::support::trade_service;

#[tokio::test]
async fn test_execute_trade_init_succeeds_and_returns_account_summary() {
    let (service, store) = trade_service();

    let output = execute_trade_command_with_service(
        TradeCommands::Init {
            capital: Some(1500000.0),
            commission_rate: Some(0.0003),
            commission_min: Some(3.0),
            stamp_duty_rate: None,
            transfer_fee_rate: None,
        },
        &service,
    )
    .await
    .unwrap();

    match output {
        TradeCommandOutput::AccountInitialized(account) => {
            assert_eq!(account.account_id, "default");
            assert_eq!(account.initial_capital, dec!(1500000));
            assert_eq!(account.available_cash, dec!(1500000));
            assert_eq!(account.fee_config.commission_rate, dec!(0.0003));
            assert_eq!(account.fee_config.commission_min, dec!(3));
        }
        other => panic!("unexpected output: {:?}", other),
    }

    let state = store.snapshot().unwrap();
    assert!(state.account.is_some());
    assert!(state.trade_records.is_empty());
}

#[tokio::test]
async fn test_execute_trade_reset_clears_previous_state() {
    let (service, store) = trade_service();

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

    let output = execute_trade_command_with_service(
        TradeCommands::Reset {
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

    match output {
        TradeCommandOutput::AccountReset(account) => {
            assert_eq!(account.initial_capital, dec!(500000));
            assert_eq!(account.available_cash, dec!(500000));
            assert!(account.positions.is_empty());
        }
        other => panic!("unexpected output: {:?}", other),
    }

    let state = store.snapshot().unwrap();
    assert!(state.trade_records.is_empty());
    assert!(state.account.unwrap().positions.is_empty());
}

#[tokio::test]
async fn test_execute_trade_buy_succeeds_and_returns_trade_summary() {
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

    let output = execute_trade_command_with_service(
        TradeCommands::Buy {
            code: "000001".to_string(),
            price: 15.0,
            volume: 1000,
        },
        &service,
    )
    .await
    .unwrap();

    match output {
        TradeCommandOutput::TradeExecuted(record) => {
            assert_eq!(record.side, TradeSide::Buy);
            assert_eq!(record.code, "000001");
            assert_eq!(record.price, dec!(15));
            assert_eq!(record.volume, 1000);
        }
        other => panic!("unexpected output: {:?}", other),
    }
}

#[tokio::test]
async fn test_execute_trade_sell_succeeds_and_returns_trade_summary() {
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
            price: 15.0,
            volume: 1000,
        },
        &service,
    )
    .await
    .unwrap();

    let output = execute_trade_command_with_service(
        TradeCommands::Sell {
            code: "000001".to_string(),
            price: 16.0,
            volume: 400,
        },
        &service,
    )
    .await
    .unwrap();

    match output {
        TradeCommandOutput::TradeExecuted(record) => {
            assert_eq!(record.side, TradeSide::Sell);
            assert_eq!(record.code, "000001");
            assert_eq!(record.price, dec!(16));
            assert_eq!(record.volume, 400);
        }
        other => panic!("unexpected output: {:?}", other),
    }
}

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

    let output = execute_trade_command_with_service(TradeCommands::Position, &service)
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
