//! Live HTTP smoke test for `TRADE_DATES`. Gated by `QUANTIX_OPENSTOCK_LIVE=1`.

#![cfg(test)]

use quantix_cli::sources::openstock_client::OpenStockClient;

#[tokio::test]
#[ignore = "live OpenStock HTTP; set QUANTIX_OPENSTOCK_LIVE=1 to run"]
async fn fetch_trade_dates_live() {
    if std::env::var("QUANTIX_OPENSTOCK_LIVE").ok().as_deref() != Some("1") {
        eprintln!("skipping: QUANTIX_OPENSTOCK_LIVE not set");
        return;
    }
    let year: u32 = std::env::var("OPENSTOCK_LIVE_YEAR")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(2026);
    let client = OpenStockClient::from_env().expect("OPENSTOCK_BASE_URL + OPENSTOCK_API_KEY");
    let resp = client.fetch_trade_dates(year).await.expect("fetch ok");
    assert!(
        !resp.records.is_empty(),
        "TRADE_DATES should return records"
    );
    assert_eq!(resp.artifact_hash.len(), 64);
    println!(
        "TRADE_DATES live (year={}): {} records, source={}, latency_ms={:?}",
        year,
        resp.records.len(),
        resp.source,
        resp.latency_ms
    );
}
