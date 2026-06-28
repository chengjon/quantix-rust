use chrono::NaiveDate;
use quantix_cli::sources::openstock::{
    LiveShadowRequest, LiveShadowStatus, validate_live_shadow_payload,
};

fn request(symbol: &str, start: &str, end: &str, limit: Option<u32>) -> LiveShadowRequest {
    LiveShadowRequest {
        symbol: symbol.to_string(),
        period: "daily".to_string(),
        start_date: start.to_string(),
        end_date: end.to_string(),
        limit,
    }
}

fn payload(records: &[&str]) -> String {
    let body = records.join(",");
    format!(
        r#"{{"provider":"openstock","period":"daily","adjust_type":"none","records":[{body}]}}"#
    )
}

fn payload_owned(records: &[String]) -> String {
    let refs: Vec<&str> = records.iter().map(String::as_str).collect();
    payload(&refs)
}

const RECORD_600000_DAY1: &str = r#"{"symbol":"600000","time":"2026-06-22","open":"9.80","high":"10.15","low":"9.70","close":"10.05","volume":1234567,"amount":"12345678.90","period":"daily"}"#;
const RECORD_600000_DAY2: &str = r#"{"symbol":"600000","time":"2026-06-23","open":"10.05","high":"10.30","low":"9.95","close":"10.20","volume":2345678,"amount":"23456789.01","period":"daily"}"#;

#[test]
fn maps_live_payload_into_dry_run_report_without_drift() {
    let raw = payload(&[RECORD_600000_DAY1, RECORD_600000_DAY2]);
    let report =
        validate_live_shadow_payload(&raw, &request("600000", "2026-06-22", "2026-06-23", None))
            .expect("valid live payload should produce a report");

    assert_eq!(report.status, LiveShadowStatus::Ok);
    assert!(report.dry_run, "report must be marked dry_run");
    assert_eq!(report.source, "openstock_live_shadow");
    assert_eq!(report.record_count, 2);
    assert_eq!(report.mapped_count, 2);
    assert_eq!(report.symbol.as_deref(), Some("600000"));
    assert_eq!(report.period.as_deref(), Some("daily"));
    assert_eq!(
        report.received_date_range,
        Some((
            NaiveDate::from_ymd_opt(2026, 6, 22).unwrap(),
            NaiveDate::from_ymd_opt(2026, 6, 23).unwrap()
        ))
    );
    assert!(report.drifts.is_empty(), "no drift expected");
    assert!(
        report.fail_closed_errors.is_empty(),
        "no parse errors expected"
    );
}

#[test]
fn flags_drift_when_service_returns_more_records_than_limit() {
    let raw = payload(&[RECORD_600000_DAY1, RECORD_600000_DAY2]);
    let report = validate_live_shadow_payload(
        &raw,
        &request("600000", "2026-06-22", "2026-06-23", Some(1)),
    )
    .expect("drift detection should not hard-fail the report");

    assert_eq!(report.status, LiveShadowStatus::Drift);
    assert_eq!(report.record_count, 2);
    assert_eq!(report.drifts.len(), 1, "limit drift should be recorded");
}

#[test]
fn flags_drift_when_returned_range_falls_outside_requested_window() {
    let raw = payload(&[RECORD_600000_DAY1, RECORD_600000_DAY2]);
    let report =
        validate_live_shadow_payload(&raw, &request("600000", "2026-06-24", "2026-06-30", None))
            .expect("out-of-window drift should still produce a report");

    assert_eq!(report.status, LiveShadowStatus::Drift);
    assert!(
        report
            .drifts
            .iter()
            .any(|d| d.rule.contains("out_of_requested_window"))
    );
}

#[test]
fn fail_closes_on_missing_symbol_field() {
    let broken = r#"{"symbol":null,"time":"2026-06-22","open":"9.80","high":"10.15","low":"9.70","close":"10.05","volume":1234567,"period":"daily"}"#;
    let raw = payload(&[broken]);
    let report =
        validate_live_shadow_payload(&raw, &request("600000", "2026-06-22", "2026-06-23", None))
            .expect("fail-closed errors must not abort the report");

    assert_eq!(report.status, LiveShadowStatus::FailClosed);
    assert_eq!(report.record_count, 1);
    assert_eq!(report.mapped_count, 0);
    assert_eq!(report.fail_closed_errors.len(), 1);
}

#[test]
fn fail_closes_on_unparseable_time() {
    let broken = r#"{"symbol":"600000","time":"20260622","open":"9.80","high":"10.15","low":"9.70","close":"10.05","volume":1234567,"period":"daily"}"#;
    let raw = payload(&[broken]);
    let report =
        validate_live_shadow_payload(&raw, &request("600000", "2026-06-22", "2026-06-23", None))
            .expect("bad time should fail-closed, not abort");

    assert_eq!(report.status, LiveShadowStatus::FailClosed);
    assert_eq!(report.fail_closed_errors.len(), 1);
}

#[test]
fn fail_closes_on_non_daily_period_in_record() {
    let broken = r#"{"symbol":"600000","time":"2026-06-22","open":"9.80","high":"10.15","low":"9.70","close":"10.05","volume":1234567,"period":"minute"}"#;
    let raw = payload(&[broken]);
    let report =
        validate_live_shadow_payload(&raw, &request("600000", "2026-06-22", "2026-06-23", None))
            .expect("non-daily period should fail-closed");

    assert_eq!(report.status, LiveShadowStatus::FailClosed);
}

#[test]
fn fail_closes_when_record_symbol_mismatches_request() {
    let raw = payload_owned(&[
        RECORD_600000_DAY1.to_string(),
        RECORD_600000_DAY2.replace("600000", "000001"),
    ]);
    let report =
        validate_live_shadow_payload(&raw, &request("600000", "2026-06-22", "2026-06-23", None))
            .expect("mixed symbol should fail-closed");

    assert_eq!(report.status, LiveShadowStatus::FailClosed);
    assert!(
        report
            .fail_closed_errors
            .iter()
            .any(|e| e.to_string().contains("000001")),
        "expected the offending symbol in the error"
    );
}

#[test]
fn rejects_invalid_envelope_json() {
    let err = validate_live_shadow_payload(
        "not json",
        &request("600000", "2026-06-22", "2026-06-23", None),
    )
    .expect_err("invalid JSON envelope must be rejected");
    assert!(err.to_string().contains("invalid"));
}

#[test]
fn rejects_empty_records_envelope() {
    let raw = payload(&[]);
    let err =
        validate_live_shadow_payload(&raw, &request("600000", "2026-06-22", "2026-06-23", None))
            .expect_err("empty records envelope must be rejected");
    assert!(err.to_string().contains("no records"));
}

#[test]
fn report_implements_display_for_dry_run_log() {
    let raw = payload(&[RECORD_600000_DAY1, RECORD_600000_DAY2]);
    let report =
        validate_live_shadow_payload(&raw, &request("600000", "2026-06-22", "2026-06-23", None))
            .expect("valid payload");
    let rendered = format!("{report}");
    assert!(rendered.contains("OpenStock live shadow validation"));
    assert!(rendered.contains("dry_run: true"));
    assert!(rendered.contains("symbol: 600000"));
    assert!(rendered.contains("status: ok"));
}

// ============================================================
// Real-envelope shape (observed 2026-06-28 against NAS openstock service)
// Top-level: {"data": [...], "source": ..., "data_category": ...}
// Record:    symbol=`sh600000`, time=`2026-01-23T15:00:00+08:00`, period=`day`
// ============================================================

fn real_envelope(records: &[&str]) -> String {
    let body = records.join(",");
    format!(r#"{{"data":[{body}],"source":"eltdx","data_category":"KLINES"}}"#)
}

const REAL_RECORD_DAY1: &str = r#"{"symbol":"sh600000","time":"2026-06-22T15:00:00+08:00","open":10.5,"high":10.66,"low":10.47,"close":10.51,"volume":132324400,"amount":1394174208.0,"period":"day"}"#;
const REAL_RECORD_DAY2: &str = r#"{"symbol":"sh600000","time":"2026-06-23T15:00:00+08:00","open":10.49,"high":10.53,"low":10.34,"close":10.35,"volume":149077100,"amount":1552120320.0,"period":"day"}"#;

#[test]
fn accepts_real_envelope_with_data_field_and_normalizes_symbol_prefix() {
    let raw = real_envelope(&[REAL_RECORD_DAY1, REAL_RECORD_DAY2]);
    let report =
        validate_live_shadow_payload(&raw, &request("600000", "2026-06-22", "2026-06-23", None))
            .expect("real envelope should parse");

    assert_eq!(report.status, LiveShadowStatus::Ok);
    assert_eq!(report.record_count, 2);
    assert_eq!(report.mapped_count, 2);
    assert_eq!(report.symbol.as_deref(), Some("600000"));
    assert_eq!(report.period.as_deref(), Some("day"));
    assert_eq!(
        report.received_date_range,
        Some((
            NaiveDate::from_ymd_opt(2026, 6, 22).unwrap(),
            NaiveDate::from_ymd_opt(2026, 6, 23).unwrap()
        ))
    );
}

#[test]
fn accepts_request_symbol_with_exchange_prefix_against_bare_record_symbol() {
    let raw = real_envelope(&[REAL_RECORD_DAY1, REAL_RECORD_DAY2]);
    let req = request("sh600000", "2026-06-22", "2026-06-23", None);
    let report = validate_live_shadow_payload(&raw, &req).expect("prefix-form request ok");
    assert_eq!(report.status, LiveShadowStatus::Ok);
    assert_eq!(report.symbol.as_deref(), Some("600000"));
}

#[test]
fn fail_closes_when_record_symbol_resolves_to_different_bare_code() {
    let mismatch = r#"{"symbol":"sh000001","time":"2026-06-22T15:00:00+08:00","open":10.5,"high":10.66,"low":10.47,"close":10.51,"volume":132324400,"period":"day"}"#;
    let raw = real_envelope(&[mismatch]);
    let report =
        validate_live_shadow_payload(&raw, &request("600000", "2026-06-22", "2026-06-23", None))
            .expect("mismatch should fail-closed");
    assert_eq!(report.status, LiveShadowStatus::FailClosed);
}

#[test]
fn rejects_minute_period_in_real_envelope() {
    let broken = r#"{"symbol":"sh600000","time":"2026-06-22T15:00:00+08:00","open":10.5,"high":10.66,"low":10.47,"close":10.51,"volume":132324400,"period":"minute"}"#;
    let raw = real_envelope(&[broken]);
    let report =
        validate_live_shadow_payload(&raw, &request("600000", "2026-06-22", "2026-06-23", None))
            .expect("non-daily period should fail-closed");
    assert_eq!(report.status, LiveShadowStatus::FailClosed);
}

#[test]
fn rejects_invalid_rfc3339_time() {
    let broken = r#"{"symbol":"sh600000","time":"2026-06-22 15:00","open":10.5,"high":10.66,"low":10.47,"close":10.51,"volume":132324400,"period":"day"}"#;
    let raw = real_envelope(&[broken]);
    let report =
        validate_live_shadow_payload(&raw, &request("600000", "2026-06-22", "2026-06-23", None))
            .expect("bad time should fail-closed");
    assert_eq!(report.status, LiveShadowStatus::FailClosed);
    assert!(
        report
            .fail_closed_errors
            .iter()
            .any(|e| e.to_string().contains("RFC3339"))
    );
}
