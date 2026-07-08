//! Top-level batch scheduler (P0.15b).
//!
//! Fetches active codes, generates a batch_id, constructs OpenStock +
//! ClickHouse clients ONCE, then iterates calling ImportEngine.
//! continue-on-error: a per-code business error becomes a CodeResult
//! with Status::Failed; only infra-fatal errors abort the batch.

use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

use crate::core::error::Result;
use crate::core::runtime::OpenStockSettings;
use crate::data::models::{AdjustType, MinutePeriod};
use crate::db::ClickHouseClient;
use crate::sources::openstock_client::OpenStockClient;
use crate::tasks::openstock_import::engine::{CodeResult, ImportEngine};
use crate::tasks::openstock_import::fetcher::StockListFetchTrait;
use crate::tasks::openstock_import::state::{ImportStateStoreTrait, Status};

/// Counts per kind for a batch summary.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct KlineShareCount {
    pub klines: u32,
    pub share: u32,
}

/// One failure entry in the summary's `failures` list.
#[derive(Debug, Clone, serde::Serialize)]
pub struct FailureEntry {
    pub code: String,
    pub kind: String,
    pub reason: String,
}

/// Outcome of one batch run. Serialized for `--format json`.
#[derive(Debug, Clone, serde::Serialize)]
pub struct BatchSummary {
    pub batch_id: String,
    pub date: NaiveDate,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub total_codes: usize,
    pub success_count: KlineShareCount,
    pub failed_count: KlineShareCount,
    pub failures: Vec<FailureEntry>,
}

impl BatchSummary {
    pub fn new(batch_id: String, date: NaiveDate) -> Self {
        Self {
            batch_id,
            date,
            started_at: Utc::now(),
            finished_at: None,
            total_codes: 0,
            success_count: KlineShareCount::default(),
            failed_count: KlineShareCount::default(),
            failures: Vec::new(),
        }
    }

    pub fn push(&mut self, result: CodeResult) {
        self.total_codes += 1;
        match result.klines {
            Status::Success => self.success_count.klines += 1,
            Status::Failed { reason } => {
                self.failed_count.klines += 1;
                self.failures.push(FailureEntry {
                    code: result.code.clone(),
                    kind: "klines".into(),
                    reason,
                });
            }
        }
        match result.share {
            Status::Success => self.success_count.share += 1,
            Status::Failed { reason } => {
                self.failed_count.share += 1;
                self.failures.push(FailureEntry {
                    code: result.code.clone(),
                    kind: "share".into(),
                    reason,
                });
            }
        }
    }

    pub fn finish(&mut self) {
        self.finished_at = Some(Utc::now());
    }
}

/// Top-level scheduler. Holds a fetcher, state store, and settings
/// sufficient to build clients. `run()` constructs clients once and
/// drives the per-code loop.
pub struct BatchScheduler<'a, F: StockListFetchTrait, S: ImportStateStoreTrait> {
    pub(crate) fetcher: &'a F,
    pub(crate) state_store: &'a S,
    pub(crate) settings: &'a OpenStockSettings,
    pub(crate) period: MinutePeriod,
    pub(crate) adjust: AdjustType,
    pub(crate) will_apply: bool,
}

impl<'a, F: StockListFetchTrait, S: ImportStateStoreTrait> BatchScheduler<'a, F, S> {
    pub fn new(
        fetcher: &'a F,
        state_store: &'a S,
        settings: &'a OpenStockSettings,
        period: MinutePeriod,
        adjust: AdjustType,
        will_apply: bool,
    ) -> Self {
        Self {
            fetcher,
            state_store,
            settings,
            period,
            adjust,
            will_apply,
        }
    }

    pub async fn run(&self, date: NaiveDate, dry_run: bool) -> Result<BatchSummary> {
        let codes = self.fetcher.list_active_codes().await?;
        let batch_id = Uuid::new_v4().to_string();
        let mut summary = BatchSummary::new(batch_id.clone(), date);

        if dry_run {
            summary.total_codes = codes.len();
            summary.finish();
            return Ok(summary);
        }

        let openstock = OpenStockClient::from_settings(self.settings)?;
        let clickhouse = ClickHouseClient::with_default_config().await?;
        let engine = ImportEngine {
            openstock: &openstock,
            clickhouse: &clickhouse,
            state: self.state_store,
            batch_id: batch_id.clone(),
            period: self.period,
            adjust: self.adjust,
            will_apply: self.will_apply,
        };

        for code in &codes {
            match engine.process_code(code, date).await {
                Ok(result) => summary.push(result),
                Err(e) => {
                    tracing::error!(
                        code,
                        date = %date,
                        error = %e,
                        "infra-fatal during process_code; aborting batch"
                    );
                    return Err(e);
                }
            }
        }

        summary.finish();
        Ok(summary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tasks::openstock_import::fetcher::MockFetcher;
    use crate::tasks::openstock_import::state::MockStateStore;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).expect("valid test date")
    }

    fn settings_for_test() -> OpenStockSettings {
        OpenStockSettings {
            base_url: Some("http://localhost:9999".into()),
            api_key: Some("stub".into()),
            timeout_secs: 5,
        }
    }

    #[test]
    fn batch_summary_aggregates_success_only() {
        let mut s = BatchSummary::new("b1".into(), d(2026, 7, 8));
        s.push(CodeResult {
            code: "sh600000".into(),
            klines: Status::Success,
            share: Status::Success,
        });
        s.push(CodeResult {
            code: "sz000001".into(),
            klines: Status::Success,
            share: Status::Success,
        });
        s.finish();
        assert_eq!(s.total_codes, 2);
        assert_eq!(s.success_count.klines, 2);
        assert_eq!(s.success_count.share, 2);
        assert_eq!(s.failed_count.klines, 0);
        assert!(s.failures.is_empty());
    }

    #[test]
    fn batch_summary_aggregates_mixed() {
        let mut s = BatchSummary::new("b1".into(), d(2026, 7, 8));
        s.push(CodeResult {
            code: "sh600000".into(),
            klines: Status::Success,
            share: Status::Failed {
                reason: "timeout".into(),
            },
        });
        s.push(CodeResult {
            code: "sh999999".into(),
            klines: Status::Failed {
                reason: "404".into(),
            },
            share: Status::Failed {
                reason: "404".into(),
            },
        });
        assert_eq!(s.total_codes, 2);
        assert_eq!(s.success_count.klines, 1);
        assert_eq!(s.success_count.share, 0);
        assert_eq!(s.failed_count.klines, 1);
        assert_eq!(s.failed_count.share, 2);
        assert_eq!(s.failures.len(), 3);
    }

    #[test]
    fn batch_summary_serializes_to_json() {
        let mut s = BatchSummary::new("b1".into(), d(2026, 7, 8));
        s.push(CodeResult {
            code: "sh600000".into(),
            klines: Status::Success,
            share: Status::Failed {
                reason: "timeout".into(),
            },
        });
        s.finish();
        let json = serde_json::to_string(&s).expect("summary serializes");
        assert!(json.contains("\"batch_id\":\"b1\""));
        assert!(json.contains("\"total_codes\":1"));
        assert!(json.contains("\"klines\":1"));
        assert!(json.contains("\"share\":1"));
        assert!(json.contains("\"reason\":\"timeout\""));
    }

    #[tokio::test]
    async fn dry_run_returns_summary_without_state_writes() {
        let fetcher = MockFetcher::new(vec!["sh600000".into(), "sz000001".into()]);
        let state = MockStateStore::new();
        let settings = settings_for_test();
        let sched = BatchScheduler::new(
            &fetcher,
            &state,
            &settings,
            MinutePeriod::Minute5,
            AdjustType::QFQ,
            false,
        );
        let summary = sched.run(d(2026, 7, 8), true).await.expect("dry-run ok");
        assert_eq!(summary.total_codes, 2);
        assert_eq!(summary.success_count.klines, 0);
        // No state writes:
        let got = state
            .get_status("sh600000", d(2026, 7, 8), "klines")
            .await
            .expect("mock state read");
        assert!(got.is_none());
    }
}
