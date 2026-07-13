#![allow(clippy::too_many_arguments, clippy::collapsible_if)]

use crate::core::{QuantixError, Result};
use crate::monitor::MonitorQuoteRow;
use crate::stop::models::{
    StopEvaluationResult, StopHistoryEvent, StopHistoryEventType, StopHistoryFilter, StopRule,
    StopRuleUpdate, StopStatusRow,
};
use chrono::{DateTime, NaiveDate, Utc};
use std::collections::HashMap;
mod eval;
mod types;

use eval::{
    build_history_event, evaluate_rule_state, merge_stop_rule_patch, validate_stop_rule_inputs,
};

pub use types::{StopRuleStore, StopService};
impl<RS> StopService<RS>
where
    RS: StopRuleStore,
{
    /// 用给定的规则存储后端构造止损/止盈服务。
    pub fn new(store: RS) -> Self {
        Self { store }
    }

    /// 新建或覆盖止损/止盈规则并写入 `Set` 历史事件。
    /// 输入校验失败（互斥参数同时给出、全为空、值非正等）返回 `QuantixError::Other`。
    pub async fn set_rule(
        &self,
        code: &str,
        stop_loss_price: Option<f64>,
        take_profit_price: Option<f64>,
        stop_loss_pct: Option<f64>,
        take_profit_pct: Option<f64>,
        trailing_pct: Option<f64>,
        reference_price: Option<f64>,
        now: DateTime<Utc>,
    ) -> Result<StopRule> {
        validate_stop_rule_inputs(
            stop_loss_price,
            take_profit_price,
            stop_loss_pct,
            take_profit_pct,
            trailing_pct,
        )?;

        let rule = self
            .store
            .upsert_rule(StopRule {
                code: code.to_string(),
                stop_loss_price,
                take_profit_price,
                stop_loss_pct,
                take_profit_pct,
                trailing_pct,
                highest_price: None,
                reference_price,
                last_triggered_at: None,
                created_at: now,
                updated_at: now,
            })
            .await?;
        self.store
            .append_history(build_history_event(&rule, StopHistoryEventType::Set, now)?)
            .await?;
        Ok(rule)
    }

    /// 返回当前全部止损/止盈规则。
    pub async fn list_rules(&self) -> Result<Vec<StopRule>> {
        self.store.list_rules().await
    }

    /// 按 code 查找规则；不存在时返回 `None`。
    pub async fn get_rule(&self, code: &str) -> Result<Option<StopRule>> {
        self.store.get_rule(code).await
    }

    /// 按 code/date/event_type/limit 过滤历史事件；所有过滤项均可为 `None` 表示不限制。
    pub async fn history(
        &self,
        code: Option<&str>,
        date: Option<NaiveDate>,
        event_type: Option<StopHistoryEventType>,
        limit: Option<usize>,
    ) -> Result<Vec<StopHistoryEvent>> {
        self.store
            .list_history(StopHistoryFilter {
                code: code.map(ToOwned::to_owned),
                date,
                event_type,
                limit,
            })
            .await
    }

    /// 局部更新规则：`None` 字段保留原值，合并后重新校验，并写 `Update` 历史事件；规则不存在时返回错误。
    pub async fn update_rule(
        &self,
        code: &str,
        patch: StopRuleUpdate,
        now: DateTime<Utc>,
    ) -> Result<StopRule> {
        let existing = self
            .store
            .get_rule(code)
            .await?
            .ok_or_else(|| QuantixError::Other(format!("stop update 未找到规则: {code}")))?;
        let merged = merge_stop_rule_patch(existing, patch, now)?;
        let saved = self.store.upsert_rule(merged).await?;
        self.store
            .append_history(build_history_event(
                &saved,
                StopHistoryEventType::Update,
                now,
            )?)
            .await?;
        Ok(saved)
    }

    /// 删除规则；成功时附写 `Remove` 历史事件并返回 `true`，规则不存在返回 `false`。
    pub async fn remove_rule(&self, code: &str, now: DateTime<Utc>) -> Result<bool> {
        let existing = self.store.get_rule(code).await?;
        let removed = self.store.remove_rule(code).await?;
        if removed {
            if let Some(rule) = existing {
                self.store
                    .append_history(build_history_event(
                        &rule,
                        StopHistoryEventType::Remove,
                        now,
                    )?)
                    .await?;
            }
        }
        Ok(removed)
    }

    /// 评估单条规则（无锚价映射），返回新规则状态与可能的触发事件；缺报价返回 `QuoteMissing`。
    pub fn evaluate_rule(
        &self,
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

    /// 按 quote_rows 中的最新价批量评估规则；不使用持仓成本锚价。
    pub fn evaluate_rules(
        &self,
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
            .map(|rule| {
                self.evaluate_rule(
                    rule,
                    quote_map.get(rule.code.as_str()).copied().flatten(),
                    observed_at,
                )
            })
            .collect()
    }

    /// 批量评估规则；`avg_cost_by_code` 提供按 code 索引的持仓成本锚价（优先于 reference_price）。
    pub fn evaluate_rules_with_anchor_map(
        &self,
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

    /// 生成 CLI 状态展示行：在 `evaluate_rules_with_anchor_map` 基础上额外携带阈值/锚价/触发时间等汇总字段。
    pub fn status_rows(
        &self,
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
}
