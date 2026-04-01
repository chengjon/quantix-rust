use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use std::collections::HashMap;
use uuid::Uuid;

use crate::core::{QuantixError, Result};
use crate::monitor::MonitorQuoteRow;
use crate::stop::models::{
    StopEvaluationResult, StopHistoryEvent, StopHistoryEventType, StopHistoryFilter, StopRule,
    StopRuleUpdate, StopStatusRow,
};

#[async_trait]
pub trait StopRuleStore: Send + Sync {
    async fn upsert_rule(&self, rule: StopRule) -> Result<StopRule>;

    async fn list_rules(&self) -> Result<Vec<StopRule>>;

    async fn get_rule(&self, code: &str) -> Result<Option<StopRule>>;

    async fn append_history(&self, event: StopHistoryEvent) -> Result<()>;

    async fn list_history(&self, filter: StopHistoryFilter) -> Result<Vec<StopHistoryEvent>>;

    async fn remove_rule(&self, code: &str) -> Result<bool>;
}

#[derive(Debug, Clone)]
pub struct StopService<RS> {
    store: RS,
}

impl<RS> StopService<RS>
where
    RS: StopRuleStore,
{
    pub fn new(store: RS) -> Self {
        Self { store }
    }

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
            .append_history(build_history_event(
                &rule,
                StopHistoryEventType::Set,
                now,
            )?)
            .await?;
        Ok(rule)
    }

    pub async fn list_rules(&self) -> Result<Vec<StopRule>> {
        self.store.list_rules().await
    }

    pub async fn get_rule(&self, code: &str) -> Result<Option<StopRule>> {
        self.store.get_rule(code).await
    }

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

    pub fn evaluate_rule(
        &self,
        rule: &StopRule,
        current_price: Option<f64>,
        observed_at: DateTime<Utc>,
    ) -> StopEvaluationResult {
        super::service_eval::evaluate_rule(rule, current_price, observed_at)
    }

    pub fn evaluate_rules(
        &self,
        rules: &[StopRule],
        quote_rows: &[MonitorQuoteRow],
        observed_at: DateTime<Utc>,
    ) -> Vec<StopEvaluationResult> {
        super::service_eval::evaluate_rules(rules, quote_rows, observed_at)
    }

    pub fn evaluate_rules_with_anchor_map(
        &self,
        rules: &[StopRule],
        quote_rows: &[MonitorQuoteRow],
        avg_cost_by_code: &HashMap<String, f64>,
        observed_at: DateTime<Utc>,
    ) -> Vec<StopEvaluationResult> {
        super::service_eval::evaluate_rules_with_anchor_map(
            rules,
            quote_rows,
            avg_cost_by_code,
            observed_at,
        )
    }

    pub fn status_rows(
        &self,
        rules: &[StopRule],
        quote_rows: &[MonitorQuoteRow],
        avg_cost_by_code: &HashMap<String, f64>,
        observed_at: DateTime<Utc>,
    ) -> Vec<StopStatusRow> {
        super::service_eval::status_rows(rules, quote_rows, avg_cost_by_code, observed_at)
    }
}

fn validate_stop_rule_inputs(
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

fn merge_stop_rule_patch(
    existing: StopRule,
    patch: StopRuleUpdate,
    now: DateTime<Utc>,
) -> Result<StopRule> {
    let merged = StopRule {
        code: existing.code,
        stop_loss_price: patch
            .stop_loss_price
            .unwrap_or(existing.stop_loss_price),
        take_profit_price: patch
            .take_profit_price
            .unwrap_or(existing.take_profit_price),
        stop_loss_pct: patch.stop_loss_pct.unwrap_or(existing.stop_loss_pct),
        take_profit_pct: patch
            .take_profit_pct
            .unwrap_or(existing.take_profit_pct),
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

fn build_history_event(
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
        anchor_source: rule
            .reference_price
            .map(|_| "reference_price".to_string()),
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
