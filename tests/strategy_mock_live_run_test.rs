use async_trait::async_trait;
use chrono::{DateTime, Duration, NaiveDate, TimeZone, Utc};
use quantix_cli::core::Result;
use quantix_cli::data::models::{AdjustType, Kline};
use quantix_cli::execution::kernel::{
    ExecutionKernel, ExecutionRunRequest, FillDeltaApplier, RiskDecision, RiskEvaluator,
};
use quantix_cli::execution::mock_live::{MockLiveClock, MockLiveExecutionAdapter};
use quantix_cli::execution::models::{
    ExecutionPolicy, FillDeltaContext, FillDeltaResult, MockLiveFillStep, MockLiveOrderState,
    OrderIntent, OrderSide, OrderStatus,
};
use quantix_cli::execution::reconciliation::ReconciliationService;
use quantix_cli::execution::runtime_store::StrategyRuntimeStore;
use quantix_cli::strategy::runtime::{StrategyBarLoader, StrategyRuntime};
use quantix_cli::strategy::trait_def::Signal;
use quantix_cli::trade::{InitAccountRequest, JsonPaperTradeStore, PaperTradeStore, TradeService};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal_macros::dec;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use tempfile::tempdir;

fn fixed_ts() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 17, 9, 30, 0).unwrap()
}

fn make_test_date(offset_days: usize) -> NaiveDate {
    NaiveDate::from_ymd_opt(2024, 1, 1)
        .unwrap()
        .checked_add_signed(Duration::days(offset_days as i64))
        .unwrap()
}

fn buy_fixture() -> Vec<Kline> {
    let prices = [
        10.0, 9.0, 8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0, 1.0, 1.0, 1.0, 1.0, 12.0,
    ];
    prices
        .iter()
        .enumerate()
        .map(|(idx, price)| Kline {
            code: "000001".to_string(),
            date: make_test_date(idx),
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

#[derive(Clone)]
struct FakeBarLoader {
    bars: Vec<Kline>,
}

#[async_trait]
impl StrategyBarLoader for FakeBarLoader {
    async fn load_daily_bars(&self, code: &str, limit: usize) -> Result<Vec<Kline>> {
        let mut rows: Vec<Kline> = self
            .bars
            .iter()
            .filter(|bar| bar.code == code)
            .cloned()
            .collect();
        if rows.len() > limit {
            rows = rows.split_off(rows.len() - limit);
        }
        Ok(rows)
    }
}

#[derive(Clone)]
struct FixedRisk {
    decision: RiskDecision,
}

#[async_trait]
impl RiskEvaluator for FixedRisk {
    async fn evaluate(&self, _intent: OrderIntent) -> Result<RiskDecision> {
        Ok(self.decision.clone())
    }

    async fn sync_after_fill(&self) -> Result<()> {
        Ok(())
    }
}

#[derive(Clone, Copy)]
struct FixedClock;

impl MockLiveClock for FixedClock {
    fn now(&self) -> DateTime<Utc> {
        fixed_ts()
    }
}

#[derive(Clone)]
struct TradeFillDeltaApplier<Store> {
    trade_service: TradeService<Store>,
    seen_fill_ids: Arc<Mutex<HashSet<(String, u64)>>>,
}

impl<Store> TradeFillDeltaApplier<Store>
where
    Store: PaperTradeStore,
{
    fn new(store: Store) -> Self {
        Self {
            trade_service: TradeService::new(store),
            seen_fill_ids: Arc::new(Mutex::new(HashSet::new())),
        }
    }
}

#[async_trait]
impl<Store> FillDeltaApplier for TradeFillDeltaApplier<Store>
where
    Store: PaperTradeStore + Clone,
{
    async fn apply_fill_delta(&self, ctx: FillDeltaContext) -> Result<FillDeltaResult> {
        let Some(fill_details) = ctx.fill_details else {
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
            return Ok(FillDeltaResult {
                applied: false,
                delta_quantity: 0,
                trade_record_id: None,
            });
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

        Ok(FillDeltaResult {
            applied: true,
            delta_quantity: fill_details.fill_quantity,
            trade_record_id: Some(record.id),
        })
    }
}

fn sample_request(
    run_id: &str,
    client_order_id: &str,
    bar_end: DateTime<Utc>,
) -> ExecutionRunRequest {
    ExecutionRunRequest {
        run_id: run_id.to_string(),
        strategy_name: "ma_cross".to_string(),
        mode: "mock_live".to_string(),
        trigger: "once".to_string(),
        symbol: "000001".to_string(),
        timeframe: "1d".to_string(),
        bar_end,
        market_price: dec!(12),
        held_volume: None,
        policy: ExecutionPolicy {
            fixed_cash_per_buy: dec!(12000),
            slippage_bps: 0,
        },
        client_order_id: client_order_id.to_string(),
    }
}

#[tokio::test]
async fn successful_mock_live_run_writes_runtime_rows_without_mutating_account() {
    let dir = tempdir().unwrap();
    let trade_store = JsonPaperTradeStore::new(dir.path().join("paper_trade.json"));
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();

    let trade_service = TradeService::new(trade_store.clone());
    trade_service
        .init_account(
            InitAccountRequest::new(Some(100000.0), None, None, None, None).unwrap(),
            fixed_ts(),
        )
        .await
        .unwrap();

    let loader = FakeBarLoader {
        bars: buy_fixture(),
    };
    let envelope = StrategyRuntime::new(loader.clone())
        .run_ma_cross_once("000001", 5, 10)
        .await
        .unwrap();
    assert_eq!(envelope.signal, Signal::Buy);

    let kernel = ExecutionKernel::new(
        runtime_store.clone(),
        MockLiveExecutionAdapter::new(runtime_store.clone(), FixedClock),
        FixedRisk {
            decision: RiskDecision::Allow,
        },
    );

    let result = kernel
        .execute_once(
            sample_request("run-mock-1", "run-mock-1_000001_1", fixed_ts()),
            envelope,
        )
        .await
        .unwrap();

    assert_eq!(result.order_status, Some(OrderStatus::Accepted));
    assert_eq!(runtime_store.count_runs().await.unwrap(), 1);
    assert_eq!(runtime_store.count_signal_events().await.unwrap(), 1);
    assert_eq!(runtime_store.count_orders().await.unwrap(), 1);

    let order = runtime_store
        .find_order_by_client_order_id("run-mock-1_000001_1")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(order.status, OrderStatus::Accepted);

    let state = trade_store.load_state().await.unwrap().unwrap();
    assert!(state.account.unwrap().positions.is_empty());
}

#[tokio::test]
async fn second_mock_live_run_on_same_bar_is_deduplicated_by_run_key() {
    let dir = tempdir().unwrap();
    let trade_store = JsonPaperTradeStore::new(dir.path().join("paper_trade.json"));
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();

    let trade_service = TradeService::new(trade_store.clone());
    trade_service
        .init_account(
            InitAccountRequest::new(Some(100000.0), None, None, None, None).unwrap(),
            fixed_ts(),
        )
        .await
        .unwrap();

    let loader = FakeBarLoader {
        bars: buy_fixture(),
    };
    let envelope = StrategyRuntime::new(loader)
        .run_ma_cross_once("000001", 5, 10)
        .await
        .unwrap();

    let kernel = ExecutionKernel::new(
        runtime_store.clone(),
        MockLiveExecutionAdapter::new(runtime_store.clone(), FixedClock),
        FixedRisk {
            decision: RiskDecision::Allow,
        },
    );

    let first = kernel
        .execute_once(
            sample_request("run-mock-1", "dup-mock-order", fixed_ts()),
            envelope.clone(),
        )
        .await
        .unwrap();
    let second = kernel
        .execute_once(
            sample_request("run-mock-2", "dup-mock-order", fixed_ts()),
            envelope,
        )
        .await
        .unwrap();

    assert_eq!(first.order_status, Some(OrderStatus::Accepted));
    assert_eq!(second.order_status, Some(OrderStatus::Accepted));
    assert_eq!(runtime_store.count_runs().await.unwrap(), 1);
    assert_eq!(runtime_store.count_orders().await.unwrap(), 1);
}

#[tokio::test]
async fn mock_live_recovery_advances_runtime_order_status() {
    let dir = tempdir().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let loader = FakeBarLoader {
        bars: buy_fixture(),
    };
    let envelope = StrategyRuntime::new(loader)
        .run_ma_cross_once("000001", 5, 10)
        .await
        .unwrap();

    let kernel = ExecutionKernel::new(
        runtime_store.clone(),
        MockLiveExecutionAdapter::with_state_template(
            runtime_store.clone(),
            FixedClock,
            MockLiveOrderState {
                fill_plan: vec![
                    MockLiveFillStep {
                        quantity: 50,
                        delay_secs: 0,
                    },
                    MockLiveFillStep {
                        quantity: 50,
                        delay_secs: 0,
                    },
                ],
                ..Default::default()
            },
        ),
        FixedRisk {
            decision: RiskDecision::Allow,
        },
    );

    let result = kernel
        .execute_once(
            sample_request("run-mock-rec-1", "run-mock-rec-1_000001_1", fixed_ts()),
            envelope,
        )
        .await
        .unwrap();
    assert_eq!(result.order_status, Some(OrderStatus::Accepted));

    let summary = kernel.recover_pending_orders().await.unwrap();
    assert_eq!(summary.scanned, 1);
    assert_eq!(summary.recovered, 1);

    let order = runtime_store
        .find_order_by_client_order_id("run-mock-rec-1_000001_1")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(order.status, OrderStatus::PartiallyFilled);
    assert_eq!(order.filled_quantity, 50);
}

#[tokio::test]
async fn mock_live_recovery_applies_only_new_fill_deltas_to_account() {
    let dir = tempdir().unwrap();
    let trade_store = JsonPaperTradeStore::new(dir.path().join("paper_trade.json"));
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();

    let trade_service = TradeService::new(trade_store.clone());
    trade_service
        .init_account(
            InitAccountRequest::new(Some(100000.0), None, None, None, None).unwrap(),
            fixed_ts(),
        )
        .await
        .unwrap();

    let loader = FakeBarLoader {
        bars: buy_fixture(),
    };
    let envelope = StrategyRuntime::new(loader)
        .run_ma_cross_once("000001", 5, 10)
        .await
        .unwrap();

    let kernel = ExecutionKernel::with_fill_delta(
        runtime_store.clone(),
        MockLiveExecutionAdapter::with_state_template(
            runtime_store.clone(),
            FixedClock,
            MockLiveOrderState {
                fill_plan: vec![
                    MockLiveFillStep {
                        quantity: 50,
                        delay_secs: 0,
                    },
                    MockLiveFillStep {
                        quantity: 50,
                        delay_secs: 0,
                    },
                ],
                ..Default::default()
            },
        ),
        TradeFillDeltaApplier::new(trade_store.clone()),
        FixedRisk {
            decision: RiskDecision::Allow,
        },
    );

    let result = kernel
        .execute_once(
            sample_request("run-mock-rec-2", "run-mock-rec-2_000001_1", fixed_ts()),
            envelope,
        )
        .await
        .unwrap();
    assert_eq!(result.order_status, Some(OrderStatus::Accepted));

    let before_recovery = trade_store.load_state().await.unwrap().unwrap();
    assert!(
        before_recovery
            .account
            .as_ref()
            .unwrap()
            .positions
            .is_empty()
    );
    assert_eq!(before_recovery.trade_records.len(), 0);

    let first_summary = kernel.recover_pending_orders().await.unwrap();
    assert_eq!(first_summary.scanned, 1);
    assert_eq!(first_summary.recovered, 1);

    let after_first = trade_store.load_state().await.unwrap().unwrap();
    assert_eq!(
        after_first
            .account
            .as_ref()
            .unwrap()
            .positions
            .get("000001")
            .unwrap()
            .volume,
        50
    );
    assert_eq!(after_first.trade_records.len(), 1);

    let second_summary = kernel.recover_pending_orders().await.unwrap();
    assert_eq!(second_summary.scanned, 1);
    assert_eq!(second_summary.recovered, 1);

    let after_second = trade_store.load_state().await.unwrap().unwrap();
    assert_eq!(
        after_second
            .account
            .as_ref()
            .unwrap()
            .positions
            .get("000001")
            .unwrap()
            .volume,
        100
    );
    assert_eq!(after_second.trade_records.len(), 2);
}

#[tokio::test]
async fn mock_live_reconciliation_updates_runtime_order_without_mutating_account() {
    let dir = tempdir().unwrap();
    let trade_store = JsonPaperTradeStore::new(dir.path().join("paper_trade.json"));
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();

    let trade_service = TradeService::new(trade_store.clone());
    trade_service
        .init_account(
            InitAccountRequest::new(Some(100000.0), None, None, None, None).unwrap(),
            fixed_ts(),
        )
        .await
        .unwrap();

    let loader = FakeBarLoader {
        bars: buy_fixture(),
    };
    let envelope = StrategyRuntime::new(loader)
        .run_ma_cross_once("000001", 5, 10)
        .await
        .unwrap();

    let kernel = ExecutionKernel::new(
        runtime_store.clone(),
        MockLiveExecutionAdapter::with_state_template(
            runtime_store.clone(),
            FixedClock,
            MockLiveOrderState {
                fill_plan: vec![
                    MockLiveFillStep {
                        quantity: 40,
                        delay_secs: 0,
                    },
                    MockLiveFillStep {
                        quantity: 60,
                        delay_secs: 0,
                    },
                ],
                next_step_index: 1,
                simulated_fill_price: Some(dec!(12)),
                ..Default::default()
            },
        ),
        FixedRisk {
            decision: RiskDecision::Allow,
        },
    );

    let result = kernel
        .execute_once(
            sample_request(
                "run-mock-reconcile-1",
                "run-mock-reconcile-1_000001_1",
                fixed_ts(),
            ),
            envelope,
        )
        .await
        .unwrap();
    assert_eq!(result.order_status, Some(OrderStatus::Accepted));

    let order = runtime_store
        .find_order_by_client_order_id("run-mock-reconcile-1_000001_1")
        .await
        .unwrap()
        .unwrap();
    runtime_store
        .update_order(&order.order_id, OrderStatus::Unknown, 0, None, fixed_ts())
        .await
        .unwrap();

    let before = trade_store.load_state().await.unwrap().unwrap();
    assert!(before.account.as_ref().unwrap().positions.is_empty());
    assert_eq!(before.trade_records.len(), 0);

    let service = ReconciliationService::new(runtime_store.clone());
    let report = service.reconcile_all().await.unwrap();

    assert_eq!(report.summary.total_open_orders, 1);
    assert_eq!(report.summary.recovered_orders, 1);

    let saved = runtime_store
        .find_order_by_client_order_id("run-mock-reconcile-1_000001_1")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved.status, OrderStatus::PartiallyFilled);
    assert_eq!(saved.filled_quantity, 40);
    assert_eq!(saved.remaining_quantity, saved.requested_quantity - 40);
    assert_eq!(saved.avg_fill_price, Some(dec!(12)));

    let after = trade_store.load_state().await.unwrap().unwrap();
    assert!(after.account.as_ref().unwrap().positions.is_empty());
    assert_eq!(after.trade_records.len(), 0);
}
