//! P0.8h — Wider OpenStock fixture → analysis/strategy loop.
//!
//! Consumes the read-only public API exercised by P0.8d
//! (`parse_daily_kline_json`) and fans the resulting `Vec<Kline>` through
//! the broader indicator and strategy surface area:
//!
//! - indicator fan-out: `sma`, `ema`, `wma`, `bollinger_bands`, `atr`,
//!   `obv`, `cci`, `williams_r`
//! - strategy drive: `MACrossStrategy::new` fed bar-by-bar via
//!   `Strategy::on_bar`, producing a `Signal` sequence
//!
//! This is a test-only slice (card P0.8h). No production Rust source
//! changes; no ClickHouse; no live OpenStock network.

use quantix_cli::analysis::{atr, bollinger_bands, cci, ema, obv, sma, williams_r, wma};
use quantix_cli::core::signal::Signal;
use quantix_cli::sources::parse_daily_kline_json;
use quantix_cli::strategy::ma_cross::MACrossStrategy;
use quantix_cli::strategy::trait_def::Strategy;

const DAILY_KLINE_30D_FIXTURE: &str = include_str!("fixtures/openstock/daily_kline_30d.json");

const EXPECTED_RECORD_COUNT: usize = 30;
const INDICATOR_PERIOD: usize = 5;

#[tokio::test]
async fn openstock_30d_fixture_feeds_indicator_and_strategy_loops() {
    // 1. Parse the 30-day fixture and assert structural shape.
    let klines = parse_daily_kline_json(DAILY_KLINE_30D_FIXTURE)
        .expect("30d fixture should parse via parse_daily_kline_json");
    assert_eq!(
        klines.len(),
        EXPECTED_RECORD_COUNT,
        "fixture should contain exactly 30 daily records"
    );
    for kline in &klines {
        assert_eq!(
            kline.code, "600000",
            "every fixture record must be for the single test symbol 600000"
        );
    }

    // 2. Fan the OHLCV series through every targeted indicator.
    let closes: Vec<_> = klines.iter().map(|k| k.close).collect();
    let highs: Vec<_> = klines.iter().map(|k| k.high).collect();
    let lows: Vec<_> = klines.iter().map(|k| k.low).collect();
    let volumes: Vec<_> = klines.iter().map(|k| k.volume).collect();

    // 2a. Pure-close indicators.
    let sma_values = sma(&closes, INDICATOR_PERIOD);
    let ema_values = ema(&closes, INDICATOR_PERIOD);
    let wma_values = wma(&closes, INDICATOR_PERIOD);
    let boll_values = bollinger_bands(&closes, INDICATOR_PERIOD, 2);

    assert_eq!(
        sma_values.len(),
        EXPECTED_RECORD_COUNT,
        "sma output length must match input length"
    );
    assert_eq!(
        ema_values.len(),
        EXPECTED_RECORD_COUNT,
        "ema output length must match input length"
    );
    assert_eq!(
        wma_values.len(),
        EXPECTED_RECORD_COUNT,
        "wma output length must match input length"
    );
    assert_eq!(
        boll_values.len(),
        EXPECTED_RECORD_COUNT,
        "bollinger_bands output length must match input length"
    );

    // Last-window outputs must be Some(...) for each pure-close indicator.
    assert!(
        sma_values[EXPECTED_RECORD_COUNT - 1].is_some(),
        "sma final window must be Some"
    );
    assert!(
        ema_values[EXPECTED_RECORD_COUNT - 1].is_some(),
        "ema final window must be Some"
    );
    assert!(
        wma_values[EXPECTED_RECORD_COUNT - 1].is_some(),
        "wma final window must be Some"
    );
    assert!(
        boll_values[EXPECTED_RECORD_COUNT - 1].is_some(),
        "bollinger_bands final window must be Some"
    );

    // 2b. High/low/close indicators.
    let atr_values = atr(&highs, &lows, &closes, INDICATOR_PERIOD);
    let cci_values = cci(&highs, &lows, &closes, INDICATOR_PERIOD);
    let williams_values = williams_r(&highs, &lows, &closes, INDICATOR_PERIOD);

    assert_eq!(
        atr_values.len(),
        EXPECTED_RECORD_COUNT,
        "atr output length must match input length"
    );
    assert_eq!(
        cci_values.len(),
        EXPECTED_RECORD_COUNT,
        "cci output length must match input length"
    );
    assert_eq!(
        williams_values.len(),
        EXPECTED_RECORD_COUNT,
        "williams_r output length must match input length"
    );
    assert!(
        atr_values[EXPECTED_RECORD_COUNT - 1].is_some(),
        "atr final window must be Some"
    );
    assert!(
        cci_values[EXPECTED_RECORD_COUNT - 1].is_some(),
        "cci final window must be Some"
    );
    assert!(
        williams_values[EXPECTED_RECORD_COUNT - 1].is_some(),
        "williams_r final window must be Some"
    );

    // 2c. OBV uses i64 cumulative volume.
    let obv_values = obv(&closes, &volumes);
    assert_eq!(
        obv_values.len(),
        EXPECTED_RECORD_COUNT,
        "obv output length must match input length"
    );
    assert!(
        obv_values[EXPECTED_RECORD_COUNT - 1].is_some(),
        "obv final slot must be Some"
    );

    // 3. Strategy drive: feed each Kline bar into MACrossStrategy(2, 5).
    let mut strategy = MACrossStrategy::new(2, 5);
    let mut signals = Vec::with_capacity(EXPECTED_RECORD_COUNT);
    for kline in &klines {
        let signal = strategy
            .on_bar(kline)
            .await
            .expect("MACrossStrategy::on_bar must not error on fixture bars");
        signals.push(signal);
    }
    assert_eq!(
        signals.len(),
        EXPECTED_RECORD_COUNT,
        "strategy must emit one signal per fixture bar"
    );

    let has_action_signal = signals
        .iter()
        .any(|signal| matches!(signal, Signal::Buy | Signal::Sell));
    assert!(
        has_action_signal,
        "MACrossStrategy(2, 5) must emit at least one Buy or Sell across the 30-bar fixture"
    );
}
