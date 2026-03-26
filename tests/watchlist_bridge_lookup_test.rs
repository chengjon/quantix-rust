use quantix_cli::bridge::client::BridgeHttpClient;
use quantix_cli::watchlist::{BridgeTdxWatchlistQuoteLookup, WatchlistQuoteLookup};
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn bridge_watchlist_lookup_preserves_raw_input_code_keys() {
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
    let lookup = BridgeTdxWatchlistQuoteLookup::new(client);

    let quotes = lookup.lookup_quotes(&["000001".to_string()]).await.unwrap();

    assert!(quotes.contains_key("000001"));
    assert_eq!(quotes["000001"].latest_price.to_string(), "15.5");
}
