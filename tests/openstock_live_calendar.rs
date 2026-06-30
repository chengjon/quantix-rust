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
    // 优先 start/end；否则退到 year（向后兼容旧 OPENSTOCK_LIVE_YEAR env）
    let start = std::env::var("OPENSTOCK_LIVE_START").ok();
    let end = std::env::var("OPENSTOCK_LIVE_END").ok();
    let (effective_start, effective_end, hint): (Option<String>, Option<String>, String) =
        if start.is_some() || end.is_some() {
            (
                start.clone(),
                end.clone(),
                format!("start={:?}, end={:?}", start, end),
            )
        } else {
            let year: u32 = std::env::var("OPENSTOCK_LIVE_YEAR")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(2026);
            (
                Some(format!("{:04}-01-01", year)),
                Some(format!("{:04}-12-31", year)),
                format!("year={} → 展开 1-1..12-31", year),
            )
        };
    let client = OpenStockClient::from_env().expect("OPENSTOCK_BASE_URL + OPENSTOCK_API_KEY");
    let resp = client
        .fetch_trade_dates(effective_start.as_deref(), effective_end.as_deref())
        .await
        .expect("fetch ok");
    assert!(
        !resp.records.is_empty(),
        "TRADE_DATES should return records"
    );
    assert_eq!(resp.artifact_hash.len(), 64);
    println!(
        "TRADE_DATES live ({}): {} records, source={}, latency_ms={:?}",
        hint,
        resp.records.len(),
        resp.source,
        resp.latency_ms
    );
}
