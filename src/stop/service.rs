use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

use crate::core::{QuantixError, Result};
use crate::monitor::MonitorQuoteRow;
use crate::stop::models::{StopEvaluationResult, StopRule, StopTriggerKind, TriggeredStop};

#[async_trait]
pub trait StopRuleStore: Send + Sync {
    async fn upsert_rule(&self, rule: StopRule) -> Result<StopRule>;

    async fn list_rules(&self) -> Result<Vec<StopRule>>;

    async fn remove_rule(&self, code: &str) -> Result<bool>;

    async fn update_runtime_state(
        &self,
        code: &str,
        highest_price: Option<f64>,
        last_triggered_at: Option<DateTime<Utc>>,
        updated_at: DateTime<Utc>,
    ) -> Result<bool>;
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
        trailing_pct: Option<f64>,
        now: DateTime<Utc>,
    ) -> Result<StopRule> {
        validate_stop_rule_inputs(stop_loss_price, take_profit_price, trailing_pct)?;

        self.store
            .upsert_rule(StopRule {
                code: code.to_string(),
                stop_loss_price,
                take_profit_price,
                trailing_pct,
                highest_price: None,
                last_triggered_at: None,
                created_at: now,
                updated_at: now,
            })
            .await
    }

    pub async fn list_rules(&self) -> Result<Vec<StopRule>> {
        self.store.list_rules().await
    }

    pub async fn remove_rule(&self, code: &str) -> Result<bool> {
        self.store.remove_rule(code).await
    }

    pub fn evaluate_rule(
        &self,
        rule: &StopRule,
        current_price: Option<f64>,
        observed_at: DateTime<Utc>,
    ) -> StopEvaluationResult {
        let Some(current_price) = current_price else {
            return StopEvaluationResult {
                updated_rule: rule.clone(),
                triggered_stop: None,
            };
        };

        let mut updated_rule = rule.clone();
        let mut triggered_stop = None;

        if let Some(trailing_pct) = updated_rule.trailing_pct {
            let highest_price = updated_rule.highest_price.unwrap_or(current_price);
            let highest_price = highest_price.max(current_price);
            let threshold_price = highest_price * (1.0 - trailing_pct / 100.0);
            updated_rule.highest_price = Some(highest_price);

            if current_price <= threshold_price {
                triggered_stop = Some(TriggeredStop {
                    code: updated_rule.code.clone(),
                    kind: StopTriggerKind::TrailingLoss,
                    current_price,
                    threshold_price,
                    highest_price: Some(highest_price),
                    triggered_at: Some(observed_at),
                });
            }
        } else if let Some(stop_loss_price) = updated_rule.stop_loss_price {
            if current_price <= stop_loss_price {
                triggered_stop = Some(TriggeredStop {
                    code: updated_rule.code.clone(),
                    kind: StopTriggerKind::Loss,
                    current_price,
                    threshold_price: stop_loss_price,
                    highest_price: updated_rule.highest_price,
                    triggered_at: Some(observed_at),
                });
            }
        }

        if triggered_stop.is_none() {
            if let Some(take_profit_price) = updated_rule.take_profit_price {
                if current_price >= take_profit_price {
                    triggered_stop = Some(TriggeredStop {
                        code: updated_rule.code.clone(),
                        kind: StopTriggerKind::Profit,
                        current_price,
                        threshold_price: take_profit_price,
                        highest_price: updated_rule.highest_price,
                        triggered_at: Some(observed_at),
                    });
                }
            }
        }

        if triggered_stop.is_some() {
            updated_rule.last_triggered_at = Some(observed_at);
            updated_rule.updated_at = observed_at;
        } else if updated_rule.highest_price != rule.highest_price {
            updated_rule.updated_at = observed_at;
        }

        StopEvaluationResult {
            updated_rule,
            triggered_stop,
        }
    }

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

        rules.iter()
            .map(|rule| {
                self.evaluate_rule(
                    rule,
                    quote_map.get(rule.code.as_str()).copied().flatten(),
                    observed_at,
                )
            })
            .collect()
    }
}

fn validate_stop_rule_inputs(
    stop_loss_price: Option<f64>,
    take_profit_price: Option<f64>,
    trailing_pct: Option<f64>,
) -> Result<()> {
    if stop_loss_price.is_none() && take_profit_price.is_none() && trailing_pct.is_none() {
        return Err(QuantixError::Other(
            "stop set 至少需要一个条件：--loss、--profit、--trailing".to_string(),
        ));
    }

    if stop_loss_price.is_some() && trailing_pct.is_some() {
        return Err(QuantixError::Other(
            "stop set 不能同时指定 --loss 和 --trailing".to_string(),
        ));
    }

    Ok(())
}
