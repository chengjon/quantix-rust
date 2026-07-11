use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::core::signal::Signal;
use crate::core::{QuantixError, Result};
use crate::execution::adapter::{AdapterOrderRequest, ExecutionAdapter};
use crate::execution::models::{
    ExecutionPolicy, FillDeltaContext, FillDeltaResult, OrderEventRecord, OrderIntent, OrderRecord,
    OrderStatus, SignalEnvelope, SignalEventRecord, StrategyRunRecord, StrategyRunStatus,
    translate_signal,
};
use crate::execution::runtime_store::StrategyRuntimeStore;

mod recovery;

/// 风控决策：Allow 放行、Reject 拒绝（含原因）。由 RiskEvaluator 返回，决定 kernel 是否提交订单到 adapter。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RiskDecision {
    Allow,
    Reject { reason: String },
}

/// 风控评估器 trait：evaluate 在下单前评估 intent 决定是否放行；sync_after_fill 在成交后同步风控内部状态（如持仓/当日基线）。
#[async_trait]
pub trait RiskEvaluator: Send + Sync {
    async fn evaluate(&self, intent: OrderIntent) -> Result<RiskDecision>;

    async fn sync_after_fill(&self) -> Result<()>;
}

/// 成交增量应用 trait：apply_fill_delta 根据 new/old filled_quantity 差值计算本轮成交增量，并产生可选 trade_record（用于 paper/回测的持仓与账务联动）。
#[async_trait]
pub trait FillDeltaApplier: Send + Sync {
    async fn apply_fill_delta(&self, ctx: FillDeltaContext) -> Result<FillDeltaResult>;
}

/// 空实现的 FillDeltaApplier：计算 delta 但不产生 trade_record_id；用于不需要 broker 价差模拟的执行通道（如 qmt_live 直接走真实 broker）。
#[derive(Debug, Clone, Copy, Default)]
pub struct NoopFillDeltaApplier;

#[async_trait]
impl FillDeltaApplier for NoopFillDeltaApplier {
    async fn apply_fill_delta(&self, ctx: FillDeltaContext) -> Result<FillDeltaResult> {
        let delta_quantity = (ctx.new_filled_quantity - ctx.old_filled_quantity).max(0);
        Ok(FillDeltaResult {
            applied: delta_quantity > 0,
            delta_quantity,
            trade_record_id: None,
        })
    }
}

/// 执行请求（来自策略信号）：run_id 策略运行 ID、strategy_name/mode/trigger 策略与触发上下文、symbol/timeframe 标的与周期、bar_end/bar 截止时间、market_price 信号价、held_volume 持仓、policy 执行策略、client_order_id 幂等键。
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

/// 已解析出的执行请求（含 intent）：由 ExecutionRunRequest 经信号翻译 + risk gate 前构造；execute_request 接收此结构走完整下单事务。
#[derive(Debug, Clone, PartialEq)]
pub struct PreparedExecutionRequest {
    pub run_id: String,
    pub strategy_name: String,
    pub mode: String,
    pub trigger: String,
    pub symbol: String,
    pub timeframe: String,
    pub bar_end: DateTime<Utc>,
    pub signal: Signal,
    pub signal_payload_json: serde_json::Value,
    pub intent: OrderIntent,
    pub client_order_id: String,
}

/// 执行内核结果：run_id、signal 信号、order_status 可选订单终态、client_order_id 可选幂等键。order_status 为 None 表示信号被 hold/无 intent，未产生订单。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KernelExecutionResult {
    pub run_id: String,
    pub signal: Signal,
    pub order_status: Option<OrderStatus>,
    pub client_order_id: Option<String>,
}

/// 崩溃恢复扫描汇总：scanned 扫描数、recovered 恢复数、unchanged 未变数、failed 失败数、skipped 跳过数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoverySummary {
    pub scanned: usize,
    pub recovered: usize,
    pub unchanged: usize,
    pub failed: usize,
    pub skipped: usize,
}

/// 执行内核：组合 StrategyRuntimeStore + ExecutionAdapter + FillDeltaApplier + RiskEvaluator，提供 execute_once/execute_request 入口，承担信号翻译、风控、下单、成交增量应用、状态机持久化、幂等去重等完整事务。
#[derive(Debug, Clone)]
pub struct ExecutionKernel<A, F, R> {
    store: StrategyRuntimeStore,
    adapter: A,
    fill_delta: F,
    risk: R,
}

impl<A, R> ExecutionKernel<A, NoopFillDeltaApplier, R> {
    /// 构造 ExecutionKernel：注入 store / adapter / risk；不挂 fill delta（内部用 NoopFillDeltaApplier，覆盖价不计入 broker 模拟价差）。
    pub fn new(store: StrategyRuntimeStore, adapter: A, risk: R) -> Self {
        Self {
            store,
            adapter,
            fill_delta: NoopFillDeltaApplier,
            risk,
        }
    }
}

impl<A, F, R> ExecutionKernel<A, F, R> {
    /// 构造 ExecutionKernel 并注入 fill_delta：用于 paper/回测模式下对 broker 成交价做滑点或价差模拟，与真实 adapter 协同写入成交细节。
    pub fn with_fill_delta(
        store: StrategyRuntimeStore,
        adapter: A,
        fill_delta: F,
        risk: R,
    ) -> Self {
        Self {
            store,
            adapter,
            fill_delta,
            risk,
        }
    }
}

impl<A, F, R> ExecutionKernel<A, F, R>
where
    A: ExecutionAdapter,
    F: FillDeltaApplier,
    R: RiskEvaluator,
{
    /// 处理已构造好的 PreparedExecutionRequest：先 insert_run、再 risk gate、再 adapter submit、再根据返回状态写入 signal/fill/run 状态。整套事务保证 dedupe key 不重复执行；任一环节失败会把 strategy_run_status 标记为 failed 并透传错误。
    pub async fn execute_request(
        &self,
        request: PreparedExecutionRequest,
    ) -> Result<KernelExecutionResult> {
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
                signal: signal_to_str(request.signal).to_string(),
                ts: now,
                payload_json: request.signal_payload_json.clone(),
            })
            .await?;

        self.execute_prepared_order_flow(request).await
    }

    /// 单次执行入口：从 envelope + ExecutionRunRequest 组装 PreparedExecutionRequest 并走 execute_request。若已有相同 dedupe key 的成功执行，直接返回已有结果而不重复下单；避免重复入金/重复成交。
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

        self.execute_prepared_order_flow(PreparedExecutionRequest {
            run_id: request.run_id,
            strategy_name: request.strategy_name,
            mode: request.mode,
            trigger: request.trigger,
            symbol: request.symbol,
            timeframe: request.timeframe,
            bar_end: request.bar_end,
            signal: envelope.signal,
            signal_payload_json: envelope.metadata_json,
            intent,
            client_order_id: request.client_order_id,
        })
        .await
    }

    async fn execute_prepared_order_flow(
        &self,
        request: PreparedExecutionRequest,
    ) -> Result<KernelExecutionResult> {
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
                signal: request.signal,
                order_status: Some(existing.status),
                client_order_id: Some(existing.client_order_id),
            });
        }

        let intent = request.intent.clone();
        match self.risk.evaluate(intent.clone()).await? {
            RiskDecision::Reject { reason } => {
                let now = Utc::now();
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
                    signal: request.signal,
                    order_status: Some(OrderStatus::Rejected),
                    client_order_id: Some(request.client_order_id),
                })
            }
            RiskDecision::Allow => {
                let now = Utc::now();
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

                let event_time = Utc::now();
                if response.filled_quantity > 0 {
                    let fill_result = match self
                        .fill_delta
                        .apply_fill_delta(FillDeltaContext {
                            order_id: order_id.clone(),
                            client_order_id: request.client_order_id.clone(),
                            symbol: request.symbol.clone(),
                            side: intent.side,
                            requested_price: intent.requested_price,
                            old_filled_quantity: 0,
                            new_filled_quantity: response.filled_quantity,
                            fill_details: response.fill_details.clone(),
                            event_time,
                        })
                        .await
                    {
                        Ok(result) => result,
                        Err(err) => {
                            self.store
                                .insert_order_event(&OrderEventRecord {
                                    event_id: Uuid::new_v4().to_string(),
                                    order_id: order_id.clone(),
                                    client_order_id: request.client_order_id.clone(),
                                    event_type: "fill_apply_failed".to_string(),
                                    event_time,
                                    details_json: serde_json::json!({
                                        "error": err.to_string(),
                                        "proposed_status": response.latest_status.as_str(),
                                        "proposed_filled_quantity": response.filled_quantity,
                                        "avg_fill_price": response.avg_fill_price,
                                        "fill_details": fill_details_json(response.fill_details.as_ref()),
                                    }),
                                })
                                .await?;
                            self.store
                                .update_run_status(
                                    &request.run_id,
                                    StrategyRunStatus::Failed,
                                    Some(event_time),
                                )
                                .await?;
                            return Err(err);
                        }
                    };

                    if fill_result.applied {
                        self.store
                            .insert_order_event(&OrderEventRecord {
                                event_id: Uuid::new_v4().to_string(),
                                order_id: order_id.clone(),
                                client_order_id: request.client_order_id.clone(),
                                event_type: response.latest_status.as_str().to_string(),
                                event_time,
                                details_json: serde_json::json!({
                                    "filled_quantity": response.filled_quantity,
                                    "avg_fill_price": response.avg_fill_price,
                                }),
                            })
                            .await?;
                        self.store
                            .insert_order_event(&OrderEventRecord {
                                event_id: Uuid::new_v4().to_string(),
                                order_id: order_id.clone(),
                                client_order_id: request.client_order_id.clone(),
                                event_type: "fill_applied".to_string(),
                                event_time,
                                details_json: serde_json::json!({
                                    "delta_quantity": fill_result.delta_quantity,
                                    "trade_record_id": fill_result.trade_record_id,
                                    "fill_details": fill_details_json(response.fill_details.as_ref()),
                                }),
                            })
                            .await?;
                        self.store
                            .update_order(
                                &order_id,
                                response.latest_status,
                                response.filled_quantity,
                                response.avg_fill_price,
                                event_time,
                            )
                            .await?;
                        self.mark_mock_live_fill_applied(
                            &order_id,
                            &request.client_order_id,
                            response.fill_details.as_ref(),
                        )
                        .await?;
                        self.risk.sync_after_fill().await?;
                    }
                } else {
                    self.store
                        .insert_order_event(&OrderEventRecord {
                            event_id: Uuid::new_v4().to_string(),
                            order_id: order_id.clone(),
                            client_order_id: request.client_order_id.clone(),
                            event_type: response.latest_status.as_str().to_string(),
                            event_time,
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
                            event_time,
                        )
                        .await?;
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
                    signal: request.signal,
                    order_status: Some(response.latest_status),
                    client_order_id: Some(request.client_order_id),
                })
            }
        }
    }

    async fn mark_mock_live_fill_applied(
        &self,
        order_id: &str,
        adapter_order_id: &str,
        fill_details: Option<&crate::execution::models::FillDetails>,
    ) -> Result<()> {
        let Some(fill_details) = fill_details else {
            return Ok(());
        };
        let Some(mut state) = self.store.get_mock_live_order_state(order_id).await? else {
            return Ok(());
        };

        if fill_details.fill_id <= state.last_applied_fill_id {
            return Ok(());
        }

        state.last_applied_fill_id = fill_details.fill_id;
        self.store
            .update_mock_live_order_state(order_id, Some(adapter_order_id), &state)
            .await
    }
}

fn signal_to_str(signal: Signal) -> &'static str {
    match signal {
        Signal::Buy => "buy",
        Signal::Sell => "sell",
        Signal::Hold => "hold",
    }
}

fn fill_details_json(
    fill_details: Option<&crate::execution::models::FillDetails>,
) -> serde_json::Value {
    match fill_details {
        Some(fill) => serde_json::json!({
            "fill_id": fill.fill_id,
            "fill_quantity": fill.fill_quantity,
            "fill_price": fill.fill_price,
        }),
        None => serde_json::Value::Null,
    }
}
