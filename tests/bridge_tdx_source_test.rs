use chrono::NaiveDate;
use quantix_cli::bridge::client::BridgeHttpClient;
use quantix_cli::data::fetcher::Fetcher;
use quantix_cli::sources::bridge_tdx::BridgeTdxSource;
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn bridge_tdx_source_maps_batch_quotes_into_stock_quote() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/data/tdx/quotes"))
        .and(header("x-quantix-api-key", "bridge-test-key"))
        .and(body_json(serde_json::json!({
            "symbols": ["000001.SZ"]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "quotes": [
                {
                    "symbol": "000001.SZ",
                    "name": "平安银行",
                    "last": 15.5,
                    "bid": 15.49,
                    "ask": 15.51,
                    "open": 15.45,
                    "high": 15.6,
                    "low": 15.4,
                    "pre_close": 15.3,
                    "volume": 12345678,
                    "turnover": 191234567.89,
                    "timestamp": "2026-03-26T14:30:00Z",
                    "source": "tdx_bridge"
                }
            ]
        })))
        .mount(&server)
        .await;

    let client = BridgeHttpClient::new(server.uri(), Some("bridge-test-key".to_string())).unwrap();
    let source = BridgeTdxSource::new(client);

    let quotes = source.fetch_quotes_batch(&[(0, "000001")]).await.unwrap();

    assert_eq!(quotes.len(), 1);
    assert_eq!(quotes[0].code, "000001");
    assert_eq!(quotes[0].name, "平安银行");
    assert_eq!(quotes[0].market, 0);
    assert!((quotes[0].change_percent - 1.307).abs() < 0.01);
}

#[tokio::test]
async fn bridge_tdx_source_maps_kline_into_existing_model() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/data/tdx/kline/000001.SZ"))
        .and(header("x-quantix-api-key", "bridge-test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "symbol": "000001.SZ",
            "period": "1d",
            "bars": [
                {
                    "datetime": "2026-03-26",
                    "open": 14.8,
                    "high": 15.1,
                    "low": 14.75,
                    "close": 15.0,
                    "volume": 87654321,
                    "turnover": 1312345678.0
                }
            ],
            "source": "tdx_bridge"
        })))
        .mount(&server)
        .await;

    let client = BridgeHttpClient::new(server.uri(), Some("bridge-test-key".to_string())).unwrap();
    let source = BridgeTdxSource::new(client);

    let bars = source
        .get_kline(
            "000001",
            NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 3, 26).unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(bars.len(), 1);
    assert_eq!(bars[0].code, "000001");
    assert_eq!(bars[0].date, NaiveDate::from_ymd_opt(2026, 3, 26).unwrap());
}
