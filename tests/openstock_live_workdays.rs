//! Live HTTP smoke test for `WORKDAYS`. Gated by `QUANTIX_OPENSTOCK_LIVE=1`.

#![cfg(test)]

use quantix_cli::sources::openstock_client::OpenStockClient;

#[tokio::test]
#[ignore = "live OpenStock HTTP; set QUANTIX_OPENSTOCK_LIVE=1 to run"]
async fn fetch_workdays_today_live() {
    if std::env::var("QUANTIX_OPENSTOCK_LIVE").ok().as_deref() != Some("1") {
        eprintln!("skipping: QUANTIX_OPENSTOCK_LIVE not set");
        return;
    }
    let action =
        std::env::var("OPENSTOCK_LIVE_WORKDAY_ACTION").unwrap_or_else(|_| "today".to_string());
    let date = std::env::var("OPENSTOCK_LIVE_WORKDAY_DATE").ok();
    let start = std::env::var("OPENSTOCK_LIVE_WORKDAY_START").ok();
    let end = std::env::var("OPENSTOCK_LIVE_WORKDAY_END").ok();
    let client = OpenStockClient::from_env().expect("OPENSTOCK_BASE_URL + OPENSTOCK_API_KEY");
    let resp = client
        .fetch_workdays(&action, date.as_deref(), start.as_deref(), end.as_deref())
        .await
        .expect("fetch ok");
    assert!(!resp.records.is_empty(), "WORKDAYS should return records");
    assert_eq!(resp.artifact_hash.len(), 64);
    println!(
        "WORKDAYS live (action={}): {} records, source={}, latency_ms={:?}",
        action,
        resp.records.len(),
        resp.source,
        resp.latency_ms
    );
}
