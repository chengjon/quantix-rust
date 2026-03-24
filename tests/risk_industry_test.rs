use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use quantix_cli::core::{QuantixError, Result};
use quantix_cli::risk::{
    IndustryResolver, IndustrySnapshotRecord, IndustrySourceTier, LatestIndustryReader,
    LatestIndustryRecord, SqliteIndustrySnapshotStore,
};
use tempfile::tempdir;

#[derive(Clone)]
struct FakeLatestIndustryReader {
    responses: std::sync::Arc<std::sync::Mutex<Vec<Result<Option<LatestIndustryRecord>>>>>,
}

impl FakeLatestIndustryReader {
    fn new(responses: Vec<Result<Option<LatestIndustryRecord>>>) -> Self {
        Self {
            responses: std::sync::Arc::new(std::sync::Mutex::new(responses)),
        }
    }
}

#[async_trait]
impl LatestIndustryReader for FakeLatestIndustryReader {
    async fn load_latest_industry(&self, _code: &str) -> Result<Option<LatestIndustryRecord>> {
        let mut responses = self.responses.lock().unwrap();
        if responses.is_empty() {
            return Ok(None);
        }
        responses.remove(0)
    }
}

fn fixed_ts() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 24, 9, 35, 0).unwrap()
}

fn latest_record(
    code: &str,
    industry_name: &str,
    captured_at: DateTime<Utc>,
) -> LatestIndustryRecord {
    LatestIndustryRecord {
        code: code.to_string(),
        industry_name: industry_name.to_string(),
        source: "sector_daily".to_string(),
        captured_at,
    }
}

fn snapshot_record(
    month: &str,
    code: &str,
    industry_name: &str,
    captured_at: DateTime<Utc>,
) -> IndustrySnapshotRecord {
    IndustrySnapshotRecord {
        snapshot_month: month.to_string(),
        code: code.to_string(),
        industry_name: industry_name.to_string(),
        source: "sector_daily".to_string(),
        captured_at,
    }
}

#[tokio::test]
async fn latest_source_lookup_returns_industry_and_freezes_query_month_snapshot() {
    let dir = tempdir().unwrap();
    let risk_path = dir.path().join("risk").join("risk_state.json");
    let store = SqliteIndustrySnapshotStore::from_risk_path(&risk_path)
        .await
        .unwrap();
    let resolver = IndustryResolver::new(
        store.clone(),
        FakeLatestIndustryReader::new(vec![Ok(Some(latest_record("000001", "银行", fixed_ts())))]),
    );

    let resolved = resolver
        .resolve("000001", NaiveDate::from_ymd_opt(2026, 3, 11).unwrap())
        .await
        .unwrap();
    let snapshot = store
        .find_month_snapshot("2026-03", "000001")
        .await
        .unwrap()
        .unwrap();

    assert_eq!(resolved.industry_name, "银行");
    assert_eq!(resolved.source_tier, IndustrySourceTier::Latest);
    assert_eq!(snapshot.industry_name, "银行");
    assert_eq!(snapshot.snapshot_month, "2026-03");
}

#[tokio::test]
async fn second_latest_source_success_in_same_month_does_not_overwrite_existing_snapshot_row() {
    let dir = tempdir().unwrap();
    let store = SqliteIndustrySnapshotStore::new(dir.path().join("industry_snapshots.db"))
        .await
        .unwrap();
    let resolver = IndustryResolver::new(
        store.clone(),
        FakeLatestIndustryReader::new(vec![
            Ok(Some(latest_record("000001", "银行", fixed_ts()))),
            Ok(Some(latest_record(
                "000001",
                "证券",
                fixed_ts() + chrono::Duration::days(2),
            ))),
        ]),
    );

    resolver
        .resolve("000001", NaiveDate::from_ymd_opt(2026, 3, 11).unwrap())
        .await
        .unwrap();
    let second = resolver
        .resolve("000001", NaiveDate::from_ymd_opt(2026, 3, 21).unwrap())
        .await
        .unwrap();
    let frozen = store
        .find_month_snapshot("2026-03", "000001")
        .await
        .unwrap()
        .unwrap();

    assert_eq!(second.industry_name, "证券");
    assert_eq!(second.source_tier, IndustrySourceTier::Latest);
    assert_eq!(frozen.industry_name, "银行");
    assert_eq!(frozen.captured_at, fixed_ts());
}

#[tokio::test]
async fn primary_lookup_failure_falls_back_to_query_month_snapshot() {
    let dir = tempdir().unwrap();
    let store = SqliteIndustrySnapshotStore::new(dir.path().join("industry_snapshots.db"))
        .await
        .unwrap();
    store
        .insert_if_missing(&snapshot_record("2026-03", "000001", "银行", fixed_ts()))
        .await
        .unwrap();
    let resolver = IndustryResolver::new(
        store,
        FakeLatestIndustryReader::new(vec![Err(QuantixError::DataSource(
            "upstream unavailable".to_string(),
        ))]),
    );

    let resolved = resolver
        .resolve("000001", NaiveDate::from_ymd_opt(2026, 3, 25).unwrap())
        .await
        .unwrap();

    assert_eq!(resolved.industry_name, "银行");
    assert_eq!(resolved.source_tier, IndustrySourceTier::SnapshotMonth);
}

#[tokio::test]
async fn query_month_miss_falls_back_to_most_recent_available_snapshot() {
    let dir = tempdir().unwrap();
    let store = SqliteIndustrySnapshotStore::new(dir.path().join("industry_snapshots.db"))
        .await
        .unwrap();
    store
        .insert_if_missing(&snapshot_record("2026-02", "000001", "银行", fixed_ts()))
        .await
        .unwrap();
    let resolver = IndustryResolver::new(store, FakeLatestIndustryReader::new(vec![Ok(None)]));

    let resolved = resolver
        .resolve("000001", NaiveDate::from_ymd_opt(2026, 3, 25).unwrap())
        .await
        .unwrap();

    assert_eq!(resolved.industry_name, "银行");
    assert_eq!(resolved.source_tier, IndustrySourceTier::SnapshotFallback);
}

#[tokio::test]
async fn all_three_tiers_missing_return_a_hard_resolution_error() {
    let dir = tempdir().unwrap();
    let store = SqliteIndustrySnapshotStore::new(dir.path().join("industry_snapshots.db"))
        .await
        .unwrap();
    let resolver = IndustryResolver::new(store, FakeLatestIndustryReader::new(vec![Ok(None)]));

    let err = resolver
        .resolve("000001", NaiveDate::from_ymd_opt(2026, 3, 25).unwrap())
        .await
        .unwrap_err();

    assert!(err.to_string().contains("latest/monthly/fallback"));
}

#[tokio::test]
async fn store_derives_sqlite_path_from_risk_path_and_creates_missing_parent_directories() {
    let dir = tempdir().unwrap();
    let risk_path = dir
        .path()
        .join("nested")
        .join("risk")
        .join("risk_state.json");
    let store = SqliteIndustrySnapshotStore::from_risk_path(&risk_path)
        .await
        .unwrap();

    assert_eq!(
        store.path(),
        dir.path()
            .join("nested")
            .join("risk")
            .join("industry_snapshots.db")
    );
    assert!(store.path().exists());
}
