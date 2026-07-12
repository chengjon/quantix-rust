use super::helpers::*;

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
