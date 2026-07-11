//! Tests for the klines family (fetch_index_klines, fetch_historical_klines,
//! fetch_tick_data, fetch_daily_klines, fetch_klines).

use super::test_support::fast_test_cfg;
use super::*;

// -----------------------------------------------------------------
// fetch_klines tests (wiremock-based, P0.13a Task 1.2)
// -----------------------------------------------------------------

#[tokio::test]
async fn fetch_klines_day_none_sends_period_day_and_omits_adjust() {
    use crate::data::models::{AdjustType, BarPeriod};
    use wiremock::matchers::{body_partial_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    let client = OpenStockClient::new(fast_test_cfg(server.uri())).expect("client build");

    let body = serde_json::json!({
        "data": [
            {
                "time": "2026-06-01T15:00:00+08:00",
                "open": 10.5,
                "high": 11.0,
                "low": 10.2,
                "close": 10.8,
                "volume": 1000000.0,
                "amount": 10800000.0,
            }
        ]
    });
    Mock::given(method("POST"))
        .and(path("/data/bars"))
        .and(body_partial_json(serde_json::json!({
            "symbol": "600000",
            "period": "day",
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&server)
        .await;

    let klines = client
        .fetch_klines("600000", BarPeriod::Day, AdjustType::None, None, None)
        .await
        .expect("fetch_klines ok");
    assert_eq!(klines.len(), 1);
    assert_eq!(klines[0].code, "600000");
    assert_eq!(klines[0].adjust_type, AdjustType::None);
}

#[tokio::test]
async fn fetch_klines_qfq_sends_adjust_qfq_and_stamps_records() {
    use crate::data::models::{AdjustType, BarPeriod};
    use wiremock::matchers::{body_partial_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    let client = OpenStockClient::new(fast_test_cfg(server.uri())).expect("client build");

    let body = serde_json::json!({
        "data": [
            {
                "time": "2026-06-02T15:00:00+08:00",
                "open": 5.0,
                "high": 5.5,
                "low": 4.9,
                "close": 5.2,
                "volume": 2000000.0,
                "amount": 10400000.0,
            }
        ]
    });
    Mock::given(method("POST"))
        .and(path("/data/bars"))
        .and(body_partial_json(serde_json::json!({
            "symbol": "000001",
            "period": "week",
            "adjust": "qfq",
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&server)
        .await;

    let klines = client
        .fetch_klines(
            "000001",
            BarPeriod::Week,
            AdjustType::QFQ,
            Some("2026-01-01"),
            Some("2026-06-30"),
        )
        .await
        .expect("fetch_klines ok");
    assert_eq!(klines.len(), 1);
    assert_eq!(klines[0].code, "000001");
    assert_eq!(klines[0].adjust_type, AdjustType::QFQ);
}

#[tokio::test]
async fn fetch_klines_propagates_4xx() {
    use crate::data::models::{AdjustType, BarPeriod};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    let client = OpenStockClient::new(fast_test_cfg(server.uri())).expect("client build");

    Mock::given(method("POST"))
        .and(path("/data/bars"))
        .respond_with(
            ResponseTemplate::new(400)
                .set_body_string(r#"{"code":"bad_request","message":"nope"}"#),
        )
        .expect(1) // no retry on 4xx — matches fetch_daily_klines
        .mount(&server)
        .await;

    let err = client
        .fetch_klines("600000", BarPeriod::Month, AdjustType::HFQ, None, None)
        .await
        .expect_err("should fail");
    let msg = format!("{:?}", err);
    assert!(msg.contains("/data/bars returned 400"), "msg={}", msg);
}
