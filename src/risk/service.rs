use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::core::{QuantixError, Result};
use crate::risk::models::{
    BuyLockState, DailyRiskBaseline, PositionRiskRow, ProjectedBuyImpact, RiskAccountSnapshot,
    RiskLockStateSource, RiskLogEvent, RiskLogEventType, RiskRule, RiskRuleSnapshot, RiskRuleType,
    RiskState, RiskStatus, RuleValue,
};

const DEFAULT_RISK_EVENT_LIMIT: usize = 100;

#[async_trait]
pub trait RiskStore: Send + Sync {
    async fn load_state(&self) -> Result<Option<RiskState>>;

    async fn save_state(&self, state: &RiskState) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct RiskService<Store> {
    store: Store,
    event_limit: usize,
}

impl<Store> RiskService<Store>
where
    Store: RiskStore,
{
    pub fn new(store: Store) -> Self {
        Self {
            store,
            event_limit: DEFAULT_RISK_EVENT_LIMIT,
        }
    }

    pub async fn set_rule(
        &self,
        rule_type: &str,
        value: &str,
        now: DateTime<Utc>,
    ) -> Result<RiskRule> {
        let mut state = self.load_state().await?;
        let parsed_type = RiskRuleType::parse(rule_type)?;
        let parsed_value = RuleValue::parse(parsed_type, value)?;

        let rule = upsert_rule(&mut state, parsed_type, parsed_value, now);
        push_risk_event(
            &mut state,
            self.event_limit,
            RiskLogEvent {
                ts: now,
                event_type: RiskLogEventType::RuleSet,
                trading_date: None,
                detail: format!("{} = {}", rule.rule_type.as_cli_str(), rule.value.display()),
            },
        );
        self.store.save_state(&state).await?;
        Ok(rule)
    }

    pub async fn list_rules(&self) -> Result<Vec<RiskRule>> {
        let mut rules = self.load_state().await?.rules;
        rules.sort_by_key(|rule| rule.rule_type);
        Ok(rules)
    }

    pub async fn list_log(
        &self,
        limit: Option<usize>,
        date: Option<NaiveDate>,
        event_type: Option<RiskLogEventType>,
    ) -> Result<Vec<RiskLogEvent>> {
        let state = self.load_state().await?;
        let limit = limit.unwrap_or(20);
        Ok(state
            .events
            .iter()
            .rev()
            .filter(|event| {
                date.map(|target| event.ts.date_naive() == target)
                    .unwrap_or(true)
                    && event_type
                        .map(|target| event.event_type == target)
                        .unwrap_or(true)
            })
            .take(limit)
            .cloned()
            .collect())
    }

    pub async fn enable_rule(&self, rule_type: &str, now: DateTime<Utc>) -> Result<RiskRule> {
        self.toggle_rule(rule_type, true, now).await
    }

    pub async fn disable_rule(&self, rule_type: &str, now: DateTime<Utc>) -> Result<RiskRule> {
        self.toggle_rule(rule_type, false, now).await
    }

    pub async fn status(
        &self,
        snapshot: &RiskAccountSnapshot,
        now: DateTime<Utc>,
    ) -> Result<RiskStatus> {
        let mut state = self.load_state().await?;
        let status = self.refresh_state(&mut state, snapshot, now);
        self.store.save_state(&state).await?;
        Ok(status)
    }

    pub async fn check_buy(
        &self,
        snapshot: &RiskAccountSnapshot,
        projected_buy: &ProjectedBuyImpact,
        now: DateTime<Utc>,
    ) -> Result<()> {
        let mut state = self.load_state().await?;
        self.refresh_state(&mut state, snapshot, now);
        self.store.save_state(&state).await?;

        if state.buy_lock.locked {
            return Err(QuantixError::Other(format!(
                "risk buy 已锁定: {}",
                state
                    .buy_lock
                    .reason
                    .clone()
                    .unwrap_or_else(|| "daily-loss-limit 已触发".to_string())
            )));
        }

        if let Some(rule) = find_enabled_rule(&state, RiskRuleType::PositionLimit) {
            check_position_limit(rule, projected_buy)?;
        }

        Ok(())
    }

    pub async fn sync_after_trade_snapshot(
        &self,
        snapshot: &RiskAccountSnapshot,
        now: DateTime<Utc>,
    ) -> Result<RiskStatus> {
        self.status(snapshot, now).await
    }

    pub async fn sync_after_trade_reset(
        &self,
        snapshot: &RiskAccountSnapshot,
        now: DateTime<Utc>,
    ) -> Result<RiskStatus> {
        let mut state = self.load_state().await?;
        state.account_id = snapshot.account_id.clone();
        state.daily_baseline = Some(DailyRiskBaseline {
            trading_date: now.date_naive(),
            starting_total_assets: snapshot.total_assets,
        });
        if state.buy_lock.locked || state.buy_lock.released_for_date.is_some() {
            push_risk_event(
                &mut state,
                self.event_limit,
                RiskLogEvent {
                    ts: now,
                    event_type: RiskLogEventType::BuyLockCleared,
                    trading_date: Some(now.date_naive()),
                    detail: "trade init/reset".to_string(),
                },
            );
        }
        state.buy_lock = BuyLockState::default();

        let status = build_status(&state, snapshot, now.date_naive());
        self.store.save_state(&state).await?;
        Ok(status)
    }

    pub async fn release_buy_lock(&self, now: DateTime<Utc>) -> Result<BuyLockState> {
        let mut state = self.load_state().await?;
        let trading_date = now.date_naive();

        if state.buy_lock.locked {
            let previous_reason = state
                .buy_lock
                .reason
                .clone()
                .unwrap_or_else(|| "manual release".to_string());
            state.buy_lock.locked = false;
            state.buy_lock.released_for_date = Some(trading_date);
            push_risk_event(
                &mut state,
                self.event_limit,
                RiskLogEvent {
                    ts: now,
                    event_type: RiskLogEventType::BuyLockReleased,
                    trading_date: Some(trading_date),
                    detail: previous_reason,
                },
            );
            let released = state.buy_lock.clone();
            self.store.save_state(&state).await?;
            return Ok(released);
        }

        if state.buy_lock.released_for_date == Some(trading_date) {
            return Ok(state.buy_lock.clone());
        }

        Err(QuantixError::Other("当前无活动买入锁".to_string()))
    }

    async fn toggle_rule(
        &self,
        rule_type: &str,
        enabled: bool,
        now: DateTime<Utc>,
    ) -> Result<RiskRule> {
        let mut state = self.load_state().await?;
        let parsed_type = RiskRuleType::parse(rule_type)?;
        let rule = state
            .rules
            .iter_mut()
            .find(|rule| rule.rule_type == parsed_type)
            .ok_or_else(|| {
                QuantixError::Other(format!("risk rule {} 尚未配置", parsed_type.as_cli_str()))
            })?;

        rule.enabled = enabled;
        rule.updated_at = now;

        let updated = rule.clone();
        push_risk_event(
            &mut state,
            self.event_limit,
            RiskLogEvent {
                ts: now,
                event_type: if enabled {
                    RiskLogEventType::RuleEnabled
                } else {
                    RiskLogEventType::RuleDisabled
                },
                trading_date: None,
                detail: parsed_type.as_cli_str().to_string(),
            },
        );
        self.store.save_state(&state).await?;
        Ok(updated)
    }

    async fn load_state(&self) -> Result<RiskState> {
        Ok(self.store.load_state().await?.unwrap_or_default())
    }

    fn refresh_state(
        &self,
        state: &mut RiskState,
        snapshot: &RiskAccountSnapshot,
        now: DateTime<Utc>,
    ) -> RiskStatus {
        let trading_date = now.date_naive();
        state.account_id = snapshot.account_id.clone();

        let baseline_needs_reset = state
            .daily_baseline
            .as_ref()
            .map(|baseline| baseline.trading_date != trading_date)
            .unwrap_or(true);

        if baseline_needs_reset {
            if state.buy_lock.locked || state.buy_lock.released_for_date.is_some() {
                push_risk_event(
                    state,
                    self.event_limit,
                    RiskLogEvent {
                        ts: now,
                        event_type: RiskLogEventType::BuyLockCleared,
                        trading_date: Some(trading_date),
                        detail: "day rollover".to_string(),
                    },
                );
            }
            state.daily_baseline = Some(DailyRiskBaseline {
                trading_date,
                starting_total_assets: snapshot.total_assets,
            });
            state.buy_lock = BuyLockState::default();
        }

        if let Some(rule) = find_enabled_rule(state, RiskRuleType::DailyLossLimit).cloned() {
            apply_daily_loss_rule(state, self.event_limit, &rule, snapshot.total_assets, now);
        }

        build_status(state, snapshot, trading_date)
    }
}

fn upsert_rule(
    state: &mut RiskState,
    rule_type: RiskRuleType,
    value: RuleValue,
    now: DateTime<Utc>,
) -> RiskRule {
    if let Some(existing) = state
        .rules
        .iter_mut()
        .find(|rule| rule.rule_type == rule_type)
    {
        existing.value = value;
        existing.updated_at = now;
        return existing.clone();
    }

    let rule = RiskRule {
        rule_type,
        value,
        enabled: true,
        created_at: now,
        updated_at: now,
    };
    state.rules.push(rule.clone());
    state.rules.sort_by_key(|item| item.rule_type);
    rule
}

fn find_enabled_rule(state: &RiskState, rule_type: RiskRuleType) -> Option<&RiskRule> {
    state
        .rules
        .iter()
        .find(|rule| rule.rule_type == rule_type && rule.enabled)
}

fn apply_daily_loss_rule(
    state: &mut RiskState,
    event_limit: usize,
    rule: &RiskRule,
    current_total_assets: Decimal,
    now: DateTime<Utc>,
) {
    let baseline = state.daily_baseline.as_ref().expect("baseline initialized");
    let daily_pnl = current_total_assets - baseline.starting_total_assets;
    let daily_pnl_pct = pct_change(daily_pnl, baseline.starting_total_assets);

    let triggered = match rule.value {
        RuleValue::Amount(limit) => daily_pnl <= -limit,
        RuleValue::Percentage(limit_pct) => daily_pnl_pct <= -limit_pct,
    };

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
}

fn check_position_limit(rule: &RiskRule, projected_buy: &ProjectedBuyImpact) -> Result<()> {
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

fn build_status(
    state: &RiskState,
    snapshot: &RiskAccountSnapshot,
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
    }
}

fn pct_change(numerator: Decimal, denominator: Decimal) -> Decimal {
    if denominator.is_zero() {
        Decimal::ZERO
    } else {
        numerator / denominator * dec!(100)
    }
}

fn push_risk_event(state: &mut RiskState, event_limit: usize, event: RiskLogEvent) {
    state.events.push(event);
    if state.events.len() > event_limit {
        let overflow = state.events.len() - event_limit;
        state.events.drain(0..overflow);
    }
}
