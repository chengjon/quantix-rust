use async_trait::async_trait;
use chrono::{NaiveDate, TimeZone, Utc};
use quantix_cli::core::Result;
use quantix_cli::risk::{
    ClassificationStandard, IndustryClassificationLevel, IndustrySyncSource, SqliteIndustryStore,
    ShenwanCurrentSeedRow, ShenwanHistoricalSeedRow, sync_industry_reference_data_at,
};
use tempfile::tempdir;

fn fixed_ts() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 25, 10, 0, 0).unwrap()
}

fn current_row(code: &str, industry_name: &str) -> ShenwanCurrentSeedRow {
    ShenwanCurrentSeedRow {
        security_code: code.to_string(),
        industry_name: industry_name.to_string(),
        source: "fake_current_sync".to_string(),
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
        source: "fake_history_sync".to_string(),
    }
}

#[derive(Debug)]
struct FakeIndustrySyncSource {
    current_rows: Vec<ShenwanCurrentSeedRow>,
    historical_rows: Vec<ShenwanHistoricalSeedRow>,
}

#[async_trait]
impl IndustrySyncSource for FakeIndustrySyncSource {
    async fn fetch_shenwan_current_rows(&self) -> Result<Vec<ShenwanCurrentSeedRow>> {
        Ok(self.current_rows.clone())
    }

    async fn fetch_shenwan_history_rows(&self) -> Result<Vec<ShenwanHistoricalSeedRow>> {
        Ok(self.historical_rows.clone())
    }
}

#[tokio::test]
async fn sync_industry_reference_data_refreshes_current_and_history_tables() {
    let dir = tempdir().unwrap();
    let risk_state_path = dir.path().join("risk").join("risk_state.json");
    let source = FakeIndustrySyncSource {
        current_rows: vec![
            current_row("000001.SZ", "银行"),
            current_row("600000.SH", "非银金融"),
        ],
        historical_rows: vec![
            historical_row(
                "000001",
                "银行",
                NaiveDate::from_ymd_opt(2014, 1, 1).unwrap(),
                Some(NaiveDate::from_ymd_opt(2021, 7, 29).unwrap()),
            ),
            historical_row(
                "000001",
                "非银金融",
                NaiveDate::from_ymd_opt(2021, 7, 30).unwrap(),
                None,
            ),
        ],
    };

    let summary = sync_industry_reference_data_at(
        &risk_state_path,
        ClassificationStandard::Shenwan,
        &source,
        fixed_ts(),
    )
    .await
    .unwrap();

    assert_eq!(summary.standard, ClassificationStandard::Shenwan);
    assert_eq!(summary.current_rows, 2);
    assert_eq!(summary.history_rows, 2);

    let store = SqliteIndustryStore::from_risk_state_path(&risk_state_path)
        .await
        .unwrap();
    let current = store
        .lookup_current(
            ClassificationStandard::Shenwan,
            IndustryClassificationLevel::FirstLevel,
            "000001",
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(current.industry_name, "银行");

    let historical = store
        .lookup_historical(
            ClassificationStandard::Shenwan,
            IndustryClassificationLevel::FirstLevel,
            "000001",
            NaiveDate::from_ymd_opt(2022, 1, 3).unwrap(),
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(historical.industry_name, "非银金融");
}

#[tokio::test]
async fn sync_industry_reference_data_rejects_unsupported_standard() {
    let dir = tempdir().unwrap();
    let risk_state_path = dir.path().join("risk").join("risk_state.json");
    let source = FakeIndustrySyncSource {
        current_rows: vec![],
        historical_rows: vec![],
    };

    let err = sync_industry_reference_data_at(
        &risk_state_path,
        ClassificationStandard::Csrc,
        &source,
        fixed_ts(),
    )
    .await
    .unwrap_err();

    assert!(err.to_string().contains("shenwan"));
}
