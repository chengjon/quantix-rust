//! Tests for `fetch_minute_share` (P0.13b-2).

use super::test_support::fast_test_cfg;
use super::*;

#[tokio::test]
async fn fetch_minute_share_sends_minute_data_category_and_date() {
    use rust_decimal_macros::dec;
    use wiremock::matchers::{body_partial_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/data/fetch"))
        .and(body_partial_json(serde_json::json!({
            "data_category": "MINUTE_DATA",
            "params": { "code": "sh600000", "date": "2026-07-01" }
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "source": "eltdx",
            "artifact_hash": "abc123",
            "latency_ms": 42,
            "data": [
                {
                    "meta": { "trading_date": "2026-07-01" },
                    "points": [
                        { "time_minutes": null, "time": "09:30", "price": 10.50, "volume": 12300, "amount": 129150.0, "avg_price": 10.50, "index": 0, "price_milli": 10500 },
                        { "time_minutes": null, "time": "09:31", "price": 10.51, "volume": 8800, "amount": 92488.0, "avg_price": 10.505, "index": 1, "price_milli": 10510 }
                    ]
                }
            ]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = OpenStockClient::new(fast_test_cfg(server.uri())).expect("client build");
    let date = chrono::NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();
    let shares = client
        .fetch_minute_share("sh600000", crate::data::models::DateOrRange::Date(date))
        .await
        .expect("fetch ok");
    assert_eq!(shares.len(), 2);
    assert_eq!(shares[0].code, "sh600000");
    assert_eq!(shares[0].timestamp, date.and_hms_opt(9, 30, 0).unwrap());
    assert_eq!(shares[0].price, Some(dec!(10.50)));
    assert_eq!(shares[0].volume, Some(12300));
    assert_eq!(shares[1].timestamp, date.and_hms_opt(9, 31, 0).unwrap());
}

/// Regression test for BUG-B (OPENSTOCK_HANDOFF_2026-07-07.md): live
/// OpenStock returns `time_minutes: null` and populates `time: "HH:MM"`
/// instead. The parser must fall back to `time` rather than fail the
/// whole envelope deserialization.
#[tokio::test]
async fn fetch_minute_share_falls_back_to_time_when_time_minutes_null() {
    use rust_decimal_macros::dec;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/data/fetch"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "source": "eltdx",
            "data": [
                {
                    "meta": { "trading_date": "2026-07-03" },
                    "points": [
                        { "time_minutes": null, "time": "09:31", "price": 8.71, "volume": 100, "amount": 871.0, "avg_price": 8.71, "index": 0 },
                        { "time_minutes": null, "time": "09:32", "price": 8.72, "volume": 200, "amount": 1744.0, "avg_price": 8.715, "index": 1 }
                    ]
                }
            ]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = OpenStockClient::new(fast_test_cfg(server.uri())).expect("client build");
    let date = chrono::NaiveDate::from_ymd_opt(2026, 7, 3).unwrap();
    let shares = client
        .fetch_minute_share("sh600000", crate::data::models::DateOrRange::Date(date))
        .await
        .expect("fetch ok despite null time_minutes");
    assert_eq!(shares.len(), 2, "both records must parse via time fallback");
    assert_eq!(shares[0].timestamp, date.and_hms_opt(9, 31, 0).unwrap());
    assert_eq!(shares[1].timestamp, date.and_hms_opt(9, 32, 0).unwrap());
    assert_eq!(shares[0].price, Some(dec!(8.71)));
}

/// A record skips only when time is unparseable. Numeric fields are
/// independently optional — missing price/volume/amount/avg_price does
/// NOT skip (BUG-C, OPENSTOCK_HANDOFF_2026-07-07.md).
#[tokio::test]
async fn fetch_minute_share_skips_records_with_unparseable_time_only() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/data/fetch"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "source": "eltdx",
            "data": [
                {
                    "meta": { "trading_date": "2026-07-01" },
                    "points": [
                        // ok: all fields
                        { "time_minutes": "09:30", "price": 10.50, "volume": 100, "amount": 1050.0, "avg_price": 10.50 },
                        // ok: numeric fields partially missing — still kept
                        { "time_minutes": "09:31", "price": 10.51, "volume": 200 },
                        // skip: invalid time "99:99"
                        { "time_minutes": "99:99", "price": 10.53, "volume": 300, "amount": 3159.0, "avg_price": 10.53 },
                        // skip: time_minutes and time both absent
                        { "price": 10.54, "volume": 400, "amount": 4216.0, "avg_price": 10.54 },
                        // ok: time_minutes null + time "11:30" fallback (BUG-B)
                        { "time_minutes": null, "time": "11:30", "price": 10.55, "volume": 500 }
                    ]
                }
            ]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = OpenStockClient::new(fast_test_cfg(server.uri())).expect("client build");
    let date = chrono::NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();
    let shares = client
        .fetch_minute_share("sh600000", crate::data::models::DateOrRange::Date(date))
        .await
        .expect("fetch ok");
    assert_eq!(
        shares.len(),
        3,
        "expected 3 valid records (only time failures skip), got {:?}",
        shares
    );
    assert_eq!(shares[0].timestamp, date.and_hms_opt(9, 30, 0).unwrap());
    // record 1 had only price+volume — those should be Some, others None
    assert_eq!(shares[1].timestamp, date.and_hms_opt(9, 31, 0).unwrap());
    assert_eq!(shares[1].price, Some(rust_decimal_macros::dec!(10.51)));
    assert_eq!(shares[1].volume, Some(200));
    assert_eq!(shares[1].amount, None);
    assert_eq!(shares[1].avg_price, None);
    // record 4 used BUG-B time fallback
    assert_eq!(shares[2].timestamp, date.and_hms_opt(11, 30, 0).unwrap());
}

#[tokio::test]
async fn fetch_minute_share_propagates_4xx() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/data/fetch"))
        .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
            "error": { "code": "NOT_FOUND", "message": "unknown code" }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = OpenStockClient::new(fast_test_cfg(server.uri())).expect("client build");
    let date = chrono::NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();
    let err = client
        .fetch_minute_share("invalid_code", crate::data::models::DateOrRange::Date(date))
        .await
        .expect_err("must error");
    let msg = format!("{err}");
    assert!(
        msg.contains("404") || msg.contains("NOT_FOUND") || msg.contains("unknown"),
        "expected error to mention status/error, got: {msg}"
    );
}

#[tokio::test]
async fn fetch_minute_share_range_loops_per_day() {
    // W3: Range triggers N single-day requests, each yielding records
    // stamped with meta.trading_date (NOT request date — INV-2C).
    use crate::data::models::{DateOrRange, iter_dates_inclusive};
    use wiremock::matchers::{body_partial_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    // Single Mock with closure responder: each call inspects params.date
    // and returns a record stamped with the requested trading_date.
    // wiremock 0.6 supports closure responders.
    Mock::given(method("POST"))
        .and(path("/data/fetch"))
        .and(body_partial_json(
            serde_json::json!({ "data_category": "MINUTE_DATA" }),
        ))
        .respond_with(|request: &wiremock::Request| {
            let body: serde_json::Value = serde_json::from_slice(&request.body).unwrap_or_default();
            let req_date = body["params"]["date"].as_str().unwrap_or("");
            let resp = serde_json::json!({
                "status": "ok",
                "source": "eltdx",
                "artifact_hash": format!("hash-{}", req_date),
                "data": [{
                    "meta": { "trading_date": req_date },
                    "points": [{
                        "time_minutes": "0931",
                        "price": 10.0,
                        "volume": 100,
                        "amount": 1000.0,
                        "avg_price": 10.0
                    }]
                }]
            });
            ResponseTemplate::new(200).set_body_json(resp)
        })
        .expect(3)
        .mount(&server)
        .await;

    let cfg = fast_test_cfg(server.uri());
    let client = OpenStockClient::new(cfg).expect("client build");
    let start = chrono::NaiveDate::from_ymd_opt(2026, 6, 28).unwrap();
    let end = chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap();
    let shares = client
        .fetch_minute_share("sh600000", DateOrRange::Range { start, end })
        .await
        .expect("fetch ok");

    assert_eq!(shares.len(), 3, "one record per day × 3 days");
    let dates: Vec<chrono::NaiveDate> = shares.iter().map(|s| s.timestamp.date()).collect();
    // Verify all days present
    for d in iter_dates_inclusive(start, end) {
        assert!(dates.contains(&d), "expected day {} in results", d);
    }
    // Verify ascending order
    let mut sorted = dates.clone();
    sorted.sort();
    assert_eq!(dates, sorted, "results must be in ascending date order");
}

#[tokio::test]
async fn fetch_minute_share_range_skips_non_trading_days() {
    // W5: Range iterates all days client-side; non-trading days return
    // empty points arrays → no records contributed for that day.
    use crate::data::models::DateOrRange;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/data/fetch"))
        .respond_with(|request: &wiremock::Request| {
            let body: serde_json::Value = serde_json::from_slice(&request.body).unwrap_or_default();
            let req_date = body["params"]["date"].as_str().unwrap_or("");
            // For "2026-06-28" (Sunday) return empty points
            let points: Vec<serde_json::Value> = if req_date == "2026-06-28" {
                vec![]
            } else {
                vec![serde_json::json!({
                    "time_minutes": "1000",
                    "price": 10.0, "volume": 100,
                    "amount": 1000.0, "avg_price": 10.0,
                })]
            };
            let resp = serde_json::json!({
                "status": "ok",
                "source": "eltdx",
                "artifact_hash": "x",
                "data": [{
                    "meta": { "trading_date": req_date },
                    "points": points
                }]
            });
            ResponseTemplate::new(200).set_body_json(resp)
        })
        .expect(3)
        .mount(&server)
        .await;

    let cfg = fast_test_cfg(server.uri());
    let client = OpenStockClient::new(cfg).expect("client build");
    let start = chrono::NaiveDate::from_ymd_opt(2026, 6, 28).unwrap();
    let end = chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap();
    let shares = client
        .fetch_minute_share("sh600000", DateOrRange::Range { start, end })
        .await
        .expect("fetch ok");

    // Sunday returns empty, so only 2 trading days × 1 record = 2 records
    assert_eq!(shares.len(), 2, "non-trading day must contribute 0 records");
    let dates: Vec<chrono::NaiveDate> = shares.iter().map(|s| s.timestamp.date()).collect();
    assert!(
        !dates.contains(&start),
        "non-trading day must not appear in results"
    );
}
