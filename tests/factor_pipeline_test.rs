use async_trait::async_trait;
use chrono::NaiveDate;
use polars::prelude::*;
use quantix_cli::core::Result;
use quantix_cli::factor::{
    FactorCategory, FactorComputeRequest, FactorDataLoader, FactorDataset, FactorLoadRequest,
    FactorMeta, MissingPolicy, builtin_factor_catalog, cs_rank, factor_result_to_csv_string,
    factor_result_to_json_string, ts_delay, ts_delta,
};

struct MockFactorLoader {
    frame: DataFrame,
}

#[async_trait]
impl FactorDataLoader for MockFactorLoader {
    async fn load_bars(&self, _request: &FactorLoadRequest) -> Result<DataFrame> {
        Ok(self.frame.clone())
    }
}

fn mock_factor_frame() -> DataFrame {
    df!(
        "date" => &[
            "2026-01-01", "2026-01-01", "2026-01-01",
            "2026-01-02", "2026-01-02", "2026-01-02",
        ],
        "symbol" => &[
            "000001.SZ", "600000.SH", "000002.SZ",
            "000001.SZ", "600000.SH", "000002.SZ",
        ],
        "open" => &[10.0, 20.0, 30.0, 11.0, 21.0, 31.0],
        "high" => &[10.5, 20.5, 30.5, 11.5, 21.5, 31.5],
        "low" => &[9.5, 19.5, 29.5, 10.5, 20.5, 30.5],
        "close" => &[10.2, 20.2, 30.2, 11.2, 21.2, 31.2],
        "volume" => &[1000i64, 2000, 3000, 1100, 2100, 3100],
    )
    .unwrap()
}

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

#[tokio::test]
async fn dataset_from_loader_normalizes_and_checks_schema() {
    let loader = MockFactorLoader {
        frame: mock_factor_frame(),
    };
    let request = FactorLoadRequest {
        symbols: vec![
            "000001.SZ".to_string(),
            "600000.SH".to_string(),
            "000002.SZ".to_string(),
        ],
        start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        end: NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
        required_fields: vec!["close".to_string()],
    };

    let dataset = FactorDataset::from_loader(&loader, &request).await.unwrap();
    assert_eq!(dataset.frame().height(), 6);
    dataset.ensure_time_aligned().unwrap();
    dataset.validate_no_lookahead_basic().unwrap();
}

#[test]
fn operators_compute_aligned_series() {
    let df = mock_factor_frame();

    let rank = cs_rank(&df, "close").unwrap();
    assert_eq!(rank.len(), df.height());

    let delay = ts_delay(&df, "close", 1).unwrap();
    assert_eq!(delay.len(), df.height());

    let delta = ts_delta(&df, "close", 1).unwrap();
    assert_eq!(delta.len(), df.height());
}

#[tokio::test]
async fn catalog_lists_and_computes_rank_close() {
    let loader = MockFactorLoader {
        frame: mock_factor_frame(),
    };
    let request = FactorLoadRequest {
        symbols: vec![
            "000001.SZ".to_string(),
            "600000.SH".to_string(),
            "000002.SZ".to_string(),
        ],
        start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        end: NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
        required_fields: vec!["close".to_string()],
    };
    let dataset = FactorDataset::from_loader(&loader, &request).await.unwrap();
    let catalog = builtin_factor_catalog();

    assert!(catalog.list().iter().any(|meta| meta.id == "rank_close"));
    let result = catalog.compute("rank_close", &dataset).unwrap();
    assert_eq!(result.factor_id, "rank_close");
    assert_eq!(result.frame.height(), dataset.frame().height());
}

#[tokio::test]
async fn factor_result_exports_csv_and_json_strings() {
    let loader = MockFactorLoader {
        frame: mock_factor_frame(),
    };
    let request = FactorLoadRequest {
        symbols: vec![
            "000001.SZ".to_string(),
            "600000.SH".to_string(),
            "000002.SZ".to_string(),
        ],
        start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        end: NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
        required_fields: vec!["close".to_string()],
    };
    let dataset = FactorDataset::from_loader(&loader, &request).await.unwrap();
    let result = builtin_factor_catalog()
        .compute("rank_close", &dataset)
        .unwrap();

    let csv = factor_result_to_csv_string(&result).unwrap();
    assert!(csv.contains("date,symbol,value"));

    let json = factor_result_to_json_string(&result).unwrap();
    assert!(json.contains("rank_close"));
}
