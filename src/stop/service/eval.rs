//! 止损规则评估、合并、校验与历史事件构造的辅助函数。

use crate::core::{QuantixError, Result};
use crate::stop::models::{
    StopAnchorSource, StopEvalState, StopHistoryEvent, StopHistoryEventType, StopRule,
    StopRuleUpdate, StopTriggerKind, TriggeredStop,
};
use crate::stop::service::types::EvaluatedRuleState;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// 评估单条规则：解析锚价，计算移动止损/固定止损/止盈阈值，必要时生成触发事件并更新规则状态。
pub(crate) fn evaluate_rule_state(
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

fn resolve_anchor(
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

pub(crate) fn validate_stop_rule_inputs(
    stop_loss_price: Option<f64>,
    take_profit_price: Option<f64>,
    stop_loss_pct: Option<f64>,
    take_profit_pct: Option<f64>,
    trailing_pct: Option<f64>,
) -> Result<()> {
    if stop_loss_price.is_none()
        && take_profit_price.is_none()
        && stop_loss_pct.is_none()
        && take_profit_pct.is_none()
        && trailing_pct.is_none()
    {
        return Err(QuantixError::Other(
            "stop set 至少需要一个条件：--loss、--profit、--loss-pct、--profit-pct、--trailing"
                .to_string(),
        ));
    }

    if stop_loss_price.is_some() && stop_loss_pct.is_some() {
        return Err(QuantixError::Other(
            "stop set 不能同时指定 --loss 和 --loss-pct".to_string(),
        ));
    }

    if take_profit_price.is_some() && take_profit_pct.is_some() {
        return Err(QuantixError::Other(
            "stop set 不能同时指定 --profit 和 --profit-pct".to_string(),
        ));
    }

    if trailing_pct.is_some() && (stop_loss_price.is_some() || stop_loss_pct.is_some()) {
        return Err(QuantixError::Other(
            "stop set 不能同时指定 --trailing 和 --loss/--loss-pct".to_string(),
        ));
    }

    validate_positive_price("--loss", stop_loss_price)?;
    validate_positive_price("--profit", take_profit_price)?;
    validate_positive_percentage("--loss-pct", stop_loss_pct)?;
    validate_positive_percentage("--profit-pct", take_profit_pct)?;
    validate_trailing_pct(trailing_pct)?;

    Ok(())
}

pub(crate) fn merge_stop_rule_patch(
    existing: StopRule,
    patch: StopRuleUpdate,
    now: DateTime<Utc>,
) -> Result<StopRule> {
    let merged = StopRule {
        code: existing.code,
        stop_loss_price: patch.stop_loss_price.unwrap_or(existing.stop_loss_price),
        take_profit_price: patch
            .take_profit_price
            .unwrap_or(existing.take_profit_price),
        stop_loss_pct: patch.stop_loss_pct.unwrap_or(existing.stop_loss_pct),
        take_profit_pct: patch.take_profit_pct.unwrap_or(existing.take_profit_pct),
        trailing_pct: patch.trailing_pct.unwrap_or(existing.trailing_pct),
        highest_price: existing.highest_price,
        reference_price: patch.reference_price.unwrap_or(existing.reference_price),
        last_triggered_at: existing.last_triggered_at,
        created_at: existing.created_at,
        updated_at: now,
    };

    validate_stop_rule_inputs(
        merged.stop_loss_price,
        merged.take_profit_price,
        merged.stop_loss_pct,
        merged.take_profit_pct,
        merged.trailing_pct,
    )?;

    Ok(merged)
}

pub(crate) fn build_history_event(
    rule: &StopRule,
    event_type: StopHistoryEventType,
    now: DateTime<Utc>,
) -> Result<StopHistoryEvent> {
    Ok(StopHistoryEvent {
        id: Uuid::new_v4().to_string(),
        code: rule.code.clone(),
        event_type,
        trigger_kind: None,
        trigger_price: None,
        anchor_price: rule.reference_price,
        anchor_source: rule.reference_price.map(|_| "reference_price".to_string()),
        snapshot_json: serde_json::to_value(rule)?,
        created_at: now,
    })
}

fn validate_positive_price(flag: &str, value: Option<f64>) -> Result<()> {
    let Some(value) = value else {
        return Ok(());
    };

    if !value.is_finite() || value <= 0.0 {
        return Err(QuantixError::Other(format!(
            "stop set {} 必须是有限正数",
            flag
        )));
    }

    Ok(())
}

fn validate_trailing_pct(value: Option<f64>) -> Result<()> {
    let Some(value) = value else {
        return Ok(());
    };

    if !value.is_finite() || value <= 0.0 || value >= 100.0 {
        return Err(QuantixError::Other(
            "stop set --trailing 必须在 0 到 100 之间".to_string(),
        ));
    }

    Ok(())
}

fn validate_positive_percentage(flag: &str, value: Option<f64>) -> Result<()> {
    let Some(value) = value else {
        return Ok(());
    };

    if !value.is_finite() || value <= 0.0 {
        return Err(QuantixError::Other(format!(
            "stop set {} 必须是有限正数",
            flag
        )));
    }

    Ok(())
}
