use super::*;
use quantix_cli::stop::{StopAnchorSource, StopEvalState, StopTriggerKind};
use std::collections::HashMap;

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
fn evaluate_rule_allows_price_between_fixed_loss_and_profit_thresholds() {
    let service = StopService::new(FakeStopRuleStore::default());
    let mut rule = sample_rule("000001");
    rule.stop_loss_price = Some(14.5);
    rule.take_profit_price = Some(16.5);

    let result = service.evaluate_rule(&rule, Some(15.5), sample_time());

    assert_eq!(result.updated_rule, rule);
    assert_eq!(result.triggered_stop, None);
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

#[test]
fn status_rows_prefer_position_cost_anchor_when_available() {
    let service = StopService::new(FakeStopRuleStore::default());
    let mut rule = sample_rule("000001");
    rule.stop_loss_price = None;
    rule.stop_loss_pct = Some(5.0);
    rule.reference_price = Some(15.2);

    let rows = service.status_rows(
        &[rule],
        &[quote_row("000001", 19.0)],
        &HashMap::from([("000001".to_string(), 20.0)]),
        sample_time(),
    );

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].anchor_source, Some(StopAnchorSource::PositionCost));
    assert_eq!(rows[0].anchor_price, Some(20.0));
    assert_eq!(rows[0].loss_threshold, Some(19.0));
    assert_eq!(rows[0].eval_state, StopEvalState::Triggered);
}

#[test]
fn status_rows_fall_back_to_reference_price_without_position() {
    let service = StopService::new(FakeStopRuleStore::default());
    let mut rule = sample_rule("000001");
    rule.stop_loss_price = None;
    rule.stop_loss_pct = Some(5.0);
    rule.reference_price = Some(15.2);

    let rows = service.status_rows(
        &[rule],
        &[quote_row("000001", 15.0)],
        &HashMap::new(),
        sample_time(),
    );

    assert_eq!(
        rows[0].anchor_source,
        Some(StopAnchorSource::ReferencePrice)
    );
    assert_eq!(rows[0].anchor_price, Some(15.2));
    assert_eq!(rows[0].loss_threshold, Some(14.44));
    assert_eq!(rows[0].eval_state, StopEvalState::Armed);
}

#[test]
fn status_rows_report_loss_trigger_diagnostics_from_position_anchor() {
    let service = StopService::new(FakeStopRuleStore::default());
    let mut rule = sample_rule("000001");
    rule.stop_loss_pct = Some(5.0);

    let rows = service.status_rows(
        &[rule],
        &[quote_row("000001", 14.9)],
        &HashMap::from([("000001".to_string(), 16.0)]),
        sample_time(),
    );

    assert_eq!(rows[0].last_price, Some(14.9));
    assert_eq!(rows[0].anchor_source, Some(StopAnchorSource::PositionCost));
    assert_eq!(rows[0].anchor_price, Some(16.0));
    assert_eq!(rows[0].loss_threshold, Some(15.2));
    assert_eq!(rows[0].profit_threshold, None);
    assert_eq!(rows[0].last_triggered_at, Some(sample_time()));
    assert_eq!(rows[0].eval_state, StopEvalState::Triggered);
}

#[test]
fn status_rows_report_anchor_missing_for_percent_rules_without_any_anchor() {
    let service = StopService::new(FakeStopRuleStore::default());
    let mut rule = sample_rule("000001");
    rule.stop_loss_price = None;
    rule.stop_loss_pct = Some(5.0);
    rule.reference_price = None;

    let rows = service.status_rows(
        &[rule],
        &[quote_row("000001", 15.0)],
        &HashMap::new(),
        sample_time(),
    );

    assert_eq!(rows[0].anchor_source, None);
    assert_eq!(rows[0].anchor_price, None);
    assert_eq!(rows[0].loss_threshold, None);
    assert_eq!(rows[0].eval_state, StopEvalState::AnchorMissing);
}

#[test]
fn status_rows_report_quote_missing_without_failing() {
    let service = StopService::new(FakeStopRuleStore::default());
    let mut rule = sample_rule("000001");
    rule.stop_loss_pct = Some(5.0);

    let rows = service.status_rows(&[rule], &[], &HashMap::new(), sample_time());

    assert_eq!(rows[0].last_price, None);
    assert_eq!(rows[0].eval_state, StopEvalState::QuoteMissing);
}
