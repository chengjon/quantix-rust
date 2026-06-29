//! P0.8g-impl integration tests against a live ClickHouse instance.
//!
//! Gated by `QUANTIX_SHADOW_INTEGRATION=1`. Skipped by default. To
//! run locally:
//!
//! ```bash
//! QUANTIX_SHADOW_INTEGRATION=1 \
//! QUANTIX_SHADOW_PERSIST_CONFIRM=yes \
//! cargo test --test openstock_shadow_persistence_integration_test -- --ignored
//! ```
//!
//! Prerequisite: `db/schema/quantix_shadow_init.sql` must have been
//! applied to the target ClickHouse instance.

use quantix_cli::core::runtime::ClickHouseSettings;
use quantix_cli::db::clickhouse::ClickHouseClient;
use quantix_cli::sources::openstock::{LiveShadowRequest, validate_live_shadow_payload};
use quantix_cli::sources::openstock_shadow::{
    new_batch_id, rollback_shadow_batch, verify_shadow_batch, write_shadow_klines,
};

const VALID_PAYLOAD: &str = r#"{"data":[
    {"symbol":"600000","time":"2026-06-01","open":"10.00","high":"10.20","low":"9.95","close":"10.10","volume":1000,"amount":"10100.00","period":"daily"},
    {"symbol":"600000","time":"2026-06-02","open":"10.10","high":"10.30","low":"10.05","close":"10.25","volume":1100,"amount":"11275.00","period":"daily"}
]}"#;

fn integration_enabled() -> bool {
    std::env::var("QUANTIX_SHADOW_INTEGRATION").ok().as_deref() == Some("1")
}

async fn client() -> ClickHouseClient {
    let settings = ClickHouseSettings::from_env();
    ClickHouseClient::from_settings(&settings)
        .await
        .expect("ClickHouse client construction")
}

#[tokio::test]
#[ignore = "requires QUANTIX_SHADOW_INTEGRATION=1 + live ClickHouse"]
async fn persist_live_apply_writes_rows() {
    if !integration_enabled() {
        eprintln!("skipped: QUANTIX_SHADOW_INTEGRATION not set");
        return;
    }
    let request = LiveShadowRequest {
        symbol: "600000".to_string(),
        period: "daily".to_string(),
        start_date: "2026-06-01".to_string(),
        end_date: "2026-06-02".to_string(),
        limit: Some(100),
    };
    let report = validate_live_shadow_payload(VALID_PAYLOAD, &request).unwrap();
    let client = client().await;
    let batch_id = new_batch_id();

    let written = write_shadow_klines(&client, &report, VALID_PAYLOAD, &batch_id, "ci", true, true)
        .await
        .expect("apply write");
    assert!(written.applied);
    assert_eq!(written.row_count, 2);

    let count = verify_shadow_batch(&client, &batch_id).await.unwrap();
    assert_eq!(count, 2);

    // Idempotent rollback.
    let removed = rollback_shadow_batch(&client, &batch_id).await.unwrap();
    let _ = removed;
    let after = verify_shadow_batch(&client, &batch_id).await.unwrap();
    assert_eq!(after, 0);
}

#[tokio::test]
#[ignore = "requires QUANTIX_SHADOW_INTEGRATION=1 + live ClickHouse"]
async fn rollback_removes_batch() {
    if !integration_enabled() {
        eprintln!("skipped: QUANTIX_SHADOW_INTEGRATION not set");
        return;
    }
    let request = LiveShadowRequest {
        symbol: "600000".to_string(),
        period: "daily".to_string(),
        start_date: "2026-06-01".to_string(),
        end_date: "2026-06-02".to_string(),
        limit: Some(100),
    };
    let report = validate_live_shadow_payload(VALID_PAYLOAD, &request).unwrap();
    let client = client().await;
    let batch_id = new_batch_id();

    write_shadow_klines(&client, &report, VALID_PAYLOAD, &batch_id, "ci", true, true)
        .await
        .unwrap();
    let before = verify_shadow_batch(&client, &batch_id).await.unwrap();
    assert_eq!(before, 2);

    let _ = rollback_shadow_batch(&client, &batch_id).await.unwrap();
    let _ = rollback_shadow_batch(&client, &batch_id).await.unwrap(); // idempotent
    let after = verify_shadow_batch(&client, &batch_id).await.unwrap();
    assert_eq!(after, 0);
}
