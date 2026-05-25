use chrono::{TimeZone, Utc};
use quantix_cli::execution::config::{AutoApprovalMode, JsonExecutionConfigStore};
use quantix_cli::execution::daemon::consume_next_pending_request_with_components;
use quantix_cli::execution::models::{
    ApprovalStatus, ExecutionRequestStatus, SignalStatus, StrategyRunRecord, StrategyRunStatus,
    StrategySignalRecord,
};
use quantix_cli::execution::runtime_store::StrategyRuntimeStore;
use quantix_cli::risk::{JsonRiskStore, RiskService, ShenwanCurrentSeedRow, SqliteIndustryStore};
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

async fn seed_current_industry(risk_state_path: &std::path::Path, code: &str, industry_name: &str) {
    let store = SqliteIndustryStore::from_risk_state_path(risk_state_path)
        .await
        .unwrap();
    store
        .upsert_shenwan_current_rows(
            &[ShenwanCurrentSeedRow {
                security_code: code.to_string(),
                industry_name: industry_name.to_string(),
                source: "test_seed".to_string(),
            }],
            fixed_ts(),
        )
        .await
        .unwrap();
}

fn invalid_runtime_risk_state_path(dir: &tempfile::TempDir) -> std::path::PathBuf {
    let risk_dir = dir.path().join("invalid-runtime-risk");
    std::fs::create_dir_all(&risk_dir).unwrap();
    std::fs::create_dir_all(risk_dir.join("industry_reference.db")).unwrap();
    risk_dir.join("risk_state.json")
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
    assert_eq!(
        saved.payload_json["execution_diagnostics"]["code"],
        "request_completed_order_terminal"
    );

    let state = trade_store.load_state().await.unwrap().unwrap();
    assert_eq!(state.trade_records.len(), 1);
}

#[tokio::test]
async fn daemon_run_once_rejects_pending_request_when_industry_is_blocked() {
    let dir = tempdir().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let trade_store = JsonPaperTradeStore::new(dir.path().join("paper_trade.json"));
    let risk_state_path = dir.path().join("risk_state.json");
    seed_current_industry(&risk_state_path, "000001.SZ", "银行").await;
    let risk_store = JsonRiskStore::new(risk_state_path.clone());
    let risk_service = RiskService::from_json_store(risk_store.clone())
        .await
        .unwrap();

    risk_service
        .set_rule("industry-blocklist", "银行,地产", fixed_ts())
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

    let run = sample_run(fixed_ts());
    runtime_store.insert_run(&run).await.unwrap();
    let signal = sample_signal(&run.run_id, "signal-daemon-industry-1", fixed_ts());
    runtime_store.insert_signal(&signal).await.unwrap();
    let request = runtime_store
        .approve_signal_and_create_request(
            "signal-daemon-industry-1",
            "paper",
            "default",
            Some("cli"),
        )
        .await
        .unwrap();

    let summary = consume_next_pending_request_with_components(
        &runtime_store,
        trade_store.clone(),
        risk_store,
    )
    .await
    .unwrap();

    let saved = runtime_store
        .get_execution_request(&request.request_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(summary.claimed, 1);
    assert_eq!(summary.completed, 1);
    assert_eq!(summary.failed, 0);
    assert_eq!(saved.request_status, ExecutionRequestStatus::Completed);
    assert_eq!(
        saved.payload_json["execution_result"]["order_status"],
        "rejected"
    );
    assert_eq!(
        saved.payload_json["execution_diagnostics"]["code"],
        "request_completed_order_terminal"
    );

    let client_order_id = saved.payload_json["execution_result"]["client_order_id"]
        .as_str()
        .unwrap();
    let order = runtime_store
        .find_order_by_client_order_id(client_order_id)
        .await
        .unwrap()
        .unwrap();
    let events = runtime_store
        .list_order_events(&order.order_id)
        .await
        .unwrap();
    assert_eq!(events[0].event_type, "risk_rejected");
    assert!(
        events[0].details_json["reason"]
            .as_str()
            .unwrap()
            .contains("industry-blocklist")
    );

    let state = trade_store.load_state().await.unwrap().unwrap();
    assert!(state.trade_records.is_empty());
}

#[tokio::test]
async fn daemon_run_once_marks_request_failed_when_industry_resolver_misses() {
    let dir = tempdir().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let trade_store = JsonPaperTradeStore::new(dir.path().join("paper_trade.json"));
    let risk_state_path = dir.path().join("risk_state.json");
    let risk_store = JsonRiskStore::new(risk_state_path.clone());
    let risk_service = RiskService::from_json_store(risk_store.clone())
        .await
        .unwrap();

    risk_service
        .set_rule("industry-blocklist", "银行,地产", fixed_ts())
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

    let run = sample_run(fixed_ts());
    runtime_store.insert_run(&run).await.unwrap();
    let signal = sample_signal(&run.run_id, "signal-daemon-miss-1", fixed_ts());
    runtime_store.insert_signal(&signal).await.unwrap();
    let request = runtime_store
        .approve_signal_and_create_request("signal-daemon-miss-1", "paper", "default", Some("cli"))
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
    assert_eq!(summary.completed, 0);
    assert_eq!(summary.failed, 1);

    let saved = runtime_store
        .get_execution_request(&request.request_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved.request_status, ExecutionRequestStatus::Failed);
    assert_eq!(
        saved.payload_json["execution_diagnostics"]["code"],
        "execution_error_unclassified"
    );
    assert!(
        saved.payload_json["execution_error"]["message"]
            .as_str()
            .unwrap()
            .contains("industry-blocklist")
    );
    assert!(
        saved.payload_json["execution_error"]["message"]
            .as_str()
            .unwrap()
            .contains("检查失败")
    );

    let order = runtime_store
        .find_first_order_for_run(&run.run_id)
        .await
        .unwrap();
    assert!(order.is_none());

    let state = trade_store.load_state().await.unwrap().unwrap();
    assert!(state.trade_records.is_empty());
}

#[tokio::test]
async fn daemon_run_once_returns_unsupported_for_live_request_before_sqlite_setup() {
    let dir = tempdir().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let trade_store = JsonPaperTradeStore::new(dir.path().join("paper_trade.json"));
    let risk_store = JsonRiskStore::new(invalid_runtime_risk_state_path(&dir));

    let run = sample_run(fixed_ts());
    runtime_store.insert_run(&run).await.unwrap();
    let signal = sample_signal(&run.run_id, "signal-daemon-live-1", fixed_ts());
    runtime_store.insert_signal(&signal).await.unwrap();
    let request = runtime_store
        .approve_signal_and_create_request("signal-daemon-live-1", "live", "default", Some("cli"))
        .await
        .unwrap();

    let summary =
        consume_next_pending_request_with_components(&runtime_store, trade_store, risk_store)
            .await
            .unwrap();
    assert_eq!(summary.claimed, 1);
    assert_eq!(summary.completed, 0);
    assert_eq!(summary.failed, 1);

    let saved = runtime_store
        .get_execution_request(&request.request_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved.request_status, ExecutionRequestStatus::Failed);
    assert_eq!(
        saved.payload_json["execution_diagnostics"]["code"],
        "daemon_live_mode_unsupported"
    );
    assert!(
        saved.payload_json["execution_error"]["message"]
            .as_str()
            .unwrap()
            .contains("live 模式尚未实现")
    );
}

#[tokio::test]
async fn daemon_run_once_rejects_qmt_live_request_with_manual_bridge_guidance() {
    let dir = tempdir().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let trade_store = JsonPaperTradeStore::new(dir.path().join("paper_trade.json"));
    let risk_store = JsonRiskStore::new(invalid_runtime_risk_state_path(&dir));

    let run = sample_run(fixed_ts());
    runtime_store.insert_run(&run).await.unwrap();
    let signal = sample_signal(&run.run_id, "signal-daemon-qmt-live-1", fixed_ts());
    runtime_store.insert_signal(&signal).await.unwrap();
    let request = runtime_store
        .approve_signal_and_create_request(
            "signal-daemon-qmt-live-1",
            "qmt_live",
            "default",
            Some("cli"),
        )
        .await
        .unwrap();

    let summary =
        consume_next_pending_request_with_components(&runtime_store, trade_store, risk_store)
            .await
            .unwrap();
    assert_eq!(summary.claimed, 1);
    assert_eq!(summary.completed, 0);
    assert_eq!(summary.failed, 1);

    let saved = runtime_store
        .get_execution_request(&request.request_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved.request_status, ExecutionRequestStatus::Failed);
    assert_eq!(
        saved.payload_json["execution_diagnostics"]["code"],
        "daemon_qmt_live_manual_bridge_required"
    );
    let message = saved.payload_json["execution_error"]["message"]
        .as_str()
        .unwrap();
    assert!(message.contains("qmt_live"));
    assert!(message.contains("execution bridge qmt-live"));
}

#[tokio::test]
async fn daemon_run_once_writes_non_terminal_completion_diagnostics_for_mock_live() {
    let dir = tempdir().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let trade_store = JsonPaperTradeStore::new(dir.path().join("paper_trade.json"));
    let risk_store = JsonRiskStore::new(dir.path().join("risk_state.json"));

    TradeService::new(trade_store.clone())
        .init_account(
            InitAccountRequest::new(Some(100000.0), None, None, None, None).unwrap(),
            fixed_ts(),
        )
        .await
        .unwrap();

    let run = sample_run(fixed_ts());
    runtime_store.insert_run(&run).await.unwrap();
    let signal = sample_signal(&run.run_id, "signal-daemon-mock-live-1", fixed_ts());
    runtime_store.insert_signal(&signal).await.unwrap();
    let request = runtime_store
        .approve_signal_and_create_request(
            "signal-daemon-mock-live-1",
            "mock_live",
            "default",
            Some("cli"),
        )
        .await
        .unwrap();

    let summary =
        consume_next_pending_request_with_components(&runtime_store, trade_store, risk_store)
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
        "accepted"
    );
    assert_eq!(
        saved.payload_json["execution_diagnostics"]["code"],
        "request_completed_order_non_terminal"
    );
}
