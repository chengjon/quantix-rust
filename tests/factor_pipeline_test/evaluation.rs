use super::helpers::*;

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
