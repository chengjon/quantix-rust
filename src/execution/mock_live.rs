use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

use crate::execution::adapter::{
    AdapterError, AdapterOrderRequest, ExecutionAdapter, OrderInitialResponse, OrderQueryResponse,
};
use crate::execution::models::{FillDetails, MockLiveOrderState, OrderStatus};
use crate::execution::runtime_store::StrategyRuntimeStore;

pub trait MockLiveClock: Clone + Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SystemMockLiveClock;

impl MockLiveClock for SystemMockLiveClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
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
            return Err(AdapterError::Network("Simulated network timeout".to_string()));
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
            return Err(AdapterError::Network("Simulated network disconnect".to_string()));
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

        let fill_price = match state.simulated_fill_price {
            Some(price) => price,
            None => self.load_requested_price(order_id).await?,
        };
        let fill_details = if Self::has_unapplied_fill(&state) {
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
            latest_status: Self::latest_status_for_state(&state),
            filled_quantity: Self::cumulative_filled_quantity(&state),
            avg_fill_price: (state.next_step_index > 0).then_some(fill_price),
            fill_details,
            rejection_reason: None,
        })
    }

    async fn cancel_order(&self, order_id: &str) -> std::result::Result<(), AdapterError> {
        let mut state = self.load_state(order_id).await?;
        state.cancel_requested = true;
        self.save_state(order_id, &state).await
    }
}
