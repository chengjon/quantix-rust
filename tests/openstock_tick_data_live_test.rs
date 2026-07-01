//! Live smoke for `quantix data import-ticks` (OpenStock source).
//! Verifies the dry-run parse path: OpenStock TICK_DATA fetch succeeds,
//! `parse_tick_data` produces a non-empty `Vec<Tick>`, and TDengine is
//! NOT touched (no --apply).
//! Gated by `QUANTIX_OPENSTOCK_LIVE=1`.

#![cfg(test)]

use chrono::Datelike;
use quantix_cli::sources::openstock_client::OpenStockClient;
use quantix_cli::sources::openstock_envelope::OpenStockEnvelope;
use quantix_cli::sources::openstock_ticks::{TickEnvelopeRecord, parse_tick_data};

#[tokio::test]
#[ignore = "live OpenStock HTTP; set QUANTIX_OPENSTOCK_LIVE=1 to run"]
async fn import_ticks_openstock_dry_run_live() {
    if std::env::var("QUANTIX_OPENSTOCK_LIVE").ok().as_deref() != Some("1") {
        eprintln!("skipping: QUANTIX_OPENSTOCK_LIVE not set");
        return;
    }
    // 600000 = 浦发银行, TICK_DATA 已 live-Verified 2026-07-01
    let symbol =
        std::env::var("OPENSTOCK_LIVE_TICK_SYMBOL").unwrap_or_else(|_| "600000".to_string());
    let date = std::env::var("OPENSTOCK_LIVE_TICK_DATE").ok().or_else(|| {
        // 默认昨天 (live-verified 样本是 20260630)
        let today = chrono::Local::now().date_naive();
        if today.year() == 2026 && today.month() == 7 {
            Some(today.pred_opt().unwrap().format("%Y%m%d").to_string())
        } else {
            None
        }
    });

    let client = OpenStockClient::from_env().expect("OPENSTOCK_BASE_URL + OPENSTOCK_API_KEY");
    let resp = client
        .fetch_tick_data(&symbol, date.as_deref())
        .await
        .expect("fetch ok");

    let record_count = resp.records.len();
    assert!(
        record_count > 0,
        "TICK_DATA should return at least one envelope-record for {symbol}"
    );

    let envelope: OpenStockEnvelope<TickEnvelopeRecord> = OpenStockEnvelope {
        data: resp.records,
        source: Some(resp.source.clone()),
        data_category: Some("TICK_DATA".to_string()),
        request_id: None,
        route_decision_id: None,
        quality_flags: Vec::new(),
        cache_state: None,
        circuit_state: None,
        latency_ms: resp.latency_ms,
        received_at: resp.received_at.clone(),
    };
    let (meta, ticks) = parse_tick_data(envelope).expect("parse_tick_data ok");
    assert!(!ticks.is_empty(), "parser should produce Tick rows");

    println!(
        "import-ticks openstock dry-run live (symbol={}, date={:?}): {} envelope-records, {} ticks, source={}, trading_date={:?}",
        symbol,
        date,
        record_count,
        ticks.len(),
        resp.source,
        meta.trading_date
    );
}
