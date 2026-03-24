use chrono::{TimeZone, Utc};
use quantix_cli::execution::config::{AutoApprovalMode, JsonExecutionConfigStore};
use quantix_cli::execution::daemon::consume_next_pending_request_with_components;
use quantix_cli::execution::models::{
    ApprovalStatus, ExecutionRequestStatus, SignalStatus, StrategyRunRecord, StrategyRunStatus,
    StrategySignalRecord,
};
use quantix_cli::execution::runtime_store::StrategyRuntimeStore;
use quantix_cli::risk::JsonRiskStore;
use quantix_cli::trade::{InitAccountRequest, JsonPaperTradeStore, PaperTradeStore, TradeService};
use serde_json::json;
use tempfile::tempdir;

#[test]
fn config_load_or_create_persists_default_execution_daemon_settings() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("execution").join("config.json");

    let store = JsonExecutionConfigStore::new(&path);
    let config = store.load_or_create().unwrap();

    assert_eq!(config.poll_interval_secs, 10);
    assert_eq!(config.max_requests_per_iteration, 1);
    assert_eq!(config.auto_approval.mode, AutoApprovalMode::Manual);

    let saved = std::fs::read_to_string(&path).unwrap();
    assert!(saved.contains("\"poll_interval_secs\": 10"));
    assert!(saved.contains("\"max_requests_per_iteration\": 1"));
    assert!(saved.contains("\"mode\": \"manual\""));
}

fn fixed_ts() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 23, 9, 30, 0).unwrap()
}

fn sample_run(bar_end: chrono::DateTime<Utc>) -> StrategyRunRecord {
    StrategyRunRecord {
        run_id: "execution-daemon-run".to_string(),
        strategy_name: "ma_cross".to_string(),
        mode: "signal".to_string(),
        trigger: "daemon".to_string(),
        status: StrategyRunStatus::Running,
        symbol: "000001".to_string(),
        timeframe: "1d".to_string(),
        bar_end,
        started_at: fixed_ts(),
        finished_at: None,
        metadata_json: json!({}),
    }
}

fn sample_signal(
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

#[tokio::test]
async fn daemon_run_once_returns_empty_summary_when_no_pending_request_exists() {
    let dir = tempdir().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let trade_store = JsonPaperTradeStore::new(dir.path().join("paper_trade.json"));
    let risk_store = JsonRiskStore::new(dir.path().join("risk_state.json"));

    let summary =
        consume_next_pending_request_with_components(&runtime_store, trade_store, risk_store)
            .await
            .unwrap();

    assert_eq!(summary.claimed, 0);
    assert_eq!(summary.completed, 0);
    assert_eq!(summary.failed, 0);
}

#[tokio::test]
async fn daemon_run_once_consumes_one_pending_paper_request() {
    let dir = tempdir().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let trade_store = JsonPaperTradeStore::new(dir.path().join("paper_trade.json"));
    let risk_store = JsonRiskStore::new(dir.path().join("risk_state.json"));

    let trade_service = TradeService::new(trade_store.clone());
    trade_service
        .init_account(
            InitAccountRequest::new(Some(100000.0), None, None, None, None).unwrap(),
            fixed_ts(),
        )
        .await
        .unwrap();

    let run = sample_run(fixed_ts());
    runtime_store.insert_run(&run).await.unwrap();
    let signal = sample_signal(&run.run_id, "signal-daemon-1", fixed_ts());
    runtime_store.insert_signal(&signal).await.unwrap();
    let request = runtime_store
        .approve_signal_and_create_request("signal-daemon-1", "paper", "default", Some("cli"))
        .await
        .unwrap();

    let summary = consume_next_pending_request_with_components(
        &runtime_store,
        trade_store.clone(),
        risk_store,
    )
    .await
    .unwrap();

    assert_eq!(summary.claimed, 1);
    assert_eq!(summary.completed, 1);
    assert_eq!(summary.failed, 0);

    let saved = runtime_store
        .get_execution_request(&request.request_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved.request_status, ExecutionRequestStatus::Completed);
    assert_eq!(
        saved.payload_json["execution_result"]["order_status"],
        "filled"
    );

    let state = trade_store.load_state().await.unwrap().unwrap();
    assert_eq!(state.trade_records.len(), 1);
}
