//! Tests for the minute-data family (fetch_minute_klines,
//! fetch_minute_share, and stream variants).

use super::test_support::fast_test_cfg;
use super::*;

// -----------------------------------------------------------------
// fetch_minute_klines tests (wiremock-based, P0.13b-1 Task 2)
// -----------------------------------------------------------------

#[tokio::test]
async fn fetch_minute_klines_1m_none_sends_period_1m_and_date() {
    use crate::data::models::{AdjustType, MinutePeriod};
    use wiremock::matchers::{body_partial_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    let client = OpenStockClient::new(fast_test_cfg(server.uri())).expect("client build");

    let body = serde_json::json!({
        "data": [
            {"time": "2026-07-02T09:31:00+08:00", "open": 10.0, "high": 10.5, "low": 9.9, "close": 10.2, "volume": 1000.0, "amount": 10200.0},
            {"time": "2026-07-02T09:32:00+08:00", "open": 10.2, "high": 10.4, "low": 10.1, "close": 10.3, "volume": 800.0, "amount": 8240.0},
        ]
    });
    Mock::given(method("POST"))
        .and(path("/data/bars"))
        .and(body_partial_json(serde_json::json!({
            "symbol": "sh600000",
            "period": "1m",
            "date": "2026-07-02"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .expect(1)
        .mount(&server)
        .await;

    let date = chrono::NaiveDate::from_ymd_opt(2026, 7, 2).unwrap();
    let bars = client
        .fetch_minute_klines(
            "sh600000",
            MinutePeriod::Minute1,
            crate::data::models::DateOrRange::Date(date),
            AdjustType::None,
        )
        .await
        .expect("fetch_minute_klines ok");

    assert_eq!(bars.len(), 2);
    assert_eq!(bars[0].code, "sh600000");
    assert_eq!(
        bars[0].timestamp,
        chrono::NaiveDateTime::parse_from_str("2026-07-02T09:31:00", "%Y-%m-%dT%H:%M:%S").unwrap()
    );
    assert_eq!(bars[0].adjust_type, AdjustType::None);
    assert_eq!(bars[1].volume, 800);

    // W2: Date path wire body must NOT contain range fields (INV-2A backward compat).
    let received = server.received_requests().await.expect("at least one");
    let req_body: serde_json::Value =
        serde_json::from_slice(&received[0].body).expect("body is json");
    assert!(
        req_body.get("start_date").is_none(),
        "Date body must not include start_date, got: {:?}",
        req_body
    );
    assert!(
        req_body.get("end_date").is_none(),
        "Date body must not include end_date, got: {:?}",
        req_body
    );
}

#[tokio::test]
async fn fetch_minute_klines_5m_qfq_sends_adjust_and_stamps_records() {
    use crate::data::models::{AdjustType, MinutePeriod};
    use wiremock::matchers::{body_partial_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    let client = OpenStockClient::new(fast_test_cfg(server.uri())).expect("client build");

    Mock::given(method("POST"))
        .and(path("/data/bars"))
        .and(body_partial_json(serde_json::json!({
            "symbol": "sh600000",
            "period": "5m",
            "date": "2026-07-02",
            "adjust": "qfq"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {"time": "2026-07-02T09:35:00+08:00", "open": 11.0, "high": 11.2, "low": 10.9, "close": 11.1, "volume": 500.0, "amount": 5550.0}
            ]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let date = chrono::NaiveDate::from_ymd_opt(2026, 7, 2).unwrap();
    let bars = client
        .fetch_minute_klines(
            "sh600000",
            MinutePeriod::Minute5,
            crate::data::models::DateOrRange::Date(date),
            AdjustType::QFQ,
        )
        .await
        .expect("fetch_minute_klines ok");

    assert_eq!(bars.len(), 1);
    assert_eq!(bars[0].adjust_type, AdjustType::QFQ);
}

#[tokio::test]
async fn fetch_minute_klines_propagates_4xx() {
    use crate::data::models::{AdjustType, MinutePeriod};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    let client = OpenStockClient::new(fast_test_cfg(server.uri())).expect("client build");

    Mock::given(method("POST"))
        .and(path("/data/bars"))
        .respond_with(ResponseTemplate::new(400).set_body_string("bad period"))
        .expect(1) // no retry on 4xx — matches fetch_klines
        .mount(&server)
        .await;

    let date = chrono::NaiveDate::from_ymd_opt(2026, 7, 2).unwrap();
    let result = client
        .fetch_minute_klines(
            "sh600000",
            MinutePeriod::Minute15,
            crate::data::models::DateOrRange::Date(date),
            AdjustType::None,
        )
        .await;

    let err = result.expect_err("expected error on 400");
    let msg = format!("{:?}", err);
    assert!(
        msg.contains("/data/bars returned 400"),
        "expected '/data/bars returned 400' in error, got: {}",
        msg
    );
}

#[tokio::test]
async fn fetch_minute_klines_range_sends_start_date_end_date_body() {
    // W1: Range mode sends start_date + end_date (NOT date) — spec §3.2 row, §6 D4.
    use crate::data::models::{AdjustType, DateOrRange, MinutePeriod};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    let body = serde_json::json!({
        "data": [
            {"time": "2026-06-01T09:31:00+08:00", "open": 10.0, "high": 10.5, "low": 9.9, "close": 10.2, "volume": 1000.0, "amount": 10200.0},
            {"time": "2026-06-30T15:00:00+08:00", "open": 11.0, "high": 11.2, "low": 10.8, "close": 11.1, "volume": 500.0, "amount": 5550.0}
        ]
    });
    Mock::given(method("POST"))
        .and(path("/data/bars"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .expect(1)
        .mount(&server)
        .await;

    let cfg = fast_test_cfg(server.uri());
    let client = OpenStockClient::new(cfg).expect("client build");
    let start = chrono::NaiveDate::parse_from_str("2026-06-01", "%Y-%m-%d").unwrap();
    let end = chrono::NaiveDate::parse_from_str("2026-06-30", "%Y-%m-%d").unwrap();
    let bars = client
        .fetch_minute_klines(
            "sh600000",
            MinutePeriod::Minute1,
            DateOrRange::Range { start, end },
            AdjustType::None,
        )
        .await
        .expect("fetch ok");

    assert_eq!(bars.len(), 2);

    let received = server.received_requests().await.expect("at least one");
    assert_eq!(received.len(), 1);
    let req_body: serde_json::Value =
        serde_json::from_slice(&received[0].body).expect("body is json");
    assert_eq!(req_body["start_date"], "2026-06-01");
    assert_eq!(req_body["end_date"], "2026-06-30");
    assert!(
        req_body.get("date").is_none(),
        "Range body must not include 'date', got: {:?}",
        req_body
    );
    assert_eq!(req_body["symbol"], "sh600000");
    assert_eq!(req_body["period"], "1m");
}

// -----------------------------------------------------------------
// fetch_minute_share (P0.13b-2)
// -----------------------------------------------------------------

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
// -----------------------------------------------------------------
// Stream API unit tests (P0.13d Task 4: INV-1A / INV-5A / INV-5B)
// -----------------------------------------------------------------

#[tokio::test]
async fn fetch_minute_klines_stream_yields_expected_chunk_count() {
    // S5: stream yields one Vec per chunk with expected record count.
    // INV-1A equivalence is enforced by L1 live test only.
    use crate::data::models::{AdjustType, DateOrRange, MinutePeriod};
    use chrono::NaiveDate;
    use futures::StreamExt;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    let client = OpenStockClient::new(fast_test_cfg(server.uri())).expect("client build");

    Mock::given(method("POST"))
        .and(path("/data/bars"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {"time":"2026-06-01T09:31:00+08:00","open":1.0,"high":2.0,"low":0.5,"close":1.5,"volume":100.0,"amount":150.0},
                {"time":"2026-06-01T09:32:00+08:00","open":1.5,"high":2.5,"low":1.0,"close":2.0,"volume":200.0,"amount":400.0},
            ]
        })))
        .expect(2) // 14 days / 7 = 2 chunks
        .mount(&server)
        .await;

    let start = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2026, 6, 14).unwrap();
    let s = client.fetch_minute_klines_stream(
        "sh600000",
        MinutePeriod::Minute1,
        DateOrRange::Range { start, end },
        AdjustType::None,
    );
    futures::pin_mut!(s);

    let mut total = 0usize;
    while let Some(batch) = s.next().await {
        let batch = batch.expect("batch ok");
        total += batch.len();
    }
    assert_eq!(total, 4, "2 chunks × 2 records = 4");
}

#[tokio::test]
async fn fetch_minute_klines_stream_terminates_on_first_batch_error() {
    // S6 / INV-5A: first Err yields, subsequent next() returns None.
    // Stream must not retry / advance to the next chunk after an Err.
    use crate::data::models::{AdjustType, DateOrRange, MinutePeriod};
    use chrono::NaiveDate;
    use futures::StreamExt;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    let client = OpenStockClient::new(fast_test_cfg(server.uri())).expect("client build");

    // 14-day range => 2 chunks; mock fails on every call but we expect
    // only the first chunk's call to fire (stream short-circuits).
    Mock::given(method("POST"))
        .and(path("/data/bars"))
        .respond_with(ResponseTemplate::new(500).set_body_string("simulated server error"))
        .expect(1)
        .mount(&server)
        .await;

    let start = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2026, 6, 14).unwrap();
    let s = client.fetch_minute_klines_stream(
        "sh600000",
        MinutePeriod::Minute1,
        DateOrRange::Range { start, end },
        AdjustType::None,
    );
    futures::pin_mut!(s);

    let first = s.next().await.expect("first item exists");
    assert!(first.is_err(), "first batch must be Err");

    // Stream must terminate after the error (no second chunk polled).
    let next = s.next().await;
    assert!(next.is_none(), "stream must return None after first Err");
}

#[tokio::test]
async fn fetch_minute_share_stream_yields_empty_vec_for_non_trading_days() {
    // S7 / INV-5B: server returns no records for non-trading days; stream
    // still yields an empty Vec for each day (not skipped).
    // batch count == calendar day count.
    use crate::data::models::DateOrRange;
    use chrono::NaiveDate;
    use futures::StreamExt;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    let client = OpenStockClient::new(fast_test_cfg(server.uri())).expect("client build");

    // Every MINUTE_DATA request returns empty points array.
    Mock::given(method("POST"))
        .and(path("/data/fetch"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {"meta": {"trading_date": "2026-06-01"}, "points": []}
            ]
        })))
        .expect(3) // one per calendar day
        .mount(&server)
        .await;

    let start = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2026, 6, 3).unwrap();
    let s = client.fetch_minute_share_stream("sh600000", DateOrRange::Range { start, end });
    futures::pin_mut!(s);

    let mut batch_count = 0usize;
    let mut total_records = 0usize;
    while let Some(batch) = s.next().await {
        let batch = batch.expect("batch ok");
        batch_count += 1;
        total_records += batch.len();
    }
    assert_eq!(batch_count, 3, "INV-5B: one batch per calendar day");
    assert_eq!(total_records, 0, "no records for non-trading days");
}

// -----------------------------------------------------------------
// Stream API wiremock tests (P0.13d Task 5: INV-2A / INV-2B wire shape)
// -----------------------------------------------------------------

#[tokio::test]
async fn fetch_minute_klines_stream_emits_per_chunk_subrange_body() {
    // W1 / INV-2A: each chunk request body uses start_date/end_date of that
    // chunk (no `date` field) for a multi-week Range input.
    use crate::data::models::{AdjustType, DateOrRange, MinutePeriod};
    use chrono::NaiveDate;
    use futures::StreamExt;
    use wiremock::matchers::{body_partial_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    let client = OpenStockClient::new(fast_test_cfg(server.uri())).expect("client build");

    // 14-day range => chunk 1: 06-01..=06-07, chunk 2: 06-08..=06-14
    let start = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2026, 6, 14).unwrap();

    // Chunk 1 body assertion
    Mock::given(method("POST"))
        .and(path("/data/bars"))
        .and(body_partial_json(serde_json::json!({
            "start_date": "2026-06-01",
            "end_date": "2026-06-07",
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": []})))
        .expect(1)
        .mount(&server)
        .await;

    // Chunk 2 body assertion
    Mock::given(method("POST"))
        .and(path("/data/bars"))
        .and(body_partial_json(serde_json::json!({
            "start_date": "2026-06-08",
            "end_date": "2026-06-14",
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": []})))
        .expect(1)
        .mount(&server)
        .await;

    let s = client.fetch_minute_klines_stream(
        "sh600000",
        MinutePeriod::Minute1,
        DateOrRange::Range { start, end },
        AdjustType::None,
    );
    futures::pin_mut!(s);
    while let Some(b) = s.next().await {
        b.expect("ok");
    }
}

#[tokio::test]
async fn fetch_minute_share_stream_emits_one_request_per_calendar_day() {
    // W2 / INV-2B: each calendar day emits one /data/fetch MINUTE_DATA request.
    use crate::data::models::DateOrRange;
    use chrono::NaiveDate;
    use futures::StreamExt;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    let client = OpenStockClient::new(fast_test_cfg(server.uri())).expect("client build");

    // 5-day range
    let start = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2026, 6, 5).unwrap();

    Mock::given(method("POST"))
        .and(path("/data/fetch"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"meta": {"trading_date": "2026-06-01"}, "points": []}]
        })))
        .expect(5) // one per calendar day
        .mount(&server)
        .await;

    let s = client.fetch_minute_share_stream("sh600000", DateOrRange::Range { start, end });
    futures::pin_mut!(s);
    while let Some(b) = s.next().await {
        b.expect("ok");
    }
}

#[tokio::test]
async fn fetch_minute_klines_stream_date_mode_emits_single_batch_with_date_field() {
    // W3 / INV-2A Date path: Date(d) -> 1 chunk (d,d) -> body has `date` only.
    use crate::data::models::{AdjustType, DateOrRange, MinutePeriod};
    use chrono::NaiveDate;
    use futures::StreamExt;
    use wiremock::matchers::{body_partial_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    let client = OpenStockClient::new(fast_test_cfg(server.uri())).expect("client build");

    let d = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();

    Mock::given(method("POST"))
        .and(path("/data/bars"))
        .and(body_partial_json(serde_json::json!({"date": "2026-06-01"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": []})))
        .expect(1)
        .mount(&server)
        .await;

    let s = client.fetch_minute_klines_stream(
        "sh600000",
        MinutePeriod::Minute1,
        DateOrRange::Date(d),
        AdjustType::None,
    );
    futures::pin_mut!(s);

    let mut batches = 0usize;
    while let Some(b) = s.next().await {
        b.expect("ok");
        batches += 1;
    }
    assert_eq!(batches, 1, "Date(d) must produce exactly 1 batch");
}
