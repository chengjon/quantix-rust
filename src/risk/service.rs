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
    BuyLockState, ProjectedBuyImpact, RiskAccountSnapshot, RiskLogEvent, RiskLogEventType,
    RiskRule, RiskRuleType, RiskState, RiskStatus, RuleValue,
};
use crate::risk::storage::JsonRiskStore;
use crate::risk::volatility::{evaluate_volatility_limit, DefaultRiskBarLoader, RiskBarLoader};

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
        Self::with_dependencies(
            store,
            DefaultRiskBarLoader::from_env(),
            None::<IndustryResolver>,
        )
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
        Self::with_dependencies(
            store,
            DefaultRiskBarLoader::from_env(),
            Some(industry_resolver),
        )
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

        let rule = super::service_state::upsert_rule(&mut state, parsed_type, parsed_value, now);
        super::service_state::push_risk_event(
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
        Ok(super::service_state::list_log_events(
            &state, limit, date, event_type,
        ))
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

        if let Some(rule) =
            super::service_state::find_enabled_rule(&state, RiskRuleType::PositionLimit)
        {
            check_position_limit(rule, projected_buy)?;
        }

        if let Some(rule) =
            super::service_state::find_enabled_rule(&state, RiskRuleType::VolatilityLimit).cloned()
        {
            evaluate_volatility_limit(&rule, projected_buy, self.bar_loader.as_ref()).await?;
        }

        // 行业集中度检查（如果有行业映射数据）
        if let Some(rule) =
            super::service_state::find_enabled_rule(&state, RiskRuleType::IndustryLimit)
        {
            check_industry_limit(rule, snapshot, projected_buy)?;
        }

        if let Some(rule) =
            super::service_state::find_enabled_rule(&state, RiskRuleType::IndustryBlocklist)
                .cloned()
        {
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
        let status = super::service_state::sync_after_trade_reset_state(
            &mut state,
            self.event_limit,
            snapshot,
            now,
        );
        self.store.save_state(&state).await?;
        Ok(status)
    }

    pub async fn release_buy_lock(&self, now: DateTime<Utc>) -> Result<BuyLockState> {
        let mut state = self.load_state().await?;
        let released =
            super::service_state::release_buy_lock_state(&mut state, self.event_limit, now)?;
        self.store.save_state(&state).await?;
        Ok(released)
    }

    async fn toggle_rule(
        &self,
        rule_type: &str,
        enabled: bool,
        now: DateTime<Utc>,
    ) -> Result<RiskRule> {
        let mut state = self.load_state().await?;
        let parsed_type = RiskRuleType::parse(rule_type)?;
        let updated = super::service_state::toggle_rule_state(
            &mut state,
            self.event_limit,
            parsed_type,
            enabled,
            now,
        )?;
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
        super::service_state::refresh_state(state, self.event_limit, snapshot, now)
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
    let rule = super::service_state::find_enabled_rule(state, RiskRuleType::AutoReduce)?;

    let baseline = state.daily_baseline.as_ref()?;
    let daily_pnl = snapshot.total_assets - baseline.starting_total_assets;
    let daily_pnl_pct = super::service_state::pct_change(daily_pnl, baseline.starting_total_assets);

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
