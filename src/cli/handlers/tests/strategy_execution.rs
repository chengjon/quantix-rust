use super::*;

pub(super) fn make_kline(code: &str, day: u32, close: Decimal, volume: i64) -> Kline {
    Kline {
        code: code.to_string(),
        date: NaiveDate::from_ymd_opt(2024, 1, day).unwrap(),
        open: close,
        high: close + dec!(1),
        low: close - dec!(1),
        close,
        volume,
        amount: None,
        adjust_type: AdjustType::None,
    }
}

pub(super) fn fixed_ts() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 17, 9, 30, 0).unwrap()
}

fn sample_run(symbol: &str, bar_end: DateTime<Utc>) -> crate::execution::models::StrategyRunRecord {
    crate::execution::models::StrategyRunRecord {
        run_id: uuid::Uuid::new_v4().to_string(),
        strategy_name: "ma_cross".to_string(),
        mode: "signal".to_string(),
        trigger: "daemon".to_string(),
        status: crate::execution::models::StrategyRunStatus::Running,
        symbol: symbol.to_string(),
        timeframe: "1d".to_string(),
        bar_end,
        started_at: fixed_ts(),
        finished_at: None,
        metadata_json: serde_json::json!({}),
    }
}

fn sample_signal(
    run_id: &str,
    signal_id: &str,
    bar_end: DateTime<Utc>,
) -> crate::execution::models::StrategySignalRecord {
    crate::execution::models::StrategySignalRecord {
        signal_id: signal_id.to_string(),
        strategy_instance_id: "ma_fast_5_slow_20".to_string(),
        strategy_name: "ma_cross".to_string(),
        symbol: "000001".to_string(),
        timeframe: "1d".to_string(),
        bar_end,
        signal_value: "buy".to_string(),
        signal_status: crate::execution::models::SignalStatus::New,
        approval_status: crate::execution::models::ApprovalStatus::Pending,
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

#[derive(Debug, Clone, Default)]
pub(super) struct FakeLoader {
    pub(super) data: HashMap<String, Vec<Kline>>,
}

impl StrategyBarLoadTelemetry for FakeLoader {
    fn last_source(&self) -> Option<crate::strategy::StrategyBarLoadSource> {
        Some(crate::strategy::StrategyBarLoadSource {
            source_id: "test-primary".to_string(),
            fallback_used: false,
        })
    }
}

#[async_trait]
impl DailyKlineLoader for FakeLoader {
    async fn load_daily_klines(
        &self,
        code: &str,
        lookback: usize,
    ) -> crate::core::Result<Vec<Kline>> {
        let mut rows = self.data.get(code).cloned().unwrap_or_default();
        if rows.len() > lookback {
            rows = rows[rows.len() - lookback..].to_vec();
        }
        Ok(rows)
    }
}

#[async_trait]
impl StrategyBarLoader for FakeLoader {
    async fn load_daily_bars(&self, code: &str, limit: usize) -> crate::core::Result<Vec<Kline>> {
        let mut rows = self.data.get(code).cloned().unwrap_or_default();
        if rows.len() > limit {
            rows = rows[rows.len() - limit..].to_vec();
        }
        Ok(rows)
    }
}

#[async_trait]
impl crate::risk::RiskBarLoader for FakeLoader {
    async fn load_daily_bars(&self, code: &str, limit: usize) -> crate::core::Result<Vec<Kline>> {
        let mut rows = self.data.get(code).cloned().unwrap_or_default();
        if rows.len() > limit {
            rows = rows[rows.len() - limit..].to_vec();
        }
        Ok(rows)
    }
}

#[tokio::test]
async fn test_strategy_paper_requires_explicit_code() {
    let dir = tempdir().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();

    let err = execute_strategy_run_with_components(
        "ma_cross",
        "paper",
        None,
        FakeLoader::default(),
        JsonPaperTradeStore::new(dir.path().join("paper_trade.json")),
        JsonRiskStore::new(dir.path().join("risk_state.json")),
        &runtime_store,
    )
    .await
    .unwrap_err();

    assert!(err.to_string().contains("--code"));
    assert!(err.to_string().contains("--mode paper"));
}

#[tokio::test]
async fn test_strategy_mock_live_requires_explicit_code() {
    let dir = tempdir().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();

    let err = execute_strategy_run_with_components(
        "ma_cross",
        "mock_live",
        None,
        FakeLoader::default(),
        JsonPaperTradeStore::new(dir.path().join("paper_trade.json")),
        JsonRiskStore::new(dir.path().join("risk_state.json")),
        &runtime_store,
    )
    .await
    .unwrap_err();

    assert!(err.to_string().contains("--code"));
    assert!(err.to_string().contains("--mode mock_live"));
}

#[tokio::test]
async fn test_strategy_mock_live_rejects_when_kill_switch_enabled() {
    let _lock = env_lock();
    let dir = tempdir().unwrap();
    let home_dir = tempdir().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let trade_store = JsonPaperTradeStore::new(dir.path().join("paper_trade.json"));
    let trade_service = TradeService::new(trade_store.clone());
    trade_service
        .init_account(
            crate::trade::InitAccountRequest::new(Some(100000.0), None, None, None, None).unwrap(),
            fixed_ts(),
        )
        .await
        .unwrap();
    let loader = FakeLoader {
        data: HashMap::from([(
            "000001".to_string(),
            [
                dec!(10),
                dec!(9),
                dec!(8),
                dec!(7),
                dec!(6),
                dec!(5),
                dec!(4),
                dec!(3),
                dec!(2),
                dec!(1),
                dec!(1),
                dec!(1),
                dec!(1),
                dec!(1),
                dec!(12),
            ]
            .into_iter()
            .enumerate()
            .map(|(idx, close)| make_kline("000001", (idx + 1) as u32, close, 1000))
            .collect(),
        )]),
    };
    let risk_service = crate::risk::RiskService::with_bar_loader(
        JsonRiskStore::new(dir.path().join("risk_state.json")),
        loader.clone(),
    );

    struct HomeGuard {
        original: Option<String>,
    }

    impl Drop for HomeGuard {
        fn drop(&mut self) {
            match &self.original {
                Some(value) => unsafe { std::env::set_var("HOME", value) },
                None => unsafe { std::env::remove_var("HOME") },
            }
        }
    }

    let _home_guard = {
        let original = std::env::var("HOME").ok();
        unsafe { std::env::set_var("HOME", home_dir.path()) };
        HomeGuard { original }
    };

    let kill_switch_store = crate::safety::JsonKillSwitchStore::with_default_path().unwrap();
    execute_safety_kill_switch_command_with_store_at(
        SafetyKillSwitchCommands::Enable {
            reason: "broker instability".to_string(),
        },
        &kill_switch_store,
        fixed_ts(),
    )
    .unwrap();

    let err = execute_strategy_run_with_risk_service(
        "ma_cross",
        "mock_live",
        Some("000001".to_string()),
        loader,
        trade_store,
        risk_service,
        &runtime_store,
    )
    .await
    .unwrap_err();

    let message = err.to_string();
    assert!(message.contains("kill switch"));
    assert!(message.contains("mock_live"));
    assert!(message.contains("broker instability"));
}

#[tokio::test]
async fn test_strategy_paper_allows_execution_when_kill_switch_enabled() {
    let dir = tempdir().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let trade_store = JsonPaperTradeStore::new(dir.path().join("paper_trade.json"));
    let trade_service = TradeService::new(trade_store.clone());
    trade_service
        .init_account(
            crate::trade::InitAccountRequest::new(Some(100000.0), None, None, None, None).unwrap(),
            fixed_ts(),
        )
        .await
        .unwrap();
    let loader = FakeLoader {
        data: HashMap::from([(
            "000001".to_string(),
            [
                dec!(10),
                dec!(9),
                dec!(8),
                dec!(7),
                dec!(6),
                dec!(5),
                dec!(4),
                dec!(3),
                dec!(2),
                dec!(1),
                dec!(1),
                dec!(1),
                dec!(1),
                dec!(1),
                dec!(12),
            ]
            .into_iter()
            .enumerate()
            .map(|(idx, close)| make_kline("000001", (idx + 1) as u32, close, 1000))
            .collect(),
        )]),
    };
    let risk_service = crate::risk::RiskService::with_bar_loader(
        JsonRiskStore::new(dir.path().join("risk_state.json")),
        loader.clone(),
    );
    let kill_switch_store =
        crate::safety::JsonKillSwitchStore::new(dir.path().join("kill_switch.json"));

    execute_safety_kill_switch_command_with_store_at(
        SafetyKillSwitchCommands::Enable {
            reason: "broker instability".to_string(),
        },
        &kill_switch_store,
        fixed_ts(),
    )
    .unwrap();

    let summary = execute_strategy_run_with_risk_service_and_kill_switch(
        "ma_cross",
        "paper",
        Some("000001".to_string()),
        loader,
        trade_store,
        risk_service,
        &runtime_store,
        &kill_switch_store,
    )
    .await
    .unwrap();

    assert_eq!(summary.mode, "paper");
    assert!(matches!(
        summary.order_status,
        Some(OrderStatus::Accepted | OrderStatus::Filled)
    ));

    let order = runtime_store
        .find_first_order_for_run(&summary.run_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(order.adapter, "paper");
}

#[tokio::test]
async fn test_strategy_paper_requires_initialized_account() {
    let dir = tempdir().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let loader = FakeLoader {
        data: HashMap::from([(
            "000001".to_string(),
            (1..=30)
                .map(|day| make_kline("000001", day, dec!(10) + Decimal::from(day), 1000))
                .collect(),
        )]),
    };

    let err = execute_strategy_run_with_components(
        "ma_cross",
        "paper",
        Some("000001".to_string()),
        loader,
        JsonPaperTradeStore::new(dir.path().join("paper_trade.json")),
        JsonRiskStore::new(dir.path().join("risk_state.json")),
        &runtime_store,
    )
    .await
    .unwrap_err();

    assert!(err.to_string().contains("trade init"));
}

#[tokio::test]
async fn test_strategy_live_remains_unsupported() {
    let dir = tempdir().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();

    let err = execute_strategy_run_with_components(
        "ma_cross",
        "live",
        Some("000001".to_string()),
        FakeLoader::default(),
        JsonPaperTradeStore::new(dir.path().join("paper_trade.json")),
        JsonRiskStore::new(dir.path().join("risk_state.json")),
        &runtime_store,
    )
    .await
    .unwrap_err();

    assert!(matches!(err, QuantixError::Unsupported(_)));
    let message = err.to_string();
    assert!(message.contains("live 模式尚未实现"));
    assert!(message.contains("qmt_live"));
    assert!(message.contains("execution bridge qmt-live"));
    assert!(message.contains("qmt.mode=live"));
}

#[tokio::test]
async fn test_strategy_mock_live_returns_non_final_status() {
    let dir = tempdir().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let loader = FakeLoader {
        data: HashMap::from([(
            "000001".to_string(),
            [
                dec!(10),
                dec!(9),
                dec!(8),
                dec!(7),
                dec!(6),
                dec!(5),
                dec!(4),
                dec!(3),
                dec!(2),
                dec!(1),
                dec!(1),
                dec!(1),
                dec!(1),
                dec!(1),
                dec!(12),
            ]
            .into_iter()
            .enumerate()
            .map(|(idx, close)| make_kline("000001", (idx + 1) as u32, close, 1000))
            .collect(),
        )]),
    };
    let trade_store = JsonPaperTradeStore::new(dir.path().join("paper_trade.json"));
    let trade_service = TradeService::new(trade_store.clone());
    trade_service
        .init_account(
            crate::trade::InitAccountRequest::new(Some(100000.0), None, None, None, None).unwrap(),
            fixed_ts(),
        )
        .await
        .unwrap();

    let summary = execute_strategy_run_with_components(
        "ma_cross",
        "mock_live",
        Some("000001".to_string()),
        loader,
        trade_store,
        JsonRiskStore::new(dir.path().join("risk_state.json")),
        &runtime_store,
    )
    .await
    .unwrap();

    assert_eq!(summary.mode, "mock_live");
    assert_eq!(summary.order_status, Some(OrderStatus::Accepted));
    assert!(summary.message.contains("order_status=accepted"));
}

#[tokio::test]
async fn test_execute_execution_bridge_qmt_live_rejects_preview_only_bridge_mode() {
    let _lock = env_lock();
    let _guard = RuntimeEnvGuard::capture();
    let dir = tempdir().unwrap();
    let runtime_db_path = dir.path().join("runtime.db");

    unsafe {
        std::env::set_var(STRATEGY_RUNTIME_DB_PATH_ENV, &runtime_db_path);
        std::env::set_var(BRIDGE_API_KEY_ENV, "bridge-test-key");
    }

    let server = MockServer::start().await;
    unsafe {
        std::env::set_var(BRIDGE_BASE_URL_ENV, server.uri());
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
                "mode": "preview_only",
                "supports": ["account_status", "order_preview"]
            }
        })))
        .mount(&server)
        .await;

    let runtime_store = StrategyRuntimeStore::new(&runtime_db_path).await.unwrap();
    let run = sample_run("000001", fixed_ts());
    runtime_store.insert_run(&run).await.unwrap();

    let signal = sample_signal(&run.run_id, "signal-qmt-live-gate", fixed_ts());
    runtime_store.insert_signal(&signal).await.unwrap();

    let request = runtime_store
        .approve_signal_and_create_request(
            "signal-qmt-live-gate",
            "qmt_live",
            "default",
            Some("cli"),
        )
        .await
        .unwrap();

    let err = execute_execution_bridge_qmt_live(&request.request_id, true)
        .await
        .unwrap_err();

    assert!(
        err.to_string().contains("preview_only"),
        "expected preview_only safety gate error, got: {err}"
    );

    let saved = runtime_store
        .get_execution_request(&request.request_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        saved.request_status,
        crate::execution::models::ExecutionRequestStatus::Failed
    );
    assert_eq!(
        saved.payload_json["execution_diagnostics"]["code"],
        "bridge_qmt_mode_not_live"
    );
    assert!(
        saved.payload_json["execution_error"]["message"]
            .as_str()
            .unwrap()
            .contains("preview_only")
    );
}

#[tokio::test]
async fn test_execute_execution_bridge_qmt_live_rejects_when_qmt_capability_is_disabled() {
    let _lock = env_lock();
    let _guard = RuntimeEnvGuard::capture();
    let dir = tempdir().unwrap();
    let runtime_db_path = dir.path().join("runtime.db");

    unsafe {
        std::env::set_var(STRATEGY_RUNTIME_DB_PATH_ENV, &runtime_db_path);
        std::env::set_var(BRIDGE_API_KEY_ENV, "bridge-test-key");
    }

    let server = MockServer::start().await;
    unsafe {
        std::env::set_var(BRIDGE_BASE_URL_ENV, server.uri());
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
                "enabled": false,
                "mode": "preview_only",
                "supports": ["account_status", "order_preview"]
            }
        })))
        .mount(&server)
        .await;

    let runtime_store = StrategyRuntimeStore::new(&runtime_db_path).await.unwrap();
    let run = sample_run("000001", fixed_ts());
    runtime_store.insert_run(&run).await.unwrap();

    let signal = sample_signal(
        &run.run_id,
        "signal-qmt-live-capability-disabled",
        fixed_ts(),
    );
    runtime_store.insert_signal(&signal).await.unwrap();

    let request = runtime_store
        .approve_signal_and_create_request(
            "signal-qmt-live-capability-disabled",
            "qmt_live",
            "default",
            Some("cli"),
        )
        .await
        .unwrap();

    let err = execute_execution_bridge_qmt_live(&request.request_id, true)
        .await
        .unwrap_err();

    assert!(
        err.to_string().contains("capability 未启用"),
        "expected disabled capability error, got: {err}"
    );

    let saved = runtime_store
        .get_execution_request(&request.request_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        saved.request_status,
        crate::execution::models::ExecutionRequestStatus::Failed
    );
    assert_eq!(
        saved.payload_json["execution_diagnostics"]["code"],
        "bridge_qmt_capability_disabled"
    );
}

#[tokio::test]
async fn test_execute_execution_bridge_qmt_live_rejects_when_capability_check_fails() {
    let _lock = env_lock();
    let _guard = RuntimeEnvGuard::capture();
    let dir = tempdir().unwrap();
    let runtime_db_path = dir.path().join("runtime.db");

    unsafe {
        std::env::set_var(STRATEGY_RUNTIME_DB_PATH_ENV, &runtime_db_path);
        std::env::set_var(BRIDGE_API_KEY_ENV, "bridge-test-key");
    }

    let server = MockServer::start().await;
    unsafe {
        std::env::set_var(BRIDGE_BASE_URL_ENV, server.uri());
    }

    Mock::given(method("GET"))
        .and(path("/api/v1/capabilities"))
        .and(header("x-quantix-api-key", "bridge-test-key"))
        .respond_with(ResponseTemplate::new(503).set_body_string("bridge unavailable"))
        .mount(&server)
        .await;

    let runtime_store = StrategyRuntimeStore::new(&runtime_db_path).await.unwrap();
    let run = sample_run("000001", fixed_ts());
    runtime_store.insert_run(&run).await.unwrap();

    let signal = sample_signal(
        &run.run_id,
        "signal-qmt-live-capability-check-failed",
        fixed_ts(),
    );
    runtime_store.insert_signal(&signal).await.unwrap();

    let request = runtime_store
        .approve_signal_and_create_request(
            "signal-qmt-live-capability-check-failed",
            "qmt_live",
            "default",
            Some("cli"),
        )
        .await
        .unwrap();

    let err = execute_execution_bridge_qmt_live(&request.request_id, true)
        .await
        .unwrap_err();

    assert!(
        err.to_string().contains("能力检查失败"),
        "expected capability check failure error, got: {err}"
    );

    let saved = runtime_store
        .get_execution_request(&request.request_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        saved.request_status,
        crate::execution::models::ExecutionRequestStatus::Failed
    );
    assert_eq!(
        saved.payload_json["execution_diagnostics"]["code"],
        "bridge_qmt_capability_check_failed"
    );
    assert_eq!(
        saved.payload_json["execution_diagnostics"]["diagnostic_source"],
        "qmt_live_gate"
    );
    assert_eq!(
        saved.payload_json["execution_diagnostics"]["qmt_live_failure_category"],
        "capability_check_failed"
    );
    assert_eq!(
        saved.payload_json["execution_diagnostics"]["compatibility_requirement"],
        "bridge /api/v1/capabilities returns qmt capability metadata"
    );
}

#[tokio::test]
async fn test_execute_execution_bridge_qmt_live_persists_task_identity_into_related_order() {
    let _lock = env_lock();
    let _guard = RuntimeEnvGuard::capture();
    let dir = tempdir().unwrap();
    let runtime_db_path = dir.path().join("runtime.db");

    unsafe {
        std::env::set_var(STRATEGY_RUNTIME_DB_PATH_ENV, &runtime_db_path);
        std::env::set_var(BRIDGE_API_KEY_ENV, "bridge-test-key");
    }

    let server = MockServer::start().await;
    unsafe {
        std::env::set_var(BRIDGE_BASE_URL_ENV, server.uri());
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
                "supports": ["account_status", "order_preview", "order_submit"]
            }
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/api/v1/task/execute"))
        .and(header("x-bridge-contract-version", "miniqmt.v1"))
        .and(header("x-quantix-api-key", "bridge-test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "task_id": "task-1",
            "status": "bridge_task_accepted",
            "receipt_timestamp": "2026-05-01T09:30:00Z",
            "bridge_contract_version": "miniqmt.v1",
            "source_name": "miniqmt"
        })))
        .mount(&server)
        .await;

    let runtime_store = StrategyRuntimeStore::new(&runtime_db_path).await.unwrap();
    let run = sample_run("000001", fixed_ts());
    runtime_store.insert_run(&run).await.unwrap();

    let signal = sample_signal(&run.run_id, "signal-qmt-live-success", fixed_ts());
    runtime_store.insert_signal(&signal).await.unwrap();

    let request = runtime_store
        .approve_signal_and_create_request(
            "signal-qmt-live-success",
            "qmt_live",
            "default",
            Some("cli"),
        )
        .await
        .unwrap();

    execute_execution_bridge_qmt_live(&request.request_id, true)
        .await
        .unwrap();

    let saved = runtime_store
        .get_execution_request(&request.request_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        saved.request_status,
        crate::execution::models::ExecutionRequestStatus::Completed
    );
    assert_eq!(
        saved.payload_json["execution_result"]["order_status"],
        "pending_submit"
    );
    assert_eq!(
        saved.payload_json["execution_diagnostics"]["code"],
        "request_completed_order_non_terminal"
    );
    assert_eq!(
        saved.payload_json["execution_result"]["adapter_order_id"],
        "task-1"
    );
    let saved_order = runtime_store
        .find_order_by_client_order_id(&request.request_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved_order.adapter, "qmt_live");
    assert_eq!(
        saved_order.status,
        crate::execution::models::OrderStatus::PendingSubmit
    );
    assert_eq!(
        saved_order.payload_json["qmt_live"]["task_identity"]["task_id"],
        "task-1"
    );
    assert_eq!(
        saved_order.payload_json["qmt_live"]["task_identity"]["client_order_id"],
        request.request_id
    );
    assert!(
        saved_order.payload_json["qmt_live"]["task_identity"]["local_submission_id"]
            .as_str()
            .is_some_and(|value| !value.is_empty())
    );
}

#[tokio::test]
async fn test_build_execution_bridge_qmt_query_output_prefers_task_result_lookup() {
    let server = MockServer::start().await;
    let client = BridgeHttpClient::new(server.uri(), Some("bridge-test-key".to_string())).unwrap();

    Mock::given(method("GET"))
        .and(path("/api/v1/task/result/task-1"))
        .and(header("x-bridge-contract-version", "miniqmt.v1"))
        .and(header("x-quantix-api-key", "bridge-test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "task_id": "task-1",
            "status": "completed",
            "bridge_contract_version": "miniqmt.v1",
            "result": {
                "client_order_id": "req-1",
                "local_submission_id": "local-1",
                "account_scope": "default",
                "event_id": "evt-1",
                "occurred_at": "2026-05-03T09:32:00Z",
                "source_name": "miniqmt",
                "broker_event_type": "acknowledgement",
                "external_order_id": "broker-1",
                "reason_code": null,
                "reason_detail": null,
                "evidence_ref": null
            }
        })))
        .mount(&server)
        .await;

    let output = build_execution_bridge_qmt_query_output(&client, "task-1")
        .await
        .unwrap();

    assert_eq!(output["query_mode"], "task_result");
    assert_eq!(output["adapter_order_id"], "task-1");
    assert_eq!(output["latest_status"], "accepted");
    assert_eq!(output["external_order_id"], "broker-1");
    assert_eq!(output["client_order_id"], "req-1");
    assert_eq!(output["local_submission_id"], "local-1");
    assert_eq!(output["source_name"], "miniqmt");
    assert_eq!(output["broker_event_type"], "Acknowledgement");
}

#[tokio::test]
async fn test_build_execution_bridge_qmt_query_output_falls_back_to_legacy_order_lookup() {
    let server = MockServer::start().await;
    let client = BridgeHttpClient::new(server.uri(), Some("bridge-test-key".to_string())).unwrap();

    Mock::given(method("GET"))
        .and(path("/api/v1/task/result/broker-legacy-1"))
        .and(header("x-bridge-contract-version", "miniqmt.v1"))
        .and(header("x-quantix-api-key", "bridge-test-key"))
        .respond_with(
            ResponseTemplate::new(404)
                .set_body_json(serde_json::json!({ "reason_detail": "task not found" })),
        )
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v1/broker/qmt/orders/broker-legacy-1"))
        .and(header("x-quantix-api-key", "bridge-test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "adapter_order_id": "broker-legacy-1",
            "latest_status": "accepted",
            "filled_quantity": 100,
            "avg_fill_price": "10.50",
            "fill_details": null
        })))
        .mount(&server)
        .await;

    let output = build_execution_bridge_qmt_query_output(&client, "broker-legacy-1")
        .await
        .unwrap();

    assert_eq!(output["query_mode"], "legacy_order");
    assert_eq!(output["adapter_order_id"], "broker-legacy-1");
    assert_eq!(output["latest_status"], "accepted");
    assert_eq!(output["filled_quantity"], 100);
    assert_eq!(output["avg_fill_price"], "10.50");
}

#[tokio::test]
async fn test_execute_execution_bridge_qmt_cancel_with_client_resolves_task_id_to_external_order_id()
 {
    let server = MockServer::start().await;
    let client = BridgeHttpClient::new(server.uri(), Some("bridge-test-key".to_string())).unwrap();

    Mock::given(method("GET"))
        .and(path("/api/v1/task/result/task-cancel-1"))
        .and(header("x-bridge-contract-version", "miniqmt.v1"))
        .and(header("x-quantix-api-key", "bridge-test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "task_id": "task-cancel-1",
            "status": "completed",
            "bridge_contract_version": "miniqmt.v1",
            "result": {
                "client_order_id": "req-cancel-1",
                "local_submission_id": "local-cancel-1",
                "account_scope": "default",
                "event_id": "evt-cancel-1",
                "occurred_at": "2026-05-03T09:40:00Z",
                "source_name": "miniqmt",
                "broker_event_type": "acknowledgement",
                "external_order_id": "broker-cancel-1",
                "reason_code": null,
                "reason_detail": null,
                "evidence_ref": null
            }
        })))
        .mount(&server)
        .await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/broker/qmt/orders/broker-cancel-1"))
        .and(header("x-quantix-api-key", "bridge-test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "order_id": "broker-cancel-1",
            "error_message": null
        })))
        .mount(&server)
        .await;

    let result = execute_execution_bridge_qmt_cancel_with_client(&client, "task-cancel-1")
        .await
        .unwrap();

    assert_eq!(result.requested_order_id, "task-cancel-1");
    assert_eq!(result.cancel_order_id, "broker-cancel-1");
    assert!(result.resolved_from_task_result);
    assert!(result.response.success);
    assert_eq!(result.response.order_id, "broker-cancel-1");
}

#[tokio::test]
async fn test_execute_execution_bridge_qmt_cancel_with_client_falls_back_to_direct_order_id() {
    let server = MockServer::start().await;
    let client = BridgeHttpClient::new(server.uri(), Some("bridge-test-key".to_string())).unwrap();

    Mock::given(method("GET"))
        .and(path("/api/v1/task/result/broker-cancel-legacy-1"))
        .and(header("x-bridge-contract-version", "miniqmt.v1"))
        .and(header("x-quantix-api-key", "bridge-test-key"))
        .respond_with(
            ResponseTemplate::new(404)
                .set_body_json(serde_json::json!({ "reason_detail": "task not found" })),
        )
        .mount(&server)
        .await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/broker/qmt/orders/broker-cancel-legacy-1"))
        .and(header("x-quantix-api-key", "bridge-test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "order_id": "broker-cancel-legacy-1",
            "error_message": null
        })))
        .mount(&server)
        .await;

    let result = execute_execution_bridge_qmt_cancel_with_client(&client, "broker-cancel-legacy-1")
        .await
        .unwrap();

    assert_eq!(result.requested_order_id, "broker-cancel-legacy-1");
    assert_eq!(result.cancel_order_id, "broker-cancel-legacy-1");
    assert!(!result.resolved_from_task_result);
    assert!(result.response.success);
    assert_eq!(result.response.order_id, "broker-cancel-legacy-1");
}

#[tokio::test]
async fn test_execute_execution_bridge_qmt_live_rejects_live_mode_without_order_submit_support() {
    let _lock = env_lock();
    let _guard = RuntimeEnvGuard::capture();
    let dir = tempdir().unwrap();
    let runtime_db_path = dir.path().join("runtime.db");

    unsafe {
        std::env::set_var(STRATEGY_RUNTIME_DB_PATH_ENV, &runtime_db_path);
        std::env::set_var(BRIDGE_API_KEY_ENV, "bridge-test-key");
    }

    let server = MockServer::start().await;
    unsafe {
        std::env::set_var(BRIDGE_BASE_URL_ENV, server.uri());
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

    let runtime_store = StrategyRuntimeStore::new(&runtime_db_path).await.unwrap();
    let run = sample_run("000001", fixed_ts());
    runtime_store.insert_run(&run).await.unwrap();

    let signal = sample_signal(&run.run_id, "signal-qmt-live-submit-capability", fixed_ts());
    runtime_store.insert_signal(&signal).await.unwrap();

    let request = runtime_store
        .approve_signal_and_create_request(
            "signal-qmt-live-submit-capability",
            "qmt_live",
            "default",
            Some("cli"),
        )
        .await
        .unwrap();

    let err = execute_execution_bridge_qmt_live(&request.request_id, true)
        .await
        .unwrap_err();

    assert!(
        err.to_string().contains("order_submit"),
        "expected order_submit safety gate error, got: {err}"
    );

    let saved = runtime_store
        .get_execution_request(&request.request_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        saved.request_status,
        crate::execution::models::ExecutionRequestStatus::Failed
    );
    assert_eq!(
        saved.payload_json["execution_diagnostics"]["code"],
        "bridge_qmt_order_submit_capability_missing"
    );
    assert_eq!(
        saved.payload_json["execution_diagnostics"]["diagnostic_source"],
        "qmt_live_gate"
    );
    assert_eq!(
        saved.payload_json["execution_diagnostics"]["qmt_live_failure_category"],
        "missing_required_capability"
    );
    assert_eq!(
        saved.payload_json["execution_diagnostics"]["compatibility_requirement"],
        "bridge qmt.supports includes order_submit"
    );
    assert_eq!(saved.payload_json["execution_error"]["adapter"], "qmt_live");
    assert!(
        saved.payload_json["execution_error"]["message"]
            .as_str()
            .unwrap()
            .contains("order_submit")
    );
    assert!(
        saved.payload_json["execution_error"]["failed_at"]
            .as_str()
            .is_some()
    );
}

#[tokio::test]
async fn test_execute_execution_bridge_qmt_live_rejects_when_kill_switch_enabled() {
    let dir = tempdir().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let kill_switch_store =
        crate::safety::JsonKillSwitchStore::new(dir.path().join("kill_switch.json"));

    execute_safety_kill_switch_command_with_store_at(
        SafetyKillSwitchCommands::Enable {
            reason: "broker instability".to_string(),
        },
        &kill_switch_store,
        fixed_ts(),
    )
    .unwrap();

    let run = sample_run("000001", fixed_ts());
    runtime_store.insert_run(&run).await.unwrap();

    let signal = sample_signal(&run.run_id, "signal-qmt-live-kill-switch", fixed_ts());
    runtime_store.insert_signal(&signal).await.unwrap();

    let request = runtime_store
        .approve_signal_and_create_request(
            "signal-qmt-live-kill-switch",
            "qmt_live",
            "default",
            Some("cli"),
        )
        .await
        .unwrap();

    let err = execute_execution_bridge_qmt_live_with_runtime_store_and_kill_switch(
        &runtime_store,
        &kill_switch_store,
        &request.request_id,
        true,
    )
    .await
    .unwrap_err();

    let message = err.to_string();
    assert!(message.contains("kill switch"));
    assert!(message.contains("qmt_live"));
    assert!(message.contains("broker instability"));

    let saved = runtime_store
        .get_execution_request(&request.request_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        saved.request_status,
        crate::execution::models::ExecutionRequestStatus::Failed
    );
    assert_eq!(
        saved.payload_json["execution_diagnostics"]["code"],
        "kill_switch_blocked"
    );
    assert_eq!(saved.payload_json["execution_error"]["adapter"], "qmt_live");
    assert_eq!(
        saved.payload_json["kill_switch"]["reason"],
        "broker instability"
    );
    assert_eq!(saved.payload_json["kill_switch"]["target_mode"], "qmt_live");
    assert_eq!(
        saved.payload_json["kill_switch"]["enabled_at"],
        fixed_ts().to_rfc3339()
    );
    assert!(
        saved.payload_json["kill_switch"]["blocked_at"]
            .as_str()
            .is_some()
    );
}

#[tokio::test]
async fn test_strategy_paper_risk_bridge_surfaces_volatility_limit_reason() {
    let dir = tempdir().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let loader = FakeLoader {
        data: HashMap::from([(
            "000001".to_string(),
            [
                dec!(10),
                dec!(9),
                dec!(8),
                dec!(7),
                dec!(6),
                dec!(5),
                dec!(4),
                dec!(3),
                dec!(2),
                dec!(1),
                dec!(1),
                dec!(1),
                dec!(1),
                dec!(1),
                dec!(12),
            ]
            .into_iter()
            .enumerate()
            .map(|(idx, close)| make_kline("000001", (idx + 1) as u32, close, 1000))
            .collect(),
        )]),
    };
    let trade_store = JsonPaperTradeStore::new(dir.path().join("paper_trade.json"));
    let trade_service = TradeService::new(trade_store.clone());
    trade_service
        .init_account(
            crate::trade::InitAccountRequest::new(Some(100000.0), None, None, None, None).unwrap(),
            fixed_ts(),
        )
        .await
        .unwrap();
    let risk_service = RiskService::with_bar_loader(
        JsonRiskStore::new(dir.path().join("risk_state.json")),
        loader.clone(),
    );
    risk_service
        .set_rule("volatility-limit", "4%", fixed_ts())
        .await
        .unwrap();

    let summary = execute_strategy_run_with_risk_service(
        "ma_cross",
        "paper",
        Some("000001".to_string()),
        loader,
        trade_store,
        risk_service,
        &runtime_store,
    )
    .await
    .unwrap();

    assert_eq!(summary.order_status, Some(OrderStatus::Rejected));

    let order = runtime_store
        .find_first_order_for_run(&summary.run_id)
        .await
        .unwrap()
        .unwrap();
    let events = runtime_store
        .list_order_events(&order.order_id)
        .await
        .unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, "risk_rejected");
    assert!(
        events[0].details_json["reason"]
            .as_str()
            .unwrap()
            .contains("volatility-limit")
    );
}

#[tokio::test]
async fn test_strategy_mock_live_risk_bridge_surfaces_volatility_limit_reason() {
    let dir = tempdir().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let loader = FakeLoader {
        data: HashMap::from([(
            "000001".to_string(),
            [
                dec!(10),
                dec!(9),
                dec!(8),
                dec!(7),
                dec!(6),
                dec!(5),
                dec!(4),
                dec!(3),
                dec!(2),
                dec!(1),
                dec!(1),
                dec!(1),
                dec!(1),
                dec!(1),
                dec!(12),
            ]
            .into_iter()
            .enumerate()
            .map(|(idx, close)| make_kline("000001", (idx + 1) as u32, close, 1000))
            .collect(),
        )]),
    };
    let trade_store = JsonPaperTradeStore::new(dir.path().join("paper_trade.json"));
    let trade_service = TradeService::new(trade_store.clone());
    trade_service
        .init_account(
            crate::trade::InitAccountRequest::new(Some(100000.0), None, None, None, None).unwrap(),
            fixed_ts(),
        )
        .await
        .unwrap();
    let risk_service = RiskService::with_bar_loader(
        JsonRiskStore::new(dir.path().join("risk_state.json")),
        loader.clone(),
    );
    risk_service
        .set_rule("volatility-limit", "4%", fixed_ts())
        .await
        .unwrap();

    let summary = execute_strategy_run_with_risk_service(
        "ma_cross",
        "mock_live",
        Some("000001".to_string()),
        loader,
        trade_store.clone(),
        risk_service,
        &runtime_store,
    )
    .await
    .unwrap();

    assert_eq!(summary.order_status, Some(OrderStatus::Rejected));

    let order = runtime_store
        .find_first_order_for_run(&summary.run_id)
        .await
        .unwrap()
        .unwrap();
    let events = runtime_store
        .list_order_events(&order.order_id)
        .await
        .unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, "risk_rejected");
    assert!(
        events[0].details_json["reason"]
            .as_str()
            .unwrap()
            .contains("volatility-limit")
    );

    let state = trade_store.load_state().await.unwrap().unwrap();
    assert!(state.account.unwrap().positions.is_empty());
}

#[test]
fn test_execute_strategy_config_init_creates_default_file() {
    let dir = tempdir().unwrap();
    let store =
        crate::strategy::JsonStrategyConfigStore::new(dir.path().join("strategy-config.json"));

    let config = execute_strategy_config_init_to_store(&store).unwrap();

    assert_eq!(config.check_interval_secs, 60);
    assert!(dir.path().join("strategy-config.json").exists());
}

#[test]
fn test_execute_strategy_config_show_returns_saved_config() {
    let dir = tempdir().unwrap();
    let store =
        crate::strategy::JsonStrategyConfigStore::new(dir.path().join("strategy-config.json"));
    let expected = store.load_or_create().unwrap();

    let shown = execute_strategy_config_show_from_store(&store).unwrap();

    assert_eq!(shown, expected);
}

#[test]
fn test_execute_strategy_service_config_show_reports_not_configured_when_missing() {
    let dir = tempdir().unwrap();
    let store = crate::strategy::JsonStrategyServiceConfigStore::new(
        dir.path().join("strategy-service.json"),
    );

    let shown = execute_strategy_service_config_command_with_store(
        StrategyServiceConfigCommands::Show,
        &store,
    )
    .unwrap();

    assert!(shown.is_none());
}

#[test]
fn test_execute_strategy_service_config_set_persists_values() {
    let dir = tempdir().unwrap();
    let binary_path = dir.path().join("quantix");
    std::fs::write(&binary_path, "#!/bin/sh\nexit 0\n").unwrap();
    let mut perms = std::fs::metadata(&binary_path).unwrap().permissions();
    use std::os::unix::fs::PermissionsExt;
    perms.set_mode(0o755);
    std::fs::set_permissions(&binary_path, perms).unwrap();

    let store = crate::strategy::JsonStrategyServiceConfigStore::new(
        dir.path().join("strategy-service.json"),
    );

    let shown = execute_strategy_service_config_command_with_store(
        StrategyServiceConfigCommands::Set {
            quantix_bin: binary_path.display().to_string(),
            env_file: Some("/tmp/strategy.env".to_string()),
        },
        &store,
    )
    .unwrap()
    .unwrap();

    assert_eq!(shown.quantix_bin_path, binary_path);
    assert_eq!(
        shown.environment_file_path,
        Some(std::path::PathBuf::from("/tmp/strategy.env"))
    );

    let saved = store.load().unwrap();
    assert_eq!(saved.quantix_bin_path, binary_path);
    assert_eq!(
        saved.environment_file_path,
        Some(std::path::PathBuf::from("/tmp/strategy.env"))
    );
}

#[derive(Default)]
struct FakeStrategyServiceInstaller {
    status_output: Option<String>,
}

impl StrategyServiceInstallerOps for FakeStrategyServiceInstaller {
    fn install(&self) -> Result<()> {
        Ok(())
    }

    fn uninstall(&self) -> Result<()> {
        Ok(())
    }

    fn start(&self) -> Result<()> {
        Ok(())
    }

    fn stop(&self) -> Result<()> {
        Ok(())
    }

    fn enable(&self) -> Result<()> {
        Ok(())
    }

    fn disable(&self) -> Result<()> {
        Ok(())
    }

    fn status(&self) -> Result<String> {
        Ok(self
            .status_output
            .clone()
            .unwrap_or_else(|| "installed: yes".to_string()))
    }

    fn status_summary(&self) -> Result<StrategyServiceStatusSummary> {
        Ok(StrategyServiceStatusSummary {
            installed: true,
            enabled: false,
            active: "inactive".to_string(),
            unit_path: std::path::PathBuf::from("~/.config/systemd/user/quantix-strategy.service"),
            wrapper_path: std::path::PathBuf::from("~/.local/bin/quantix-strategy-run"),
            quantix_bin_path: std::path::PathBuf::from("/bin/echo"),
            environment_file_path: None,
            raw_status: None,
        })
    }
}

#[test]
fn test_execute_strategy_service_install_returns_message() {
    let message = execute_strategy_service_command_with_installer(
        StrategyServiceCommands::Install,
        &FakeStrategyServiceInstaller::default(),
    )
    .unwrap();

    assert_eq!(message, "strategy service installed");
}

#[test]
fn test_execute_strategy_service_status_returns_status_text() {
    let message = execute_strategy_service_command_with_installer(
        StrategyServiceCommands::Status,
        &FakeStrategyServiceInstaller {
            status_output: Some("installed: yes\nenabled: no".to_string()),
        },
    )
    .unwrap();

    assert!(message.contains("installed: yes"));
    assert!(message.contains("enabled: no"));
}

#[tokio::test]
async fn test_execute_strategy_daemon_once_bootstraps_and_then_emits_signal() {
    let dir = tempdir().unwrap();
    let config_store =
        crate::strategy::JsonStrategyConfigStore::new(dir.path().join("strategy-config.json"));
    config_store.load_or_create().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let mut loader = FakeLoader::default();
    loader.data.insert(
        "000001".to_string(),
        vec![
            make_kline("000001", 1, dec!(10), 1000),
            make_kline("000001", 2, dec!(10), 1000),
            make_kline("000001", 3, dec!(10), 1000),
            make_kline("000001", 4, dec!(9), 1000),
            make_kline("000001", 5, dec!(9), 1000),
            make_kline("000001", 6, dec!(20), 1000),
        ],
    );

    let first = execute_strategy_daemon_run_once_with_components(
        loader.clone(),
        &config_store,
        &runtime_store,
    )
    .await
    .unwrap();
    assert!(first.is_none());
    assert_eq!(runtime_store.count_signals().await.unwrap(), 0);

    loader
        .data
        .get_mut("000001")
        .unwrap()
        .push(make_kline("000001", 7, dec!(21), 1000));

    let second =
        execute_strategy_daemon_run_once_with_components(loader, &config_store, &runtime_store)
            .await
            .unwrap();
    assert_eq!(
        second.map(|signal| signal.metadata_json["bar_source_id"].clone()),
        Some(json!("test-primary"))
    );
    assert_eq!(runtime_store.count_signals().await.unwrap(), 1);
}
