#[path = "service/industry_checks.rs"]
mod industry_checks;
mod state_helpers;

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::sync::Arc;
use tokio::sync::Mutex;

use self::industry_checks::{evaluate_industry_blocklist, evaluate_industry_limit};
use self::state_helpers::{
    apply_daily_loss_rule, build_status, check_position_limit, find_enabled_rule, push_risk_event,
};
use crate::core::{QuantixError, Result};
use crate::risk::industry::ResolvedIndustry;
use crate::risk::industry_resolver::IndustryResolver;
use crate::risk::industry_store::SqliteIndustryStore;
use crate::risk::models::{
    AutoReduceRecommendation, BuyLockState, DailyRiskBaseline, PositionRiskRow, ProjectedBuyImpact,
    RiskAccountSnapshot, RiskLockStateSource, RiskLogEvent, RiskLogEventType, RiskRule,
    RiskRuleSnapshot, RiskRuleType, RiskState, RiskStatus, RuleValue,
};
use crate::risk::storage::JsonRiskStore;
use crate::risk::volatility::{DefaultRiskBarLoader, RiskBarLoader, evaluate_volatility_limit};

pub use self::industry_checks::{AutoReduceDecision, check_auto_reduce_trigger};

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

/// 风控服务：管理规则、计算实时持仓比例、生成日亏损 / 行业集中度等告警。
#[derive(Debug, Clone)]
pub struct RiskService<Store> {
    store: Store,
    event_limit: usize,
    bar_loader: Arc<dyn RiskBarLoader>,
    industry_resolver: Option<Arc<dyn RiskIndustryResolver>>,
}

/// 运行时复合服务：持有 base 与惰性构建的 buy_checks 两份 [`RiskService`] 实例。
#[derive(Debug, Clone)]
pub struct RuntimeJsonRiskServices {
    store: JsonRiskStore,
    base: RiskService<JsonRiskStore>,
    buy_checks: Arc<Mutex<Option<RiskService<JsonRiskStore>>>>,
}

impl RuntimeJsonRiskServices {
    /// 用 base 服务 + 共享 store 构造，buy_checks 在首次访问时惰性初始化。
    pub fn new(store: JsonRiskStore) -> Self {
        Self {
            base: RiskService::new(store.clone()),
            store,
            buy_checks: Arc::new(Mutex::new(None)),
        }
    }

    /// 返回基础风控服务（只读、规则管理）。
    pub fn base(&self) -> &RiskService<JsonRiskStore> {
        &self.base
    }

    /// 返回带完整依赖（bar_loader / industry_resolver）的买入校验服务。
    ///
    /// 首次调用时从 store 重建并缓存，后续调用复用缓存实例。
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
    /// 用默认 bar_loader 与无行业解析器构造服务。
    pub fn new(store: Store) -> Self {
        Self::with_dependencies(
            store,
            DefaultRiskBarLoader::from_env(),
            None::<IndustryResolver>,
        )
    }

    /// 用自定义 bar_loader 构造（无行业解析器）。
    pub fn with_bar_loader<Loader>(store: Store, bar_loader: Loader) -> Self
    where
        Loader: RiskBarLoader + 'static,
    {
        Self::with_dependencies(store, bar_loader, None::<IndustryResolver>)
    }

    /// 用自定义行业解析器构造（bar_loader 用默认）。
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

    /// 同时指定 bar_loader 与行业解析器。
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

    /// 完整依赖注入入口，其他构造函数都委托到这里。
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

    /// 创建或更新规则，并写入 `RuleSet` 日志事件。
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

    /// 列出所有规则（按类型升序）。
    pub async fn list_rules(&self) -> Result<Vec<RiskRule>> {
        let mut rules = self.load_state().await?.rules;
        rules.sort_by_key(|rule| rule.rule_type);
        Ok(rules)
    }

    /// 查询日志事件，按时间倒序、可按日期 / 事件类型过滤；`limit` 缺省 20。
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

    /// 启用指定规则，并写入 `RuleEnabled` 日志事件。
    pub async fn enable_rule(&self, rule_type: &str, now: DateTime<Utc>) -> Result<RiskRule> {
        self.toggle_rule(rule_type, true, now).await
    }

    /// 禁用指定规则，并写入 `RuleDisabled` 日志事件。
    pub async fn disable_rule(&self, rule_type: &str, now: DateTime<Utc>) -> Result<RiskRule> {
        self.toggle_rule(rule_type, false, now).await
    }

    /// 基于账户快照生成风控状态：日盈亏、买入锁状态、持仓比例、规则快照、自动减仓建议。
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

    /// 买入前风控校验：刷新状态 → 检查买入锁 → 仓位 / 波动率 / 行业集中度等规则。
    ///
    /// 通过返回 `Ok(())`，违反任一规则返回 [`QuantixError`]。
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
        if let Some(rule) = find_enabled_rule(&state, RiskRuleType::IndustryLimit).cloned() {
            evaluate_industry_limit(
                &rule,
                snapshot,
                projected_buy,
                self.industry_resolver.as_deref(),
                now,
            )
            .await?;
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

    /// 交易后状态同步：等同 [`Self::status`]，用于 trade 模块触发刷新。
    pub async fn sync_after_trade_snapshot(
        &self,
        snapshot: &RiskAccountSnapshot,
        now: DateTime<Utc>,
    ) -> Result<RiskStatus> {
        self.status(snapshot, now).await
    }

    /// trade init/reset 后同步：重置日基准为今日 + 当前总资产，并清除买入锁（写日志）。
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

        let status = build_status(&state, snapshot, now, now.date_naive());
        self.store.save_state(&state).await?;
        Ok(status)
    }

    /// 手动释放买入锁：标记当日已释放（不影响锁本身的状态），并写日志事件。
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

        Ok(build_status(state, snapshot, now, trading_date))
    }
}

impl RiskService<JsonRiskStore> {
    /// 从 JSON store 构造带完整行业解析器的服务（行业映射来自同一目录的 SQLite）。
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
