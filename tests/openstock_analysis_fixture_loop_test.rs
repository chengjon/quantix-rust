use quantix_cli::analysis::sma;
use quantix_cli::sources::parse_daily_kline_json;
use rust_decimal_macros::dec;

const DAILY_KLINE_FIXTURE: &str = include_str!("fixtures/openstock/daily_kline.json");

#[test]
fn openstock_daily_fixture_feeds_existing_sma_indicator_path() {
    let klines = parse_daily_kline_json(DAILY_KLINE_FIXTURE).expect("fixture should parse");

    let closes: Vec<_> = klines.iter().map(|kline| kline.close).collect();
    let sma_values = sma(&closes, 2);

    assert_eq!(klines.len(), 2);
    assert_eq!(klines[0].code, "600000");
    assert_eq!(klines[0].date.to_string(), "2026-06-22");
    assert_eq!(klines[1].date.to_string(), "2026-06-23");
    assert_eq!(closes, vec![dec!(10.05), dec!(10.20)]);
    assert_eq!(sma_values, vec![None, Some(dec!(10.125))]);
}
