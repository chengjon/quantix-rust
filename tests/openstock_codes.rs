//! Integration tests for `openstock_codes` parsers (fixture-driven).

use quantix_cli::sources::openstock_codes::{
    StockCodeParseError, parse_all_stocks, parse_stock_codes,
};
use quantix_cli::sources::openstock_envelope::OpenStockEnvelope;

const CODES_FIXTURE: &str = include_str!("fixtures/openstock/codes.json");
const CODES_EMPTY_FIXTURE: &str = include_str!("fixtures/openstock/codes_empty.json");
const ALL_STOCKS_FIXTURE: &str = include_str!("fixtures/openstock/all_stocks.json");

#[test]
fn parse_stock_codes_fixture() {
    let env: OpenStockEnvelope<_> = serde_json::from_str(CODES_FIXTURE).unwrap();
    let codes = parse_stock_codes(env).unwrap();
    assert_eq!(codes.len(), 3);
    assert_eq!(codes[0].code, "600000");
    assert_eq!(codes[0].name.as_deref(), Some("浦发银行"));
    assert_eq!(codes[2].code, "000002");
}

#[test]
fn parse_stock_codes_empty_fixture_errors() {
    let env: OpenStockEnvelope<_> = serde_json::from_str(CODES_EMPTY_FIXTURE).unwrap();
    assert_eq!(
        parse_stock_codes(env),
        Err(StockCodeParseError::EmptyRecords)
    );
}

#[test]
fn parse_all_stocks_fixture() {
    let env: OpenStockEnvelope<_> = serde_json::from_str(ALL_STOCKS_FIXTURE).unwrap();
    let entries = parse_all_stocks(env).unwrap();
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].market.as_deref(), Some("sh"));
    assert!(entries[0].listing_date.is_some());
    assert_eq!(entries[2].code, "688981");
}
