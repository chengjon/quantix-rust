//! Live HTTP smoke test for `ALL_STOCKS`. Gated by `QUANTIX_OPENSTOCK_LIVE=1`.

#![cfg(test)]

use quantix_cli::sources::openstock_client::OpenStockClient;

#[tokio::test]
#[ignore = "live OpenStock HTTP; set QUANTIX_OPENSTOCK_LIVE=1 to run"]
async fn fetch_all_stocks_live() {
    if std::env::var("QUANTIX_OPENSTOCK_LIVE").ok().as_deref() != Some("1") {
        eprintln!("skipping: QUANTIX_OPENSTOCK_LIVE not set");
        return;
    }
    let day = std::env::var("OPENSTOCK_LIVE_DAY").ok();
    let client = OpenStockClient::from_env().expect("OPENSTOCK_BASE_URL + OPENSTOCK_API_KEY");
    let resp = client
        .fetch_all_stocks(day.as_deref())
        .await
        .expect("fetch ok");
    assert!(!resp.records.is_empty(), "ALL_STOCKS should return records");
    assert_eq!(resp.artifact_hash.len(), 64);
    println!(
        "ALL_STOCKS live (day={:?}): {} records, source={}, latency_ms={:?}",
        day,
        resp.records.len(),
        resp.source,
        resp.latency_ms
    );
}
