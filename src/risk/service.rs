use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::core::{QuantixError, Result};
use crate::risk::industry::{IndustryResolver, ResolvedIndustry};
use crate::risk::industry_store::SqliteIndustryStore;
use crate::risk::models::{
    BuyLockState, DailyRiskBaseline, PositionRiskRow, ProjectedBuyImpact, RiskAccountSnapshot,
    RiskLockStateSource, RiskLogEvent, RiskLogEventType, RiskRule, RiskRuleSnapshot, RiskRuleType,
    RiskState, RiskStatus, RuleValue,
};
use crate::risk::storage::JsonRiskStore;
use crate::risk::volatility::{DefaultRiskBarLoader, RiskBarLoader, evaluate_volatility_limit};

const DEFAULT_RISK_EVENT_LIMIT: usize = 100;

#[async_trait]
pub trait RiskStore: Send + Sync {
    async fn load_state(&self) -> Result<Option<RiskState>>;

    async fn save_state(&self, state: &RiskState) -> Result<()>;
}

#[async_trait]
pub trait RiskIndustryResolver: Send + Sync + std::fmt::Debug {
    async fn resolve(
        &self,
        code: &str,
        query_date: NaiveDate,
        captured_at: DateTime<Utc>,
    ) -> Result<ResolvedIndustry>;
}

#[async_trait]
impl RiskIndustryResolver for IndustryResolver {
    async fn resolve(
        &self,
        code: &str,
        query_date: NaiveDate,
        captured_at: DateTime<Utc>,
    ) -> Result<ResolvedIndustry> {
        IndustryResolver::resolve(self, code, query_date, captured_at).await
    }
}

#[derive(Debug, Clone)]
pub struct RiskService<Store> {
    store: Store,
    event_limit: usize,
    bar_loader: Arc<dyn RiskBarLoader>,
    industry_resolver: Option<Arc<dyn RiskIndustryResolver>>,
}

#[derive(Debug, Clone)]
pub struct RuntimeJsonRiskServices {
    store: JsonRiskStore,
    base: RiskService<JsonRiskStore>,
    buy_checks: Arc<Mutex<Option<RiskService<JsonRiskStore>>>>,
}

impl RuntimeJsonRiskServices {
    pub fn new(store: JsonRiskStore) -> Self {
        Self {
            base: RiskService::new(store.clone()),
            store,
            buy_checks: Arc::new(Mutex::new(None)),
        }
    }

    pub fn base(&self) -> &RiskService<JsonRiskStore> {
        &self.base
    }

    pub async fn buy_checks(&self) -> Result<RiskService<JsonRiskStore>> {
        let mut guard = self.buy_checks.lock().await;
        if let Some(service) = guard.as_ref() {
            return Ok(service.clone());
        }

        let service = RiskService::from_json_store(self.store.clone()).await?;
        *guard = Some(service.clone());
        Ok(service)
    }
}

impl<Store> RiskService<Store>
where
    Store: RiskStore,
{
    pub fn new(store: Store) -> Self {
        Self::with_dependencies(store, DefaultRiskBarLoader::from_env(), None::<IndustryResolver>)
    }

    pub fn with_bar_loader<Loader>(store: Store, bar_loader: Loader) -> Self
    where
        Loader: RiskBarLoader + 'static,
    {
        Self::with_dependencies(store, bar_loader, None::<IndustryResolver>)
    }

    pub fn with_industry_resolver<Resolver>(store: Store, industry_resolver: Resolver) -> Self
    where
        Resolver: RiskIndustryResolver + 'static,
    {
        Self::with_dependencies(store, DefaultRiskBarLoader::from_env(), Some(industry_resolver))
    }

    pub fn with_bar_loader_and_industry_resolver<Loader, Resolver>(
        store: Store,
        bar_loader: Loader,
        industry_resolver: Resolver,
    ) -> Self
    where
        Loader: RiskBarLoader + 'static,
        Resolver: RiskIndustryResolver + 'static,
    {
        Self::with_dependencies(store, bar_loader, Some(industry_resolver))
    }

    pub fn with_dependencies<Loader, Resolver>(
        store: Store,
        bar_loader: Loader,
        industry_resolver: Option<Resolver>,
    ) -> Self
    where
        Loader: RiskBarLoader + 'static,
        Resolver: RiskIndustryResolver + 'static,
    {
        Self {
            store,
            event_limit: DEFAULT_RISK_EVENT_LIMIT,
            bar_loader: Arc::new(bar_loader),
            industry_resolver: industry_resolver.map(|resolver| Arc::new(resolver) as _),
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
        let status = self.refresh_state(&mut state, snapshot, now)?;
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
        self.refresh_state(&mut state, snapshot, now)?;
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

        if let Some(rule) = find_enabled_rule(&state, RiskRuleType::VolatilityLimit).cloned() {
            evaluate_volatility_limit(&rule, projected_buy, self.bar_loader.as_ref()).await?;
        }

        // 行业集中度检查（如果有行业映射数据）
        if let Some(rule) = find_enabled_rule(&state, RiskRuleType::IndustryLimit) {
            check_industry_limit(rule, snapshot, projected_buy)?;
        }

        if let Some(rule) = find_enabled_rule(&state, RiskRuleType::IndustryBlocklist).cloned() {
            evaluate_industry_blocklist(
                &rule,
                projected_buy,
                self.industry_resolver.as_deref(),
                now,
            )
            .await?;
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
    ) -> Result<RiskStatus> {
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
            apply_daily_loss_rule(state, self.event_limit, &rule, snapshot.total_assets, now)?;
        }

        Ok(build_status(state, snapshot, trading_date))
    }
}

impl RiskService<JsonRiskStore> {
    pub async fn from_json_store(store: JsonRiskStore) -> Result<Self> {
        let industry_store = SqliteIndustryStore::from_risk_state_path(store.path()).await?;
        Ok(Self::with_industry_resolver(
            store,
            IndustryResolver::new(industry_store),
        ))
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

async fn evaluate_industry_blocklist(
    rule: &RiskRule,
    projected_buy: &ProjectedBuyImpact,
    resolver: Option<&dyn RiskIndustryResolver>,
    now: DateTime<Utc>,
) -> Result<()> {
    let RuleValue::TextList(blocked_industries) = &rule.value else {
        return Err(QuantixError::Other(
            "risk rule industry-blocklist 配置无效".to_string(),
        ));
    };

    let resolver = resolver.ok_or_else(|| {
        QuantixError::Config(format!(
            "risk rule industry-blocklist 检查失败: code={} 原因=未配置行业解析器",
            projected_buy.code
        ))
    })?;

    let resolved = resolver
        .resolve(&projected_buy.code, now.date_naive(), now)
        .await
        .map_err(|err| {
            QuantixError::DataSource(format!(
                "risk rule industry-blocklist 检查失败: code={} 原因={}",
                projected_buy.code, err
            ))
        })?;

    if blocked_industries
        .iter()
        .any(|industry_name| industry_name == &resolved.industry_name)
    {
        return Err(QuantixError::Other(format!(
            "risk rule industry-blocklist 已拒绝: code={} industry={} blocked={}",
            resolved.code,
            resolved.industry_name,
            blocked_industries.join(",")
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

/// 行业集中度检查
///
/// 检查买入后单一行业的持仓占比是否超过限制
/// 注意：此功能需要行业映射数据支持，目前为占位实现
fn check_industry_limit(
    rule: &RiskRule,
    _snapshot: &RiskAccountSnapshot,
    projected_buy: &ProjectedBuyImpact,
) -> Result<()> {
    let RuleValue::Percentage(limit_pct) = rule.value.clone() else {
        return Err(QuantixError::Other(
            "risk rule industry-limit 配置无效，仅支持百分比".to_string(),
        ));
    };

    if projected_buy.projected_total_assets <= Decimal::ZERO {
        return Err(QuantixError::Other(
            "risk check projected_total_assets 必须大于 0".to_string(),
        ));
    }

    // TODO: 实现行业映射和集中度计算
    // 当前为占位实现，需要集成行业分类数据源
    // 步骤：
    // 1. 根据 projected_buy.code 获取股票所属行业
    // 2. 计算该行业在当前持仓中的总市值
    // 3. 加上本次买入的预计市值
    // 4. 检查是否超过限制

    // 暂时记录日志，不阻止交易
    tracing::debug!(
        "industry-limit check: code={}, limit={}%, placeholder implementation",
        projected_buy.code,
        limit_pct
    );

    Ok(())
}

/// 自动减仓触发检查
///
/// 检查是否需要触发自动减仓规则
/// 返回需要减仓的股票列表和减仓比例
pub fn check_auto_reduce_trigger(
    state: &RiskState,
    snapshot: &RiskAccountSnapshot,
    now: DateTime<Utc>,
) -> Option<AutoReduceDecision> {
    let rule = find_enabled_rule(state, RiskRuleType::AutoReduce)?;

    let baseline = state.daily_baseline.as_ref()?;
    let daily_pnl = snapshot.total_assets - baseline.starting_total_assets;
    let daily_pnl_pct = pct_change(daily_pnl, baseline.starting_total_assets);

    let triggered = match rule.value.clone() {
        RuleValue::Percentage(limit_pct) => daily_pnl_pct <= -limit_pct,
        RuleValue::Amount(limit) => daily_pnl <= -limit,
        RuleValue::TextList(_) => false,
    };

    if triggered {
        // 计算需要减仓的比例（简化实现：减仓50%）
        let reduce_ratio = dec!(50);
        Some(AutoReduceDecision {
            trigger_rule: rule.clone(),
            current_loss_pct: daily_pnl_pct,
            reduce_ratio,
            positions_to_reduce: snapshot.positions.clone(),
            triggered_at: now,
        })
    } else {
        None
    }
}

/// 自动减仓决策
#[derive(Debug, Clone)]
pub struct AutoReduceDecision {
    /// 触发的规则
    pub trigger_rule: RiskRule,
    /// 当前亏损百分比
    pub current_loss_pct: Decimal,
    /// 减仓比例（百分比）
    pub reduce_ratio: Decimal,
    /// 需要减仓的持仓列表
    pub positions_to_reduce: Vec<crate::risk::models::RiskPositionSnapshot>,
    /// 触发时间
    pub triggered_at: DateTime<Utc>,
}
