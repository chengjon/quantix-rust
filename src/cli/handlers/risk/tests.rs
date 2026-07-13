//! risk 命令解析、执行与状态加载的单元测试。

use super::*;
use async_trait::async_trait;
use chrono::TimeZone;
use rust_decimal_macros::dec;
use tempfile::tempdir;

use crate::risk::{
    BuyLockState, ClassificationStandard, DailyRiskBaseline, IndustryClassificationLevel,
    IndustrySyncSource, JsonRiskStore, RiskAccountSnapshot, RiskLockStateSource, RiskRuleType,
    RiskState, RiskStore, RuleValue, ShenwanCurrentSeedRow, ShenwanHistoricalSeedRow,
    SqliteIndustryStore,
};

fn fixed_ts() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 25, 10, 0, 0).unwrap()
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
async fn execute_risk_sync_industry_command_returns_summary_and_persists_rows() {
    let dir = tempdir().unwrap();
    let risk_state_path = dir.path().join("risk").join("risk_state.json");
    let service = RiskService::new(JsonRiskStore::new(&risk_state_path));
    let source = FakeIndustrySyncSource {
        current_rows: vec![ShenwanCurrentSeedRow {
            security_code: "000001.SZ".to_string(),
            industry_name: "银行".to_string(),
            source: "fake_current_sync".to_string(),
        }],
        historical_rows: vec![ShenwanHistoricalSeedRow {
            security_code: "000001".to_string(),
            industry_name: "银行".to_string(),
            effective_from: chrono::NaiveDate::from_ymd_opt(2014, 1, 1).unwrap(),
            effective_to: None,
            source: "fake_history_sync".to_string(),
        }],
    };

    let output = execute_risk_command_with_service_and_sync_at(
        RiskCommands::Sync(RiskSyncCommands::Industry {
            standard: "shenwan".to_string(),
        }),
        &service,
        &risk_state_path,
        &source,
        fixed_ts(),
    )
    .await
    .unwrap();

    match output {
        RiskCommandOutput::IndustrySync(summary) => {
            assert_eq!(summary.standard, ClassificationStandard::Shenwan);
            assert_eq!(summary.current_rows, 1);
            assert_eq!(summary.history_rows, 1);
        }
        other => panic!("unexpected output: {other:?}"),
    }

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
}

#[tokio::test]
async fn execute_risk_rule_set_industry_limit_command_returns_percentage_rule() {
    let dir = tempdir().unwrap();
    let risk_state_path = dir.path().join("risk").join("risk_state.json");
    let service = RiskService::new(JsonRiskStore::new(&risk_state_path));

    let output = execute_risk_command_with_service_at(
        RiskCommands::Rule(RiskRuleCommands::Set {
            rule_type: "industry-limit".to_string(),
            value: "30%".to_string(),
        }),
        &service,
        fixed_ts(),
    )
    .await
    .unwrap();

    match output {
        RiskCommandOutput::RuleSet(rule) => {
            assert_eq!(rule.rule_type, RiskRuleType::IndustryLimit);
            assert_eq!(rule.value, RuleValue::Percentage(dec!(30)));
            assert!(rule.enabled);
        }
        other => panic!("unexpected output: {other:?}"),
    }
}

#[test]
fn build_risk_pnl_lines_surfaces_auto_reduce_as_manual_recommendation() {
    let status = RiskStatus {
        account_id: "paper".to_string(),
        trading_date: chrono::NaiveDate::from_ymd_opt(2026, 3, 25).unwrap(),
        starting_total_assets: dec!(1000000),
        current_total_assets: dec!(930000),
        daily_pnl: dec!(-70000),
        daily_pnl_pct: dec!(-7),
        buy_locked: false,
        manual_release_active: false,
        lock_state_source: RiskLockStateSource::Open,
        lock_reason: None,
        lock_trigger_reason: None,
        lock_triggered_at: None,
        lock_effective_trading_date: None,
        position_ratios: vec![PositionRiskRow {
            code: "000001".to_string(),
            market_value: dec!(120000),
            ratio_pct: dec!(12.9),
        }],
        rules: vec![],
        auto_reduce_recommendation: Some(crate::risk::AutoReduceRecommendation {
            current_loss_pct: dec!(-7),
            reduce_ratio: dec!(50),
            position_codes: vec!["000001".to_string()],
            triggered_at: fixed_ts(),
        }),
    };

    let lines = build_risk_pnl_lines(&status).join("\n");
    assert!(lines.contains("[自动减仓建议]"));
    assert!(lines.contains("不会自动卖出"));
    assert!(lines.contains("000001"));
}

#[tokio::test]
async fn load_risk_status_for_live_import_preserves_manual_release_state() {
    let dir = tempdir().unwrap();
    let risk_state_path = dir.path().join("risk_state.json");
    let trading_date = fixed_ts().date_naive();
    let triggered_at = Utc.with_ymd_and_hms(2026, 3, 25, 9, 30, 0).unwrap();
    let mut state = RiskState {
        account_id: "live-001".to_string(),
        daily_baseline: Some(DailyRiskBaseline {
            trading_date,
            starting_total_assets: dec!(100000),
        }),
        buy_lock: BuyLockState {
            locked: false,
            reason: Some("daily-loss-limit 5% 已触发".to_string()),
            triggered_at: Some(triggered_at),
            trading_date: Some(trading_date),
            released_for_date: Some(trading_date),
        },
        ..RiskState::default()
    };
    JsonRiskStore::new(&risk_state_path)
        .save_state(&state)
        .await
        .unwrap();

    let service = RiskService::new(JsonRiskStore::new(&risk_state_path));
    let snapshot = RiskAccountSnapshot::new(
        "live-001".to_string(),
        dec!(100000),
        vec![("000001".to_string(), dec!(1520))],
    );
    let status = service.status(&snapshot, fixed_ts()).await.unwrap();

    assert_eq!(status.account_id, "live-001");
    assert!(!status.buy_locked);
    assert!(status.manual_release_active);
    assert_eq!(
        status.lock_state_source,
        RiskLockStateSource::ManualReleaseActive
    );
    assert_eq!(
        status.lock_trigger_reason.as_deref(),
        Some("daily-loss-limit 5% 已触发")
    );
    assert_eq!(status.lock_triggered_at, Some(triggered_at));
    assert_eq!(status.lock_effective_trading_date, Some(trading_date));

    state = JsonRiskStore::new(&risk_state_path)
        .load_state()
        .await
        .unwrap()
        .unwrap();
    assert_eq!(state.buy_lock.released_for_date, Some(trading_date));
    assert_eq!(
        state.buy_lock.reason.as_deref(),
        Some("daily-loss-limit 5% 已触发")
    );
}
