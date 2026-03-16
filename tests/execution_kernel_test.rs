use async_trait::async_trait;
use chrono::{Duration, NaiveDate};
use quantix_cli::data::models::{AdjustType, Kline};
use quantix_cli::execution::models::{
    translate_signal, ExecutionPolicy, OrderSide, OrderType, SignalEnvelope,
};
use quantix_cli::strategy::runtime::{StrategyBarLoader, StrategyRuntime};
use quantix_cli::strategy::trait_def::Signal;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

fn make_test_date(offset_days: usize) -> NaiveDate {
    NaiveDate::from_ymd_opt(2024, 1, 1)
        .unwrap()
        .checked_add_signed(Duration::days(offset_days as i64))
        .unwrap()
}

fn create_ma_cross_fixture() -> Vec<Kline> {
    let mut prices = Vec::new();
    let mut price = 100.0;

    for _ in 0..20 {
        prices.push(price);
        price -= 0.5;
    }

    for _ in 0..40 {
        prices.push(price);
        price += 0.5;
    }

    prices
        .iter()
        .enumerate()
        .map(|(i, price)| Kline {
            code: "000001".to_string(),
            date: make_test_date(i),
            open: Decimal::from_str_exact(&price.to_string()).unwrap(),
            high: Decimal::from_str_exact(&(price + 1.0).to_string()).unwrap(),
            low: Decimal::from_str_exact(&(price - 1.0).to_string()).unwrap(),
            close: Decimal::from_str_exact(&price.to_string()).unwrap(),
            volume: 1_000_000,
            amount: Some(
                Decimal::from_str_exact(&price.to_string()).unwrap() * Decimal::from(1_000_000),
            ),
            adjust_type: AdjustType::None,
        })
        .collect()
}

#[derive(Clone)]
struct FakeBarLoader {
    bars: Vec<Kline>,
}

#[async_trait]
impl StrategyBarLoader for FakeBarLoader {
    async fn load_daily_bars(&self, code: &str, limit: usize) -> quantix_cli::core::Result<Vec<Kline>> {
        let mut filtered: Vec<Kline> = self
            .bars
            .iter()
            .filter(|bar| bar.code == code)
            .cloned()
            .collect();
        if filtered.len() > limit {
            filtered = filtered.split_off(filtered.len() - limit);
        }
        Ok(filtered)
    }
}

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

    assert!(matches!(envelope.signal, Signal::Buy | Signal::Sell | Signal::Hold));
}
