use super::*;
use super::support::{FakeLoader, make_kline};

#[tokio::test]
async fn test_execute_screener_preset_list_returns_supported_presets() {
    let output = execute_screener_command_with_loader(
        ScreenerCommands::PresetList,
        FakeLoader::default(),
        WatchlistStorage::new("/tmp/unused-screener-watchlist.json"),
    )
    .await
    .unwrap();

    match output {
        ScreenerCommandOutput::PresetList(presets) => {
            let names: Vec<&str> = presets.iter().map(|item| item.name).collect();
            assert_eq!(
                names,
                vec![
                    "close_above_ma",
                    "close_below_ma",
                    "rsi_gte",
                    "rsi_lte",
                    "volume_ratio_gte",
                ]
            );
        }
        ScreenerCommandOutput::Rows(_) => panic!("expected preset list output"),
    }
}

#[tokio::test]
async fn test_execute_screener_run_with_codes_returns_rows() {
    let loader = FakeLoader {
        data: HashMap::from([
            (
                "000001".to_string(),
                vec![
                    make_kline("000001", 1, dec!(10), 100),
                    make_kline("000001", 2, dec!(10), 100),
                    make_kline("000001", 3, dec!(10), 100),
                    make_kline("000001", 4, dec!(11), 100),
                    make_kline("000001", 5, dec!(12), 100),
                ],
            ),
            (
                "000002".to_string(),
                vec![
                    make_kline("000002", 1, dec!(10), 100),
                    make_kline("000002", 2, dec!(10), 100),
                    make_kline("000002", 3, dec!(10), 100),
                    make_kline("000002", 4, dec!(12), 100),
                    make_kline("000002", 5, dec!(15), 100),
                ],
            ),
        ]),
    };

    let output = execute_screener_command_with_loader(
        ScreenerCommands::Run {
            codes: Some("000001,000002".to_string()),
            watchlist: false,
            group: None,
            preset: vec!["close_above_ma:period=3".to_string()],
            limit: Some(1),
            sort_by: Some("score".to_string()),
        },
        loader,
        WatchlistStorage::new("/tmp/unused-screener-watchlist.json"),
    )
    .await
    .unwrap();

    match output {
        ScreenerCommandOutput::Rows(rows) => {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].code, "000002");
            assert!(rows[0].matched);
        }
        ScreenerCommandOutput::PresetList(_) => panic!("expected rows output"),
    }
}

#[tokio::test]
async fn test_execute_screener_run_with_watchlist_group_uses_watchlist_storage() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("watchlist.json");
    let storage = WatchlistStorage::new(&path);
    let service = WatchlistService::default();
    let mut store = storage.load_or_create().unwrap();
    service.create_group(&mut store, "core", Utc::now()).unwrap();
    service
        .add(&mut store, "000001", Some("core"), Utc::now())
        .unwrap();
    service.add(&mut store, "000002", None, Utc::now()).unwrap();
    storage.save(&store).unwrap();

    let loader = FakeLoader {
        data: HashMap::from([(
            "000001".to_string(),
            vec![
                make_kline("000001", 1, dec!(10), 100),
                make_kline("000001", 2, dec!(10), 100),
                make_kline("000001", 3, dec!(10), 100),
                make_kline("000001", 4, dec!(11), 100),
                make_kline("000001", 5, dec!(12), 100),
            ],
        )]),
    };

    let output = execute_screener_command_with_loader(
        ScreenerCommands::Run {
            codes: None,
            watchlist: true,
            group: Some("core".to_string()),
            preset: vec!["close_above_ma:period=3".to_string()],
            limit: None,
            sort_by: None,
        },
        loader,
        storage,
    )
    .await
    .unwrap();

    match output {
        ScreenerCommandOutput::Rows(rows) => {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].code, "000001");
        }
        ScreenerCommandOutput::PresetList(_) => panic!("expected rows output"),
    }
}

#[tokio::test]
async fn test_execute_screener_run_rejects_invalid_preset() {
    let err = execute_screener_command_with_loader(
        ScreenerCommands::Run {
            codes: Some("000001".to_string()),
            watchlist: false,
            group: None,
            preset: vec!["unknown_rule:value=1".to_string()],
            limit: None,
            sort_by: None,
        },
        FakeLoader::default(),
        WatchlistStorage::new("/tmp/unused-screener-watchlist.json"),
    )
    .await
    .unwrap_err();

    assert!(matches!(err, QuantixError::Other(_)));
    assert!(err.to_string().contains("未知的 preset"));
}
