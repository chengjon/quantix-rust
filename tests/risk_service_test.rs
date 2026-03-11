use async_trait::async_trait;
use chrono::{DateTime, Duration, TimeZone, Utc};
use quantix_cli::core::Result;
use quantix_cli::risk::{
    ProjectedBuyImpact, RiskAccountSnapshot, RiskRuleType, RiskService, RiskState, RiskStore,
    RuleValue,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
struct FakeRiskStore {
    state: Arc<Mutex<Option<RiskState>>>,
}

impl FakeRiskStore {
    fn snapshot(&self) -> Option<RiskState> {
        self.state.lock().unwrap().clone()
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

fn fixed_ts() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 11, 9, 35, 0).unwrap()
}

fn service() -> (RiskService<FakeRiskStore>, FakeRiskStore) {
    let store = FakeRiskStore::default();
    (RiskService::new(store.clone()), store)
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
