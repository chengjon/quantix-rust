pub use chrono::{TimeZone, Utc};
pub use quantix_cli::execution::kernel::RecoverySummary;
pub use quantix_cli::execution::models::{
    ApprovalStatus, ExecutionRequestStatus, MockLiveOrderState, OrderEventRecord, OrderRecord,
    OrderSide, OrderStatus, OrderType, QmtLiveRuntimeMetadata, QmtLiveTaskIdentity,
    RunnerCheckpointRecord, SignalStatus, StrategyDaemonCheckpointRecord, StrategyRunRecord,
    StrategyRunStatus, StrategySignalRecord,
};
pub use quantix_cli::execution::runtime_store::StrategyRuntimeStore;
pub use rust_decimal_macros::dec;
pub use serde_json::json;
pub use tempfile::tempdir;
pub use uuid::Uuid;

pub(crate) fn fixed_ts() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 17, 9, 30, 0).unwrap()
}

pub(crate) fn sample_run(symbol: &str, bar_end: chrono::DateTime<Utc>) -> StrategyRunRecord {
    StrategyRunRecord {
        run_id: Uuid::new_v4().to_string(),
        strategy_name: "ma_cross".to_string(),
        mode: "paper".to_string(),
        trigger: "once".to_string(),
        status: StrategyRunStatus::Running,
        symbol: symbol.to_string(),
        timeframe: "1d".to_string(),
        bar_end,
        started_at: fixed_ts(),
        finished_at: None,
        metadata_json: json!({"short": 5, "long": 20}),
    }
}

pub(crate) fn sample_order(run_id: &str, client_order_id: &str) -> OrderRecord {
    OrderRecord {
        order_id: Uuid::new_v4().to_string(),
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
        adapter: "paper".to_string(),
        created_at: fixed_ts(),
        updated_at: fixed_ts(),
        last_transition_at: fixed_ts(),
        version: 0,
        payload_json: json!({"reason": "ma_cross_buy"}),
    }
}

pub(crate) fn sample_signal(
    run_id: &str,
    signal_id: &str,
    bar_end: chrono::DateTime<Utc>,
) -> StrategySignalRecord {
    StrategySignalRecord {
        signal_id: signal_id.to_string(),
        strategy_instance_id: "ma_fast_5_slow_20".to_string(),
        strategy_name: "ma_cross".to_string(),
        symbol: "000001".to_string(),
        timeframe: "1d".to_string(),
        bar_end,
        signal_value: "buy".to_string(),
        signal_status: SignalStatus::New,
        approval_status: ApprovalStatus::Pending,
        run_id: run_id.to_string(),
        metadata_json: json!({
            "fast": 5,
            "slow": 20,
            "market_price": "12.34",
            "signal_value": "buy",
            "execution_policy": {
                "fixed_cash_per_buy": "10000",
                "slippage_bps": 0
            },
            "bar_source_id": "test-primary",
            "bar_source_fallback": false
        }),
        created_at: fixed_ts(),
        updated_at: fixed_ts(),
    }
}

pub(crate) fn sample_daemon_checkpoint(last_run_id: &str) -> StrategyDaemonCheckpointRecord {
    StrategyDaemonCheckpointRecord {
        checkpoint_id: Uuid::new_v4().to_string(),
        strategy_instance_id: "ma_fast_5_slow_20".to_string(),
        strategy_name: "ma_cross".to_string(),
        symbol: "000001".to_string(),
        timeframe: "1d".to_string(),
        last_processed_bar: Some(fixed_ts()),
        last_run_id: Some(last_run_id.to_string()),
        state_json: json!({"bootstrap_policy": "latest_only"}),
        updated_at: fixed_ts(),
    }
}
