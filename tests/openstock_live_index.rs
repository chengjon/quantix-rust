//! Live HTTP smoke test for `INDEX_KLINES`. Gated by `QUANTIX_OPENSTOCK_LIVE=1`.

#![cfg(test)]

use quantix_cli::sources::openstock_client::OpenStockClient;

#[tokio::test]
#[ignore = "live OpenStock HTTP; set QUANTIX_OPENSTOCK_LIVE=1 to run"]
async fn fetch_index_klines_live() {
    if std::env::var("QUANTIX_OPENSTOCK_LIVE").ok().as_deref() != Some("1") {
        eprintln!("skipping: QUANTIX_OPENSTOCK_LIVE not set");
        return;
    }
    let symbol = std::env::var("OPENSTOCK_LIVE_SYMBOL").unwrap_or_else(|_| "sh000001".to_string());
    let client = OpenStockClient::from_env().expect("OPENSTOCK_BASE_URL + OPENSTOCK_API_KEY");
    let resp = client
        .fetch_index_klines(&symbol, None, None)
        .await
        .expect("fetch ok");
    assert!(
        !resp.records.is_empty(),
        "INDEX_KLINES should return records"
    );
    assert_eq!(resp.artifact_hash.len(), 64);
    println!(
        "INDEX_KLINES live (symbol={}): {} records, source={}, latency_ms={:?}",
        symbol,
        resp.records.len(),
        resp.source,
        resp.latency_ms
    );
}

/// Date-range variant. Triggered by `OPENSTOCK_LIVE_START` + `OPENSTOCK_LIVE_END`
/// env vars (YYYY-MM-DD). Verifies the runtime honors `start_date`/`end_date`
/// and returns only K-lines whose date falls inside the requested window.
///
/// Regression for the bug fixed after live integration on 2026-07-01:
/// legacy `start`/`end` parameter names were silently ignored by the
/// baostock adapter, returning the full 500-bar history from 2015.
#[tokio::test]
#[ignore = "live OpenStock HTTP; set QUANTIX_OPENSTOCK_LIVE=1 + OPENSTOCK_LIVE_START/END to run"]
async fn fetch_index_klines_live_honors_date_range() {
    if std::env::var("QUANTIX_OPENSTOCK_LIVE").ok().as_deref() != Some("1") {
        eprintln!("skipping: QUANTIX_OPENSTOCK_LIVE not set");
        return;
    }
    let start = std::env::var("OPENSTOCK_LIVE_START").expect("OPENSTOCK_LIVE_START=YYYY-MM-DD");
    let end = std::env::var("OPENSTOCK_LIVE_END").expect("OPENSTOCK_LIVE_END=YYYY-MM-DD");
    let symbol = std::env::var("OPENSTOCK_LIVE_SYMBOL").unwrap_or_else(|_| "sh000001".to_string());

    let client = OpenStockClient::from_env().expect("OPENSTOCK_BASE_URL + OPENSTOCK_API_KEY");
    let resp = client
        .fetch_index_klines(&symbol, Some(&start), Some(&end))
        .await
        .expect("fetch ok");

    assert!(
        !resp.records.is_empty(),
        "INDEX_KLINES should return records for range {}..={}",
        start,
        end
    );

    // Every returned K-line must have date in [start, end]. If the runtime
    // ignores the date params (the legacy-bug shape), the first record's
    // date would be 2015-01-05 — far earlier than `start`.
    let out_of_range: Vec<_> = resp
        .records
        .iter()
        .filter(|k| {
            let d = k.time.as_deref().unwrap_or("");
            !(d >= start.as_str() && d <= end.as_str())
        })
        .collect();
    assert!(
        out_of_range.is_empty(),
        "INDEX_KLINES returned {} K-lines outside [{}, {}]. First offender: {:?} \
         — runtime may be ignoring start_date/end_date again",
        out_of_range.len(),
        start,
        end,
        out_of_range
            .first()
            .map(|k| k.time.as_deref().unwrap_or("?")),
    );

    println!(
        "INDEX_KLINES live (symbol={}, range={}..={}): {} records all within range, source={}",
        symbol,
        start,
        end,
        resp.records.len(),
        resp.source
    );
}
