#![allow(clippy::collapsible_if)]

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

use crate::execution::adapter::{
    AdapterError, AdapterOrderRequest, ExecutionAdapter, ExecutionCancelSemantics,
    ExecutionCapabilities, ExecutionChannel, ExecutionFillSource, ExecutionStatusSource,
    OrderInitialResponse, OrderQueryResponse,
};
use crate::execution::models::{FillDetails, MockLiveOrderState, OrderStatus};
use crate::execution::runtime_store::StrategyRuntimeStore;

/// mock_live 时钟 trait：返回 UTC 当前时间，用于 mock adapter 控制订单生命周期的时间推进。Clone + Send + Sync 以适配 adapter 的并发模型。
pub trait MockLiveClock: Clone + Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

/// 系统时钟实现：基于 chrono::Utc::now() 返回真实当前时间，mock_live 默认时钟。
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemMockLiveClock;

impl MockLiveClock for SystemMockLiveClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

/// mock_live 执行适配器：基于 StrategyRuntimeStore 模拟订单生命周期（pending_submit → submitted → partially_filled/filled），由 clock 驱动时间推进；state_template 提供新订单的初始 mock 状态。
#[derive(Debug, Clone)]
pub struct MockLiveExecutionAdapter<C> {
    store: StrategyRuntimeStore,
    clock: C,
    state_template: MockLiveOrderState,
}

impl<C> MockLiveExecutionAdapter<C>
where
    C: MockLiveClock,
{
    /// 构造 mock_live 适配器：注入 store + clock，state_template 用默认值。
    pub fn new(store: StrategyRuntimeStore, clock: C) -> Self {
        Self::with_state_template(store, clock, MockLiveOrderState::default())
    }

    /// 构造 mock_live 适配器并自定义 state_template（用于覆盖 last_applied_fill_id 等初始字段）。
    pub fn with_state_template(
        store: StrategyRuntimeStore,
        clock: C,
        state_template: MockLiveOrderState,
    ) -> Self {
        Self {
            store,
            clock,
            state_template,
        }
    }

    async fn load_state(
        &self,
        order_id: &str,
    ) -> std::result::Result<MockLiveOrderState, AdapterError> {
        self.store
            .get_mock_live_order_state(order_id)
            .await
            .map_err(|err| AdapterError::Execution(err.to_string()))?
            .ok_or_else(|| {
                AdapterError::Execution(format!("mock_live order not found: {order_id}"))
            })
    }

    async fn save_state(
        &self,
        order_id: &str,
        state: &MockLiveOrderState,
    ) -> std::result::Result<(), AdapterError> {
        self.store
            .update_mock_live_order_state(order_id, Some(order_id), state)
            .await
            .map_err(|err| AdapterError::Execution(err.to_string()))
    }

    async fn load_requested_price(
        &self,
        order_id: &str,
    ) -> std::result::Result<Decimal, AdapterError> {
        self.store
            .find_order_by_client_order_id(order_id)
            .await
            .map_err(|err| AdapterError::Execution(err.to_string()))?
            .map(|order| order.requested_price)
            .ok_or_else(|| {
                AdapterError::Execution(format!(
                    "mock_live order requested price not found: {order_id}"
                ))
            })
    }

    fn cumulative_filled_quantity(state: &MockLiveOrderState) -> i64 {
        state
            .fill_plan
            .iter()
            .take(state.next_step_index)
            .map(|step| step.quantity)
            .sum()
    }

    fn has_unapplied_fill(state: &MockLiveOrderState) -> bool {
        state.last_applied_fill_id < state.next_step_index as u64
    }

    fn latest_status_for_state(state: &MockLiveOrderState) -> OrderStatus {
        if state.next_step_index == 0 {
            OrderStatus::Accepted
        } else if state.next_step_index < state.fill_plan.len() {
            OrderStatus::PartiallyFilled
        } else {
            OrderStatus::Filled
        }
    }

    fn parse_query_script(mode: &str) -> Option<Vec<OrderStatus>> {
        let script = mode.strip_prefix("query_script:")?;
        let mut statuses = Vec::new();
        for token in script.split(',') {
            let status = match token.trim().to_ascii_lowercase().as_str() {
                "submitted" => OrderStatus::Submitted,
                "accepted" => OrderStatus::Accepted,
                "partially_filled" | "partial" => OrderStatus::PartiallyFilled,
                "filled" => OrderStatus::Filled,
                "pending_cancel" => OrderStatus::PendingCancel,
                "canceled" | "cancelled" => OrderStatus::Canceled,
                "rejected" => OrderStatus::Rejected,
                "unknown" => OrderStatus::Unknown,
                _ => return None,
            };
            statuses.push(status);
        }
        Some(statuses)
    }

    async fn build_response(
        &self,
        order_id: &str,
        state: &MockLiveOrderState,
        status: OrderStatus,
    ) -> std::result::Result<OrderQueryResponse, AdapterError> {
        let fill_price = match state.simulated_fill_price {
            Some(price) => price,
            None => self.load_requested_price(order_id).await?,
        };
        let fill_details = if Self::has_unapplied_fill(state) {
            let fill_index = state.last_applied_fill_id as usize;
            let fill_step = state.fill_plan[fill_index].clone();
            Some(FillDetails {
                fill_id: fill_index as u64 + 1,
                fill_quantity: fill_step.quantity,
                fill_price,
                last_fill_price: fill_price,
                last_fill_quantity: fill_step.quantity,
                total_fills: (fill_index + 1) as i64,
                commission: Decimal::ZERO,
                fees: Decimal::ZERO,
                venue: "mock".to_string(),
                broker_fill_id: String::new(),
            })
        } else {
            None
        };

        Ok(OrderQueryResponse {
            adapter_order_id: order_id.to_string(),
            latest_status: status,
            filled_quantity: Self::cumulative_filled_quantity(state),
            avg_fill_price: (state.next_step_index > 0).then_some(fill_price),
            fill_details,
            rejection_reason: None,
        })
    }

    fn maybe_mark_recovery_exhausted(state: &mut MockLiveOrderState) {
        if state.unknown_retries > 3 {
            state.recovery_exhausted = true;
            state.exhausted_reason = Some("unknown_retry_budget_exceeded".to_string());
        }
    }

    async fn handle_query_script(
        &self,
        order_id: &str,
        state: &mut MockLiveOrderState,
    ) -> std::result::Result<Option<OrderQueryResponse>, AdapterError> {
        let Some(mode) = state
            .fault_injection
            .as_ref()
            .and_then(|fault| fault.mode.as_deref())
        else {
            return Ok(None);
        };
        let Some(script) = Self::parse_query_script(mode) else {
            return Ok(None);
        };

        loop {
            if state.query_script_index >= script.len() {
                state.fault_injection = None;
                state.query_script_fill_started = false;
                self.save_state(order_id, state).await?;
                return Ok(None);
            }

            let status = script[state.query_script_index];
            match status {
                OrderStatus::Unknown => {
                    state.unknown_retries += 1;
                    Self::maybe_mark_recovery_exhausted(state);
                    state.query_script_index += 1;
                    state.query_script_fill_started = false;
                    self.save_state(order_id, state).await?;
                    return Ok(Some(OrderQueryResponse {
                        adapter_order_id: order_id.to_string(),
                        latest_status: OrderStatus::Unknown,
                        filled_quantity: Self::cumulative_filled_quantity(state),
                        avg_fill_price: None,
                        fill_details: None,
                        rejection_reason: None,
                    }));
                }
                OrderStatus::Submitted
                | OrderStatus::Accepted
                | OrderStatus::PendingCancel
                | OrderStatus::Canceled
                | OrderStatus::Rejected => {
                    state.query_script_index += 1;
                    state.query_script_fill_started = false;
                    self.save_state(order_id, state).await?;
                    return Ok(Some(OrderQueryResponse {
                        adapter_order_id: order_id.to_string(),
                        latest_status: status,
                        filled_quantity: Self::cumulative_filled_quantity(state),
                        avg_fill_price: None,
                        fill_details: None,
                        rejection_reason: None,
                    }));
                }
                OrderStatus::PartiallyFilled | OrderStatus::Filled => {
                    if state.query_script_fill_started {
                        if Self::has_unapplied_fill(state) {
                            return self.build_response(order_id, state, status).await.map(Some);
                        }

                        state.query_script_fill_started = false;
                        state.query_script_index += 1;
                        self.save_state(order_id, state).await?;
                        continue;
                    }

                    if !Self::has_unapplied_fill(state)
                        && state.next_step_index < state.fill_plan.len()
                    {
                        state.next_step_index += 1;
                    }
                    state.query_script_fill_started = true;
                    self.save_state(order_id, state).await?;
                    return self.build_response(order_id, state, status).await.map(Some);
                }
                OrderStatus::PendingSubmit => return Ok(None),
            }
        }
    }
}

#[async_trait]
impl<C> ExecutionAdapter for MockLiveExecutionAdapter<C>
where
    C: MockLiveClock,
{
    fn adapter_name(&self) -> &'static str {
        "mock_live"
    }

    fn capabilities(&self) -> ExecutionCapabilities {
        ExecutionCapabilities {
            channel: ExecutionChannel::MockLive,
            status_source: ExecutionStatusSource::LocalSimulatedLifecycle,
            fill_source: ExecutionFillSource::LocalSimulatedMatcher,
            relies_on_broker_api: false,
            supports_pending_order_lifecycle: true,
            supports_partial_fill: true,
            cancel_semantics: ExecutionCancelSemantics::LocalLifecycle,
        }
    }

    async fn submit_order(
        &self,
        request: AdapterOrderRequest,
    ) -> std::result::Result<OrderInitialResponse, AdapterError> {
        let mut state = self.state_template.clone();
        if state.simulated_fill_price.is_none() {
            state.simulated_fill_price = Some(request.price);
        }
        if state.planned_fill_time.is_none() {
            state.planned_fill_time = Some(self.clock.now());
        }

        self.store
            .insert_mock_live_order_state(
                &request.client_order_id,
                Some(&request.client_order_id),
                &state,
            )
            .await
            .map_err(|err| AdapterError::Execution(err.to_string()))?;

        Ok(OrderInitialResponse {
            adapter_order_id: request.client_order_id,
            latest_status: OrderStatus::Accepted,
            filled_quantity: 0,
            avg_fill_price: None,
            fill_details: None,
            rejection_reason: None,
        })
    }

    async fn query_order(
        &self,
        order_id: &str,
    ) -> std::result::Result<OrderQueryResponse, AdapterError> {
        let mut state = self.load_state(order_id).await?;

        if state.cancel_requested {
            return Ok(OrderQueryResponse {
                adapter_order_id: order_id.to_string(),
                latest_status: OrderStatus::Canceled,
                filled_quantity: Self::cumulative_filled_quantity(&state),
                avg_fill_price: None,
                fill_details: None,
                rejection_reason: None,
            });
        }

        if let Some(response) = self.handle_query_script(order_id, &mut state).await? {
            return Ok(response);
        }

        if state
            .fault_injection
            .as_ref()
            .and_then(|fault| fault.mode.as_deref())
            == Some("unknown_once")
        {
            state.unknown_retries += 1;
            state.fault_injection = None;
            self.save_state(order_id, &state).await?;
            return Ok(OrderQueryResponse {
                adapter_order_id: order_id.to_string(),
                latest_status: OrderStatus::Unknown,
                filled_quantity: Self::cumulative_filled_quantity(&state),
                avg_fill_price: None,
                fill_details: None,
                rejection_reason: None,
            });
        }

        if state
            .fault_injection
            .as_ref()
            .and_then(|fault| fault.mode.as_deref())
            == Some("unknown_always")
        {
            state.unknown_retries += 1;
            Self::maybe_mark_recovery_exhausted(&mut state);
            self.save_state(order_id, &state).await?;
            return Ok(OrderQueryResponse {
                adapter_order_id: order_id.to_string(),
                latest_status: OrderStatus::Unknown,
                filled_quantity: Self::cumulative_filled_quantity(&state),
                avg_fill_price: None,
                fill_details: None,
                rejection_reason: None,
            });
        }

        // Network timeout simulation - returns error after delay
        if state
            .fault_injection
            .as_ref()
            .and_then(|fault| fault.mode.as_deref())
            == Some("network_timeout")
        {
            let delay_secs = state
                .fault_injection
                .as_ref()
                .and_then(|f| f.timeout_seconds)
                .unwrap_or(5);
            tokio::time::sleep(tokio::time::Duration::from_secs(delay_secs as u64)).await;
            state.fault_injection = None;
            self.save_state(order_id, &state).await?;
            return Err(AdapterError::Network(
                "Simulated network timeout".to_string(),
            ));
        }

        // Network disconnect simulation - returns error immediately
        if state
            .fault_injection
            .as_ref()
            .and_then(|fault| fault.mode.as_deref())
            == Some("network_disconnect")
        {
            state.fault_injection = None;
            self.save_state(order_id, &state).await?;
            return Err(AdapterError::Network(
                "Simulated network disconnect".to_string(),
            ));
        }

        // Delayed response simulation - delays response but succeeds
        if let Some(mode) = state
            .fault_injection
            .as_ref()
            .and_then(|fault| fault.mode.as_deref())
        {
            if mode.starts_with("delayed_response:") {
                let delay_secs: u64 = mode
                    .strip_prefix("delayed_response:")
                    .unwrap_or("2")
                    .parse()
                    .unwrap_or(2);
                tokio::time::sleep(tokio::time::Duration::from_secs(delay_secs)).await;
                // Clear fault after one delay
                state.fault_injection = None;
                self.save_state(order_id, &state).await?;
            }
        }

        // Simulated rejection - returns rejected status
        if let Some(mode) = state
            .fault_injection
            .as_ref()
            .and_then(|fault| fault.mode.clone())
        {
            if mode.starts_with("simulated_rejection") {
                let reason = mode
                    .strip_prefix("simulated_rejection:")
                    .unwrap_or("mock_rejection");
                state.fault_injection = None;
                self.save_state(order_id, &state).await?;
                return Ok(OrderQueryResponse {
                    adapter_order_id: order_id.to_string(),
                    latest_status: OrderStatus::Rejected,
                    filled_quantity: 0,
                    avg_fill_price: None,
                    fill_details: None,
                    rejection_reason: Some(reason.to_string()),
                });
            }
        }

        if !Self::has_unapplied_fill(&state) && state.next_step_index < state.fill_plan.len() {
            state.next_step_index += 1;
            self.save_state(order_id, &state).await?;
        }
        self.build_response(order_id, &state, Self::latest_status_for_state(&state))
            .await
    }

    async fn cancel_order(&self, order_id: &str) -> std::result::Result<(), AdapterError> {
        let mut state = self.load_state(order_id).await?;
        state.cancel_requested = true;
        self.save_state(order_id, &state).await
    }
}
