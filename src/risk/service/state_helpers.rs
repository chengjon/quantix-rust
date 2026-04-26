use super::*;

pub(super) fn find_enabled_rule(state: &RiskState, rule_type: RiskRuleType) -> Option<&RiskRule> {
    state
        .rules
        .iter()
        .find(|rule| rule.rule_type == rule_type && rule.enabled)
}

pub(super) fn apply_daily_loss_rule(
    state: &mut RiskState,
    event_limit: usize,
    rule: &RiskRule,
    current_total_assets: Decimal,
    now: DateTime<Utc>,
) -> Result<()> {
    let baseline = state.daily_baseline.as_ref().expect("baseline initialized");
    let daily_pnl = current_total_assets - baseline.starting_total_assets;
    let daily_pnl_pct = pct_change(daily_pnl, baseline.starting_total_assets);

    let triggered = evaluate_daily_loss_rule_triggered(rule, daily_pnl, daily_pnl_pct)?;

    if triggered
        && !state.buy_lock.locked
        && state.buy_lock.released_for_date != Some(now.date_naive())
    {
        let reason = format!("daily-loss-limit {} 已触发", rule.value.display());
        state.buy_lock = BuyLockState {
            locked: true,
            reason: Some(reason.clone()),
            triggered_at: Some(now),
            trading_date: Some(now.date_naive()),
            released_for_date: None,
        };
        push_risk_event(
            state,
            event_limit,
            RiskLogEvent {
                ts: now,
                event_type: RiskLogEventType::DailyLossLockTriggered,
                trading_date: Some(now.date_naive()),
                detail: reason,
            },
        );
    }

    Ok(())
}

fn evaluate_daily_loss_rule_triggered(
    rule: &RiskRule,
    daily_pnl: Decimal,
    daily_pnl_pct: Decimal,
) -> Result<bool> {
    match &rule.value {
        RuleValue::Amount(limit) => Ok(daily_pnl <= -*limit),
        RuleValue::Percentage(limit_pct) => Ok(daily_pnl_pct <= -*limit_pct),
        RuleValue::TextList(_) => Err(QuantixError::Other(
            "risk rule daily-loss-limit 配置无效".to_string(),
        )),
    }
}

pub(super) fn check_position_limit(
    rule: &RiskRule,
    projected_buy: &ProjectedBuyImpact,
) -> Result<()> {
    let RuleValue::Percentage(limit_pct) = rule.value.clone() else {
        return Err(QuantixError::Other(
            "risk rule position-limit 配置无效".to_string(),
        ));
    };

    if projected_buy.projected_total_assets <= Decimal::ZERO {
        return Err(QuantixError::Other(
            "risk check projected_total_assets 必须大于 0".to_string(),
        ));
    }

    let projected_ratio_pct =
        projected_buy.projected_position_value / projected_buy.projected_total_assets * dec!(100);
    if projected_ratio_pct > limit_pct {
        return Err(QuantixError::Other(format!(
            "risk rule position-limit 已超限: {} 预计仓位 {}%",
            limit_pct, projected_ratio_pct
        )));
    }

    Ok(())
}

pub(super) fn build_status(
    state: &RiskState,
    snapshot: &RiskAccountSnapshot,
    now: DateTime<Utc>,
    trading_date: chrono::NaiveDate,
) -> RiskStatus {
    let baseline = state.daily_baseline.as_ref().expect("baseline initialized");
    let current_total_assets = snapshot.total_assets;
    let daily_pnl = current_total_assets - baseline.starting_total_assets;
    let daily_pnl_pct = pct_change(daily_pnl, baseline.starting_total_assets);

    let mut position_ratios = snapshot
        .positions
        .iter()
        .map(|position| PositionRiskRow {
            code: position.code.clone(),
            market_value: position.market_value,
            ratio_pct: pct_change(position.market_value, current_total_assets),
        })
        .collect::<Vec<_>>();
    position_ratios.sort_by(|left, right| left.code.cmp(&right.code));

    let rules = state
        .rules
        .iter()
        .map(|rule| RiskRuleSnapshot {
            rule_type: rule.rule_type,
            value: rule.value.clone(),
            enabled: rule.enabled,
        })
        .collect();
    let auto_reduce_recommendation = build_auto_reduce_recommendation(state, snapshot, now);

    let manual_release_active =
        !state.buy_lock.locked && state.buy_lock.released_for_date == Some(trading_date);
    let lock_state_source = if state.buy_lock.locked {
        RiskLockStateSource::DailyLossLocked
    } else if manual_release_active {
        RiskLockStateSource::ManualReleaseActive
    } else {
        RiskLockStateSource::Open
    };
    let lock_trigger_reason = match lock_state_source {
        RiskLockStateSource::Open => None,
        RiskLockStateSource::DailyLossLocked | RiskLockStateSource::ManualReleaseActive => {
            state.buy_lock.reason.clone()
        }
    };
    let lock_triggered_at = match lock_state_source {
        RiskLockStateSource::Open => None,
        RiskLockStateSource::DailyLossLocked | RiskLockStateSource::ManualReleaseActive => {
            state.buy_lock.triggered_at
        }
    };
    let lock_effective_trading_date = match lock_state_source {
        RiskLockStateSource::Open => None,
        RiskLockStateSource::DailyLossLocked => state.buy_lock.trading_date,
        RiskLockStateSource::ManualReleaseActive => state
            .buy_lock
            .released_for_date
            .or(state.buy_lock.trading_date),
    };

    RiskStatus {
        account_id: state.account_id.clone(),
        trading_date,
        starting_total_assets: baseline.starting_total_assets,
        current_total_assets,
        daily_pnl,
        daily_pnl_pct,
        buy_locked: state.buy_lock.locked,
        manual_release_active,
        lock_state_source,
        lock_reason: state.buy_lock.reason.clone(),
        lock_trigger_reason,
        lock_triggered_at,
        lock_effective_trading_date,
        position_ratios,
        rules,
        auto_reduce_recommendation,
    }
}

pub(super) fn build_auto_reduce_recommendation(
    state: &RiskState,
    snapshot: &RiskAccountSnapshot,
    now: DateTime<Utc>,
) -> Option<AutoReduceRecommendation> {
    let decision = check_auto_reduce_trigger(state, snapshot, now)?;
    let mut position_codes = decision
        .positions_to_reduce
        .iter()
        .map(|position| position.code.clone())
        .collect::<Vec<_>>();
    position_codes.sort();
    position_codes.dedup();

    Some(AutoReduceRecommendation {
        current_loss_pct: decision.current_loss_pct,
        reduce_ratio: decision.reduce_ratio,
        position_codes,
        triggered_at: decision.triggered_at,
    })
}

pub(super) fn pct_change(numerator: Decimal, denominator: Decimal) -> Decimal {
    if denominator.is_zero() {
        Decimal::ZERO
    } else {
        numerator / denominator * dec!(100)
    }
}

pub(super) fn push_risk_event(state: &mut RiskState, event_limit: usize, event: RiskLogEvent) {
    state.events.push(event);
    if state.events.len() > event_limit {
        let overflow = state.events.len() - event_limit;
        state.events.drain(0..overflow);
    }
}
