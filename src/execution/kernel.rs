use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::core::{QuantixError, Result};
use crate::execution::adapter::{AdapterOrderRequest, ExecutionAdapter};
use crate::execution::models::{
    ExecutionPolicy, OrderEventRecord, OrderIntent, OrderRecord, OrderStatus, SignalEnvelope,
    SignalEventRecord, StrategyRunRecord, StrategyRunStatus, translate_signal,
};
use crate::execution::runtime_store::StrategyRuntimeStore;
use crate::strategy::trait_def::Signal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RiskDecision {
    Allow,
    Reject { reason: String },
}

#[async_trait]
pub trait RiskEvaluator: Send + Sync {
    async fn evaluate(&self, intent: OrderIntent) -> Result<RiskDecision>;

    async fn sync_after_fill(&self) -> Result<()>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionRunRequest {
    pub run_id: String,
    pub strategy_name: String,
    pub mode: String,
    pub trigger: String,
    pub symbol: String,
    pub timeframe: String,
    pub bar_end: DateTime<Utc>,
    pub market_price: rust_decimal::Decimal,
    pub held_volume: Option<i64>,
    pub policy: ExecutionPolicy,
    pub client_order_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KernelExecutionResult {
    pub run_id: String,
    pub signal: Signal,
    pub order_status: Option<OrderStatus>,
    pub client_order_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoverySummary {
    pub scanned: usize,
    pub recovered: usize,
    pub unchanged: usize,
    pub failed: usize,
    pub skipped: usize,
}

#[derive(Debug, Clone)]
pub struct ExecutionKernel<A, R> {
    store: StrategyRuntimeStore,
    adapter: A,
    risk: R,
}

impl<A, R> ExecutionKernel<A, R> {
    pub fn new(store: StrategyRuntimeStore, adapter: A, risk: R) -> Self {
        Self {
            store,
            adapter,
            risk,
        }
    }
}

impl<A, R> ExecutionKernel<A, R>
where
    A: ExecutionAdapter,
    R: RiskEvaluator,
{
    pub async fn execute_once(
        &self,
        request: ExecutionRunRequest,
        envelope: SignalEnvelope,
    ) -> Result<KernelExecutionResult> {
        if let Some(existing_run) = self
            .store
            .find_run_by_dedupe_key(
                &request.strategy_name,
                &request.mode,
                &request.symbol,
                &request.timeframe,
                request.bar_end,
            )
            .await?
        {
            let existing_order = self
                .store
                .find_first_order_for_run(&existing_run.run_id)
                .await?;
            return Ok(KernelExecutionResult {
                run_id: existing_run.run_id,
                signal: envelope.signal,
                order_status: existing_order.as_ref().map(|order| order.status),
                client_order_id: existing_order.map(|order| order.client_order_id),
            });
        }

        let now = Utc::now();
        self.store
            .insert_run(&StrategyRunRecord {
                run_id: request.run_id.clone(),
                strategy_name: request.strategy_name.clone(),
                mode: request.mode.clone(),
                trigger: request.trigger.clone(),
                status: StrategyRunStatus::Running,
                symbol: request.symbol.clone(),
                timeframe: request.timeframe.clone(),
                bar_end: request.bar_end,
                started_at: now,
                finished_at: None,
                metadata_json: serde_json::json!({}),
            })
            .await?;

        self.store
            .insert_signal_event(&SignalEventRecord {
                event_id: Uuid::new_v4().to_string(),
                run_id: request.run_id.clone(),
                strategy_name: request.strategy_name.clone(),
                symbol: request.symbol.clone(),
                signal: signal_to_str(envelope.signal).to_string(),
                ts: now,
                payload_json: envelope.metadata_json.clone(),
            })
            .await?;

        let maybe_intent = translate_signal(
            &envelope,
            &request.symbol,
            request.market_price,
            request.held_volume,
            &request.policy,
        )?;

        let Some(intent) = maybe_intent else {
            self.store
                .update_run_status(
                    &request.run_id,
                    StrategyRunStatus::Success,
                    Some(Utc::now()),
                )
                .await?;
            return Ok(KernelExecutionResult {
                run_id: request.run_id,
                signal: envelope.signal,
                order_status: None,
                client_order_id: None,
            });
        };

        if let Some(existing) = self
            .store
            .find_order_by_client_order_id(&request.client_order_id)
            .await?
        {
            self.store
                .update_run_status(
                    &request.run_id,
                    StrategyRunStatus::Success,
                    Some(Utc::now()),
                )
                .await?;
            return Ok(KernelExecutionResult {
                run_id: request.run_id,
                signal: envelope.signal,
                order_status: Some(existing.status),
                client_order_id: Some(existing.client_order_id),
            });
        }

        match self.risk.evaluate(intent.clone()).await? {
            RiskDecision::Reject { reason } => {
                let order_id = request.client_order_id.clone();
                self.store
                    .insert_order(&OrderRecord {
                        order_id: order_id.clone(),
                        client_order_id: request.client_order_id.clone(),
                        run_id: request.run_id.clone(),
                        symbol: request.symbol.clone(),
                        side: intent.side,
                        order_type: intent.order_type,
                        requested_quantity: intent.requested_quantity,
                        requested_price: intent.requested_price,
                        filled_quantity: 0,
                        remaining_quantity: intent.requested_quantity,
                        avg_fill_price: None,
                        status: OrderStatus::Rejected,
                        adapter: "risk".to_string(),
                        created_at: now,
                        updated_at: now,
                        last_transition_at: now,
                        version: 0,
                        payload_json: intent.policy_snapshot_json.clone(),
                    })
                    .await?;
                self.store
                    .insert_order_event(&OrderEventRecord {
                        event_id: Uuid::new_v4().to_string(),
                        order_id,
                        client_order_id: request.client_order_id.clone(),
                        event_type: "risk_rejected".to_string(),
                        event_time: now,
                        details_json: serde_json::json!({ "reason": reason }),
                    })
                    .await?;
                self.store
                    .update_run_status(
                        &request.run_id,
                        StrategyRunStatus::Success,
                        Some(Utc::now()),
                    )
                    .await?;
                Ok(KernelExecutionResult {
                    run_id: request.run_id,
                    signal: envelope.signal,
                    order_status: Some(OrderStatus::Rejected),
                    client_order_id: Some(request.client_order_id),
                })
            }
            RiskDecision::Allow => {
                let order_id = request.client_order_id.clone();
                self.store
                    .insert_order(&OrderRecord {
                        order_id: order_id.clone(),
                        client_order_id: request.client_order_id.clone(),
                        run_id: request.run_id.clone(),
                        symbol: request.symbol.clone(),
                        side: intent.side,
                        order_type: intent.order_type,
                        requested_quantity: intent.requested_quantity,
                        requested_price: intent.requested_price,
                        filled_quantity: 0,
                        remaining_quantity: intent.requested_quantity,
                        avg_fill_price: None,
                        status: OrderStatus::PendingSubmit,
                        adapter: self.adapter.adapter_name().to_string(),
                        created_at: now,
                        updated_at: now,
                        last_transition_at: now,
                        version: 0,
                        payload_json: intent.policy_snapshot_json.clone(),
                    })
                    .await?;
                self.store
                    .insert_order_event(&OrderEventRecord {
                        event_id: Uuid::new_v4().to_string(),
                        order_id: order_id.clone(),
                        client_order_id: request.client_order_id.clone(),
                        event_type: "pending_submit".to_string(),
                        event_time: now,
                        details_json: serde_json::json!({}),
                    })
                    .await?;

                let response = self
                    .adapter
                    .submit_order(AdapterOrderRequest {
                        client_order_id: request.client_order_id.clone(),
                        symbol: request.symbol.clone(),
                        side: intent.side,
                        quantity: intent.requested_quantity,
                        price: intent.requested_price,
                    })
                    .await
                    .map_err(|err| QuantixError::Other(err.to_string()))?;

                self.store
                    .insert_order_event(&OrderEventRecord {
                        event_id: Uuid::new_v4().to_string(),
                        order_id: order_id.clone(),
                        client_order_id: request.client_order_id.clone(),
                        event_type: response.latest_status.as_str().to_string(),
                        event_time: Utc::now(),
                        details_json: serde_json::json!({
                            "filled_quantity": response.filled_quantity,
                            "avg_fill_price": response.avg_fill_price,
                        }),
                    })
                    .await?;
                self.store
                    .update_order(
                        &order_id,
                        response.latest_status,
                        response.filled_quantity,
                        response.avg_fill_price,
                        Utc::now(),
                    )
                    .await?;

                if response.filled_quantity > 0 {
                    self.risk.sync_after_fill().await?;
                }

                self.store
                    .update_run_status(
                        &request.run_id,
                        StrategyRunStatus::Success,
                        Some(Utc::now()),
                    )
                    .await?;
                Ok(KernelExecutionResult {
                    run_id: request.run_id,
                    signal: envelope.signal,
                    order_status: Some(response.latest_status),
                    client_order_id: Some(request.client_order_id),
                })
            }
        }
    }

    pub async fn recover_pending_orders(&self) -> Result<RecoverySummary> {
        let mut summary = RecoverySummary {
            scanned: 0,
            recovered: 0,
            unchanged: 0,
            failed: 0,
            skipped: 0,
        };

        let orders = self.store.list_recoverable_mock_live_orders().await?;
        summary.scanned = orders.len();

        for order in orders {
            let before_state = self.store.get_mock_live_order_state(&order.order_id).await?;
            let response = match self.adapter.query_order(&order.client_order_id).await {
                Ok(response) => response,
                Err(_) => {
                    summary.failed += 1;
                    continue;
                }
            };
            let after_state = self.store.get_mock_live_order_state(&order.order_id).await?;

            if response.latest_status != order.status || response.filled_quantity != order.filled_quantity {
                let remaining_quantity =
                    (order.requested_quantity - response.filled_quantity).max(0);
                let updated_at = Utc::now();
                let updated = self
                    .store
                    .try_update_order_with_version(
                        &order.order_id,
                        order.version,
                        response.latest_status,
                        response.filled_quantity,
                        remaining_quantity,
                        response.avg_fill_price,
                        updated_at,
                    )
                    .await?;

                if updated {
                    self.store
                        .insert_order_event(&OrderEventRecord {
                            event_id: Uuid::new_v4().to_string(),
                            order_id: order.order_id.clone(),
                            client_order_id: order.client_order_id.clone(),
                            event_type: response.latest_status.as_str().to_string(),
                            event_time: updated_at,
                            details_json: serde_json::json!({
                                "filled_quantity": response.filled_quantity,
                                "avg_fill_price": response.avg_fill_price,
                            }),
                        })
                        .await?;

                    if response.filled_quantity > order.filled_quantity {
                        self.risk.sync_after_fill().await?;
                    }

                    summary.recovered += 1;
                    continue;
                }

                let latest = self
                    .store
                    .find_order_by_client_order_id(&order.client_order_id)
                    .await?;
                if let Some(latest) = latest {
                    let retry_updated = self
                        .store
                        .try_update_order_with_version(
                            &latest.order_id,
                            latest.version,
                            response.latest_status,
                            response.filled_quantity,
                            (latest.requested_quantity - response.filled_quantity).max(0),
                            response.avg_fill_price,
                            updated_at,
                        )
                        .await?;
                    if retry_updated {
                        self.store
                            .insert_order_event(&OrderEventRecord {
                                event_id: Uuid::new_v4().to_string(),
                                order_id: latest.order_id.clone(),
                                client_order_id: latest.client_order_id.clone(),
                                event_type: response.latest_status.as_str().to_string(),
                                event_time: updated_at,
                                details_json: serde_json::json!({
                                    "filled_quantity": response.filled_quantity,
                                    "avg_fill_price": response.avg_fill_price,
                                }),
                            })
                            .await?;

                        if response.filled_quantity > latest.filled_quantity {
                            self.risk.sync_after_fill().await?;
                        }

                        summary.recovered += 1;
                    } else {
                        summary.skipped += 1;
                    }
                } else {
                    summary.skipped += 1;
                }
                continue;
            }

            let exhaustion_transition = before_state
                .as_ref()
                .map(|state| state.recovery_exhausted)
                .unwrap_or(false)
                == false
                && after_state
                    .as_ref()
                    .map(|state| state.recovery_exhausted)
                    .unwrap_or(false);

            if exhaustion_transition {
                let after_state = after_state.unwrap();
                self.store
                    .insert_order_event(&OrderEventRecord {
                        event_id: Uuid::new_v4().to_string(),
                        order_id: order.order_id.clone(),
                        client_order_id: order.client_order_id.clone(),
                        event_type: "recovery_exhausted".to_string(),
                        event_time: Utc::now(),
                        details_json: serde_json::json!({
                            "unknown_retries": after_state.unknown_retries,
                            "reason": after_state.exhausted_reason,
                        }),
                    })
                    .await?;
                summary.recovered += 1;
            } else {
                summary.unchanged += 1;
            }
        }

        Ok(summary)
    }
}

fn signal_to_str(signal: Signal) -> &'static str {
    match signal {
        Signal::Buy => "buy",
        Signal::Sell => "sell",
        Signal::Hold => "hold",
    }
}
