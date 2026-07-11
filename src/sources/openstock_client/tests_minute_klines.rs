//! Tests for `fetch_minute_klines` (wiremock-based, P0.13b-1 Task 2).

use super::test_support::fast_test_cfg;
use super::*;

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
