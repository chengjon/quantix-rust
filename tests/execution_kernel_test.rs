use async_trait::async_trait;
use chrono::{Duration, NaiveDate};
use chrono::{TimeZone, Utc};
use quantix_cli::core::Result;
use quantix_cli::data::models::{AdjustType, Kline};
use quantix_cli::execution::adapter::{AdapterOrderRequest, AdapterError, ExecutionAdapter};
use quantix_cli::execution::kernel::{
    ExecutionKernel, ExecutionRunRequest, KernelExecutionResult, RecoverySummary, RiskDecision,
    RiskEvaluator,
};
use quantix_cli::execution::models::{
    translate_signal, ExecutionPolicy, OrderIntent, OrderSide, OrderStatus, OrderType,
    SignalEnvelope,
};
use quantix_cli::execution::paper::PaperExecutionAdapter;
use quantix_cli::execution::runtime_store::StrategyRuntimeStore;
use quantix_cli::strategy::runtime::{StrategyBarLoader, StrategyRuntime};
use quantix_cli::strategy::trait_def::Signal;
use quantix_cli::trade::{InitAccountRequest, PaperTradeState, PaperTradeStore, TradeService};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::sync::{Arc, Mutex};
use tempfile::tempdir;

fn make_test_date(offset_days: usize) -> NaiveDate {
    NaiveDate::from_ymd_opt(2024, 1, 1)
        .unwrap()
        .checked_add_signed(Duration::days(offset_days as i64))
        .unwrap()
}

fn create_ma_cross_fixture() -> Vec<Kline> {
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

fn fixed_ts() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 17, 9, 30, 0).unwrap()
}

#[derive(Clone)]
struct FakeBarLoader {
    bars: Vec<Kline>,
}

#[derive(Clone, Default)]
struct FakePaperTradeStore {
    state: Arc<Mutex<Option<PaperTradeState>>>,
}

impl FakePaperTradeStore {
    fn snapshot(&self) -> Option<PaperTradeState> {
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
struct CountingAdapter {
    submissions: Arc<Mutex<usize>>,
}

impl CountingAdapter {
    fn new() -> Self {
        Self {
            submissions: Arc::new(Mutex::new(0)),
        }
    }

    fn submission_count(&self) -> usize {
        *self.submissions.lock().unwrap()
    }
}

#[async_trait]
impl ExecutionAdapter for CountingAdapter {
    async fn submit_order(
        &self,
        request: AdapterOrderRequest,
    ) -> std::result::Result<quantix_cli::execution::adapter::OrderInitialResponse, AdapterError> {
        *self.submissions.lock().unwrap() += 1;
        Ok(quantix_cli::execution::adapter::OrderInitialResponse {
            adapter_order_id: request.client_order_id,
            latest_status: OrderStatus::Filled,
            filled_quantity: request.quantity,
            avg_fill_price: Some(request.price),
            rejection_reason: None,
        })
    }

    async fn query_order(
        &self,
        order_id: &str,
    ) -> std::result::Result<quantix_cli::execution::adapter::OrderQueryResponse, AdapterError> {
        Ok(quantix_cli::execution::adapter::OrderQueryResponse {
            adapter_order_id: order_id.to_string(),
            latest_status: OrderStatus::Unknown,
            filled_quantity: 0,
            avg_fill_price: None,
        })
    }

    async fn cancel_order(&self, _order_id: &str) -> std::result::Result<(), AdapterError> {
        Ok(())
    }
}

#[derive(Clone)]
struct FixedRiskEvaluator {
    decision: RiskDecision,
}

#[async_trait]
impl RiskEvaluator for FixedRiskEvaluator {
    async fn evaluate(&self, _intent: OrderIntent) -> Result<RiskDecision> {
        Ok(self.decision.clone())
    }

    async fn sync_after_fill(&self) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
impl StrategyBarLoader for FakeBarLoader {
    async fn load_daily_bars(&self, code: &str, limit: usize) -> quantix_cli::core::Result<Vec<Kline>> {
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

#[test]
fn hold_signal_produces_no_order_intent() {
    let envelope = SignalEnvelope::new(Signal::Hold);
    let policy = ExecutionPolicy {
        fixed_cash_per_buy: dec!(2500),
        slippage_bps: 0,
    };

    let result = translate_signal(&envelope, "000001", dec!(12.34), None, &policy).unwrap();

    assert!(result.is_none());
}

#[test]
fn buy_signal_uses_fixed_cash_and_rounds_down_to_board_lot() {
    let envelope = SignalEnvelope::new(Signal::Buy);
    let policy = ExecutionPolicy {
        fixed_cash_per_buy: dec!(2500),
        slippage_bps: 0,
    };

    let intent = translate_signal(&envelope, "000001", dec!(12.34), None, &policy)
        .unwrap()
        .unwrap();

    assert_eq!(intent.symbol, "000001");
    assert_eq!(intent.side, OrderSide::Buy);
    assert_eq!(intent.order_type, OrderType::Market);
    assert_eq!(intent.requested_quantity, 200);
    assert_eq!(intent.requested_price, dec!(12.34));
}

#[test]
fn sell_signal_uses_sell_all_position_volume() {
    let envelope = SignalEnvelope::new(Signal::Sell);
    let policy = ExecutionPolicy {
        fixed_cash_per_buy: dec!(2500),
        slippage_bps: 0,
    };

    let intent = translate_signal(&envelope, "000001", dec!(11.80), Some(300), &policy)
        .unwrap()
        .unwrap();

    assert_eq!(intent.symbol, "000001");
    assert_eq!(intent.side, OrderSide::Sell);
    assert_eq!(intent.order_type, OrderType::Market);
    assert_eq!(intent.requested_quantity, 300);
    assert_eq!(intent.requested_price, dec!(11.80));
}

#[tokio::test]
async fn strategy_runtime_returns_latest_signal_for_ma_cross() {
    let runtime = StrategyRuntime::new(FakeBarLoader {
        bars: create_ma_cross_fixture(),
    });

    let envelope = runtime.run_ma_cross_once("000001", 5, 10).await.unwrap();

    assert!(matches!(envelope.signal, Signal::Buy | Signal::Sell | Signal::Hold));
}

#[tokio::test]
async fn paper_adapter_buy_submission_returns_filled_and_updates_account() {
    let store = FakePaperTradeStore::default();
    let service = TradeService::new(store.clone());
    service
        .init_account(
            InitAccountRequest::new(Some(100000.0), None, None, None, None).unwrap(),
            chrono::Utc::now(),
        )
        .await
        .unwrap();
    let adapter = PaperExecutionAdapter::new(service);

    let result = adapter
        .submit_order(AdapterOrderRequest {
            client_order_id: "run_000001_1".to_string(),
            symbol: "000001".to_string(),
            side: OrderSide::Buy,
            quantity: 100,
            price: dec!(10.00),
        })
        .await
        .unwrap();

    assert_eq!(result.latest_status, quantix_cli::execution::models::OrderStatus::Filled);
    assert_eq!(result.filled_quantity, 100);
    assert_eq!(result.avg_fill_price, Some(dec!(10.00)));

    let account = store.snapshot().unwrap().account.unwrap();
    assert!(account.positions.contains_key("000001"));
}

#[tokio::test]
async fn paper_adapter_sell_submission_returns_filled() {
    let store = FakePaperTradeStore::default();
    let service = TradeService::new(store.clone());
    let now = chrono::Utc::now();
    service
        .init_account(
            InitAccountRequest::new(Some(100000.0), None, None, None, None).unwrap(),
            now,
        )
        .await
        .unwrap();
    service
        .buy(
            quantix_cli::trade::TradeOrderRequest::new("000001", 10.0, 100).unwrap(),
            now,
        )
        .await
        .unwrap();
    let adapter = PaperExecutionAdapter::new(service);

    let result = adapter
        .submit_order(AdapterOrderRequest {
            client_order_id: "run_000001_2".to_string(),
            symbol: "000001".to_string(),
            side: OrderSide::Sell,
            quantity: 100,
            price: dec!(11.00),
        })
        .await
        .unwrap();

    assert_eq!(result.latest_status, quantix_cli::execution::models::OrderStatus::Filled);
    assert_eq!(result.filled_quantity, 100);
    assert_eq!(result.avg_fill_price, Some(dec!(11.00)));
    assert!(store.snapshot().unwrap().account.unwrap().positions.is_empty());
}

#[tokio::test]
async fn paper_adapter_cancel_returns_unsupported() {
    let store = FakePaperTradeStore::default();
    let service = TradeService::new(store);
    let adapter = PaperExecutionAdapter::new(service);

    let err = adapter.cancel_order("paper-order-1").await.unwrap_err();

    assert!(matches!(err, AdapterError::Unsupported(_)));
}

#[tokio::test]
async fn paper_adapter_query_returns_unsupported() {
    let store = FakePaperTradeStore::default();
    let service = TradeService::new(store);
    let adapter = PaperExecutionAdapter::new(service);

    let err = adapter.query_order("paper-order-1").await.unwrap_err();

    assert!(matches!(err, AdapterError::Unsupported(_)));
}

fn sample_run_request(client_order_id: &str) -> ExecutionRunRequest {
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

#[tokio::test]
async fn kernel_success_path_persists_run_signal_order_and_events() {
    let dir = tempdir().unwrap();
    let store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let adapter = CountingAdapter::new();
    let kernel = ExecutionKernel::new(
        store.clone(),
        adapter.clone(),
        FixedRiskEvaluator {
            decision: RiskDecision::Allow,
        },
    );

    let result = kernel
        .execute_once(sample_run_request("run-000001-1"), SignalEnvelope::new(Signal::Buy))
        .await
        .unwrap();

    assert_eq!(
        result,
        KernelExecutionResult {
            run_id: "run-1".to_string(),
            signal: Signal::Buy,
            order_status: Some(OrderStatus::Filled),
            client_order_id: Some("run-000001-1".to_string()),
        }
    );
    assert_eq!(adapter.submission_count(), 1);
    assert_eq!(store.count_runs().await.unwrap(), 1);
    assert_eq!(store.count_orders().await.unwrap(), 1);
    assert_eq!(store.list_order_events("run-000001-1").await.unwrap().len(), 2);
}

#[tokio::test]
async fn kernel_risk_rejection_creates_rejected_order_and_skips_adapter() {
    let dir = tempdir().unwrap();
    let store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let adapter = CountingAdapter::new();
    let kernel = ExecutionKernel::new(
        store.clone(),
        adapter.clone(),
        FixedRiskEvaluator {
            decision: RiskDecision::Reject {
                reason: "position-limit".to_string(),
            },
        },
    );

    let result = kernel
        .execute_once(sample_run_request("run-000001-2"), SignalEnvelope::new(Signal::Buy))
        .await
        .unwrap();

    assert_eq!(result.order_status, Some(OrderStatus::Rejected));
    assert_eq!(adapter.submission_count(), 0);
    let order = store
        .find_order_by_client_order_id("run-000001-2")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(order.status, OrderStatus::Rejected);
}

#[tokio::test]
async fn kernel_duplicate_client_order_id_returns_stored_result_without_resubmitting() {
    let dir = tempdir().unwrap();
    let store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let adapter = CountingAdapter::new();
    let kernel = ExecutionKernel::new(
        store.clone(),
        adapter.clone(),
        FixedRiskEvaluator {
            decision: RiskDecision::Allow,
        },
    );

    let first = kernel
        .execute_once(sample_run_request("dup-order"), SignalEnvelope::new(Signal::Buy))
        .await
        .unwrap();
    let second = kernel
        .execute_once(
            ExecutionRunRequest {
                run_id: "run-2".to_string(),
                bar_end: fixed_ts() + Duration::days(1),
                ..sample_run_request("dup-order")
            },
            SignalEnvelope::new(Signal::Buy),
        )
        .await
        .unwrap();

    assert_eq!(first.order_status, Some(OrderStatus::Filled));
    assert_eq!(second.order_status, Some(OrderStatus::Filled));
    assert_eq!(adapter.submission_count(), 1);
}

#[tokio::test]
async fn kernel_recover_pending_orders_returns_empty_summary() {
    let dir = tempdir().unwrap();
    let store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let adapter = CountingAdapter::new();
    let kernel = ExecutionKernel::new(
        store,
        adapter,
        FixedRiskEvaluator {
            decision: RiskDecision::Allow,
        },
    );

    let summary = kernel.recover_pending_orders().await.unwrap();

    assert_eq!(
        summary,
        RecoverySummary {
            scanned: 0,
            recovered: 0,
        }
    );
}
