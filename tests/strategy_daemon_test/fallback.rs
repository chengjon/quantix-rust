use super::*;
use quantix_cli::strategy::FallbackStrategyBarLoader;

#[tokio::test]
async fn fallback_loader_prefers_primary_rows_when_available() {
    let root = tempdir().unwrap();
    write_day_file(root.path(), "sh", "000001", &[800, 900, 1000]);

    let primary = FakeBarLoader::default();
    primary.set_bars("000001", vec![kline(1, 10), kline(2, 11)]);

    let loader = FallbackStrategyBarLoader::new(primary, Some(root.path().to_path_buf()));
    let rows = loader.load_daily_bars("000001", 10).await.unwrap();

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].close, dec!(10));
    assert_eq!(rows[1].close, dec!(11));
    assert_eq!(
        loader.last_source(),
        Some(StrategyBarLoadSource {
            source_id: "primary".to_string(),
            fallback_used: false,
        })
    );
}

#[tokio::test]
async fn fallback_loader_uses_tdx_when_primary_returns_empty() {
    let root = tempdir().unwrap();
    write_day_file(root.path(), "sh", "000001", &[1200, 1300, 1400]);

    let loader =
        FallbackStrategyBarLoader::new(FakeBarLoader::default(), Some(root.path().to_path_buf()));
    let rows = loader.load_daily_bars("000001", 10).await.unwrap();

    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].close, dec!(12));
    assert_eq!(rows[2].close, dec!(14));
    assert_eq!(
        loader.last_source(),
        Some(StrategyBarLoadSource {
            source_id: "tdx-day-file".to_string(),
            fallback_used: true,
        })
    );
}

#[tokio::test]
async fn fallback_loader_uses_preferred_market_to_resolve_ambiguous_code() {
    let root = tempdir().unwrap();
    write_day_file(root.path(), "sh", "000001", &[800, 900]);
    write_day_file(root.path(), "sz", "000001", &[1200, 1300]);

    let loader = FallbackStrategyBarLoader::with_options(
        FakeBarLoader::default(),
        "primary",
        Some(root.path().to_path_buf()),
        Some("sz".to_string()),
    );
    let rows = loader.load_daily_bars("000001", 10).await.unwrap();

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].close, dec!(12));
    assert_eq!(rows[1].close, dec!(13));
}

#[tokio::test]
async fn fallback_loader_uses_tdx_when_primary_errors() {
    let root = tempdir().unwrap();
    write_day_file(root.path(), "sh", "000001", &[1500, 1600]);

    let loader = FallbackStrategyBarLoader::new(ErrorBarLoader, Some(root.path().to_path_buf()));
    let rows = loader.load_daily_bars("000001", 10).await.unwrap();

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].close, dec!(15));
    assert_eq!(rows[1].close, dec!(16));
}
