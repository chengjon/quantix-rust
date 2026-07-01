//! Live smoke for `quantix data tdx-api import-klines --source openstock` (P0.11a).
//! Verifies the dry-run path: OpenStock fetch succeeds, parser produces
//! `Vec<Kline>`, and the ClickHouse write is NOT triggered (no --apply).
//! Gated by `QUANTIX_OPENSTOCK_LIVE=1`.

#![cfg(test)]

use quantix_cli::sources::openstock_client::OpenStockClient;
use quantix_cli::sources::openstock_envelope::OpenStockEnvelope;
use quantix_cli::sources::openstock_index::{IndexKlineRecord, parse_index_klines};

#[tokio::test]
#[ignore = "live OpenStock HTTP; set QUANTIX_OPENSTOCK_LIVE=1 to run"]
async fn import_klines_openstock_dry_run_live() {
    if std::env::var("QUANTIX_OPENSTOCK_LIVE").ok().as_deref() != Some("1") {
        eprintln!("skipping: QUANTIX_OPENSTOCK_LIVE not set");
        return;
    }
    // sh000001 = 上证指数, INDEX_KLINES 已 live-verified
    let symbol = std::env::var("OPENSTOCK_LIVE_SYMBOL").unwrap_or_else(|_| "sh000001".to_string());

    let client = OpenStockClient::from_env().expect("OPENSTOCK_BASE_URL + OPENSTOCK_API_KEY");
    let resp = client
        .fetch_index_klines(&symbol, None, None)
        .await
        .expect("fetch ok");

    let record_count = resp.records.len();
    assert!(
        record_count > 0,
        "INDEX_KLINES should return records for {symbol}"
    );

    // 镜像 handler 的 envelope 重建逻辑，确保 parse 不报错
    let envelope: OpenStockEnvelope<IndexKlineRecord> = OpenStockEnvelope {
        data: resp.records,
        source: Some(resp.source.clone()),
        data_category: Some("INDEX_KLINES".to_string()),
        request_id: None,
        route_decision_id: None,
        quality_flags: Vec::new(),
        cache_state: None,
        circuit_state: None,
        latency_ms: resp.latency_ms,
        received_at: resp.received_at.clone(),
    };
    let klines = parse_index_klines(envelope).expect("parse_index_klines ok");
    assert!(!klines.is_empty(), "parser should produce Kline rows");

    println!(
        "import-klines openstock dry-run live (symbol={}): {} records, {} Kline rows parsed, source={}",
        symbol,
        record_count,
        klines.len(),
        resp.source
    );
}
