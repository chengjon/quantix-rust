use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use quantix_cli::core::Result;
use quantix_cli::monitor::MonitorQuoteRow;
use quantix_cli::stop::{StopRule, StopRuleStore, StopService, StopTriggerKind};
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
struct FakeStopRuleStore {
    state: Arc<Mutex<FakeStopRuleStoreState>>,
}

#[derive(Default)]
struct FakeStopRuleStoreState {
    rules: Vec<StopRule>,
}

#[async_trait]
impl StopRuleStore for FakeStopRuleStore {
    async fn upsert_rule(&self, rule: StopRule) -> Result<StopRule> {
        let mut state = self.state.lock().unwrap();
        if let Some(existing) = state
            .rules
            .iter_mut()
            .find(|existing| existing.code == rule.code)
        {
            *existing = rule.clone();
        } else {
            state.rules.push(rule.clone());
        }
        Ok(rule)
    }

    async fn list_rules(&self) -> Result<Vec<StopRule>> {
        Ok(self.state.lock().unwrap().rules.clone())
    }

    async fn remove_rule(&self, code: &str) -> Result<bool> {
        let mut state = self.state.lock().unwrap();
        let before = state.rules.len();
        state.rules.retain(|rule| rule.code != code);
        Ok(before != state.rules.len())
    }
}

fn sample_time() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 11, 11, 0, 0).unwrap()
}

fn sample_rule(code: &str) -> StopRule {
    StopRule {
        code: code.to_string(),
        stop_loss_price: None,
        take_profit_price: None,
        trailing_pct: None,
        highest_price: None,
        last_triggered_at: None,
        created_at: sample_time(),
        updated_at: sample_time(),
    }
}

fn quote_row(code: &str, last_price: f64) -> MonitorQuoteRow {
    MonitorQuoteRow {
        code: code.to_string(),
        group: "core".to_string(),
        tags: Vec::new(),
        last_price: Some(last_price),
        change_pct: None,
        quote_time: Some(sample_time()),
        note: None,
    }
}

#[tokio::test]
async fn set_rule_persists_fixed_loss_rule() {
    let service = StopService::new(FakeStopRuleStore::default());

    let rule = service
        .set_rule("000001", Some(14.5), None, None, sample_time())
        .await
        .unwrap();

    assert_eq!(rule.code, "000001");
    assert_eq!(rule.stop_loss_price, Some(14.5));
    assert_eq!(rule.take_profit_price, None);
    assert_eq!(rule.trailing_pct, None);
    assert_eq!(rule.highest_price, None);
}

#[tokio::test]
async fn set_rule_persists_fixed_profit_rule() {
    let service = StopService::new(FakeStopRuleStore::default());

    let rule = service
        .set_rule("000001", None, Some(18.0), None, sample_time())
        .await
        .unwrap();

    assert_eq!(rule.stop_loss_price, None);
    assert_eq!(rule.take_profit_price, Some(18.0));
    assert_eq!(rule.trailing_pct, None);
}

#[tokio::test]
async fn set_rule_persists_trailing_rule() {
    let service = StopService::new(FakeStopRuleStore::default());

    let rule = service
        .set_rule("000001", None, None, Some(5.0), sample_time())
        .await
        .unwrap();

    assert_eq!(rule.stop_loss_price, None);
    assert_eq!(rule.take_profit_price, None);
    assert_eq!(rule.trailing_pct, Some(5.0));
}

#[tokio::test]
async fn set_rule_rejects_invalid_threshold_values() {
    let service = StopService::new(FakeStopRuleStore::default());

    let loss_err = service
        .set_rule("000001", Some(0.0), None, None, sample_time())
        .await
        .unwrap_err();
    assert!(loss_err.to_string().contains("--loss 必须是有限正数"));

    let profit_err = service
        .set_rule("000001", None, Some(-1.0), None, sample_time())
        .await
        .unwrap_err();
    assert!(profit_err.to_string().contains("--profit 必须是有限正数"));

    let trailing_err = service
        .set_rule("000001", None, None, Some(150.0), sample_time())
        .await
        .unwrap_err();
    assert!(trailing_err.to_string().contains("--trailing 必须在 0 到 100 之间"));
}

#[tokio::test]
async fn set_rule_rejects_non_finite_threshold_values() {
    let service = StopService::new(FakeStopRuleStore::default());

    let loss_err = service
        .set_rule("000001", Some(f64::NAN), None, None, sample_time())
        .await
        .unwrap_err();
    assert!(loss_err.to_string().contains("--loss 必须是有限正数"));

    let profit_err = service
        .set_rule("000001", None, Some(f64::INFINITY), None, sample_time())
        .await
        .unwrap_err();
    assert!(profit_err.to_string().contains("--profit 必须是有限正数"));

    let trailing_err = service
        .set_rule("000001", None, None, Some(f64::NEG_INFINITY), sample_time())
        .await
        .unwrap_err();
    assert!(trailing_err.to_string().contains("--trailing 必须在 0 到 100 之间"));
}

#[tokio::test]
async fn list_rules_returns_store_rules() {
    let store = FakeStopRuleStore::default();
    store
        .state
        .lock()
        .unwrap()
        .rules
        .extend([sample_rule("000001"), sample_rule("000002")]);
    let service = StopService::new(store);

    let rules = service.list_rules().await.unwrap();

    assert_eq!(rules.len(), 2);
    assert_eq!(rules[0].code, "000001");
    assert_eq!(rules[1].code, "000002");
}

#[tokio::test]
async fn remove_rule_deletes_rule_from_store() {
    let store = FakeStopRuleStore::default();
    store
        .state
        .lock()
        .unwrap()
        .rules
        .push(sample_rule("000001"));
    let service = StopService::new(store.clone());

    let removed = service.remove_rule("000001").await.unwrap();

    assert!(removed);
    assert!(store.state.lock().unwrap().rules.is_empty());
}

#[test]
fn evaluate_rule_detects_fixed_loss_trigger() {
    let service = StopService::new(FakeStopRuleStore::default());
    let mut rule = sample_rule("000001");
    rule.stop_loss_price = Some(14.5);

    let result = service.evaluate_rule(&rule, Some(14.2), sample_time());

    assert_eq!(result.updated_rule.stop_loss_price, Some(14.5));
    assert_eq!(result.triggered_stop.unwrap().kind, StopTriggerKind::Loss);
}

#[test]
fn evaluate_rule_detects_fixed_profit_trigger() {
    let service = StopService::new(FakeStopRuleStore::default());
    let mut rule = sample_rule("000001");
    rule.take_profit_price = Some(18.0);

    let result = service.evaluate_rule(&rule, Some(18.2), sample_time());

    assert_eq!(result.triggered_stop.unwrap().kind, StopTriggerKind::Profit);
}

#[test]
fn evaluate_rule_updates_trailing_highest_price() {
    let service = StopService::new(FakeStopRuleStore::default());
    let mut rule = sample_rule("000001");
    rule.trailing_pct = Some(5.0);
    rule.highest_price = Some(15.0);

    let result = service.evaluate_rule(&rule, Some(16.2), sample_time());

    assert_eq!(result.updated_rule.highest_price, Some(16.2));
    assert_eq!(result.triggered_stop, None);
}

#[test]
fn evaluate_rule_triggers_trailing_stop_after_drawdown() {
    let service = StopService::new(FakeStopRuleStore::default());
    let mut rule = sample_rule("000001");
    rule.trailing_pct = Some(5.0);
    rule.highest_price = Some(20.0);

    let result = service.evaluate_rule(&rule, Some(18.8), sample_time());

    let triggered = result.triggered_stop.unwrap();
    assert_eq!(triggered.kind, StopTriggerKind::TrailingLoss);
    assert_eq!(triggered.threshold_price, 19.0);
    assert_eq!(triggered.highest_price, Some(20.0));
}

#[test]
fn evaluate_rule_with_missing_quote_produces_no_trigger() {
    let service = StopService::new(FakeStopRuleStore::default());
    let mut rule = sample_rule("000001");
    rule.stop_loss_price = Some(14.5);

    let result = service.evaluate_rule(&rule, None, sample_time());

    assert_eq!(result.updated_rule, rule);
    assert_eq!(result.triggered_stop, None);
}

#[test]
fn evaluate_rules_matches_quotes_by_code() {
    let service = StopService::new(FakeStopRuleStore::default());
    let mut loss_rule = sample_rule("000001");
    loss_rule.stop_loss_price = Some(14.5);
    let mut profit_rule = sample_rule("000002");
    profit_rule.take_profit_price = Some(18.0);

    let results = service.evaluate_rules(
        &[loss_rule, profit_rule],
        &[quote_row("000001", 14.2), quote_row("000002", 18.3)],
        sample_time(),
    );

    assert_eq!(results.len(), 2);
    assert_eq!(
        results[0]
            .triggered_stop
            .as_ref()
            .map(|trigger| trigger.kind),
        Some(StopTriggerKind::Loss)
    );
    assert_eq!(
        results[1]
            .triggered_stop
            .as_ref()
            .map(|trigger| trigger.kind),
        Some(StopTriggerKind::Profit)
    );
}
