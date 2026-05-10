use chrono::NaiveDate;
use quantix_cli::factor::{
    FactorCategory, FactorComputeRequest, FactorLoadRequest, FactorMeta, MissingPolicy,
};

#[test]
fn factor_core_types_have_first_slice_fields() {
    let meta = FactorMeta {
        id: "rank_close".to_string(),
        category: FactorCategory::Technical,
        description: "Cross-sectional rank of close by date".to_string(),
        author: Some("quantix".to_string()),
        source: Some("p1".to_string()),
        refresh_frequency: Some("daily".to_string()),
        required_fields: vec!["close".to_string()],
        missing_policy: MissingPolicy::KeepNull,
    };

    let load = FactorLoadRequest {
        symbols: vec!["000001.SZ".to_string()],
        start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        end: NaiveDate::from_ymd_opt(2026, 1, 10).unwrap(),
        required_fields: meta.required_fields.clone(),
    };

    let compute = FactorComputeRequest {
        factors: vec![meta.id.clone()],
        symbols: load.symbols.clone(),
        start: load.start,
        end: load.end,
        run_checks: true,
    };

    assert_eq!(compute.factors, vec!["rank_close"]);
    assert_eq!(load.required_fields, vec!["close"]);
}
