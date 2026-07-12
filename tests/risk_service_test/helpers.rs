pub use async_trait::async_trait;
pub use chrono::{DateTime, Datelike, Duration, TimeZone, Utc};
pub use quantix_cli::core::{QuantixError, Result};
pub use quantix_cli::risk::service::RiskIndustryResolver;
pub use quantix_cli::risk::{
    ClassificationStandard, IndustryClassificationLevel, IndustrySourceTier, ProjectedBuyImpact,
    ResolvedIndustry, RiskAccountSnapshot, RiskLockStateSource, RiskLogEventType, RiskRule,
    RiskRuleType, RiskService, RiskState, RiskStore, RuleValue,
};
pub use rust_decimal::Decimal;
pub use rust_decimal_macros::dec;
pub use std::collections::HashMap;
pub use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub(crate) struct FakeRiskStore {
    pub(crate) state: Arc<Mutex<Option<RiskState>>>,
}

impl FakeRiskStore {
    pub(crate) fn snapshot(&self) -> Option<RiskState> {
        self.state.lock().unwrap().clone()
    }

    pub(crate) fn set_snapshot(&self, state: RiskState) {
        *self.state.lock().unwrap() = Some(state);
    }
}

#[async_trait]
impl RiskStore for FakeRiskStore {
    async fn load_state(&self) -> Result<Option<RiskState>> {
        Ok(self.snapshot())
    }

    async fn save_state(&self, state: &RiskState) -> Result<()> {
        *self.state.lock().unwrap() = Some(state.clone());
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct FakeIndustryResolver {
    pub(crate) industries: Arc<Mutex<HashMap<String, String>>>,
}

impl FakeIndustryResolver {
    pub(crate) fn with_rows(rows: &[(&str, &str)]) -> Self {
        Self {
            industries: Arc::new(Mutex::new(
                rows.iter()
                    .map(|(code, industry)| ((*code).to_string(), (*industry).to_string()))
                    .collect(),
            )),
        }
    }
}

#[async_trait]
impl RiskIndustryResolver for FakeIndustryResolver {
    async fn resolve(
        &self,
        code: &str,
        query_date: chrono::NaiveDate,
        _captured_at: DateTime<Utc>,
    ) -> Result<ResolvedIndustry> {
        let industries = self.industries.lock().unwrap();
        let industry_name = industries.get(code).cloned().ok_or_else(|| {
            quantix_cli::core::QuantixError::Other(format!("resolver miss: {code}"))
        })?;

        Ok(ResolvedIndustry {
            code: code.to_string(),
            industry_name,
            standard: ClassificationStandard::Shenwan,
            level: IndustryClassificationLevel::FirstLevel,
            source_tier: IndustrySourceTier::CurrentActive,
            query_month: format!("{:04}-{:02}", query_date.year(), query_date.month()),
        })
    }
}

pub(crate) fn fixed_ts() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 11, 9, 35, 0).unwrap()
}

pub(crate) fn service() -> (RiskService<FakeRiskStore>, FakeRiskStore) {
    let store = FakeRiskStore::default();
    (RiskService::new(store.clone()), store)
}

pub(crate) fn service_with_industry_resolver(
    resolver: FakeIndustryResolver,
) -> (RiskService<FakeRiskStore>, FakeRiskStore) {
    let store = FakeRiskStore::default();
    (
        RiskService::with_industry_resolver(store.clone(), resolver),
        store,
    )
}

pub(crate) fn snapshot(
    total_assets: Decimal,
    positions: &[(&str, Decimal)],
) -> RiskAccountSnapshot {
    RiskAccountSnapshot::new(
        "default",
        total_assets,
        positions
            .iter()
            .map(|(code, market_value)| ((*code).to_string(), *market_value))
            .collect(),
    )
}
