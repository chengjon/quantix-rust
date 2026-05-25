use async_trait::async_trait;
use chrono::NaiveDate;
use polars::prelude::*;
use polars::prelude::{ParquetReader, SerReader};
use quantix_cli::cli::handlers::run_factor_command;
use quantix_cli::cli::{Cli, Commands, FactorCommands, FactorOutputFormat};
use quantix_cli::core::Result;
use quantix_cli::factor::{
    FactorCategory, FactorComputeRequest, FactorDataLoader, FactorDataset, FactorLoadRequest,
    FactorMeta, LayeredBacktestRequest, MissingPolicy, NeutralizationRequest,
    builtin_factor_catalog, cs_rank, evaluate_factor_ic, factor_ic_result_to_csv_string,
    factor_ic_result_to_parquet_file, factor_result_to_csv_string, factor_result_to_json_string,
    factor_result_to_parquet_file, factor_value_correlation, neutralize_factor_cross_sectional,
    run_layered_factor_backtest, ts_delay, ts_delta, ts_rank,
};
use std::collections::BTreeMap;
use std::fs::{File, write};

use clap::Parser;

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

fn mock_factor_frame_10d() -> DataFrame {
    let mut dates = Vec::new();
    let mut symbols = Vec::new();
    let mut open = Vec::new();
    let mut high = Vec::new();
    let mut low = Vec::new();
    let mut close = Vec::new();
    let mut volume = Vec::new();
    let universe = ["000001.SZ", "600000.SH", "000002.SZ"];

    for day in 1..=10 {
        for (idx, symbol) in universe.iter().enumerate() {
            let base = 10.0 + idx as f64 * 10.0 + day as f64;
            dates.push(format!("2026-01-{day:02}"));
            symbols.push((*symbol).to_string());
            open.push(base);
            high.push(base + 0.5);
            low.push(base - 0.5);
            close.push(if day == 5 && idx == 1 {
                None
            } else {
                Some(base + 0.2)
            });
            volume.push(1000i64 + day as i64 * 10 + idx as i64);
        }
    }

    df!(
        "date" => dates,
        "symbol" => symbols,
        "open" => open,
        "high" => high,
        "low" => low,
        "close" => close,
        "volume" => volume,
    )
    .unwrap()
}

fn mock_alpha101_frame() -> DataFrame {
    mock_alpha_frame_days(15)
}

fn mock_alpha_frame_days(days: usize) -> DataFrame {
    let mut dates = Vec::new();
    let mut symbols = Vec::new();
    let mut open = Vec::new();
    let mut high = Vec::new();
    let mut low = Vec::new();
    let mut close = Vec::new();
    let mut volume = Vec::new();
    let mut amount = Vec::new();
    let universe = ["000001.SZ", "600000.SH", "000002.SZ"];

    for day in 1..=days {
        for (idx, symbol) in universe.iter().enumerate() {
            let base = 20.0 + day as f64 * 0.15;
            let open_pattern = ((day * (idx + 2)) % 7) as f64;
            let close_pattern = (((day + idx) * (idx + 3)) % 9) as f64;
            dates.push(format!("2026-01-{day:02}"));
            symbols.push((*symbol).to_string());
            let open_value = base + open_pattern * 0.4 + idx as f64 * 0.05;
            let close_value = base + 0.2 + close_pattern * 0.35;
            open.push(open_value);
            high.push(open_value.max(close_value) + 0.8);
            low.push(open_value.min(close_value) - 0.6);
            close.push(close_value);
            let vol =
                1000i64 + ((day * day + idx * 37 + day * idx * 11 + (day % 4) * 29) % 500) as i64;
            volume.push(vol);
            amount.push(((open_value + close_value) / 2.0) * vol as f64);
        }
    }

    df!(
        "date" => dates,
        "symbol" => symbols,
        "open" => open,
        "high" => high,
        "low" => low,
        "close" => close,
        "volume" => volume,
        "amount" => amount,
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

#[test]
fn factor_score_cli_shape_parses() {
    let cli = Cli::try_parse_from([
        "quantix",
        "factor",
        "score",
        "--input",
        "/tmp/factor-input.csv",
        "--factor",
        "rank_close",
        "delta_close_1",
        "--symbol",
        "000001.SZ",
        "600000.SH",
        "--start",
        "2026-01-01",
        "--end",
        "2026-01-10",
        "--format",
        "csv",
        "--output",
        "/tmp/factor-score.csv",
        "--skip-checks",
    ])
    .unwrap();

    match cli.command {
        Commands::Factor(FactorCommands::Score {
            input,
            factors,
            symbols,
            start,
            end,
            format,
            output,
            top,
            skip_checks,
        }) => {
            assert_eq!(input, "/tmp/factor-input.csv");
            assert_eq!(factors, vec!["rank_close", "delta_close_1"]);
            assert_eq!(symbols, vec!["000001.SZ", "600000.SH"]);
            assert_eq!(start, "2026-01-01");
            assert_eq!(end, "2026-01-10");
            assert_eq!(format, FactorOutputFormat::Csv);
            assert_eq!(output.as_deref(), Some("/tmp/factor-score.csv"));
            assert_eq!(top, None);
            assert!(skip_checks);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[tokio::test(flavor = "multi_thread")]
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

    let ts_rank_values = ts_rank(&df, "close", 2).unwrap();
    assert_eq!(ts_rank_values.len(), df.height());
    assert_eq!(ts_rank_values.null_count(), 3);
    let ts_rank_values = ts_rank_values.f64().unwrap();
    assert_eq!(ts_rank_values.get(3), Some(2.0));
    assert_eq!(ts_rank_values.get(4), Some(2.0));
    assert_eq!(ts_rank_values.get(5), Some(2.0));
}

#[tokio::test(flavor = "multi_thread")]
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

#[tokio::test(flavor = "multi_thread")]
async fn catalog_lists_and_computes_ts_rank_close_5() {
    let loader = MockFactorLoader {
        frame: mock_factor_frame_10d(),
    };
    let request = FactorLoadRequest {
        symbols: vec![
            "000001.SZ".to_string(),
            "600000.SH".to_string(),
            "000002.SZ".to_string(),
        ],
        start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        end: NaiveDate::from_ymd_opt(2026, 1, 10).unwrap(),
        required_fields: vec!["close".to_string()],
    };
    let dataset = FactorDataset::from_loader(&loader, &request).await.unwrap();
    let catalog = builtin_factor_catalog();

    assert!(
        catalog
            .list()
            .iter()
            .any(|meta| meta.id == "ts_rank_close_5")
    );
    let result = catalog.compute("ts_rank_close_5", &dataset).unwrap();
    assert_eq!(result.factor_id, "ts_rank_close_5");
    assert_eq!(result.frame.height(), dataset.frame().height());

    let value = result.frame.column("value").unwrap().f64().unwrap();
    // Three symbols have four warmup nulls each; the explicit missing close on
    // 600000.SH day 5 invalidates that symbol's day 5-9 rolling windows.
    assert_eq!(value.null_count(), 17);
}

#[tokio::test(flavor = "multi_thread")]
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

#[tokio::test(flavor = "multi_thread")]
async fn factor_result_exports_parquet_file() {
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
    let tempdir = tempfile::tempdir().unwrap();
    let path = tempdir.path().join("rank_close.parquet");

    factor_result_to_parquet_file(&result, &path).unwrap();

    let frame = ParquetReader::new(File::open(path).unwrap())
        .finish()
        .unwrap();
    assert_eq!(frame.height(), result.frame.height());
    assert_eq!(frame.get_column_names(), vec!["date", "symbol", "value"]);
}

#[tokio::test(flavor = "multi_thread")]
async fn factor_compute_cli_writes_parquet_output() {
    let tempdir = tempfile::tempdir().unwrap();
    let input = tempdir.path().join("bars.csv");
    let output = tempdir.path().join("rank_close.parquet");
    write(
        &input,
        "date,symbol,open,high,low,close,volume\n\
         2026-01-01,000001.SZ,10.0,11.0,9.0,10.0,1000\n\
         2026-01-01,600000.SH,20.0,21.0,19.0,20.0,2000\n\
         2026-01-02,000001.SZ,11.0,12.0,10.0,11.0,1100\n\
         2026-01-02,600000.SH,19.0,20.0,18.0,19.0,1900\n",
    )
    .unwrap();

    run_factor_command(FactorCommands::Compute {
        input: input.to_string_lossy().to_string(),
        factors: vec!["rank_close".to_string()],
        symbols: vec!["000001.SZ".to_string(), "600000.SH".to_string()],
        start: "2026-01-01".to_string(),
        end: "2026-01-02".to_string(),
        format: FactorOutputFormat::Parquet,
        output: Some(output.to_string_lossy().to_string()),
        skip_checks: false,
    })
    .await
    .unwrap();

    let frame = ParquetReader::new(File::open(output).unwrap())
        .finish()
        .unwrap();
    assert_eq!(frame.height(), 4);
    assert_eq!(frame.get_column_names(), vec!["date", "symbol", "value"]);
}

#[tokio::test(flavor = "multi_thread")]
async fn factor_score_cli_writes_csv_output() {
    let tempdir = tempfile::tempdir().unwrap();
    let input = tempdir.path().join("bars.csv");
    let output = tempdir.path().join("factor_score.csv");
    write(
        &input,
        "date,symbol,open,high,low,close,volume\n\
         2026-01-01,000001.SZ,10.0,11.0,9.0,10.0,1000\n\
         2026-01-01,600000.SH,20.0,21.0,19.0,20.0,2000\n\
         2026-01-01,000002.SZ,30.0,31.0,29.0,30.0,3000\n\
         2026-01-02,000001.SZ,11.0,12.0,10.0,11.0,1100\n\
         2026-01-02,600000.SH,19.0,20.0,18.0,19.0,1900\n\
         2026-01-02,000002.SZ,32.0,33.0,31.0,32.0,3200\n",
    )
    .unwrap();

    run_factor_command(FactorCommands::Score {
        input: input.to_string_lossy().to_string(),
        factors: vec!["rank_close".to_string(), "delta_close_1".to_string()],
        symbols: vec![
            "000001.SZ".to_string(),
            "600000.SH".to_string(),
            "000002.SZ".to_string(),
        ],
        start: "2026-01-01".to_string(),
        end: "2026-01-02".to_string(),
        format: FactorOutputFormat::Csv,
        output: Some(output.to_string_lossy().to_string()),
        top: Some(2),
        skip_checks: false,
    })
    .await
    .unwrap();

    let csv = std::fs::read_to_string(output).unwrap();
    assert!(csv.starts_with("date,symbol,score,factor_count\n"));
    assert!(csv.contains("2026-01-02,000002.SZ,1.0,2\n"));
    assert_eq!(csv.lines().count(), 3);
}

#[tokio::test(flavor = "multi_thread")]
async fn p1_pipeline_computes_rank_close_with_mock_loader() {
    let loader = MockFactorLoader {
        frame: mock_factor_frame_10d(),
    };
    let request = FactorLoadRequest {
        symbols: vec![
            "000001.SZ".to_string(),
            "600000.SH".to_string(),
            "000002.SZ".to_string(),
        ],
        start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        end: NaiveDate::from_ymd_opt(2026, 1, 10).unwrap(),
        required_fields: vec!["close".to_string()],
    };

    let dataset = FactorDataset::from_loader(&loader, &request).await.unwrap();
    dataset.ensure_time_aligned().unwrap();
    dataset.validate_no_lookahead_basic().unwrap();

    let result = builtin_factor_catalog()
        .compute("rank_close", &dataset)
        .unwrap();
    assert_eq!(result.factor_id, "rank_close");
    assert_eq!(result.frame.height(), 30);
    assert_eq!(
        result.frame.get_column_names(),
        vec!["date", "symbol", "value"]
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn catalog_lists_and_computes_alpha101_first_batch() {
    let loader = MockFactorLoader {
        frame: mock_alpha101_frame(),
    };
    let request = FactorLoadRequest {
        symbols: vec![
            "000001.SZ".to_string(),
            "600000.SH".to_string(),
            "000002.SZ".to_string(),
        ],
        start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        end: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
        required_fields: vec![
            "open".to_string(),
            "low".to_string(),
            "close".to_string(),
            "volume".to_string(),
            "amount".to_string(),
        ],
    };
    let dataset = FactorDataset::from_loader(&loader, &request).await.unwrap();
    let catalog = builtin_factor_catalog();
    let expected = [
        "alpha101_002",
        "alpha101_003",
        "alpha101_005",
        "alpha101_006",
        "alpha101_012",
    ];

    for factor_id in expected {
        assert!(
            catalog.list().iter().any(|meta| meta.id == factor_id),
            "missing catalog metadata for {factor_id}"
        );

        let result = catalog.compute(factor_id, &dataset).unwrap();
        assert_eq!(result.factor_id, factor_id);
        assert_eq!(result.frame.height(), dataset.frame().height());
        assert_eq!(
            result.frame.get_column_names(),
            vec!["date", "symbol", "value"]
        );
        assert!(
            result.frame.column("value").unwrap().null_count() < result.frame.height(),
            "{factor_id} should produce at least one non-null value"
        );
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn catalog_lists_and_computes_alpha191_first_batch() {
    let loader = MockFactorLoader {
        frame: mock_alpha101_frame(),
    };
    let request = FactorLoadRequest {
        symbols: vec![
            "000001.SZ".to_string(),
            "600000.SH".to_string(),
            "000002.SZ".to_string(),
        ],
        start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        end: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
        required_fields: vec![
            "open".to_string(),
            "high".to_string(),
            "low".to_string(),
            "close".to_string(),
            "volume".to_string(),
        ],
    };
    let dataset = FactorDataset::from_loader(&loader, &request).await.unwrap();
    let catalog = builtin_factor_catalog();
    let expected = ["alpha191_101", "alpha191_102", "alpha191_103"];

    for factor_id in expected {
        assert!(
            catalog.list().iter().any(|meta| meta.id == factor_id),
            "missing catalog metadata for {factor_id}"
        );

        let result = catalog.compute(factor_id, &dataset).unwrap();
        assert_eq!(result.factor_id, factor_id);
        assert_eq!(result.frame.height(), dataset.frame().height());
        assert_eq!(
            result.frame.get_column_names(),
            vec!["date", "symbol", "value"]
        );
        assert!(
            result.frame.column("value").unwrap().null_count() < result.frame.height(),
            "{factor_id} should produce at least one non-null value"
        );
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn catalog_lists_and_computes_alpha191_second_batch() {
    let loader = MockFactorLoader {
        frame: mock_alpha101_frame(),
    };
    let request = FactorLoadRequest {
        symbols: vec![
            "000001.SZ".to_string(),
            "600000.SH".to_string(),
            "000002.SZ".to_string(),
        ],
        start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        end: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
        required_fields: vec![
            "open".to_string(),
            "high".to_string(),
            "low".to_string(),
            "close".to_string(),
            "volume".to_string(),
        ],
    };
    let dataset = FactorDataset::from_loader(&loader, &request).await.unwrap();
    let catalog = builtin_factor_catalog();
    let expected = [
        "alpha191_104",
        "alpha191_105",
        "alpha191_106",
        "alpha191_107",
        "alpha191_108",
        "alpha191_109",
        "alpha191_110",
    ];

    for factor_id in expected {
        assert!(
            catalog.list().iter().any(|meta| meta.id == factor_id),
            "missing catalog metadata for {factor_id}"
        );

        let result = catalog.compute(factor_id, &dataset).unwrap();
        assert_eq!(result.factor_id, factor_id);
        assert_eq!(result.frame.height(), dataset.frame().height());
        assert_eq!(
            result.frame.get_column_names(),
            vec!["date", "symbol", "value"]
        );
        assert!(
            result.frame.column("value").unwrap().null_count() < result.frame.height(),
            "{factor_id} should produce at least one non-null value"
        );
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn catalog_lists_and_computes_alpha191_third_batch() {
    let loader = MockFactorLoader {
        frame: mock_alpha_frame_days(25),
    };
    let request = FactorLoadRequest {
        symbols: vec![
            "000001.SZ".to_string(),
            "600000.SH".to_string(),
            "000002.SZ".to_string(),
        ],
        start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        end: NaiveDate::from_ymd_opt(2026, 1, 25).unwrap(),
        required_fields: vec![
            "open".to_string(),
            "high".to_string(),
            "low".to_string(),
            "close".to_string(),
            "volume".to_string(),
        ],
    };
    let dataset = FactorDataset::from_loader(&loader, &request).await.unwrap();
    let catalog = builtin_factor_catalog();
    let expected = [
        "alpha191_111",
        "alpha191_112",
        "alpha191_113",
        "alpha191_114",
        "alpha191_115",
        "alpha191_116",
        "alpha191_117",
        "alpha191_118",
        "alpha191_119",
        "alpha191_120",
    ];

    for factor_id in expected {
        assert!(
            catalog.list().iter().any(|meta| meta.id == factor_id),
            "missing catalog metadata for {factor_id}"
        );

        let result = catalog.compute(factor_id, &dataset).unwrap();
        assert_eq!(result.factor_id, factor_id);
        assert_eq!(result.frame.height(), dataset.frame().height());
        assert_eq!(
            result.frame.get_column_names(),
            vec!["date", "symbol", "value"]
        );
        assert!(
            result.frame.column("value").unwrap().null_count() < result.frame.height(),
            "{factor_id} should produce at least one non-null value"
        );
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn evaluation_computes_ic_ir_and_factor_correlation() {
    let loader = MockFactorLoader {
        frame: mock_alpha101_frame(),
    };
    let request = FactorLoadRequest {
        symbols: vec![
            "000001.SZ".to_string(),
            "600000.SH".to_string(),
            "000002.SZ".to_string(),
        ],
        start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        end: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
        required_fields: vec![
            "open".to_string(),
            "close".to_string(),
            "volume".to_string(),
            "amount".to_string(),
        ],
    };
    let dataset = FactorDataset::from_loader(&loader, &request).await.unwrap();
    let catalog = builtin_factor_catalog();
    let rank_close = catalog.compute("rank_close", &dataset).unwrap();
    let alpha012 = catalog.compute("alpha101_012", &dataset).unwrap();

    let evaluation = evaluate_factor_ic(&dataset, &rank_close, 1).unwrap();
    assert_eq!(evaluation.summary.factor_id, "rank_close");
    assert_eq!(evaluation.summary.horizon, 1);
    assert!(evaluation.summary.observations > 0);
    assert!(evaluation.summary.ic_mean.is_some());
    assert!(evaluation.summary.ir.is_some());
    assert_eq!(evaluation.by_date.get_column_names(), vec!["date", "ic"]);
    let csv = factor_ic_result_to_csv_string(&evaluation).unwrap();
    assert!(csv.starts_with("date,ic\n"));
    assert!(csv.contains("2026-01-"));

    let corr = factor_value_correlation(&rank_close, &alpha012).unwrap();
    assert!(corr >= -1.0);
    assert!(corr <= 1.0);
}

#[tokio::test(flavor = "multi_thread")]
async fn factor_evaluation_exports_parquet_file() {
    let loader = MockFactorLoader {
        frame: mock_alpha101_frame(),
    };
    let request = FactorLoadRequest {
        symbols: vec![
            "000001.SZ".to_string(),
            "600000.SH".to_string(),
            "000002.SZ".to_string(),
        ],
        start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        end: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
        required_fields: vec!["close".to_string()],
    };
    let dataset = FactorDataset::from_loader(&loader, &request).await.unwrap();
    let rank_close = builtin_factor_catalog()
        .compute("rank_close", &dataset)
        .unwrap();
    let evaluation = evaluate_factor_ic(&dataset, &rank_close, 1).unwrap();
    let tempdir = tempfile::tempdir().unwrap();
    let path = tempdir.path().join("rank_close_ic.parquet");

    factor_ic_result_to_parquet_file(&evaluation, &path).unwrap();

    let frame = ParquetReader::new(File::open(path).unwrap())
        .finish()
        .unwrap();
    assert_eq!(frame.height(), evaluation.by_date.height());
    assert_eq!(frame.get_column_names(), vec!["date", "ic"]);
}

#[tokio::test(flavor = "multi_thread")]
async fn factor_evaluate_cli_writes_parquet_output() {
    let tempdir = tempfile::tempdir().unwrap();
    let input = tempdir.path().join("bars.csv");
    let output = tempdir.path().join("rank_close_ic.parquet");
    write(
        &input,
        "date,symbol,open,high,low,close,volume\n\
         2026-01-01,000001.SZ,10.0,11.0,9.0,10.0,1000\n\
         2026-01-01,600000.SH,20.0,21.0,19.0,20.0,2000\n\
         2026-01-01,000002.SZ,30.0,31.0,29.0,30.0,3000\n\
         2026-01-02,000001.SZ,11.0,12.0,10.0,11.0,1100\n\
         2026-01-02,600000.SH,19.0,20.0,18.0,19.0,1900\n\
         2026-01-02,000002.SZ,32.0,33.0,31.0,32.0,3200\n\
         2026-01-03,000001.SZ,10.5,11.5,9.5,10.5,1050\n\
         2026-01-03,600000.SH,21.0,22.0,20.0,21.0,2100\n\
         2026-01-03,000002.SZ,31.0,32.0,30.0,31.0,3100\n",
    )
    .unwrap();

    run_factor_command(FactorCommands::Evaluate {
        input: input.to_string_lossy().to_string(),
        factor: "rank_close".to_string(),
        symbols: vec![
            "000001.SZ".to_string(),
            "600000.SH".to_string(),
            "000002.SZ".to_string(),
        ],
        start: "2026-01-01".to_string(),
        end: "2026-01-03".to_string(),
        horizon: 1,
        format: FactorOutputFormat::Parquet,
        output: Some(output.to_string_lossy().to_string()),
        skip_checks: false,
    })
    .await
    .unwrap();

    let frame = ParquetReader::new(File::open(output).unwrap())
        .finish()
        .unwrap();
    assert!(frame.height() > 0);
    assert_eq!(frame.get_column_names(), vec!["date", "ic"]);
}

#[tokio::test(flavor = "multi_thread")]
async fn neutralization_removes_cross_sectional_exposure_by_date() {
    let loader = MockFactorLoader {
        frame: mock_alpha101_frame(),
    };
    let request = FactorLoadRequest {
        symbols: vec![
            "000001.SZ".to_string(),
            "600000.SH".to_string(),
            "000002.SZ".to_string(),
        ],
        start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        end: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
        required_fields: vec!["close".to_string(), "volume".to_string()],
    };
    let dataset = FactorDataset::from_loader(&loader, &request).await.unwrap();
    let rank_close = builtin_factor_catalog()
        .compute("rank_close", &dataset)
        .unwrap();

    let neutralized = neutralize_factor_cross_sectional(
        &dataset,
        &rank_close,
        &NeutralizationRequest {
            exposures: vec!["volume".to_string()],
            add_intercept: true,
        },
    )
    .unwrap();

    assert_eq!(neutralized.factor_id, "rank_close_neutralized");
    assert_eq!(neutralized.frame.height(), rank_close.frame.height());
    assert_eq!(
        neutralized.frame.get_column_names(),
        vec!["date", "symbol", "value"]
    );

    let correlation = factor_value_correlation(&neutralized, &rank_close).unwrap();
    assert!(correlation.abs() < 1.0);

    let dates = neutralized.frame.column("date").unwrap();
    let values = neutralized.frame.column("value").unwrap().f64().unwrap();
    let mut residuals_by_date: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    for row in 0..neutralized.frame.height() {
        if let Some(value) = values.get(row) {
            residuals_by_date
                .entry(dates.get(row).unwrap().to_string())
                .or_default()
                .push(value);
        }
    }
    for residuals in residuals_by_date.values() {
        let mean = residuals.iter().sum::<f64>() / residuals.len() as f64;
        assert!(mean.abs() < 1e-9);
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn layered_backtest_computes_group_returns_and_long_short() {
    let loader = MockFactorLoader {
        frame: mock_alpha101_frame(),
    };
    let request = FactorLoadRequest {
        symbols: vec![
            "000001.SZ".to_string(),
            "600000.SH".to_string(),
            "000002.SZ".to_string(),
        ],
        start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        end: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
        required_fields: vec!["close".to_string()],
    };
    let dataset = FactorDataset::from_loader(&loader, &request).await.unwrap();
    let rank_close = builtin_factor_catalog()
        .compute("rank_close", &dataset)
        .unwrap();

    let backtest = run_layered_factor_backtest(
        &dataset,
        &rank_close,
        &LayeredBacktestRequest {
            groups: 3,
            horizon: 1,
        },
    )
    .unwrap();

    assert_eq!(backtest.summary.factor_id, "rank_close");
    assert_eq!(backtest.summary.groups, 3);
    assert_eq!(backtest.summary.horizon, 1);
    assert!(backtest.summary.periods > 0);
    assert!(backtest.summary.long_short_mean.is_some());
    assert_eq!(
        backtest.by_period.get_column_names(),
        vec!["date", "group", "return", "count"]
    );
    assert_eq!(
        backtest.long_short.get_column_names(),
        vec!["date", "long_short"]
    );
    assert_eq!(backtest.long_short.height(), backtest.summary.periods);
    assert_eq!(
        backtest.by_period.height(),
        backtest.summary.periods * backtest.summary.groups
    );
}
