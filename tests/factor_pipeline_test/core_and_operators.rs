use super::helpers::*;

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
