//! Integration tests for `openstock_index` parser (fixture-driven).

use quantix_cli::sources::openstock_envelope::OpenStockEnvelope;
use quantix_cli::sources::openstock_index::{IndexKlineParseError, parse_index_klines};

const INDEX_FIXTURE: &str = include_str!("fixtures/openstock/index_klines.json");
const INDEX_EMPTY_FIXTURE: &str = include_str!("fixtures/openstock/index_klines_empty.json");

#[test]
fn parse_index_klines_fixture() {
    let env: OpenStockEnvelope<_> = serde_json::from_str(INDEX_FIXTURE).unwrap();
    let klines = parse_index_klines(env).unwrap();
    assert_eq!(klines.len(), 3);
    assert_eq!(klines[0].code, "000001");
    assert_eq!(klines[0].date.to_string(), "2026-01-02");
}

#[test]
fn parse_index_klines_empty_fixture_errors() {
    let env: OpenStockEnvelope<_> = serde_json::from_str(INDEX_EMPTY_FIXTURE).unwrap();
    match parse_index_klines(env) {
        Err(IndexKlineParseError::EmptyRecords) => {}
        other => panic!("expected EmptyRecords, got {:?}", other),
    }
}

#[test]
fn parse_index_klines_high_below_low_inline_errors() {
    let raw = r#"{
        "data":[{"symbol":"sh000001","time":"2026-01-02","open":"10","high":"5","low":"20","close":"15","volume":"1","amount":"1"}]
    }"#;
    let env: OpenStockEnvelope<_> = serde_json::from_str(raw).unwrap();
    match parse_index_klines(env) {
        Err(IndexKlineParseError::HighBelowLow { code, .. }) => {
            assert_eq!(code, "000001");
        }
        other => panic!("expected HighBelowLow, got {:?}", other),
    }
}
