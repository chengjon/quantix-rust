use async_trait::async_trait;
use chrono::{DateTime, Duration, NaiveDate, TimeZone, Utc};
use quantix_cli::core::Result;
use quantix_cli::core::signal::Signal;
use quantix_cli::data::models::{AdjustType, Kline};
use quantix_cli::execution::kernel::{
    ExecutionKernel, ExecutionRunRequest, RiskDecision, RiskEvaluator,
};
use quantix_cli::execution::models::{ExecutionPolicy, OrderIntent, OrderStatus};
use quantix_cli::execution::paper::PaperExecutionAdapter;
use quantix_cli::execution::runtime_store::StrategyRuntimeStore;
use quantix_cli::strategy::runtime::{StrategyBarLoader, StrategyRuntime};
use quantix_cli::trade::{InitAccountRequest, JsonPaperTradeStore, PaperTradeStore, TradeService};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
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

fn sample_request(
    run_id: &str,
    client_order_id: &str,
    bar_end: DateTime<Utc>,
) -> ExecutionRunRequest {
    ExecutionRunRequest {
        run_id: run_id.to_string(),
        strategy_name: "ma_cross".to_string(),
        mode: "paper".to_string(),
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
async fn successful_paper_run_writes_runtime_rows_and_updates_account() {
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
        PaperExecutionAdapter::new(trade_service),
        FixedRisk {
            decision: RiskDecision::Allow,
        },
    );

    let result = kernel
        .execute_once(
            sample_request("run-1", "run-1_000001_1", fixed_ts()),
            envelope,
        )
        .await
        .unwrap();

    assert_eq!(result.order_status, Some(OrderStatus::Filled));
    assert_eq!(runtime_store.count_runs().await.unwrap(), 1);
    assert_eq!(runtime_store.count_signal_events().await.unwrap(), 1);
    assert_eq!(runtime_store.count_orders().await.unwrap(), 1);

    let state = trade_store.load_state().await.unwrap().unwrap();
    assert!(state.account.unwrap().positions.contains_key("000001"));
}

#[tokio::test]
async fn second_run_on_same_bar_is_deduplicated_by_run_key() {
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
        PaperExecutionAdapter::new(trade_service),
        FixedRisk {
            decision: RiskDecision::Allow,
        },
    );

    let first = kernel
        .execute_once(
            sample_request("run-1", "run-1_000001_1", fixed_ts()),
            envelope.clone(),
        )
        .await
        .unwrap();
    let second = kernel
        .execute_once(
            sample_request("run-2", "run-2_000001_1", fixed_ts()),
            envelope,
        )
        .await
        .unwrap();

    assert_eq!(first.order_status, Some(OrderStatus::Filled));
    assert_eq!(second.order_status, Some(OrderStatus::Filled));
    assert_eq!(runtime_store.count_runs().await.unwrap(), 1);
    assert_eq!(runtime_store.count_orders().await.unwrap(), 1);
}

#[tokio::test]
async fn risk_rejection_keeps_account_unchanged_and_records_rejected_order() {
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
        PaperExecutionAdapter::new(trade_service),
        FixedRisk {
            decision: RiskDecision::Reject {
                reason: "position-limit".to_string(),
            },
        },
    );

    let result = kernel
        .execute_once(
            sample_request("run-3", "run-3_000001_1", fixed_ts()),
            envelope,
        )
        .await
        .unwrap();

    assert_eq!(result.order_status, Some(OrderStatus::Rejected));
    assert_eq!(runtime_store.count_orders().await.unwrap(), 1);

    let state = trade_store.load_state().await.unwrap().unwrap();
    assert!(state.account.unwrap().positions.is_empty());
}
