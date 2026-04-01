use chrono::{DateTime, Utc};
use std::collections::HashMap;

use crate::monitor::MonitorQuoteRow;
use crate::stop::models::{
    StopAnchorSource, StopEvaluationResult, StopEvalState, StopRule, StopStatusRow,
    StopTriggerKind, TriggeredStop,
};

#[derive(Debug, Clone)]
pub(super) struct EvaluatedRuleState {
    pub(super) updated_rule: StopRule,
    pub(super) triggered_stop: Option<TriggeredStop>,
    pub(super) anchor_price: Option<f64>,
    pub(super) anchor_source: Option<StopAnchorSource>,
    pub(super) loss_threshold: Option<f64>,
    pub(super) profit_threshold: Option<f64>,
    pub(super) eval_state: StopEvalState,
}

pub(super) fn evaluate_rule(
    rule: &StopRule,
    current_price: Option<f64>,
    observed_at: DateTime<Utc>,
) -> StopEvaluationResult {
    let evaluated = evaluate_rule_state(rule, current_price, None, observed_at);
    StopEvaluationResult {
        updated_rule: evaluated.updated_rule,
        triggered_stop: evaluated.triggered_stop,
    }
}

pub(super) fn evaluate_rules(
    rules: &[StopRule],
    quote_rows: &[MonitorQuoteRow],
    observed_at: DateTime<Utc>,
) -> Vec<StopEvaluationResult> {
    let quote_map = quote_rows
        .iter()
        .map(|row| (row.code.as_str(), row.last_price))
        .collect::<HashMap<_, _>>();

    rules
        .iter()
        .map(|rule| evaluate_rule(rule, quote_map.get(rule.code.as_str()).copied().flatten(), observed_at))
        .collect()
}

pub(super) fn evaluate_rules_with_anchor_map(
    rules: &[StopRule],
    quote_rows: &[MonitorQuoteRow],
    avg_cost_by_code: &HashMap<String, f64>,
    observed_at: DateTime<Utc>,
) -> Vec<StopEvaluationResult> {
    let quote_map = quote_rows
        .iter()
        .map(|row| (row.code.as_str(), row.last_price))
        .collect::<HashMap<_, _>>();

    rules
        .iter()
        .map(|rule| {
            let current_price = quote_map.get(rule.code.as_str()).copied().flatten();
            let anchor_price = avg_cost_by_code.get(&rule.code).copied();
            let evaluated = evaluate_rule_state(rule, current_price, anchor_price, observed_at);
            StopEvaluationResult {
                updated_rule: evaluated.updated_rule,
                triggered_stop: evaluated.triggered_stop,
            }
        })
        .collect()
}

pub(super) fn status_rows(
    rules: &[StopRule],
    quote_rows: &[MonitorQuoteRow],
    avg_cost_by_code: &HashMap<String, f64>,
    observed_at: DateTime<Utc>,
) -> Vec<StopStatusRow> {
    let quote_map = quote_rows
        .iter()
        .map(|row| (row.code.as_str(), row.last_price))
        .collect::<HashMap<_, _>>();

    rules
        .iter()
        .map(|rule| {
            let current_price = quote_map.get(rule.code.as_str()).copied().flatten();
            let anchor_price = avg_cost_by_code.get(&rule.code).copied();
            let evaluated = evaluate_rule_state(rule, current_price, anchor_price, observed_at);
            StopStatusRow {
                code: rule.code.clone(),
                last_price: current_price,
                anchor_price: evaluated.anchor_price,
                anchor_source: evaluated.anchor_source,
                loss_threshold: evaluated.loss_threshold,
                profit_threshold: evaluated.profit_threshold,
                trailing_pct: rule.trailing_pct,
                highest_price: evaluated.updated_rule.highest_price,
                last_triggered_at: evaluated.updated_rule.last_triggered_at,
                eval_state: evaluated.eval_state,
            }
        })
        .collect()
}

pub(super) fn evaluate_rule_state(
    rule: &StopRule,
    current_price: Option<f64>,
    position_cost: Option<f64>,
    observed_at: DateTime<Utc>,
) -> EvaluatedRuleState {
    let mut updated_rule = rule.clone();
    let has_percent_threshold = rule.stop_loss_pct.is_some() || rule.take_profit_pct.is_some();
    let (anchor_price, anchor_source) = resolve_anchor(rule, position_cost);

    let Some(current_price) = current_price else {
        return EvaluatedRuleState {
            updated_rule,
            triggered_stop: None,
            anchor_price,
            anchor_source,
            loss_threshold: None,
            profit_threshold: None,
            eval_state: StopEvalState::QuoteMissing,
        };
    };

    let trailing_threshold = if let Some(trailing_pct) = updated_rule.trailing_pct {
        let highest_price = updated_rule.highest_price.unwrap_or(current_price);
        let highest_price = highest_price.max(current_price);
        updated_rule.highest_price = Some(highest_price);
        Some(highest_price * (1.0 - trailing_pct / 100.0))
    } else {
        None
    };

    let loss_threshold = if let Some(threshold) = trailing_threshold {
        Some(threshold)
    } else if let Some(threshold) = updated_rule.stop_loss_price {
        Some(threshold)
    } else if let (Some(pct), Some(anchor)) = (updated_rule.stop_loss_pct, anchor_price) {
        Some(anchor * (1.0 - pct / 100.0))
    } else {
        None
    };

    let profit_threshold = if let Some(threshold) = updated_rule.take_profit_price {
        Some(threshold)
    } else if let (Some(pct), Some(anchor)) = (updated_rule.take_profit_pct, anchor_price) {
        Some(anchor * (1.0 + pct / 100.0))
    } else {
        None
    };

    let triggered_stop = if let Some(threshold) = trailing_threshold {
        if current_price <= threshold {
            Some(TriggeredStop {
                code: updated_rule.code.clone(),
                kind: StopTriggerKind::TrailingLoss,
                current_price,
                threshold_price: threshold,
                highest_price: updated_rule.highest_price,
                anchor_price,
                anchor_source,
                triggered_at: Some(observed_at),
            })
        } else {
            None
        }
    } else if let Some(threshold) = loss_threshold {
        if current_price <= threshold {
            Some(TriggeredStop {
                code: updated_rule.code.clone(),
                kind: StopTriggerKind::Loss,
                current_price,
                threshold_price: threshold,
                highest_price: updated_rule.highest_price,
                anchor_price,
                anchor_source,
                triggered_at: Some(observed_at),
            })
        } else {
            None
        }
    } else if let Some(threshold) = profit_threshold {
        if current_price >= threshold {
            Some(TriggeredStop {
                code: updated_rule.code.clone(),
                kind: StopTriggerKind::Profit,
                current_price,
                threshold_price: threshold,
                highest_price: updated_rule.highest_price,
                anchor_price,
                anchor_source,
                triggered_at: Some(observed_at),
            })
        } else {
            None
        }
    } else {
        None
    };

    let eval_state = if triggered_stop.is_some() {
        StopEvalState::Triggered
    } else if has_percent_threshold && anchor_price.is_none() {
        StopEvalState::AnchorMissing
    } else {
        StopEvalState::Armed
    };

    if triggered_stop.is_some() {
        updated_rule.last_triggered_at = Some(observed_at);
        updated_rule.updated_at = observed_at;
    } else if updated_rule.highest_price != rule.highest_price {
        updated_rule.updated_at = observed_at;
    }

    EvaluatedRuleState {
        updated_rule,
        triggered_stop,
        anchor_price,
        anchor_source,
        loss_threshold,
        profit_threshold,
        eval_state,
    }
}

pub(super) fn resolve_anchor(
    rule: &StopRule,
    position_cost: Option<f64>,
) -> (Option<f64>, Option<StopAnchorSource>) {
    if let Some(anchor) = position_cost {
        return (Some(anchor), Some(StopAnchorSource::PositionCost));
    }
    if let Some(anchor) = rule.reference_price {
        return (Some(anchor), Some(StopAnchorSource::ReferencePrice));
    }
    (None, None)
}
