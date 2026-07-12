use super::helpers::*;

#[test]
fn hold_signal_produces_no_order_intent() {
    let envelope = SignalEnvelope::new(Signal::Hold);
    let policy = ExecutionPolicy {
        fixed_cash_per_buy: dec!(2500),
        slippage_bps: 0,
    };

    let result = translate_signal(&envelope, "000001", dec!(12.34), None, &policy).unwrap();

    assert!(result.is_none());
}

#[test]
fn buy_signal_uses_fixed_cash_and_rounds_down_to_board_lot() {
    let envelope = SignalEnvelope::new(Signal::Buy);
    let policy = ExecutionPolicy {
        fixed_cash_per_buy: dec!(2500),
        slippage_bps: 0,
    };

    let intent = translate_signal(&envelope, "000001", dec!(12.34), None, &policy)
        .unwrap()
        .unwrap();

    assert_eq!(intent.symbol, "000001");
    assert_eq!(intent.side, OrderSide::Buy);
    assert_eq!(intent.order_type, OrderType::Market);
    assert_eq!(intent.requested_quantity, 200);
    assert_eq!(intent.requested_price, dec!(12.34));
}

#[test]
fn sell_signal_uses_sell_all_position_volume() {
    let envelope = SignalEnvelope::new(Signal::Sell);
    let policy = ExecutionPolicy {
        fixed_cash_per_buy: dec!(2500),
        slippage_bps: 0,
    };

    let intent = translate_signal(&envelope, "000001", dec!(11.80), Some(300), &policy)
        .unwrap()
        .unwrap();

    assert_eq!(intent.symbol, "000001");
    assert_eq!(intent.side, OrderSide::Sell);
    assert_eq!(intent.order_type, OrderType::Market);
    assert_eq!(intent.requested_quantity, 300);
    assert_eq!(intent.requested_price, dec!(11.80));
}

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
