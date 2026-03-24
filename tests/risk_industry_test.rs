use chrono::{NaiveDate, TimeZone, Utc};
use quantix_cli::risk::industry::{
    ClassificationStandard, IndustryClassificationLevel, IndustryResolver, IndustrySourceTier,
    ShenwanCurrentSeedRow, ShenwanHistoricalSeedRow,
};
use quantix_cli::risk::industry_store::SqliteIndustryStore;
use tempfile::{TempDir, tempdir};

fn fixed_ts() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 11, 9, 35, 0).unwrap()
}

fn query_date() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 3, 11).unwrap()
}

fn current_row(code: &str, industry_name: &str) -> ShenwanCurrentSeedRow {
    ShenwanCurrentSeedRow {
        security_code: code.to_string(),
        industry_name: industry_name.to_string(),
        source: "upstream_shenwan_sync".to_string(),
    }
}

fn historical_row(
    code: &str,
    industry_name: &str,
    effective_from: NaiveDate,
    effective_to: Option<NaiveDate>,
) -> ShenwanHistoricalSeedRow {
    ShenwanHistoricalSeedRow {
        security_code: code.to_string(),
        industry_name: industry_name.to_string(),
        effective_from,
        effective_to,
        source: "upstream_shenwan_history".to_string(),
    }
}

struct TestHarness {
    _dir: TempDir,
    store: SqliteIndustryStore,
    resolver: IndustryResolver,
}

async fn test_harness() -> TestHarness {
    let dir = tempdir().unwrap();
    let risk_state_path = dir.path().join("risk").join("risk_state.json");
    let store = SqliteIndustryStore::from_risk_state_path(&risk_state_path)
        .await
        .unwrap();
    let resolver = IndustryResolver::new(store.clone());
    TestHarness {
        _dir: dir,
        store,
        resolver,
    }
}

#[tokio::test]
async fn current_lookup_returns_industry_and_freezes_query_month_snapshot() {
    let harness = test_harness().await;
    let store = harness.store;
    let resolver = harness.resolver;
    let now = fixed_ts();

    store
        .upsert_shenwan_current_rows(&[current_row("000001.SZ", "银行")], now)
        .await
        .unwrap();

    let resolved = resolver.resolve("000001", query_date(), now).await.unwrap();
    assert_eq!(resolved.industry_name, "银行");
    assert_eq!(resolved.source_tier, IndustrySourceTier::CurrentActive);
    assert_eq!(resolved.standard, ClassificationStandard::Shenwan);
    assert_eq!(resolved.level, IndustryClassificationLevel::FirstLevel);

    let snapshot = store
        .lookup_snapshot_month(
            ClassificationStandard::Shenwan,
            IndustryClassificationLevel::FirstLevel,
            "000001",
            "2026-03",
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(snapshot.industry_name, "银行");
    assert_eq!(snapshot.source, "upstream_shenwan_sync");
    assert_eq!(snapshot.captured_at, now);
}

#[tokio::test]
async fn refresh_shenwan_current_rows_removes_stale_rows_not_present_in_new_dataset() {
    let harness = test_harness().await;
    let store = harness.store;
    let now = fixed_ts();

    store
        .upsert_shenwan_current_rows(
            &[
                current_row("000001.SZ", "银行"),
                current_row("600000.SH", "非银金融"),
            ],
            now,
        )
        .await
        .unwrap();

    store
        .refresh_shenwan_current_rows(&[current_row("600000.SH", "非银金融")], now)
        .await
        .unwrap();

    let removed = store
        .lookup_current(
            ClassificationStandard::Shenwan,
            IndustryClassificationLevel::FirstLevel,
            "000001",
        )
        .await
        .unwrap();
    assert_eq!(removed, None);

    let retained = store
        .lookup_current(
            ClassificationStandard::Shenwan,
            IndustryClassificationLevel::FirstLevel,
            "600000",
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(retained.industry_name, "非银金融");
}

#[tokio::test]
async fn second_successful_lookup_in_same_month_does_not_overwrite_snapshot_row() {
    let harness = test_harness().await;
    let store = harness.store;
    let resolver = harness.resolver;
    let first_now = fixed_ts();
    let second_now = Utc.with_ymd_and_hms(2026, 3, 18, 10, 0, 0).unwrap();

    store
        .upsert_shenwan_current_rows(&[current_row("000001.SZ", "银行")], first_now)
        .await
        .unwrap();
    let first = resolver
        .resolve("000001", query_date(), first_now)
        .await
        .unwrap();
    assert_eq!(first.industry_name, "银行");

    store
        .upsert_shenwan_current_rows(&[current_row("000001.SZ", "非银金融")], second_now)
        .await
        .unwrap();
    let second = resolver
        .resolve("000001", query_date(), second_now)
        .await
        .unwrap();
    assert_eq!(second.industry_name, "非银金融");
    assert_eq!(second.source_tier, IndustrySourceTier::CurrentActive);

    let snapshot = store
        .lookup_snapshot_month(
            ClassificationStandard::Shenwan,
            IndustryClassificationLevel::FirstLevel,
            "000001",
            "2026-03",
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(snapshot.industry_name, "银行");
    assert_eq!(snapshot.captured_at, first_now);
}

#[tokio::test]
async fn current_lookup_failure_falls_back_to_query_month_snapshot() {
    let harness = test_harness().await;
    let store = harness.store;
    let resolver = harness.resolver;
    let now = fixed_ts();

    store
        .insert_snapshot_if_missing(
            ClassificationStandard::Shenwan,
            IndustryClassificationLevel::FirstLevel,
            "2026-03",
            "000001",
            "银行",
            "current",
            now,
        )
        .await
        .unwrap();

    let resolved = resolver.resolve("000001", query_date(), now).await.unwrap();
    assert_eq!(resolved.industry_name, "银行");
    assert_eq!(resolved.source_tier, IndustrySourceTier::SnapshotMonth);
    assert_eq!(resolved.standard, ClassificationStandard::Shenwan);
}

#[tokio::test]
async fn query_month_miss_falls_back_to_local_historical_shenwan_mapping_when_available() {
    let harness = test_harness().await;
    let store = harness.store;
    let resolver = harness.resolver;
    let now = fixed_ts();

    store
        .upsert_shenwan_history_rows(
            &[historical_row(
                "000001.SZ",
                "银行",
                NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                Some(NaiveDate::from_ymd_opt(2026, 12, 31).unwrap()),
            )],
            now,
        )
        .await
        .unwrap();

    let resolved = resolver.resolve("000001", query_date(), now).await.unwrap();
    assert_eq!(resolved.industry_name, "银行");
    assert_eq!(resolved.source_tier, IndustrySourceTier::Historical);
    assert_eq!(resolved.standard, ClassificationStandard::Shenwan);
}

#[tokio::test]
async fn refresh_shenwan_history_rows_replaces_stale_rows_for_active_boundary() {
    let harness = test_harness().await;
    let store = harness.store;
    let now = fixed_ts();

    store
        .upsert_shenwan_history_rows(
            &[historical_row(
                "000001.SZ",
                "银行",
                NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                Some(NaiveDate::from_ymd_opt(2026, 12, 31).unwrap()),
            )],
            now,
        )
        .await
        .unwrap();

    store
        .refresh_shenwan_history_rows(
            &[historical_row(
                "600000.SH",
                "非银金融",
                NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                Some(NaiveDate::from_ymd_opt(2026, 12, 31).unwrap()),
            )],
            now,
        )
        .await
        .unwrap();

    let removed = store
        .lookup_historical(
            ClassificationStandard::Shenwan,
            IndustryClassificationLevel::FirstLevel,
            "000001",
            query_date(),
        )
        .await
        .unwrap();
    assert_eq!(removed, None);

    let retained = store
        .lookup_historical(
            ClassificationStandard::Shenwan,
            IndustryClassificationLevel::FirstLevel,
            "600000",
            query_date(),
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(retained.industry_name, "非银金融");
}

#[tokio::test]
async fn query_month_miss_uses_historical_row_on_inclusive_effective_to_boundary() {
    let harness = test_harness().await;
    let resolver = harness.resolver;
    let store = harness.store;
    let now = fixed_ts();

    store
        .upsert_shenwan_history_rows(
            &[historical_row(
                "000001.SZ",
                "银行",
                NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                Some(query_date()),
            )],
            now,
        )
        .await
        .unwrap();

    let resolved = resolver.resolve("000001", query_date(), now).await.unwrap();
    assert_eq!(resolved.industry_name, "银行");
    assert_eq!(resolved.source_tier, IndustrySourceTier::Historical);
}

#[tokio::test]
async fn resolver_normalizes_suffix_and_case_for_non_sz_codes() {
    let harness = test_harness().await;
    let resolver = harness.resolver;
    let store = harness.store;
    let now = fixed_ts();

    store
        .upsert_shenwan_current_rows(&[current_row("600000.sh", "银行")], now)
        .await
        .unwrap();

    let resolved = resolver.resolve("600000.SH", query_date(), now).await.unwrap();
    assert_eq!(resolved.industry_name, "银行");
    assert_eq!(resolved.code, "600000");
}

#[tokio::test]
async fn historical_miss_falls_back_to_most_recent_available_local_snapshot() {
    let harness = test_harness().await;
    let store = harness.store;
    let resolver = harness.resolver;
    let now = fixed_ts();

    store
        .insert_snapshot_if_missing(
            ClassificationStandard::Shenwan,
            IndustryClassificationLevel::FirstLevel,
            "2026-01",
            "000001",
            "旧行业",
            "current",
            Utc.with_ymd_and_hms(2026, 1, 10, 9, 0, 0).unwrap(),
        )
        .await
        .unwrap();
    store
        .insert_snapshot_if_missing(
            ClassificationStandard::Shenwan,
            IndustryClassificationLevel::FirstLevel,
            "2026-02",
            "000001",
            "银行",
            "current",
            Utc.with_ymd_and_hms(2026, 2, 10, 9, 0, 0).unwrap(),
        )
        .await
        .unwrap();

    let resolved = resolver.resolve("000001", query_date(), now).await.unwrap();
    assert_eq!(resolved.industry_name, "银行");
    assert_eq!(resolved.source_tier, IndustrySourceTier::LatestSnapshot);
    assert_eq!(resolved.standard, ClassificationStandard::Shenwan);
}

#[tokio::test]
async fn all_tiers_missing_return_a_hard_resolution_error() {
    let harness = test_harness().await;
    let resolver = harness.resolver;

    let err = resolver
        .resolve("000001", query_date(), fixed_ts())
        .await
        .unwrap_err();

    assert!(err.to_string().contains("current/monthly/history/fallback"));
}
