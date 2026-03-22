use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::execution::adapter::{
    AdapterError, AdapterOrderRequest, ExecutionAdapter, OrderInitialResponse, OrderQueryResponse,
};
use crate::execution::models::{MockLiveOrderState, OrderStatus};
use crate::execution::runtime_store::StrategyRuntimeStore;

pub trait MockLiveClock: Clone + Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

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
    pub fn new(store: StrategyRuntimeStore, clock: C) -> Self {
        Self::with_state_template(store, clock, MockLiveOrderState::default())
    }

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

    async fn load_state(&self, order_id: &str) -> std::result::Result<MockLiveOrderState, AdapterError> {
        self.store
            .get_mock_live_order_state(order_id)
            .await
            .map_err(|err| AdapterError::Execution(err.to_string()))?
            .ok_or_else(|| AdapterError::Execution(format!("mock_live order not found: {order_id}")))
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

    fn cumulative_filled_quantity(state: &MockLiveOrderState) -> i64 {
        state
            .fill_plan
            .iter()
            .take(state.next_step_index)
            .map(|step| step.quantity)
            .sum()
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

    async fn submit_order(
        &self,
        request: AdapterOrderRequest,
    ) -> std::result::Result<OrderInitialResponse, AdapterError> {
        let mut state = self.state_template.clone();
        if state.planned_fill_time.is_none() {
            state.planned_fill_time = Some(self.clock.now());
        }

        self.store
            .insert_mock_live_order_state(&request.client_order_id, Some(&request.client_order_id), &state)
            .await
            .map_err(|err| AdapterError::Execution(err.to_string()))?;

        Ok(OrderInitialResponse {
            adapter_order_id: request.client_order_id,
            latest_status: OrderStatus::Accepted,
            filled_quantity: 0,
            avg_fill_price: None,
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
            });
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
            });
        }

        if state
            .fault_injection
            .as_ref()
            .and_then(|fault| fault.mode.as_deref())
            == Some("unknown_always")
        {
            state.unknown_retries += 1;
            if state.unknown_retries > 3 {
                state.recovery_exhausted = true;
                state.exhausted_reason = Some("unknown_retry_budget_exceeded".to_string());
            }
            self.save_state(order_id, &state).await?;
            return Ok(OrderQueryResponse {
                adapter_order_id: order_id.to_string(),
                latest_status: OrderStatus::Unknown,
                filled_quantity: Self::cumulative_filled_quantity(&state),
                avg_fill_price: None,
            });
        }

        if state.next_step_index < state.fill_plan.len() {
            state.next_step_index += 1;
            state.last_applied_fill_id += 1;
            let filled_quantity = Self::cumulative_filled_quantity(&state);
            let latest_status = if state.next_step_index < state.fill_plan.len() {
                OrderStatus::PartiallyFilled
            } else {
                OrderStatus::Filled
            };
            self.save_state(order_id, &state).await?;
            return Ok(OrderQueryResponse {
                adapter_order_id: order_id.to_string(),
                latest_status,
                filled_quantity,
                avg_fill_price: None,
            });
        }

        Ok(OrderQueryResponse {
            adapter_order_id: order_id.to_string(),
            latest_status: OrderStatus::Accepted,
            filled_quantity: Self::cumulative_filled_quantity(&state),
            avg_fill_price: None,
        })
    }

    async fn cancel_order(&self, order_id: &str) -> std::result::Result<(), AdapterError> {
        let mut state = self.load_state(order_id).await?;
        state.cancel_requested = true;
        self.save_state(order_id, &state).await
    }
}
