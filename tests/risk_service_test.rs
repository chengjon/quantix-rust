use async_trait::async_trait;
use chrono::{DateTime, Datelike, Duration, TimeZone, Utc};
use quantix_cli::core::Result;
use quantix_cli::risk::{
    ClassificationStandard, IndustryClassificationLevel, IndustrySourceTier, ProjectedBuyImpact,
    ResolvedIndustry, RiskAccountSnapshot, RiskIndustryResolver, RiskLockStateSource,
    RiskLogEventType, RiskRule, RiskRuleType, RiskService, RiskState, RiskStore, RuleValue,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
struct FakeRiskStore {
    state: Arc<Mutex<Option<RiskState>>>,
}

impl FakeRiskStore {
    fn snapshot(&self) -> Option<RiskState> {
        self.state.lock().unwrap().clone()
    }

    fn set_snapshot(&self, state: RiskState) {
        *self.state.lock().unwrap() = Some(state);
    }
}

#[derive(Debug, Clone, Default)]
struct FakeIndustryResolver {
    industries: Arc<Mutex<HashMap<String, String>>>,
}

impl FakeIndustryResolver {
    fn with_rows(rows: &[(&str, &str)]) -> Self {
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
impl RiskStore for FakeRiskStore {
    async fn load_state(&self) -> Result<Option<RiskState>> {
        Ok(self.snapshot())
    }

    async fn save_state(&self, state: &RiskState) -> Result<()> {
        *self.state.lock().unwrap() = Some(state.clone());
        Ok(())
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
        let industry_name = industries
            .get(code)
            .cloned()
            .ok_or_else(|| quantix_cli::core::QuantixError::Other(format!("resolver miss: {code}")))?;

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

fn fixed_ts() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 11, 9, 35, 0).unwrap()
}

fn service() -> (RiskService<FakeRiskStore>, FakeRiskStore) {
    let store = FakeRiskStore::default();
    (RiskService::new(store.clone()), store)
}

fn service_with_industry_resolver(
    resolver: FakeIndustryResolver,
) -> (RiskService<FakeRiskStore>, FakeRiskStore) {
    let store = FakeRiskStore::default();
    (
        RiskService::with_industry_resolver(store.clone(), resolver),
        store,
    )
}

fn snapshot(total_assets: Decimal, positions: &[(&str, Decimal)]) -> RiskAccountSnapshot {
    RiskAccountSnapshot::new(
        "default",
        total_assets,
        positions
            .iter()
            .map(|(code, market_value)| ((*code).to_string(), *market_value))
            .collect(),
    )
}

#[tokio::test]
async fn set_rule_upserts_position_limit_percentage_values() {
    let (service, store) = service();
    let now = fixed_ts();

    let first = service
        .set_rule("position-limit", "20%", now)
        .await
        .unwrap();
    assert_eq!(first.rule_type, RiskRuleType::PositionLimit);
    assert_eq!(first.value, RuleValue::Percentage(dec!(20)));
    assert!(first.enabled);

    let updated = service
        .set_rule("position-limit", "25%", now + Duration::minutes(1))
        .await
        .unwrap();
    assert_eq!(updated.value, RuleValue::Percentage(dec!(25)));

    let state = store.snapshot().unwrap();
    assert_eq!(state.rules.len(), 1);
    assert_eq!(state.rules[0].value, RuleValue::Percentage(dec!(25)));
}

#[tokio::test]
async fn set_rule_upserts_daily_loss_amount_values() {
    let (service, store) = service();

    let rule = service
        .set_rule("daily-loss-limit", "50000", fixed_ts())
        .await
        .unwrap();
    assert_eq!(rule.rule_type, RiskRuleType::DailyLossLimit);
    assert_eq!(rule.value, RuleValue::Amount(dec!(50000)));

    let state = store.snapshot().unwrap();
    assert_eq!(state.rules.len(), 1);
    assert_eq!(state.rules[0].value, RuleValue::Amount(dec!(50000)));
}

#[tokio::test]
async fn set_rule_upserts_daily_loss_percentage_values() {
    let (service, store) = service();

    let rule = service
        .set_rule("daily-loss-limit", "5%", fixed_ts())
        .await
        .unwrap();
    assert_eq!(rule.value, RuleValue::Percentage(dec!(5)));

    let state = store.snapshot().unwrap();
    assert_eq!(state.rules.len(), 1);
    assert_eq!(state.rules[0].value, RuleValue::Percentage(dec!(5)));
}

#[tokio::test]
async fn set_rule_upserts_industry_blocklist_values() {
    let (service, store) = service();

    let rule = service
        .set_rule("industry-blocklist", "银行,地产", fixed_ts())
        .await
        .unwrap();
    assert_eq!(rule.rule_type, RiskRuleType::IndustryBlocklist);
    assert_eq!(
        rule.value,
        RuleValue::TextList(vec!["银行".to_string(), "地产".to_string()])
    );

    let state = store.snapshot().unwrap();
    assert_eq!(state.rules.len(), 1);
    assert_eq!(state.rules[0].rule_type, RiskRuleType::IndustryBlocklist);
    assert_eq!(
        state.rules[0].value,
        RuleValue::TextList(vec!["银行".to_string(), "地产".to_string()])
    );
}

#[tokio::test]
async fn set_rule_industry_blocklist_rejects_empty_names() {
    let (service, _) = service();

    let err = service
        .set_rule("industry-blocklist", " , , ", fixed_ts())
        .await
        .unwrap_err();
    assert!(err.to_string().contains("industry-blocklist"));
}

#[tokio::test]
async fn status_rejects_invalid_daily_loss_rule_value_type() {
    let (service, store) = service();
    let now = fixed_ts();
    let mut state = RiskState::default();
    state.rules.push(RiskRule {
        rule_type: RiskRuleType::DailyLossLimit,
        value: RuleValue::TextList(vec!["银行".to_string()]),
        enabled: true,
        created_at: now,
        updated_at: now,
    });
    store.set_snapshot(state);

    let err = service
        .status(&snapshot(dec!(1000000), &[]), now)
        .await
        .unwrap_err();

    assert!(err.to_string().contains("daily-loss-limit"));
    assert!(err.to_string().contains("配置无效"));
}

#[tokio::test]
async fn industry_blocklist_rejects_buy_when_resolved_industry_is_blocked_without_lock_or_log() {
    let (service, store) =
        service_with_industry_resolver(FakeIndustryResolver::with_rows(&[("000001", "银行")]));
    let now = fixed_ts();

    service
        .set_rule("industry-blocklist", "银行,地产", now)
        .await
        .unwrap();
    service.status(&snapshot(dec!(1000000), &[]), now).await.unwrap();
    let before = store.snapshot().unwrap();

    let err = service
        .check_buy(
            &snapshot(dec!(1000000), &[]),
            &ProjectedBuyImpact::new("000001", dec!(150000), dec!(1000000)),
            now + Duration::minutes(1),
        )
        .await
        .unwrap_err();

    assert!(err.to_string().contains("industry-blocklist"));
    assert!(err.to_string().contains("银行"));

    let state = store.snapshot().unwrap();
    assert!(!state.buy_lock.locked);
    assert_eq!(state.events.len(), before.events.len());
}

#[tokio::test]
async fn industry_blocklist_allows_buy_when_resolved_industry_is_not_blocked() {
    let (service, _) = service_with_industry_resolver(FakeIndustryResolver::with_rows(&[(
        "000001",
        "有色金属",
    )]));
    let now = fixed_ts();

    service
        .set_rule("industry-blocklist", "银行,地产", now)
        .await
        .unwrap();

    service
        .check_buy(
            &snapshot(dec!(1000000), &[]),
            &ProjectedBuyImpact::new("000001", dec!(150000), dec!(1000000)),
            now + Duration::minutes(1),
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn industry_blocklist_returns_hard_check_failure_when_resolver_misses() {
    let (service, store) = service_with_industry_resolver(FakeIndustryResolver::default());
    let now = fixed_ts();

    service
        .set_rule("industry-blocklist", "银行,地产", now)
        .await
        .unwrap();
    service.status(&snapshot(dec!(1000000), &[]), now).await.unwrap();
    let before = store.snapshot().unwrap();

    let err = service
        .check_buy(
            &snapshot(dec!(1000000), &[]),
            &ProjectedBuyImpact::new("000001", dec!(150000), dec!(1000000)),
            now + Duration::minutes(1),
        )
        .await
        .unwrap_err();

    assert!(err.to_string().contains("industry-blocklist"));
    assert!(err.to_string().contains("检查失败"));

    let state = store.snapshot().unwrap();
    assert!(!state.buy_lock.locked);
    assert_eq!(state.events.len(), before.events.len());
}

#[tokio::test]
async fn position_limit_rejects_amount_syntax() {
    let (service, _) = service();

    let err = service
        .set_rule("position-limit", "50000", fixed_ts())
        .await
        .unwrap_err();
    assert!(err.to_string().contains("position-limit"));
}

#[tokio::test]
async fn set_rule_rejects_unknown_rule_type_or_malformed_value() {
    let (service, _) = service();

    let unknown_rule = service
        .set_rule("unknown-rule", "5%", fixed_ts())
        .await
        .unwrap_err();
    assert!(unknown_rule.to_string().contains("unknown-rule"));

    let malformed_value = service
        .set_rule("daily-loss-limit", "abc%", fixed_ts())
        .await
        .unwrap_err();
    assert!(malformed_value.to_string().contains("abc%"));
}

#[tokio::test]
async fn enable_and_disable_rule_update_existing_rule() {
    let (service, _) = service();
    let now = fixed_ts();

    service
        .set_rule("daily-loss-limit", "5%", now)
        .await
        .unwrap();

    let disabled = service
        .disable_rule("daily-loss-limit", now + Duration::minutes(1))
        .await
        .unwrap();
    assert!(!disabled.enabled);

    let enabled = service
        .enable_rule("daily-loss-limit", now + Duration::minutes(2))
        .await
        .unwrap();
    assert!(enabled.enabled);
}

#[tokio::test]
async fn status_initializes_the_current_day_baseline_when_missing() {
    let (service, store) = service();

    let status = service
        .status(
            &snapshot(dec!(1000000), &[("000001", dec!(200000))]),
            fixed_ts(),
        )
        .await
        .unwrap();

    assert_eq!(status.starting_total_assets, dec!(1000000));
    assert_eq!(status.current_total_assets, dec!(1000000));
    assert_eq!(status.daily_pnl, dec!(0));
    assert_eq!(status.daily_pnl_pct, dec!(0));
    assert!(!status.buy_locked);
    assert_eq!(status.lock_state_source, RiskLockStateSource::Open);
    assert_eq!(status.lock_trigger_reason, None);
    assert_eq!(status.lock_triggered_at, None);
    assert_eq!(status.lock_effective_trading_date, None);
    assert_eq!(status.position_ratios.len(), 1);
    assert_eq!(status.position_ratios[0].code, "000001");
    assert_eq!(status.position_ratios[0].ratio_pct, dec!(20));

    let state = store.snapshot().unwrap();
    assert_eq!(
        state.daily_baseline.unwrap().trading_date,
        fixed_ts().date_naive()
    );
    assert!(!state.buy_lock.locked);
}

#[tokio::test]
async fn day_rollover_replaces_baseline_and_clears_daily_lock() {
    let (service, store) = service();
    let now = fixed_ts();

    service
        .set_rule("daily-loss-limit", "50000", now)
        .await
        .unwrap();
    service
        .status(&snapshot(dec!(1000000), &[]), now)
        .await
        .unwrap();

    let locked = service
        .sync_after_trade_snapshot(&snapshot(dec!(940000), &[]), now + Duration::hours(1))
        .await
        .unwrap();
    assert!(locked.buy_locked);

    let next_day = now + Duration::days(1);
    let reset = service
        .status(&snapshot(dec!(930000), &[]), next_day)
        .await
        .unwrap();

    assert!(!reset.buy_locked);
    assert_eq!(reset.starting_total_assets, dec!(930000));
    assert_eq!(reset.daily_pnl, dec!(0));

    let state = store.snapshot().unwrap();
    assert_eq!(
        state.daily_baseline.unwrap().trading_date,
        next_day.date_naive()
    );
    assert!(!state.buy_lock.locked);
}

#[tokio::test]
async fn daily_loss_amount_triggers_buy_lock() {
    let (service, _) = service();
    let now = fixed_ts();

    service
        .set_rule("daily-loss-limit", "50000", now)
        .await
        .unwrap();
    service
        .status(&snapshot(dec!(1000000), &[]), now)
        .await
        .unwrap();

    let status = service
        .sync_after_trade_snapshot(&snapshot(dec!(949000), &[]), now + Duration::hours(2))
        .await
        .unwrap();

    assert!(status.buy_locked);
    assert!(status.lock_reason.unwrap().contains("50000"));
}

#[tokio::test]
async fn daily_loss_percentage_triggers_buy_lock() {
    let (service, _) = service();
    let now = fixed_ts();

    service
        .set_rule("daily-loss-limit", "5%", now)
        .await
        .unwrap();
    service
        .status(&snapshot(dec!(1000000), &[]), now)
        .await
        .unwrap();

    let status = service
        .sync_after_trade_snapshot(&snapshot(dec!(949000), &[]), now + Duration::hours(2))
        .await
        .unwrap();

    assert!(status.buy_locked);
    assert_eq!(status.daily_pnl_pct, dec!(-5.1));
    assert!(status.lock_reason.unwrap().contains("5%"));
    assert_eq!(status.lock_state_source, RiskLockStateSource::DailyLossLocked);
    assert!(status.lock_trigger_reason.as_ref().unwrap().contains("5%"));
    assert_eq!(status.lock_triggered_at, Some(now + Duration::hours(2)));
    assert_eq!(status.lock_effective_trading_date, Some(now.date_naive()));
}

#[tokio::test]
async fn current_lock_blocks_new_buys() {
    let (service, _) = service();
    let now = fixed_ts();

    service
        .set_rule("daily-loss-limit", "50000", now)
        .await
        .unwrap();
    service
        .status(&snapshot(dec!(1000000), &[("000001", dec!(100000))]), now)
        .await
        .unwrap();
    service
        .sync_after_trade_snapshot(
            &snapshot(dec!(949000), &[("000001", dec!(100000))]),
            now + Duration::hours(1),
        )
        .await
        .unwrap();

    let err = service
        .check_buy(
            &snapshot(dec!(949000), &[("000001", dec!(100000))]),
            &ProjectedBuyImpact::new("000001", dec!(200000), dec!(948500)),
            now + Duration::hours(2),
        )
        .await
        .unwrap_err();

    assert!(err.to_string().contains("锁定"));
}

#[tokio::test]
async fn position_limit_rejects_a_projected_buy_that_would_exceed_the_cap() {
    let (service, _) = service();
    let now = fixed_ts();

    service
        .set_rule("position-limit", "20%", now)
        .await
        .unwrap();

    let err = service
        .check_buy(
            &snapshot(dec!(1000000), &[("000001", dec!(150000))]),
            &ProjectedBuyImpact::new("000001", dec!(250000), dec!(999500)),
            now + Duration::minutes(1),
        )
        .await
        .unwrap_err();

    assert!(err.to_string().contains("position-limit"));
}

#[tokio::test]
async fn sell_sync_remains_allowed_while_buy_lock_is_active() {
    let (service, store) = service();
    let now = fixed_ts();

    service
        .set_rule("daily-loss-limit", "50000", now)
        .await
        .unwrap();
    service
        .status(&snapshot(dec!(1000000), &[("000001", dec!(200000))]), now)
        .await
        .unwrap();
    service
        .sync_after_trade_snapshot(
            &snapshot(dec!(949000), &[("000001", dec!(180000))]),
            now + Duration::hours(1),
        )
        .await
        .unwrap();

    let status = service
        .sync_after_trade_snapshot(
            &snapshot(dec!(960000), &[("000001", dec!(120000))]),
            now + Duration::hours(2),
        )
        .await
        .unwrap();

    assert!(status.buy_locked);
    assert_eq!(status.position_ratios[0].ratio_pct, dec!(12.5));
    assert!(store.snapshot().unwrap().buy_lock.locked);
}

#[tokio::test]
async fn release_buy_lock_suppresses_same_day_relock() {
    let (service, store) = service();
    let now = fixed_ts();

    service
        .set_rule("daily-loss-limit", "50000", now)
        .await
        .unwrap();
    service.status(&snapshot(dec!(1000000), &[]), now).await.unwrap();
    service
        .sync_after_trade_snapshot(&snapshot(dec!(949000), &[]), now + Duration::hours(1))
        .await
        .unwrap();

    let released = service
        .release_buy_lock(now + Duration::hours(2))
        .await
        .unwrap();
    assert!(!released.locked);
    assert_eq!(released.released_for_date, Some(now.date_naive()));

    let status = service
        .status(&snapshot(dec!(949000), &[]), now + Duration::hours(3))
        .await
        .unwrap();
    assert!(!status.buy_locked);
    assert!(status.manual_release_active);
    assert_eq!(status.lock_state_source, RiskLockStateSource::ManualReleaseActive);
    assert!(status
        .lock_trigger_reason
        .as_ref()
        .unwrap()
        .contains("50000"));
    assert_eq!(status.lock_triggered_at, Some(now + Duration::hours(1)));
    assert_eq!(status.lock_effective_trading_date, Some(now.date_naive()));

    let state = store.snapshot().unwrap();
    assert_eq!(state.buy_lock.released_for_date, Some(now.date_naive()));
    assert!(!state.buy_lock.locked);
}

#[tokio::test]
async fn release_buy_lock_is_cleared_on_day_rollover() {
    let (service, store) = service();
    let now = fixed_ts();

    service
        .set_rule("daily-loss-limit", "50000", now)
        .await
        .unwrap();
    service.status(&snapshot(dec!(1000000), &[]), now).await.unwrap();
    service
        .sync_after_trade_snapshot(&snapshot(dec!(949000), &[]), now + Duration::hours(1))
        .await
        .unwrap();
    service
        .release_buy_lock(now + Duration::hours(2))
        .await
        .unwrap();

    let next_day_status = service
        .status(&snapshot(dec!(949000), &[]), now + Duration::days(1))
        .await
        .unwrap();

    assert!(!next_day_status.buy_locked);
    assert!(!next_day_status.manual_release_active);
    let state = store.snapshot().unwrap();
    assert_eq!(state.buy_lock.released_for_date, None);
}

#[tokio::test]
async fn release_buy_lock_errors_when_no_active_lock_exists() {
    let (service, _) = service();

    let err = service.release_buy_lock(fixed_ts()).await.unwrap_err();
    assert!(err.to_string().contains("当前无活动买入锁"));
}

#[tokio::test]
async fn list_log_returns_newest_first_rule_and_lock_events() {
    let (service, _) = service();
    let now = fixed_ts();

    service
        .set_rule("daily-loss-limit", "50000", now)
        .await
        .unwrap();
    service.status(&snapshot(dec!(1000000), &[]), now).await.unwrap();
    service
        .sync_after_trade_snapshot(&snapshot(dec!(949000), &[]), now + Duration::hours(1))
        .await
        .unwrap();
    service
        .release_buy_lock(now + Duration::hours(2))
        .await
        .unwrap();

    let events = service.list_log(Some(10), None, None).await.unwrap();
    assert_eq!(events.len(), 3);
    assert_eq!(events[0].event_type, RiskLogEventType::BuyLockReleased);
    assert_eq!(events[1].event_type, RiskLogEventType::DailyLossLockTriggered);
    assert_eq!(events[2].event_type, RiskLogEventType::RuleSet);
}

#[tokio::test]
async fn idempotent_release_does_not_append_duplicate_log_events() {
    let (service, _) = service();
    let now = fixed_ts();

    service
        .set_rule("daily-loss-limit", "50000", now)
        .await
        .unwrap();
    service.status(&snapshot(dec!(1000000), &[]), now).await.unwrap();
    service
        .sync_after_trade_snapshot(&snapshot(dec!(949000), &[]), now + Duration::hours(1))
        .await
        .unwrap();
    service
        .release_buy_lock(now + Duration::hours(2))
        .await
        .unwrap();
    service
        .release_buy_lock(now + Duration::hours(3))
        .await
        .unwrap();

    let events = service.list_log(Some(10), None, None).await.unwrap();
    let release_count = events
        .iter()
        .filter(|event| event.event_type == RiskLogEventType::BuyLockReleased)
        .count();
    assert_eq!(release_count, 1);
}

#[tokio::test]
async fn day_rollover_clearing_lock_appends_log_event() {
    let (service, _) = service();
    let now = fixed_ts();

    service
        .set_rule("daily-loss-limit", "50000", now)
        .await
        .unwrap();
    service.status(&snapshot(dec!(1000000), &[]), now).await.unwrap();
    service
        .sync_after_trade_snapshot(&snapshot(dec!(949000), &[]), now + Duration::hours(1))
        .await
        .unwrap();
    service
        .status(&snapshot(dec!(949000), &[]), now + Duration::days(1))
        .await
        .unwrap();

    let events = service.list_log(Some(10), None, None).await.unwrap();
    assert_eq!(events[0].event_type, RiskLogEventType::BuyLockCleared);
    assert!(events[0].detail.contains("day rollover"));
}

#[tokio::test]
async fn list_log_filters_by_event_write_date() {
    let (service, _) = service();
    let now = fixed_ts();

    service
        .set_rule("daily-loss-limit", "50000", now)
        .await
        .unwrap();
    service
        .set_rule("position-limit", "20%", now + Duration::days(1))
        .await
        .unwrap();

    let events = service
        .list_log(Some(10), Some(now.date_naive()), None)
        .await
        .unwrap();

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, RiskLogEventType::RuleSet);
    assert!(events[0].detail.contains("daily-loss-limit"));
}

#[tokio::test]
async fn list_log_filters_by_type_and_date_together() {
    let (service, _) = service();
    let now = fixed_ts();

    service
        .set_rule("daily-loss-limit", "50000", now)
        .await
        .unwrap();
    service.status(&snapshot(dec!(1000000), &[]), now).await.unwrap();
    service
        .sync_after_trade_snapshot(&snapshot(dec!(949000), &[]), now + Duration::hours(1))
        .await
        .unwrap();
    service
        .release_buy_lock(now + Duration::hours(2))
        .await
        .unwrap();

    let events = service
        .list_log(
            Some(10),
            Some(now.date_naive()),
            Some(RiskLogEventType::BuyLockReleased),
        )
        .await
        .unwrap();

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, RiskLogEventType::BuyLockReleased);
}
