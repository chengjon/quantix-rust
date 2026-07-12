use super::helpers::*;

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
