use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use quantix_cli::core::Result;
use quantix_cli::data::models::{AdjustType, Kline};
use quantix_cli::screener::{
    DailyKlineLoader, PresetInvocation, ScreenRunOptions, ScreenSortBy, ScreenUniverse,
    ScreenerService, parse_preset_invocation,
};
use quantix_cli::watchlist::{WatchlistService, WatchlistStorage};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;

fn make_kline(code: &str, day: u32, close: Decimal, volume: i64) -> Kline {
    Kline {
        code: code.to_string(),
        date: NaiveDate::from_ymd_opt(2024, 1, day).unwrap(),
        open: close,
        high: close + dec!(1),
        low: close - dec!(1),
        close,
        volume,
        amount: None,
        adjust_type: AdjustType::None,
    }
}

#[derive(Clone, Default)]
struct FakeLoader {
    data: HashMap<String, Vec<Kline>>,
}

#[async_trait]
impl DailyKlineLoader for FakeLoader {
    async fn load_daily_klines(&self, code: &str, lookback: usize) -> Result<Vec<Kline>> {
        let mut rows = self.data.get(code).cloned().unwrap_or_default();
        if rows.len() > lookback {
            rows = rows[rows.len() - lookback..].to_vec();
        }
        Ok(rows)
    }
}

fn default_options() -> ScreenRunOptions {
    ScreenRunOptions {
        limit: None,
        sort_by: ScreenSortBy::Code,
    }
}

fn preset(spec: &str) -> PresetInvocation {
    parse_preset_invocation(spec).unwrap()
}

#[tokio::test]
async fn resolves_explicit_code_universe() {
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
                    make_kline("000002", 1, dec!(12), 100),
                    make_kline("000002", 2, dec!(11), 100),
                    make_kline("000002", 3, dec!(10), 100),
                    make_kline("000002", 4, dec!(9), 100),
                    make_kline("000002", 5, dec!(8), 100),
                ],
            ),
        ]),
    };
    let storage = WatchlistStorage::new("/tmp/unused-watchlist.json");
    let service = ScreenerService::new(loader, storage);

    let rows = service
        .run(
            ScreenUniverse::Codes(vec!["000001".to_string(), "000002".to_string()]),
            &[preset("close_above_ma:period=3")],
            default_options(),
        )
        .await
        .unwrap();

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].code, "000001");
    assert!(rows[0].matched);
    assert_eq!(rows[1].code, "000002");
    assert!(!rows[1].matched);
}

#[tokio::test]
async fn resolves_watchlist_group_universe() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("watchlist.json");
    let storage = WatchlistStorage::new(&path);
    let watchlist_service = WatchlistService::default();
    let mut store = storage.load_or_create().unwrap();
    watchlist_service
        .create_group(&mut store, "core", Utc::now())
        .unwrap();
    watchlist_service
        .add(&mut store, "000001", Some("core"), Utc::now())
        .unwrap();
    watchlist_service.add(&mut store, "000002", None, Utc::now()).unwrap();
    storage.save(&store).unwrap();

    let service = ScreenerService::new(FakeLoader::default(), storage.clone());
    let rows = service
        .run(
            ScreenUniverse::Watchlist {
                group: Some("core".to_string()),
            },
            &[preset("close_above_ma:period=3")],
            default_options(),
        )
        .await
        .unwrap();

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].code, "000001");
}

#[tokio::test]
async fn applies_multi_preset_and_logic() {
    let loader = FakeLoader {
        data: HashMap::from([
            (
                "000001".to_string(),
                vec![
                    make_kline("000001", 1, dec!(10), 100),
                    make_kline("000001", 2, dec!(10), 100),
                    make_kline("000001", 3, dec!(10), 100),
                    make_kline("000001", 4, dec!(11), 100),
                    make_kline("000001", 5, dec!(12), 300),
                ],
            ),
            (
                "000002".to_string(),
                vec![
                    make_kline("000002", 1, dec!(10), 100),
                    make_kline("000002", 2, dec!(10), 100),
                    make_kline("000002", 3, dec!(10), 100),
                    make_kline("000002", 4, dec!(11), 100),
                    make_kline("000002", 5, dec!(12), 100),
                ],
            ),
        ]),
    };
    let storage = WatchlistStorage::new("/tmp/unused-watchlist.json");
    let service = ScreenerService::new(loader, storage);

    let rows = service
        .run(
            ScreenUniverse::Codes(vec!["000001".to_string(), "000002".to_string()]),
            &[
                preset("close_above_ma:period=3"),
                preset("volume_ratio_gte:window=5,value=1.5"),
            ],
            default_options(),
        )
        .await
        .unwrap();

    assert!(rows[0].matched);
    assert!(!rows[1].matched);
}

#[tokio::test]
async fn keeps_rows_when_kline_data_is_missing() {
    let loader = FakeLoader {
        data: HashMap::from([(
            "000001".to_string(),
            vec![
                make_kline("000001", 1, dec!(10), 100),
                make_kline("000001", 2, dec!(10), 100),
                make_kline("000001", 3, dec!(10), 100),
            ],
        )]),
    };
    let storage = WatchlistStorage::new("/tmp/unused-watchlist.json");
    let service = ScreenerService::new(loader, storage);

    let rows = service
        .run(
            ScreenUniverse::Codes(vec!["000001".to_string(), "000003".to_string()]),
            &[preset("close_above_ma:period=5")],
            default_options(),
        )
        .await
        .unwrap();

    assert_eq!(rows.len(), 2);
    assert!(!rows[0].matched);
    assert!(!rows[1].matched);
    assert!(rows[0].details[0].reason.as_deref().unwrap().contains("数据不足"));
    assert!(rows[1].details[0].reason.as_deref().unwrap().contains("数据不足"));
}

#[tokio::test]
async fn applies_sort_and_limit() {
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
    let storage = WatchlistStorage::new("/tmp/unused-watchlist.json");
    let service = ScreenerService::new(loader, storage);

    let rows = service
        .run(
            ScreenUniverse::Codes(vec!["000001".to_string(), "000002".to_string()]),
            &[preset("close_above_ma:period=3")],
            ScreenRunOptions {
                limit: Some(1),
                sort_by: ScreenSortBy::Score,
            },
        )
        .await
        .unwrap();

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].code, "000002");
}
