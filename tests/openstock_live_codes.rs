//! Live HTTP smoke test for `STOCK_CODES`. Gated by `QUANTIX_OPENSTOCK_LIVE=1`.
//! CI skips (`#[ignore]`); run locally with `--ignored` when OpenStock is up.

#![cfg(test)]

use quantix_cli::sources::openstock_client::OpenStockClient;

#[tokio::test]
#[ignore = "live OpenStock HTTP; set QUANTIX_OPENSTOCK_LIVE=1 to run"]
async fn fetch_stock_codes_live() {
    if std::env::var("QUANTIX_OPENSTOCK_LIVE").ok().as_deref() != Some("1") {
        eprintln!("skipping: QUANTIX_OPENSTOCK_LIVE not set");
        return;
    }
    let client = OpenStockClient::from_env().expect("OPENSTOCK_BASE_URL + OPENSTOCK_API_KEY");
    let resp = client.fetch_stock_codes().await.expect("fetch ok");
    assert!(
        !resp.records.is_empty(),
        "STOCK_CODES should return records"
    );
    assert_eq!(resp.artifact_hash.len(), 64);
    println!(
        "STOCK_CODES live: {} records, source={}, latency_ms={:?}",
        resp.records.len(),
        resp.source,
        resp.latency_ms
    );
}
