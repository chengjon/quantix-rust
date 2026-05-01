use chrono::{TimeZone, Utc};
use quantix_cli::core::runtime::{BRIDGE_API_KEY_ENV, BRIDGE_BASE_URL_ENV};
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
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

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

struct RuntimeEnvGuard {
    bridge_base_url: Option<String>,
    bridge_api_key: Option<String>,
}

impl RuntimeEnvGuard {
    fn capture() -> Self {
        Self {
            bridge_base_url: std::env::var(BRIDGE_BASE_URL_ENV).ok(),
            bridge_api_key: std::env::var(BRIDGE_API_KEY_ENV).ok(),
        }
    }
}

impl Drop for RuntimeEnvGuard {
    fn drop(&mut self) {
        match &self.bridge_base_url {
            Some(value) => unsafe { std::env::set_var(BRIDGE_BASE_URL_ENV, value) },
            None => unsafe { std::env::remove_var(BRIDGE_BASE_URL_ENV) },
        }

        match &self.bridge_api_key {
            Some(value) => unsafe { std::env::set_var(BRIDGE_API_KEY_ENV, value) },
            None => unsafe { std::env::remove_var(BRIDGE_API_KEY_ENV) },
        }
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
    assert!(summary.request.is_none());
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
    assert_eq!(
        summary.request.as_ref().map(|row| row.request_id.as_str()),
        Some(request.request_id.as_str())
    );
    assert_eq!(
        summary.request.as_ref().map(|row| row.request_status),
        Some(ExecutionRequestStatus::Completed)
    );

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
    assert!(
        saved.payload_json["execution_result"]["executed_at"]
            .as_str()
            .is_some()
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
    assert_eq!(
        summary.request.as_ref().map(|row| row.request_id.as_str()),
        Some(request.request_id.as_str())
    );
    assert_eq!(
        summary.request.as_ref().map(|row| row.request_status),
        Some(ExecutionRequestStatus::Completed)
    );
    assert_eq!(saved.request_status, ExecutionRequestStatus::Completed);
    assert_eq!(
        saved.payload_json["execution_result"]["order_status"],
        "rejected"
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
    assert_eq!(
        summary.request.as_ref().map(|row| row.request_id.as_str()),
        Some(request.request_id.as_str())
    );
    assert_eq!(
        summary.request.as_ref().map(|row| row.request_status),
        Some(ExecutionRequestStatus::Failed)
    );

    let saved = runtime_store
        .get_execution_request(&request.request_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved.request_status, ExecutionRequestStatus::Failed);
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
    assert!(
        saved.payload_json["execution_error"]["failed_at"]
            .as_str()
            .is_some()
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
    assert_eq!(
        summary.request.as_ref().map(|row| row.request_id.as_str()),
        Some(request.request_id.as_str())
    );
    assert_eq!(
        summary.request.as_ref().map(|row| row.request_status),
        Some(ExecutionRequestStatus::Failed)
    );

    let saved = runtime_store
        .get_execution_request(&request.request_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved.request_status, ExecutionRequestStatus::Failed);
    assert!(
        saved.payload_json["execution_error"]["message"]
            .as_str()
            .unwrap()
            .contains("live 模式尚未实现")
    );
    assert!(
        saved.payload_json["execution_error"]["message"]
            .as_str()
            .unwrap()
            .contains("qmt_live")
    );
    assert!(
        saved.payload_json["execution_error"]["message"]
            .as_str()
            .unwrap()
            .contains("qmt.mode=live")
    );
    assert!(
        saved.payload_json["execution_error"]["message"]
            .as_str()
            .unwrap()
            .contains("execution bridge qmt-live")
    );
}

#[tokio::test]
async fn daemon_run_once_rejects_qmt_live_request_without_order_submit_support() {
    let _guard = RuntimeEnvGuard::capture();
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

    let server = MockServer::start().await;
    unsafe {
        std::env::set_var(BRIDGE_BASE_URL_ENV, server.uri());
        std::env::set_var(BRIDGE_API_KEY_ENV, "bridge-test-key");
    }

    Mock::given(method("GET"))
        .and(path("/api/v1/capabilities"))
        .and(header("x-quantix-api-key", "bridge-test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "tdx": {
                "enabled": true,
                "supports": ["quote", "batch_quotes", "kline"]
            },
            "qmt": {
                "enabled": true,
                "mode": "live",
                "supports": ["account_status", "order_preview"]
            }
        })))
        .mount(&server)
        .await;

    let run = sample_run(fixed_ts());
    runtime_store.insert_run(&run).await.unwrap();
    let signal = sample_signal(
        &run.run_id,
        "signal-daemon-qmt-submit-capability",
        fixed_ts(),
    );
    runtime_store.insert_signal(&signal).await.unwrap();
    let request = runtime_store
        .approve_signal_and_create_request(
            "signal-daemon-qmt-submit-capability",
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
    assert_eq!(
        summary.request.as_ref().map(|row| row.request_id.as_str()),
        Some(request.request_id.as_str())
    );
    assert_eq!(
        summary.request.as_ref().map(|row| row.request_status),
        Some(ExecutionRequestStatus::Failed)
    );

    let saved = runtime_store
        .get_execution_request(&request.request_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved.request_status, ExecutionRequestStatus::Failed);
    let message = saved.payload_json["execution_error"]["message"]
        .as_str()
        .unwrap();
    assert!(
        message.contains("order_submit"),
        "expected order_submit gate error, got: {message}"
    );
    assert!(
        saved.payload_json["execution_error"]["failed_at"]
            .as_str()
            .is_some()
    );
}
