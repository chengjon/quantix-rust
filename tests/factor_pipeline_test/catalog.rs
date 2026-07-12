use super::helpers::*;

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
