//! Stream API unit tests for minute-data family (P0.13d Task 4 + Task 5).
//!
//! Covers INV-1A equivalence at the wire level, INV-5A error termination,
//! INV-5B empty-batch emission for non-trading days, and INV-2A/INV-2B
//! per-chunk wire shape for both klines and share streams.

use super::test_support::fast_test_cfg;
use super::*;

// -----------------------------------------------------------------
// Chunk-count + termination semantics (P0.13d Task 4: INV-1A / INV-5A / INV-5B)
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
