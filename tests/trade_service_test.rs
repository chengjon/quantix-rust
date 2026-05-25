use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use quantix_cli::core::{QuantixError, Result};
use quantix_cli::trade::{
    FeeConfig, InitAccountRequest, PaperTradeState, PaperTradeStore, TradeOrderRequest,
    TradeService, TradeSide, calculate_fee_breakdown,
};
use rust_decimal_macros::dec;
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
struct FakePaperTradeStore {
    state: Arc<Mutex<Option<PaperTradeState>>>,
}

impl FakePaperTradeStore {
    fn snapshot(&self) -> Option<PaperTradeState> {
        self.state.lock().unwrap().clone()
    }
}

#[async_trait]
impl PaperTradeStore for FakePaperTradeStore {
    async fn load_state(&self) -> Result<Option<PaperTradeState>> {
        Ok(self.snapshot())
    }

    async fn save_state(&self, state: &PaperTradeState) -> Result<()> {
        *self.state.lock().unwrap() = Some(state.clone());
        Ok(())
    }
}

fn fixed_ts() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 11, 12, 0, 0).unwrap()
}

fn service() -> (TradeService<FakePaperTradeStore>, FakePaperTradeStore) {
    let store = FakePaperTradeStore::default();
    (TradeService::new(store.clone()), store)
}

fn assert_uninitialized_account_error(err: QuantixError) {
    let QuantixError::Other(message) = err else {
        panic!("unexpected trade error: {err}");
    };
    assert_eq!(message, "trade account 尚未初始化，请先运行 trade init");
}

#[tokio::test]
async fn init_account_creates_the_default_account_with_default_capital() {
    let (service, store) = service();

    let account = service
        .init_account(
            InitAccountRequest::new(None, None, None, None, None).unwrap(),
            fixed_ts(),
        )
        .await
        .unwrap();

    assert_eq!(account.account_id, "default");
    assert_eq!(account.initial_capital, dec!(1000000));
    assert_eq!(account.available_cash, dec!(1000000));
    assert_eq!(account.fee_config, FeeConfig::default());

    let state = store.snapshot().unwrap();
    assert_eq!(state.version, 1);
    assert!(state.trade_records.is_empty());
    assert!(state.account.unwrap().positions.is_empty());
}

#[tokio::test]
async fn account_operations_return_quantix_error_before_account_initialization() {
    let (service, _store) = service();
    let request = TradeOrderRequest::new("000001", 10.0, 100).unwrap();

    let buy_err = service.buy(request.clone(), fixed_ts()).await.unwrap_err();
    assert_uninitialized_account_error(buy_err);

    let sell_err = service.sell(request.clone(), fixed_ts()).await.unwrap_err();
    assert_uninitialized_account_error(sell_err);

    let positions_err = service.positions().await.unwrap_err();
    assert_uninitialized_account_error(positions_err);

    let cash_snapshot_err = service.cash_snapshot().await.unwrap_err();
    assert_uninitialized_account_error(cash_snapshot_err);

    let state_snapshot_err = service.state_snapshot().await.unwrap_err();
    assert_uninitialized_account_error(state_snapshot_err);
}

#[tokio::test]
async fn init_account_stores_custom_fee_config() {
    let (service, _) = service();

    let account = service
        .init_account(
            InitAccountRequest::new(
                Some(1500000.0),
                Some(0.0003),
                Some(3.0),
                Some(0.0012),
                Some(0.00002),
            )
            .unwrap(),
            fixed_ts(),
        )
        .await
        .unwrap();

    assert_eq!(account.initial_capital, dec!(1500000));
    assert_eq!(account.fee_config.commission_rate, dec!(0.0003));
    assert_eq!(account.fee_config.commission_min, dec!(3));
    assert_eq!(account.fee_config.stamp_duty_rate, dec!(0.0012));
    assert_eq!(account.fee_config.transfer_fee_rate, dec!(0.00002));
}

#[tokio::test]
async fn reset_account_overwrites_an_existing_account_and_clears_old_trades() {
    let (service, store) = service();
    let now = fixed_ts();

    service
        .init_account(
            InitAccountRequest::new(None, None, None, None, None).unwrap(),
            now,
        )
        .await
        .unwrap();
    service
        .buy(
            TradeOrderRequest::new("000001", 10.0, 100).unwrap(),
            now + chrono::Duration::minutes(1),
        )
        .await
        .unwrap();

    let account = service
        .reset_account(
            InitAccountRequest::new(Some(500000.0), Some(0.0003), None, None, None).unwrap(),
            now + chrono::Duration::minutes(2),
        )
        .await
        .unwrap();

    assert_eq!(account.initial_capital, dec!(500000));
    assert_eq!(account.available_cash, dec!(500000));
    assert!(account.positions.is_empty());

    let state = store.snapshot().unwrap();
    assert!(state.trade_records.is_empty());
    assert!(state.account.unwrap().positions.is_empty());
}

#[tokio::test]
async fn buy_opens_a_new_position_and_reduces_cash_by_amount_plus_fees() {
    let (service, store) = service();
    let now = fixed_ts();

    service
        .init_account(
            InitAccountRequest::new(None, None, None, None, None).unwrap(),
            now,
        )
        .await
        .unwrap();

    let record = service
        .buy(
            TradeOrderRequest::new("000001", 10.0, 100).unwrap(),
            now + chrono::Duration::minutes(1),
        )
        .await
        .unwrap();

    assert_eq!(record.side, TradeSide::Buy);
    assert_eq!(record.amount, dec!(1000));
    assert_eq!(record.total_fee, dec!(5));

    let state = store.snapshot().unwrap();
    let account = state.account.unwrap();
    let position = account.positions.get("000001").unwrap();

    assert_eq!(account.available_cash, dec!(998995));
    assert_eq!(position.volume, 100);
    assert_eq!(position.avg_cost, dec!(10.05));
    assert_eq!(position.last_trade_price, dec!(10));
}

#[tokio::test]
async fn second_buy_updates_weighted_average_cost_including_buy_side_fees() {
    let (service, store) = service();
    let now = fixed_ts();

    service
        .init_account(
            InitAccountRequest::new(None, None, None, None, None).unwrap(),
            now,
        )
        .await
        .unwrap();
    service
        .buy(
            TradeOrderRequest::new("600000", 10.0, 1000).unwrap(),
            now + chrono::Duration::minutes(1),
        )
        .await
        .unwrap();

    service
        .buy(
            TradeOrderRequest::new("600000", 20.0, 1000).unwrap(),
            now + chrono::Duration::minutes(2),
        )
        .await
        .unwrap();

    let state = store.snapshot().unwrap();
    let account = state.account.unwrap();
    let position = account.positions.get("600000").unwrap();

    assert_eq!(account.available_cash, dec!(969989.7));
    assert_eq!(position.volume, 2000);
    assert_eq!(position.avg_cost, dec!(15.00515));
    assert_eq!(position.last_trade_price, dec!(20));
}

#[tokio::test]
async fn buy_rejects_insufficient_cash() {
    let (service, store) = service();
    let now = fixed_ts();

    service
        .init_account(
            InitAccountRequest::new(Some(1000.0), None, None, None, None).unwrap(),
            now,
        )
        .await
        .unwrap();

    let err = service
        .buy(
            TradeOrderRequest::new("000001", 10.0, 100).unwrap(),
            now + chrono::Duration::minutes(1),
        )
        .await
        .unwrap_err();

    assert!(err.to_string().contains("可用资金不足"));

    let state = store.snapshot().unwrap();
    let account = state.account.unwrap();
    assert_eq!(account.available_cash, dec!(1000));
    assert!(account.positions.is_empty());
}

#[tokio::test]
async fn sell_reduces_a_position_and_increases_cash_by_amount_minus_fees() {
    let (service, store) = service();
    let now = fixed_ts();

    service
        .init_account(
            InitAccountRequest::new(None, None, None, None, None).unwrap(),
            now,
        )
        .await
        .unwrap();
    service
        .buy(
            TradeOrderRequest::new("000001", 10.0, 100).unwrap(),
            now + chrono::Duration::minutes(1),
        )
        .await
        .unwrap();

    let record = service
        .sell(
            TradeOrderRequest::new("000001", 12.0, 40).unwrap(),
            now + chrono::Duration::minutes(2),
        )
        .await
        .unwrap();

    assert_eq!(record.side, TradeSide::Sell);
    assert_eq!(record.amount, dec!(480));
    assert_eq!(record.total_fee, dec!(5.48));

    let state = store.snapshot().unwrap();
    let account = state.account.unwrap();
    let position = account.positions.get("000001").unwrap();

    assert_eq!(account.available_cash, dec!(999469.52));
    assert_eq!(position.volume, 60);
    assert_eq!(position.avg_cost, dec!(10.05));
    assert_eq!(position.last_trade_price, dec!(12));
}

#[tokio::test]
async fn sell_removes_the_position_when_volume_reaches_zero() {
    let (service, store) = service();
    let now = fixed_ts();

    service
        .init_account(
            InitAccountRequest::new(None, None, None, None, None).unwrap(),
            now,
        )
        .await
        .unwrap();
    service
        .buy(
            TradeOrderRequest::new("000001", 10.0, 100).unwrap(),
            now + chrono::Duration::minutes(1),
        )
        .await
        .unwrap();

    service
        .sell(
            TradeOrderRequest::new("000001", 10.0, 100).unwrap(),
            now + chrono::Duration::minutes(2),
        )
        .await
        .unwrap();

    let state = store.snapshot().unwrap();
    let account = state.account.unwrap();
    assert!(!account.positions.contains_key("000001"));
}

#[tokio::test]
async fn sell_rejects_missing_positions() {
    let (service, _) = service();
    let now = fixed_ts();

    service
        .init_account(
            InitAccountRequest::new(None, None, None, None, None).unwrap(),
            now,
        )
        .await
        .unwrap();

    let err = service
        .sell(
            TradeOrderRequest::new("000001", 10.0, 100).unwrap(),
            now + chrono::Duration::minutes(1),
        )
        .await
        .unwrap_err();

    assert!(err.to_string().contains("未持有 000001"));
}

#[tokio::test]
async fn sell_rejects_insufficient_position_volume() {
    let (service, _) = service();
    let now = fixed_ts();

    service
        .init_account(
            InitAccountRequest::new(None, None, None, None, None).unwrap(),
            now,
        )
        .await
        .unwrap();
    service
        .buy(
            TradeOrderRequest::new("000001", 10.0, 100).unwrap(),
            now + chrono::Duration::minutes(1),
        )
        .await
        .unwrap();

    let err = service
        .sell(
            TradeOrderRequest::new("000001", 10.0, 101).unwrap(),
            now + chrono::Duration::minutes(2),
        )
        .await
        .unwrap_err();

    assert!(err.to_string().contains("可卖数量不足"));
}

#[tokio::test]
async fn cash_snapshot_uses_last_trade_price_to_compute_estimated_assets() {
    let (service, _) = service();
    let now = fixed_ts();

    service
        .init_account(
            InitAccountRequest::new(None, None, None, None, None).unwrap(),
            now,
        )
        .await
        .unwrap();
    service
        .buy(
            TradeOrderRequest::new("000001", 10.0, 100).unwrap(),
            now + chrono::Duration::minutes(1),
        )
        .await
        .unwrap();
    service
        .sell(
            TradeOrderRequest::new("000001", 12.0, 40).unwrap(),
            now + chrono::Duration::minutes(2),
        )
        .await
        .unwrap();

    let snapshot = service.cash_snapshot().await.unwrap();

    assert_eq!(snapshot.initial_capital, dec!(1000000));
    assert_eq!(snapshot.available_cash, dec!(999469.52));
    assert_eq!(snapshot.estimated_position_value, dec!(720));
    assert_eq!(snapshot.estimated_total_assets, dec!(1000189.52));
}

#[test]
fn invalid_capital_rate_price_and_volume_inputs_are_rejected() {
    let capital_err = InitAccountRequest::new(Some(0.0), None, None, None, None).unwrap_err();
    assert!(capital_err.to_string().contains("capital"));

    let rate_err = InitAccountRequest::new(Some(1000.0), Some(-0.1), None, None, None).unwrap_err();
    assert!(rate_err.to_string().contains("commission-rate"));

    let price_err = TradeOrderRequest::new("000001", f64::NAN, 100).unwrap_err();
    assert!(price_err.to_string().contains("price"));

    let volume_err = TradeOrderRequest::new("000001", 10.0, 0).unwrap_err();
    assert!(volume_err.to_string().contains("volume"));
}

#[test]
fn invalid_trade_code_is_rejected() {
    let short_err = TradeOrderRequest::new("12345", 10.0, 100).unwrap_err();
    assert!(short_err.to_string().contains("代码"));

    let non_digit_err = TradeOrderRequest::new("60000A", 10.0, 100).unwrap_err();
    assert!(non_digit_err.to_string().contains("代码"));
}

#[test]
fn fee_calculation_for_shanghai_buy_and_sell_applies_expected_fees() {
    let fee_config = FeeConfig::default();

    let buy = calculate_fee_breakdown(TradeSide::Buy, "600000", dec!(10000), &fee_config);
    assert_eq!(buy.commission, dec!(5));
    assert_eq!(buy.stamp_duty, dec!(0));
    assert_eq!(buy.transfer_fee, dec!(0.1));
    assert_eq!(buy.total_fee, dec!(5.1));

    let sell = calculate_fee_breakdown(TradeSide::Sell, "600000", dec!(10000), &fee_config);
    assert_eq!(sell.commission, dec!(5));
    assert_eq!(sell.stamp_duty, dec!(10));
    assert_eq!(sell.transfer_fee, dec!(0.1));
    assert_eq!(sell.total_fee, dec!(15.1));
}
