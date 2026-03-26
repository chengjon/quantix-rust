use chrono::{TimeZone, Utc};
use quantix_cli::trade::{
    FeeConfig, PaperTradeAccount, PaperTradeState, TradeOverview, TradePosition, TradeQuoteStatus,
    TradeRecord, TradeReportingService, TradeSide,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::BTreeMap;

fn fixed_ts(day: u32, hour: u32) -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, day, hour, 0, 0).unwrap()
}

fn sample_state() -> PaperTradeState {
    let opened_at = fixed_ts(10, 9);
    let updated_at = fixed_ts(12, 15);
    let mut positions = BTreeMap::new();
    positions.insert(
        "000001".to_string(),
        TradePosition {
            code: "000001".to_string(),
            volume: 1000,
            avg_cost: dec!(10.05),
            last_trade_price: dec!(10.8),
            opened_at,
            updated_at,
        },
    );
    positions.insert(
        "600000".to_string(),
        TradePosition {
            code: "600000".to_string(),
            volume: 500,
            avg_cost: dec!(20.10),
            last_trade_price: dec!(19.8),
            opened_at,
            updated_at,
        },
    );

    PaperTradeState {
        version: 1,
        account: Some(PaperTradeAccount {
            account_id: "default".to_string(),
            initial_capital: dec!(1000000),
            available_cash: dec!(979000),
            fee_config: FeeConfig::default(),
            positions,
            created_at: opened_at,
            updated_at,
        }),
        trade_records: vec![
            TradeRecord {
                id: "trade-1".to_string(),
                code: "000001".to_string(),
                side: TradeSide::Buy,
                price: dec!(10),
                volume: 1000,
                amount: dec!(10000),
                commission: dec!(5),
                stamp_duty: dec!(0),
                transfer_fee: dec!(0),
                total_fee: dec!(5),
                executed_at: fixed_ts(10, 10),
            },
            TradeRecord {
                id: "trade-2".to_string(),
                code: "600000".to_string(),
                side: TradeSide::Buy,
                price: dec!(20),
                volume: 500,
                amount: dec!(10000),
                commission: dec!(5),
                stamp_duty: dec!(0),
                transfer_fee: dec!(0.1),
                total_fee: dec!(5.1),
                executed_at: fixed_ts(11, 11),
            },
            TradeRecord {
                id: "trade-3".to_string(),
                code: "000001".to_string(),
                side: TradeSide::Sell,
                price: dec!(12),
                volume: 200,
                amount: dec!(2400),
                commission: dec!(5),
                stamp_duty: dec!(2.4),
                transfer_fee: dec!(0),
                total_fee: dec!(7.4),
                executed_at: fixed_ts(12, 12),
            },
        ],
    }
}

#[test]
fn history_rows_sort_newest_first_and_apply_filters() {
    let state = sample_state();
    let reporting = TradeReportingService::new();

    let rows = reporting.history_rows(&state, None, Some(2));
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].code, "000001");
    assert_eq!(rows[0].executed_at, fixed_ts(12, 12));
    assert_eq!(rows[0].net_cash_impact, dec!(2392.6));
    assert_eq!(rows[1].code, "600000");

    let filtered = reporting.history_rows(&state, Some("000001"), None);
    assert_eq!(filtered.len(), 2);
    assert!(filtered.iter().all(|row| row.code == "000001"));
}

#[test]
fn fee_rows_expose_fee_breakdown_and_filters() {
    let state = sample_state();
    let reporting = TradeReportingService::new();

    let rows = reporting.fee_rows(&state, Some("600000"), Some(10));
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].code, "600000");
    assert_eq!(rows[0].commission, dec!(5));
    assert_eq!(rows[0].stamp_duty, dec!(0));
    assert_eq!(rows[0].transfer_fee, dec!(0.1));
    assert_eq!(rows[0].total_fee, dec!(5.1));
}

#[test]
fn overview_aggregates_booked_totals() {
    let state = sample_state();
    let reporting = TradeReportingService::new();

    let overview = reporting.overview(&state);

    assert_eq!(
        overview,
        TradeOverview {
            initial_capital: dec!(1000000),
            available_cash: dec!(979000),
            booked_position_value: dec!(20700),
            booked_total_assets: dec!(999700),
            trade_count: 3,
            holding_count: 2,
            total_buy_amount: dec!(20000),
            total_sell_amount: dec!(2400),
            total_fee: dec!(17.5),
            live_position_value: None,
            live_total_assets: None,
            quote_coverage: None,
        }
    );
}

#[test]
fn position_rows_with_quotes_compute_unrealized_pnl() {
    let state = sample_state();
    let reporting = TradeReportingService::new();
    let quotes = BTreeMap::from([
        ("000001".to_string(), dec!(11.2)),
        ("600000".to_string(), dec!(19.5)),
    ]);

    let rows = reporting.position_rows_with_quotes(&state, &quotes);

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].code, "000001");
    assert_eq!(rows[0].current_price, Some(dec!(11.2)));
    assert_eq!(rows[0].current_market_value, Some(dec!(11200)));
    assert_eq!(rows[0].unrealized_pnl, Some(dec!(1150)));
    assert_eq!(rows[0].quote_status, TradeQuoteStatus::Live);

    assert_eq!(rows[1].code, "600000");
    assert_eq!(rows[1].current_price, Some(dec!(19.5)));
    assert_eq!(rows[1].current_market_value, Some(dec!(9750)));
    assert_eq!(rows[1].unrealized_pnl, Some(dec!(-300)));
}

#[test]
fn position_rows_without_quotes_degrade_to_missing_status() {
    let state = sample_state();
    let reporting = TradeReportingService::new();
    let quotes = BTreeMap::from([("000001".to_string(), dec!(11.2))]);

    let rows = reporting.position_rows_with_quotes(&state, &quotes);

    let missing = rows.iter().find(|row| row.code == "600000").unwrap();
    assert_eq!(missing.current_price, None);
    assert_eq!(missing.current_market_value, None);
    assert_eq!(missing.unrealized_pnl, None);
    assert_eq!(missing.quote_status, TradeQuoteStatus::Missing);
}

#[test]
fn empty_state_returns_empty_rows_and_zeroed_overview() {
    let reporting = TradeReportingService::new();
    let state = PaperTradeState::default();

    assert!(reporting.history_rows(&state, None, None).is_empty());
    assert!(reporting.fee_rows(&state, None, None).is_empty());
    assert!(reporting.position_rows(&state).is_empty());

    let overview = reporting.overview(&state);
    assert_eq!(overview.trade_count, 0);
    assert_eq!(overview.holding_count, 0);
    assert_eq!(overview.total_fee, Decimal::ZERO);
}

#[test]
fn reporting_keeps_multiple_partial_fills_as_separate_trade_rows() {
    let mut state = sample_state();
    state.trade_records = vec![
        TradeRecord {
            id: "mock-fill-1".to_string(),
            code: "000001".to_string(),
            side: TradeSide::Buy,
            price: dec!(10),
            volume: 50,
            amount: dec!(500),
            commission: dec!(5),
            stamp_duty: dec!(0),
            transfer_fee: dec!(0),
            total_fee: dec!(5),
            executed_at: fixed_ts(10, 10),
        },
        TradeRecord {
            id: "mock-fill-2".to_string(),
            code: "000001".to_string(),
            side: TradeSide::Buy,
            price: dec!(10.2),
            volume: 50,
            amount: dec!(510),
            commission: dec!(5),
            stamp_duty: dec!(0),
            transfer_fee: dec!(0),
            total_fee: dec!(5),
            executed_at: fixed_ts(10, 11),
        },
    ];

    let reporting = TradeReportingService::new();
    let history_rows = reporting.history_rows(&state, Some("000001"), None);
    assert_eq!(history_rows.len(), 2);
    assert_eq!(history_rows[0].executed_at, fixed_ts(10, 11));
    assert_eq!(history_rows[0].price, dec!(10.2));
    assert_eq!(history_rows[1].executed_at, fixed_ts(10, 10));
    assert_eq!(history_rows[1].price, dec!(10));

    let fee_rows = reporting.fee_rows(&state, Some("000001"), None);
    assert_eq!(fee_rows.len(), 2);

    let overview = reporting.overview(&state);
    assert_eq!(overview.trade_count, 2);
    assert_eq!(overview.total_buy_amount, dec!(1010));
    assert_eq!(overview.total_fee, dec!(10));
}
