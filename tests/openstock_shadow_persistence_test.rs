//! P0.8g-impl default-CI tests for `openstock_shadow`.
//!
//! These tests never connect to ClickHouse. They cover the dry-run
//! gate logic and the double-gate opt-in semantics without any
//! `QUANTIX_SHADOW_INTEGRATION=1` requirement.

use quantix_cli::sources::openstock::{LiveShadowRequest, validate_live_shadow_payload};
use quantix_cli::sources::openstock_shadow::{
    ShadowWriteError, artifact_hash, build_shadow_rows_from_report, new_batch_id,
};

fn sample_payload() -> &'static str {
    r#"{"data":[
        {"symbol":"600000","time":"2026-06-01","open":"10.00","high":"10.20","low":"9.95","close":"10.10","volume":1000,"amount":"10100.00","period":"daily"},
        {"symbol":"600000","time":"2026-06-02","open":"10.10","high":"10.30","low":"10.05","close":"10.25","volume":1100,"amount":"11275.00","period":"daily"}
    ]}"#
}

fn request_with_limit(limit: Option<u32>) -> LiveShadowRequest {
    LiveShadowRequest {
        symbol: "600000".to_string(),
        period: "daily".to_string(),
        start_date: "2026-06-01".to_string(),
        end_date: "2026-06-02".to_string(),
        limit,
    }
}

#[test]
fn artifact_hash_is_deterministic_and_distinct() {
    let raw = sample_payload();
    let a = artifact_hash(raw);
    let b = artifact_hash(raw);
    assert_eq!(a, b);
    assert_eq!(a.len(), 64);
    let c = artifact_hash(&raw.replace("600000", "600001"));
    assert_ne!(a, c);
}

#[test]
fn build_rows_rejects_drift() {
    let report =
        validate_live_shadow_payload(sample_payload(), &request_with_limit(Some(1))).unwrap();
    let batch = new_batch_id();
    let err = build_shadow_rows_from_report(&report, sample_payload(), &batch, "ci")
        .expect_err("drift must be rejected");
    assert_eq!(err, ShadowWriteError::DriftNotEmpty { count: 1 });
}

#[test]
fn build_rows_rejects_fail_closed_payload() {
    // Missing time field on every record -> all records fail to map.
    let raw = r#"{"data":[
        {"symbol":"600000","open":"10.00","high":"10.20","low":"9.95","close":"10.10","volume":1000,"amount":"10100.00","period":"daily"}
    ]}"#;
    let report = validate_live_shadow_payload(raw, &request_with_limit(Some(100))).unwrap();
    let batch = new_batch_id();
    let err = build_shadow_rows_from_report(&report, raw, &batch, "ci")
        .expect_err("fail-closed payload must be rejected");
    assert_eq!(err, ShadowWriteError::FailClosedNotEmpty { count: 1 });
}

#[test]
fn build_rows_rejects_empty_mapped_payload() {
    use quantix_cli::sources::openstock::{LiveShadowReport, LiveShadowStatus};

    let report = LiveShadowReport {
        dry_run: true,
        source: "openstock_live_shadow",
        status: LiveShadowStatus::Ok,
        record_count: 0,
        mapped_count: 0,
        symbol: None,
        period: Some("daily".to_string()),
        received_date_range: None,
        drifts: Vec::new(),
        fail_closed_errors: Vec::new(),
        klines: Vec::new(),
    };
    let err = build_shadow_rows_from_report(&report, "raw", "batch", "ci")
        .expect_err("empty mapped payload must be rejected");
    assert_eq!(err, ShadowWriteError::EmptyPayload);
}

#[test]
fn build_rows_rejects_duplicate_keys() {
    use quantix_cli::data::models::{AdjustType, Kline};
    use quantix_cli::sources::openstock::{LiveShadowReport, LiveShadowStatus};
    use rust_decimal::Decimal;

    let dup = Kline {
        code: "600000".to_string(),
        date: chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
        open: Decimal::from(10),
        high: Decimal::from(10),
        low: Decimal::from(10),
        close: Decimal::from(10),
        volume: 1,
        amount: None,
        adjust_type: AdjustType::None,
    };
    let report = LiveShadowReport {
        dry_run: true,
        source: "openstock_live_shadow",
        status: LiveShadowStatus::Ok,
        record_count: 2,
        mapped_count: 2,
        symbol: Some("600000".to_string()),
        period: Some("daily".to_string()),
        received_date_range: None,
        drifts: Vec::new(),
        fail_closed_errors: Vec::new(),
        klines: vec![dup.clone(), dup],
    };
    let err = build_shadow_rows_from_report(&report, "raw", "batch", "ci")
        .expect_err("duplicate keys must be rejected");
    assert_eq!(err, ShadowWriteError::DuplicateKeys { count: 1 });
}
