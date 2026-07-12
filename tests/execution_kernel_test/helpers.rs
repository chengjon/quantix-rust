pub use async_trait::async_trait;
pub use chrono::{Duration, NaiveDate};
pub use chrono::{TimeZone, Utc};
pub use quantix_cli::core::signal::Signal;
pub use quantix_cli::core::{QuantixError, Result};
pub use quantix_cli::data::models::{AdjustType, Kline};
pub use quantix_cli::execution::adapter::{AdapterError, AdapterOrderRequest, ExecutionAdapter};
pub use quantix_cli::execution::kernel::{
    ExecutionKernel, ExecutionRunRequest, FillDeltaApplier, KernelExecutionResult,
    PreparedExecutionRequest, RecoverySummary, RiskDecision, RiskEvaluator,
};
pub use quantix_cli::execution::mock_live::{MockLiveClock, MockLiveExecutionAdapter};
pub use quantix_cli::execution::models::{
    ExecutionPolicy, FillDeltaContext, FillDeltaResult, FillDetails, MockLiveFaultInjection,
    MockLiveFillStep, MockLiveOrderState, OrderIntent, OrderRecord, OrderSide, OrderStatus,
    OrderType, SignalEnvelope, StrategyRunRecord, StrategyRunStatus, translate_signal,
};
pub use quantix_cli::execution::paper::{IMMEDIATE_FILL_ONLY, PaperExecutionAdapter};
pub use quantix_cli::execution::runtime_store::StrategyRuntimeStore;
pub use quantix_cli::strategy::runtime::{StrategyBarLoader, StrategyRuntime};
pub use quantix_cli::trade::{InitAccountRequest, PaperTradeState, PaperTradeStore, TradeService};
pub use rust_decimal::Decimal;
pub use rust_decimal::prelude::ToPrimitive;
pub use rust_decimal_macros::dec;
pub use std::collections::HashSet;
pub use std::sync::{Arc, Mutex};
pub use tempfile::tempdir;

pub(crate) fn make_test_date(offset_days: usize) -> NaiveDate {
    NaiveDate::from_ymd_opt(2024, 1, 1)
        .unwrap()
        .checked_add_signed(Duration::days(offset_days as i64))
        .unwrap()
}

pub(crate) fn create_ma_cross_fixture() -> Vec<Kline> {
    let mut prices = Vec::new();
    let mut price = 100.0;

    for _ in 0..20 {
        prices.push(price);
        price -= 0.5;
    }

    for _ in 0..40 {
        prices.push(price);
        price += 0.5;
    }

    prices
        .iter()
        .enumerate()
        .map(|(i, price)| Kline {
            code: "000001".to_string(),
            date: make_test_date(i),
            open: Decimal::from_str_exact(&price.to_string()).unwrap(),
            high: Decimal::from_str_exact(&(price + 1.0).to_string()).unwrap(),
            low: Decimal::from_str_exact(&(price - 1.0).to_string()).unwrap(),
            close: Decimal::from_str_exact(&price.to_string()).unwrap(),
            volume: 1_000_000,
            amount: Some(
                Decimal::from_str_exact(&price.to_string()).unwrap() * Decimal::from(1_000_000),
            ),
            adjust_type: AdjustType::None,
        })
        .collect()
}

pub(crate) fn fixed_ts() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 17, 9, 30, 0).unwrap()
}

pub(crate) fn sample_run(symbol: &str, bar_end: chrono::DateTime<Utc>) -> StrategyRunRecord {
    StrategyRunRecord {
        run_id: "runtime-run".to_string(),
        strategy_name: "ma_cross".to_string(),
        mode: "mock_live".to_string(),
        trigger: "once".to_string(),
        status: StrategyRunStatus::Running,
        symbol: symbol.to_string(),
        timeframe: "1d".to_string(),
        bar_end,
        started_at: fixed_ts(),
        finished_at: None,
        metadata_json: serde_json::json!({}),
    }
}

pub(crate) fn sample_order(run_id: &str, client_order_id: &str) -> OrderRecord {
    OrderRecord {
        order_id: client_order_id.to_string(),
        client_order_id: client_order_id.to_string(),
        run_id: run_id.to_string(),
        symbol: "000001".to_string(),
        side: OrderSide::Buy,
        order_type: OrderType::Market,
        requested_quantity: 100,
        requested_price: dec!(12.34),
        filled_quantity: 0,
        remaining_quantity: 100,
        avg_fill_price: None,
        status: OrderStatus::PendingSubmit,
        adapter: "mock_live".to_string(),
        created_at: fixed_ts(),
        updated_at: fixed_ts(),
        last_transition_at: fixed_ts(),
        version: 0,
        payload_json: serde_json::json!({}),
    }
}

#[derive(Clone, Copy)]
pub(crate) struct FixedMockLiveClock;

impl MockLiveClock for FixedMockLiveClock {
    fn now(&self) -> chrono::DateTime<Utc> {
        fixed_ts()
    }
}

#[derive(Clone)]
pub(crate) struct FakeBarLoader {
    pub(crate) bars: Vec<Kline>,
}

#[derive(Clone, Default)]
pub(crate) struct FakePaperTradeStore {
    state: Arc<Mutex<Option<PaperTradeState>>>,
}

impl FakePaperTradeStore {
    pub(crate) fn snapshot(&self) -> Option<PaperTradeState> {
        self.state.lock().unwrap().clone()
    }
}

#[async_trait]
impl PaperTradeStore for FakePaperTradeStore {
    async fn load_state(&self) -> Result<Option<PaperTradeState>> {
        Ok(self.snapshot())
    }

    async fn save_state(&self, state: &PaperTradeState) -> Result<()> {
        *self.state.lock().unwrap() = Some(state.clone());
        Ok(())
    }
}

#[derive(Clone)]
pub(crate) struct RecordingTradeFillDeltaApplier<Store> {
    trade_service: TradeService<Store>,
    seen_fill_ids: Arc<Mutex<HashSet<(String, u64)>>>,
    results: Arc<Mutex<Vec<FillDeltaResult>>>,
}

impl<Store> RecordingTradeFillDeltaApplier<Store>
where
    Store: PaperTradeStore,
{
    pub(crate) fn new(store: Store) -> Self {
        Self {
            trade_service: TradeService::new(store),
            seen_fill_ids: Arc::new(Mutex::new(HashSet::new())),
            results: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub(crate) fn results(&self) -> Vec<FillDeltaResult> {
        self.results.lock().unwrap().clone()
    }
}

#[async_trait]
impl<Store> FillDeltaApplier for RecordingTradeFillDeltaApplier<Store>
where
    Store: PaperTradeStore + Clone,
{
    async fn apply_fill_delta(&self, ctx: FillDeltaContext) -> Result<FillDeltaResult> {
        let Some(fill_details) = ctx.fill_details.clone() else {
            return Ok(FillDeltaResult {
                applied: false,
                delta_quantity: 0,
                trade_record_id: None,
            });
        };

        let is_new_fill = {
            let mut seen = self.seen_fill_ids.lock().unwrap();
            seen.insert((ctx.client_order_id.clone(), fill_details.fill_id))
        };
        if !is_new_fill {
            let result = FillDeltaResult {
                applied: false,
                delta_quantity: 0,
                trade_record_id: None,
            };
            self.results.lock().unwrap().push(result.clone());
            return Ok(result);
        }

        let request = quantix_cli::trade::TradeOrderRequest::new(
            ctx.symbol.clone(),
            fill_details.fill_price.to_f64().unwrap(),
            fill_details.fill_quantity,
        )?;
        let record = match ctx.side {
            OrderSide::Buy => self.trade_service.buy(request, ctx.event_time).await?,
            OrderSide::Sell => self.trade_service.sell(request, ctx.event_time).await?,
        };
        let result = FillDeltaResult {
            applied: true,
            delta_quantity: fill_details.fill_quantity,
            trade_record_id: Some(record.id),
        };
        self.results.lock().unwrap().push(result.clone());
        Ok(result)
    }
}

#[derive(Clone, Copy, Default)]
pub(crate) struct FailingFillDeltaApplier;

#[async_trait]
impl FillDeltaApplier for FailingFillDeltaApplier {
    async fn apply_fill_delta(&self, _ctx: FillDeltaContext) -> Result<FillDeltaResult> {
        Err(QuantixError::Other("fill delta apply failed".to_string()))
    }
}

#[derive(Clone)]
pub(crate) struct CountingAdapter {
    submissions: Arc<Mutex<usize>>,
    response: Arc<Mutex<CountingAdapterResponse>>,
}

impl CountingAdapter {
    pub(crate) fn new() -> Self {
        Self {
            submissions: Arc::new(Mutex::new(0)),
            response: Arc::new(Mutex::new(CountingAdapterResponse::filled())),
        }
    }

    pub(crate) fn accepted() -> Self {
        Self {
            submissions: Arc::new(Mutex::new(0)),
            response: Arc::new(Mutex::new(CountingAdapterResponse::accepted())),
        }
    }

    pub(crate) fn partial_fill() -> Self {
        Self {
            submissions: Arc::new(Mutex::new(0)),
            response: Arc::new(Mutex::new(CountingAdapterResponse::partial_fill())),
        }
    }

    pub(crate) fn submission_count(&self) -> usize {
        *self.submissions.lock().unwrap()
    }
}

#[derive(Clone)]
pub(crate) struct CountingAdapterResponse {
    latest_status: OrderStatus,
    filled_quantity: i64,
}

impl CountingAdapterResponse {
    pub(crate) fn filled() -> Self {
        Self {
            latest_status: OrderStatus::Filled,
            filled_quantity: 200,
        }
    }

    pub(crate) fn accepted() -> Self {
        Self {
            latest_status: OrderStatus::Accepted,
            filled_quantity: 0,
        }
    }

    pub(crate) fn partial_fill() -> Self {
        Self {
            latest_status: OrderStatus::PartiallyFilled,
            filled_quantity: 50,
        }
    }
}

#[async_trait]
impl ExecutionAdapter for CountingAdapter {
    fn adapter_name(&self) -> &'static str {
        "counting"
    }

    fn capabilities(&self) -> quantix_cli::execution::adapter::ExecutionCapabilities {
        quantix_cli::execution::adapter::ExecutionCapabilities {
            channel: quantix_cli::execution::adapter::ExecutionChannel::MockLive,
            status_source:
                quantix_cli::execution::adapter::ExecutionStatusSource::LocalSimulatedLifecycle,
            fill_source:
                quantix_cli::execution::adapter::ExecutionFillSource::LocalSimulatedMatcher,
            relies_on_broker_api: false,
            supports_pending_order_lifecycle: true,
            supports_partial_fill: true,
            cancel_semantics:
                quantix_cli::execution::adapter::ExecutionCancelSemantics::LocalLifecycle,
        }
    }

    async fn submit_order(
        &self,
        request: AdapterOrderRequest,
    ) -> std::result::Result<quantix_cli::execution::adapter::OrderInitialResponse, AdapterError>
    {
        *self.submissions.lock().unwrap() += 1;
        let response = self.response.lock().unwrap().clone();
        let filled_quantity = response.filled_quantity.min(request.quantity);
        Ok(quantix_cli::execution::adapter::OrderInitialResponse {
            adapter_order_id: request.client_order_id,
            latest_status: response.latest_status,
            filled_quantity,
            avg_fill_price: Some(request.price),
            fill_details: (filled_quantity > 0).then_some(FillDetails {
                fill_id: 1,
                fill_quantity: filled_quantity,
                fill_price: request.price,
                last_fill_price: request.price,
                last_fill_quantity: filled_quantity,
                total_fills: 1,
                commission: Decimal::ZERO,
                fees: Decimal::ZERO,
                venue: "test".to_string(),
                broker_fill_id: String::new(),
            }),
            rejection_reason: None,
        })
    }

    async fn query_order(
        &self,
        order_id: &str,
    ) -> std::result::Result<quantix_cli::execution::adapter::OrderQueryResponse, AdapterError>
    {
        Ok(quantix_cli::execution::adapter::OrderQueryResponse {
            adapter_order_id: order_id.to_string(),
            latest_status: OrderStatus::Unknown,
            filled_quantity: 0,
            avg_fill_price: None,
            fill_details: None,
            rejection_reason: None,
        })
    }

    async fn cancel_order(&self, _order_id: &str) -> std::result::Result<(), AdapterError> {
        Ok(())
    }
}

#[derive(Clone)]
pub(crate) struct FixedRiskEvaluator {
    pub(crate) decision: RiskDecision,
    pub(crate) sync_calls: Arc<Mutex<usize>>,
}

#[async_trait]
impl RiskEvaluator for FixedRiskEvaluator {
    async fn evaluate(&self, _intent: OrderIntent) -> Result<RiskDecision> {
        Ok(self.decision.clone())
    }

    async fn sync_after_fill(&self) -> Result<()> {
        *self.sync_calls.lock().unwrap() += 1;
        Ok(())
    }
}

#[async_trait]
impl StrategyBarLoader for FakeBarLoader {
    async fn load_daily_bars(
        &self,
        code: &str,
        limit: usize,
    ) -> quantix_cli::core::Result<Vec<Kline>> {
        let mut filtered: Vec<Kline> = self
            .bars
            .iter()
            .filter(|bar| bar.code == code)
            .cloned()
            .collect();
        if filtered.len() > limit {
            filtered = filtered.split_off(filtered.len() - limit);
        }
        Ok(filtered)
    }
}

pub(crate) fn sample_run_request(client_order_id: &str) -> ExecutionRunRequest {
    ExecutionRunRequest {
        run_id: "run-1".to_string(),
        strategy_name: "ma_cross".to_string(),
        mode: "paper".to_string(),
        trigger: "once".to_string(),
        symbol: "000001".to_string(),
        timeframe: "1d".to_string(),
        bar_end: fixed_ts(),
        market_price: dec!(12.34),
        held_volume: Some(300),
        policy: ExecutionPolicy {
            fixed_cash_per_buy: dec!(2500),
            slippage_bps: 0,
        },
        client_order_id: client_order_id.to_string(),
    }
}

pub(crate) fn sample_prepared_request(
    mode: &str,
    client_order_id: &str,
) -> PreparedExecutionRequest {
    PreparedExecutionRequest {
        run_id: format!("prepared-{client_order_id}"),
        strategy_name: "ma_cross".to_string(),
        mode: mode.to_string(),
        trigger: "request".to_string(),
        symbol: "000001".to_string(),
        timeframe: "1d".to_string(),
        bar_end: fixed_ts(),
        signal: Signal::Buy,
        signal_payload_json: serde_json::json!({
            "source": "execution_request",
        }),
        intent: OrderIntent {
            symbol: "000001".to_string(),
            side: OrderSide::Buy,
            requested_quantity: 800,
            requested_price: dec!(12.34),
            order_type: OrderType::Market,
            reason: "signal_buy".to_string(),
            policy_snapshot_json: serde_json::json!({
                "fixed_cash_per_buy": "10000",
                "slippage_bps": 0
            }),
        },
        client_order_id: client_order_id.to_string(),
    }
}

pub(crate) fn sample_fill_delta_context(client_order_id: &str, fill_id: u64) -> FillDeltaContext {
    FillDeltaContext {
        order_id: client_order_id.to_string(),
        client_order_id: client_order_id.to_string(),
        symbol: "000001".to_string(),
        side: OrderSide::Buy,
        requested_price: dec!(12.34),
        old_filled_quantity: 0,
        new_filled_quantity: 50,
        fill_details: Some(FillDetails {
            fill_id,
            fill_quantity: 50,
            fill_price: dec!(12.34),
            last_fill_price: dec!(12.34),
            last_fill_quantity: 50,
            total_fills: 1,
            commission: Decimal::ZERO,
            fees: Decimal::ZERO,
            venue: "test".to_string(),
            broker_fill_id: String::new(),
        }),
        event_time: fixed_ts(),
    }
}
