//! Integration tests for `openstock_client` (envelope composition only — no HTTP).

use quantix_cli::sources::openstock_client::OpenStockResponse;
use quantix_cli::sources::openstock_envelope::{OpenStockEnvelope, OpenStockErrorEnvelope};

#[derive(Debug, serde::Deserialize, PartialEq)]
struct Rec {
    code: String,
}

#[test]
fn from_envelope_composes_records_source_and_hash() {
    let raw = r#"{"data":[{"code":"600000"}],"source":"eltdx","received_at":"2026-06-30T10:00:00+08:00"}"#;
    let env: OpenStockEnvelope<Rec> = serde_json::from_str(raw).unwrap();
    let resp = OpenStockResponse::from_envelope(env, raw);
    assert_eq!(resp.records.len(), 1);
    assert_eq!(resp.source, "eltdx");
    assert_eq!(resp.artifact_hash.len(), 64);
    assert!(resp.received_at.is_some());
}

#[test]
fn error_envelope_parses_and_summarizes() {
    let raw = r#"{
        "code": "provider_unavailable",
        "message": "baostock offline",
        "request_id": "req-xyz"
    }"#;
    let env: OpenStockErrorEnvelope = serde_json::from_str(raw).unwrap();
    let summary = env.to_summary();
    assert!(summary.contains("provider_unavailable"));
    assert!(summary.contains("req-xyz"));
}

#[test]
fn from_envelope_artifact_hash_matches_shadow_helper() {
    // The public re-export path must agree with the canonical impl.
    let raw = r#"{"data":[{"code":"600000"}]}"#;
    let env: OpenStockEnvelope<Rec> = serde_json::from_str(raw).unwrap();
    let resp = OpenStockResponse::from_envelope(env, raw);
    assert_eq!(
        resp.artifact_hash,
        quantix_cli::sources::openstock_artifact_hash(raw)
    );
}
