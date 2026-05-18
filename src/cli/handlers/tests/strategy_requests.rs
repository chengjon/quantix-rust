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
async fn test_execute_strategy_signal_list_approve_reject_and_request_list() {
    let dir = tempdir().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let run = sample_run("000001", fixed_ts());
    runtime_store.insert_run(&run).await.unwrap();

    let signal = crate::execution::models::StrategySignalRecord {
        signal_id: "signal-1".to_string(),
        strategy_instance_id: "ma_fast_5_slow_20".to_string(),
        strategy_name: "ma_cross".to_string(),
        symbol: "000001".to_string(),
        timeframe: "1d".to_string(),
        bar_end: fixed_ts(),
        signal_value: "buy".to_string(),
        signal_status: crate::execution::models::SignalStatus::New,
        approval_status: crate::execution::models::ApprovalStatus::Pending,
        run_id: run.run_id.clone(),
        metadata_json: json!({}),
        created_at: fixed_ts(),
        updated_at: fixed_ts(),
    };
    runtime_store.insert_signal(&signal).await.unwrap();

    let pending = execute_strategy_signal_list_with_store(&runtime_store, Some("pending"), None)
        .await
        .unwrap();
    assert_eq!(pending.len(), 1);

    let request =
        execute_strategy_signal_approve_with_store(&runtime_store, "signal-1", "paper", "default")
            .await
            .unwrap();
    assert_eq!(request.signal_id, "signal-1");

    let requests = execute_strategy_request_list_with_store(&runtime_store, Some("pending"))
        .await
        .unwrap();
    assert_eq!(requests.len(), 1);

    let second = crate::execution::models::StrategySignalRecord {
        signal_id: "signal-2".to_string(),
        bar_end: fixed_ts() + chrono::Duration::days(1),
        ..signal
    };
    runtime_store.insert_signal(&second).await.unwrap();
    execute_strategy_signal_reject_with_store(&runtime_store, "signal-2", Some("manual"))
        .await
        .unwrap();

    let rejected = runtime_store.get_signal("signal-2").await.unwrap().unwrap();
    assert_eq!(
        rejected.approval_status,
        crate::execution::models::ApprovalStatus::Rejected
    );
}

#[tokio::test]
async fn test_execute_strategy_request_execute_rejects_live_target_mode_with_qmt_guidance() {
    let dir = tempdir().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let trade_store = JsonPaperTradeStore::new(dir.path().join("paper_trade.json"));
    let risk_store = JsonRiskStore::new(dir.path().join("risk_state.json"));

    let run = sample_run("000001", fixed_ts());
    runtime_store.insert_run(&run).await.unwrap();

    let signal = crate::execution::models::StrategySignalRecord {
        signal_id: "signal-request-exec-live".to_string(),
        strategy_instance_id: "ma_fast_5_slow_20".to_string(),
        strategy_name: "ma_cross".to_string(),
        symbol: "000001".to_string(),
        timeframe: "1d".to_string(),
        bar_end: fixed_ts(),
        signal_value: "buy".to_string(),
        signal_status: crate::execution::models::SignalStatus::New,
        approval_status: crate::execution::models::ApprovalStatus::Pending,
        run_id: run.run_id.clone(),
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
    };
    runtime_store.insert_signal(&signal).await.unwrap();

    let request = runtime_store
        .approve_signal_and_create_request(
            "signal-request-exec-live",
            "live",
            "default",
            Some("cli"),
        )
        .await
        .unwrap();

    let err = execute_strategy_request_execute_with_components(
        &runtime_store,
        &request.request_id,
        trade_store,
        risk_store,
    )
    .await
    .unwrap_err();

    assert!(matches!(err, QuantixError::Unsupported(_)));
    let message = err.to_string();
    assert!(message.contains("live 模式尚未实现"));
    assert!(message.contains("qmt_live"));
    assert!(message.contains("qmt.mode=live"));
    assert!(message.contains("execution bridge qmt-live"));

    let saved = runtime_store
        .get_execution_request(&request.request_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        saved.request_status,
        crate::execution::models::ExecutionRequestStatus::Failed
    );
    assert!(
        saved.payload_json["execution_error"]["message"]
            .as_str()
            .unwrap()
            .contains("execution bridge qmt-live")
    );
}

#[tokio::test]
async fn test_execute_strategy_request_execute_and_cancel() {
    let dir = tempdir().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let trade_store = JsonPaperTradeStore::new(dir.path().join("paper_trade.json"));
    let risk_store = JsonRiskStore::new(dir.path().join("risk_state.json"));

    let trade_service = TradeService::new(trade_store.clone());
    trade_service
        .init_account(
            crate::trade::InitAccountRequest::new(Some(100000.0), None, None, None, None).unwrap(),
            fixed_ts(),
        )
        .await
        .unwrap();

    let run = sample_run("000001", fixed_ts());
    runtime_store.insert_run(&run).await.unwrap();

    let signal = crate::execution::models::StrategySignalRecord {
        signal_id: "signal-request-exec".to_string(),
        strategy_instance_id: "ma_fast_5_slow_20".to_string(),
        strategy_name: "ma_cross".to_string(),
        symbol: "000001".to_string(),
        timeframe: "1d".to_string(),
        bar_end: fixed_ts(),
        signal_value: "buy".to_string(),
        signal_status: crate::execution::models::SignalStatus::New,
        approval_status: crate::execution::models::ApprovalStatus::Pending,
        run_id: run.run_id.clone(),
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
    };
    runtime_store.insert_signal(&signal).await.unwrap();

    let request = execute_strategy_signal_approve_with_store(
        &runtime_store,
        "signal-request-exec",
        "mock_live",
        "default",
    )
    .await
    .unwrap();

    let completed = execute_strategy_request_execute_with_components(
        &runtime_store,
        &request.request_id,
        trade_store.clone(),
        risk_store.clone(),
    )
    .await
    .unwrap();
    assert_eq!(
        completed.request_status,
        crate::execution::models::ExecutionRequestStatus::Completed
    );
    assert_eq!(
        completed.payload_json["execution_result"]["order_status"],
        "accepted"
    );

    let second_signal = crate::execution::models::StrategySignalRecord {
        signal_id: "signal-request-cancel".to_string(),
        bar_end: fixed_ts() + chrono::Duration::days(1),
        ..signal
    };
    runtime_store.insert_signal(&second_signal).await.unwrap();

    let cancel_request = execute_strategy_signal_approve_with_store(
        &runtime_store,
        "signal-request-cancel",
        "paper",
        "default",
    )
    .await
    .unwrap();

    let canceled = execute_strategy_request_cancel_with_store(
        &runtime_store,
        &cancel_request.request_id,
        Some("manual cancel"),
    )
    .await
    .unwrap();
    assert_eq!(
        canceled.request_status,
        crate::execution::models::ExecutionRequestStatus::Canceled
    );
    assert_eq!(
        canceled.payload_json["cancellation"]["reason"],
        "manual cancel"
    );
}

#[test]
fn test_execute_execution_bridge_qmt_live_source_keeps_confirmation_and_request_guidance() {
    let source = std::fs::read_to_string(
        repo_root()
            .join("src")
            .join("cli")
            .join("handlers")
            .join("execution_handler.rs"),
    )
    .expect("expected src/cli/handlers/execution_handler.rs");

    assert!(
        source.contains("输入 'YES' 确认下单"),
        "expected manual qmt_live handler to keep explicit YES confirmation"
    );
    assert!(
        source.contains("查看 request 与后续收敛状态: quantix strategy request show"),
        "expected manual qmt_live handler to guide operators to request/reconciliation visibility"
    );
    assert!(
        !source.contains("查询订单状态: quantix execution bridge qmt-query --order-id"),
        "expected manual qmt_live handler to drop legacy qmt-query post-submit guidance"
    );
}

#[tokio::test]
async fn test_execute_strategy_signal_approve_rejects_live_target_mode_early() {
    let dir = tempdir().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();

    let run = sample_run("000001", fixed_ts());
    runtime_store.insert_run(&run).await.unwrap();

    let signal = crate::execution::models::StrategySignalRecord {
        signal_id: "signal-live-target".to_string(),
        strategy_instance_id: "ma_fast_5_slow_20".to_string(),
        strategy_name: "ma_cross".to_string(),
        symbol: "000001".to_string(),
        timeframe: "1d".to_string(),
        bar_end: fixed_ts(),
        signal_value: "buy".to_string(),
        signal_status: crate::execution::models::SignalStatus::New,
        approval_status: crate::execution::models::ApprovalStatus::Pending,
        run_id: run.run_id.clone(),
        metadata_json: json!({}),
        created_at: fixed_ts(),
        updated_at: fixed_ts(),
    };
    runtime_store.insert_signal(&signal).await.unwrap();

    let err = execute_strategy_signal_approve_with_store(
        &runtime_store,
        "signal-live-target",
        "live",
        "default",
    )
    .await
    .unwrap_err();

    assert!(matches!(err, QuantixError::Unsupported(_)));
    let message = err.to_string();
    assert!(message.contains("target_mode=live"));
    assert!(message.contains("target_mode=qmt_live"));
    assert!(message.contains("execution bridge qmt-live"));
    assert!(message.contains("qmt.mode=live"));

    let saved_signal = runtime_store
        .get_signal("signal-live-target")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        saved_signal.approval_status,
        crate::execution::models::ApprovalStatus::Pending
    );
    let requests = execute_strategy_request_list_with_store(&runtime_store, None)
        .await
        .unwrap();
    assert!(requests.is_empty());
}

#[tokio::test]
async fn test_execute_strategy_signal_approve_allows_qmt_live_target_mode() {
    let dir = tempdir().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();

    let run = sample_run("000001", fixed_ts());
    runtime_store.insert_run(&run).await.unwrap();

    let signal = crate::execution::models::StrategySignalRecord {
        signal_id: "signal-qmt-live-target".to_string(),
        strategy_instance_id: "ma_fast_5_slow_20".to_string(),
        strategy_name: "ma_cross".to_string(),
        symbol: "000001".to_string(),
        timeframe: "1d".to_string(),
        bar_end: fixed_ts(),
        signal_value: "buy".to_string(),
        signal_status: crate::execution::models::SignalStatus::New,
        approval_status: crate::execution::models::ApprovalStatus::Pending,
        run_id: run.run_id.clone(),
        metadata_json: json!({}),
        created_at: fixed_ts(),
        updated_at: fixed_ts(),
    };
    runtime_store.insert_signal(&signal).await.unwrap();

    let request = execute_strategy_signal_approve_with_store(
        &runtime_store,
        "signal-qmt-live-target",
        "qmt_live",
        "default",
    )
    .await
    .unwrap();

    assert_eq!(request.target_mode, "qmt_live");
    assert_eq!(
        request.request_status,
        crate::execution::models::ExecutionRequestStatus::Pending
    );
}

#[test]
fn test_format_strategy_approval_result_includes_target_and_status() {
    let row = crate::execution::models::ExecutionRequestRecord {
        request_id: "req-1".to_string(),
        signal_id: "signal-1".to_string(),
        target_mode: "paper".to_string(),
        target_account: "default".to_string(),
        request_status: crate::execution::models::ExecutionRequestStatus::Pending,
        approved_by: Some("cli".to_string()),
        created_at: fixed_ts(),
        updated_at: fixed_ts(),
        payload_json: json!({}),
    };

    let line = format_strategy_approval_result(&row);

    assert!(line.contains("req-1"));
    assert!(line.contains("signal=signal-1"));
    assert!(line.contains("target=paper/default"));
    assert!(line.contains("status=pending"));
}

#[test]
fn test_format_strategy_rejection_result_includes_reason() {
    let row = crate::execution::models::StrategySignalRecord {
        signal_id: "signal-2".to_string(),
        strategy_instance_id: "ma_fast_5_slow_20".to_string(),
        strategy_name: "ma_cross".to_string(),
        symbol: "000001".to_string(),
        timeframe: "1d".to_string(),
        bar_end: fixed_ts(),
        signal_value: "sell".to_string(),
        signal_status: crate::execution::models::SignalStatus::New,
        approval_status: crate::execution::models::ApprovalStatus::Rejected,
        run_id: "run-2".to_string(),
        metadata_json: json!({"rejection_reason": "manual reject"}),
        created_at: fixed_ts(),
        updated_at: fixed_ts(),
    };

    let line = format_strategy_rejection_result(&row);

    assert!(line.contains("signal-2"));
    assert!(line.contains("signal_status=new"));
    assert!(line.contains("approval_status=rejected"));
    assert!(line.contains("reason=manual reject"));
}

#[test]
fn test_format_strategy_request_row_includes_target_and_status() {
    let row = crate::execution::models::ExecutionRequestRecord {
        request_id: "req-2".to_string(),
        signal_id: "signal-9".to_string(),
        target_mode: "paper".to_string(),
        target_account: "swing".to_string(),
        request_status: crate::execution::models::ExecutionRequestStatus::Completed,
        approved_by: Some("cli".to_string()),
        created_at: fixed_ts(),
        updated_at: fixed_ts(),
        payload_json: json!({
            "execution_result": {
                "order_status": "accepted",
                "client_order_id": "req-2_000001_1"
            }
        }),
    };

    let line = format_strategy_request_row(&row);

    assert!(line.contains("req-2"));
    assert!(line.contains("signal=signal-9"));
    assert!(line.contains("target=paper/swing"));
    assert!(line.contains("status=completed"));
    assert!(line.contains("semantics=request_completed_order_non_terminal"));
    assert!(line.contains("result=order_status=accepted client_order_id=req-2_000001_1"));
    assert!(line.contains("created_at=2026-03-17T09:30:00Z"));
}

#[test]
fn test_format_strategy_request_detail_keeps_request_status_separate_from_order_status() {
    let row = crate::execution::models::ExecutionRequestRecord {
        request_id: "req-detail-1".to_string(),
        signal_id: "signal-detail-1".to_string(),
        target_mode: "mock_live".to_string(),
        target_account: "default".to_string(),
        request_status: crate::execution::models::ExecutionRequestStatus::Completed,
        approved_by: Some("daemon".to_string()),
        created_at: fixed_ts(),
        updated_at: fixed_ts(),
        payload_json: json!({
            "execution_result": {
                "run_id": "run-detail-1",
                "client_order_id": "req-detail-1_000001_1",
                "order_status": "accepted",
                "executed_at": "2026-03-17T09:31:00Z"
            }
        }),
    };

    let detail = format_strategy_request_detail(&row, false);

    assert!(detail.contains("target_mode: mock_live"));
    assert!(detail.contains("status: completed"));
    assert!(detail.contains("request_status: completed"));
    assert!(detail.contains("order_status: accepted"));
    assert!(detail.contains(
        "status_note: request completed only means execution layer finished; order remains accepted"
    ));
    assert!(detail.contains("client_order_id: req-detail-1_000001_1"));
}

#[test]
fn test_format_strategy_request_detail_displays_execution_diagnostics_section() {
    let row = crate::execution::models::ExecutionRequestRecord {
        request_id: "req-detail-diag".to_string(),
        signal_id: "signal-detail-diag".to_string(),
        target_mode: "qmt_live".to_string(),
        target_account: "default".to_string(),
        request_status: crate::execution::models::ExecutionRequestStatus::Failed,
        approved_by: Some("cli".to_string()),
        created_at: fixed_ts(),
        updated_at: fixed_ts(),
        payload_json: json!({
            "execution_error": {
                "message": "QMT 实盘下单被拒绝: bridge qmt.mode=preview_only，要求 bridge qmt.mode=live",
                "failed_at": "2026-03-17T09:32:00Z"
            },
            "execution_diagnostics": {
                "code": "bridge_qmt_mode_not_live",
                "category": "gate",
                "stage": "execute",
                "summary": "qmt_live 提交被阻止：bridge qmt.mode=preview_only，要求 live",
                "operator_action": "use_live_bridge_mode",
                "hint_command": "quantix execution bridge status"
            }
        }),
    };

    let detail = format_strategy_request_detail(&row, false);

    assert!(detail.contains("=== Execution Diagnostics ==="));
    assert!(detail.contains("code: bridge_qmt_mode_not_live"));
    assert!(detail.contains("category: gate"));
    assert!(detail.contains("stage: execute"));
    assert!(
        detail.contains("summary: qmt_live 提交被阻止：bridge qmt.mode=preview_only，要求 live")
    );
    assert!(detail.contains("operator_action: use_live_bridge_mode"));
    assert!(detail.contains("hint_command: quantix execution bridge status"));
}

#[test]
fn test_format_strategy_request_detail_displays_qmt_live_recovery_context_from_related_order() {
    let request = crate::execution::models::ExecutionRequestRecord {
        request_id: "req-qmt-live-detail".to_string(),
        signal_id: "signal-qmt-live-detail".to_string(),
        target_mode: "qmt_live".to_string(),
        target_account: "default".to_string(),
        request_status: crate::execution::models::ExecutionRequestStatus::Completed,
        approved_by: Some("cli".to_string()),
        created_at: fixed_ts(),
        updated_at: fixed_ts(),
        payload_json: json!({
            "execution_result": {
                "client_order_id": "req-qmt-live-detail",
                "adapter_order_id": "task-1",
                "order_status": "pending_submit"
            }
        }),
    };
    let order = crate::execution::models::OrderRecord {
        order_id: "req-qmt-live-detail".to_string(),
        client_order_id: "req-qmt-live-detail".to_string(),
        run_id: "run-1".to_string(),
        symbol: "000001".to_string(),
        side: crate::execution::models::OrderSide::Buy,
        order_type: crate::execution::models::OrderType::Limit,
        requested_quantity: 100,
        requested_price: dec!(10.50),
        filled_quantity: 0,
        remaining_quantity: 100,
        avg_fill_price: None,
        status: crate::execution::models::OrderStatus::Accepted,
        adapter: "qmt_live".to_string(),
        created_at: fixed_ts(),
        updated_at: fixed_ts(),
        last_transition_at: fixed_ts(),
        version: 1,
        payload_json: json!({
            "qmt_live": {
                "task_identity": {
                    "task_id": "task-1",
                    "client_order_id": "req-qmt-live-detail",
                    "local_submission_id": "local-1",
                    "external_order_id": "broker-1"
                },
                "last_query": {
                    "latest_status": "accepted",
                    "filled_quantity": 0,
                    "avg_fill_price": null,
                    "broker_event_type": "Acknowledgement",
                    "rejection_reason": null,
                    "updated_at": "2026-05-03T09:32:00Z"
                },
                "reconciliation": {
                    "last_action": "state_updated",
                    "last_error": null,
                    "last_attempt_at": "2026-05-03T09:32:00Z"
                }
            }
        }),
    };

    let detail = format_strategy_request_detail_with_related_order(&request, Some(&order), false);

    assert!(detail.contains("=== QMT Live Recovery ==="));
    assert!(detail.contains("task_id: task-1"));
    assert!(detail.contains("latest_status: accepted"));
    assert!(detail.contains("broker_event_type: Acknowledgement"));
    assert!(detail.contains("last_action: state_updated"));
}

#[test]
fn test_format_strategy_request_row_appends_compact_qmt_live_recovery_suffix() {
    let request = crate::execution::models::ExecutionRequestRecord {
        request_id: "req-qmt-live-row".to_string(),
        signal_id: "signal-qmt-live-row".to_string(),
        target_mode: "qmt_live".to_string(),
        target_account: "default".to_string(),
        request_status: crate::execution::models::ExecutionRequestStatus::Completed,
        approved_by: Some("cli".to_string()),
        created_at: fixed_ts(),
        updated_at: fixed_ts(),
        payload_json: json!({
            "execution_result": {
                "client_order_id": "req-qmt-live-row",
                "adapter_order_id": "task-1",
                "order_status": "pending_submit"
            }
        }),
    };
    let order = crate::execution::models::OrderRecord {
        order_id: "req-qmt-live-row".to_string(),
        client_order_id: "req-qmt-live-row".to_string(),
        run_id: "run-1".to_string(),
        symbol: "000001".to_string(),
        side: crate::execution::models::OrderSide::Buy,
        order_type: crate::execution::models::OrderType::Limit,
        requested_quantity: 100,
        requested_price: dec!(10.50),
        filled_quantity: 0,
        remaining_quantity: 100,
        avg_fill_price: None,
        status: crate::execution::models::OrderStatus::PendingSubmit,
        adapter: "qmt_live".to_string(),
        created_at: fixed_ts(),
        updated_at: fixed_ts(),
        last_transition_at: fixed_ts(),
        version: 1,
        payload_json: json!({
            "qmt_live": {
                "task_identity": {
                    "task_id": "task-1",
                    "client_order_id": "req-qmt-live-row",
                    "local_submission_id": "local-1",
                    "external_order_id": null
                },
                "last_query": {
                    "latest_status": "pending_submit",
                    "filled_quantity": 0,
                    "avg_fill_price": null,
                    "broker_event_type": null,
                    "rejection_reason": null,
                    "updated_at": "2026-05-03T09:32:00Z"
                },
                "reconciliation": {
                    "last_action": "no_action",
                    "last_error": null,
                    "last_attempt_at": "2026-05-03T09:32:00Z"
                }
            }
        }),
    };

    let line = format_strategy_request_row_with_related_order(&request, Some(&order));

    assert!(line.contains("qmt_task_id=task-1"));
    assert!(line.contains("qmt_latest_status=pending_submit"));
    assert!(line.contains("qmt_last_action=no_action"));
}

#[test]
fn test_format_execution_daemon_summary_when_idle() {
    let summary = crate::execution::daemon::ExecutionDaemonIterationSummary {
        claimed: 0,
        completed: 0,
        failed: 0,
        request: None,
    };

    let line = format_execution_daemon_summary(&summary);

    assert_eq!(line, "execution daemon 未找到 pending request");
}

#[test]
fn test_format_execution_daemon_summary_includes_request_and_result() {
    let summary = crate::execution::daemon::ExecutionDaemonIterationSummary {
        claimed: 1,
        completed: 1,
        failed: 0,
        request: Some(crate::execution::models::ExecutionRequestRecord {
            request_id: "req-daemon-1".to_string(),
            signal_id: "signal-daemon-1".to_string(),
            target_mode: "paper".to_string(),
            target_account: "default".to_string(),
            request_status: crate::execution::models::ExecutionRequestStatus::Completed,
            approved_by: Some("cli".to_string()),
            created_at: fixed_ts(),
            updated_at: fixed_ts(),
            payload_json: json!({
                "execution_result": {
                    "order_status": "filled",
                    "client_order_id": "req-daemon-1_000001_1"
                }
            }),
        }),
    };

    let line = format_execution_daemon_summary(&summary);

    assert!(line.contains("request=req-daemon-1"));
    assert!(line.contains("status=completed"));
    assert!(line.contains("result=order_status=filled client_order_id=req-daemon-1_000001_1"));
}

#[test]
fn test_format_execution_daemon_summary_marks_non_terminal_completed_orders() {
    let summary = crate::execution::daemon::ExecutionDaemonIterationSummary {
        claimed: 1,
        completed: 1,
        failed: 0,
        request: Some(crate::execution::models::ExecutionRequestRecord {
            request_id: "req-daemon-accepted".to_string(),
            signal_id: "signal-daemon-accepted".to_string(),
            target_mode: "mock_live".to_string(),
            target_account: "default".to_string(),
            request_status: crate::execution::models::ExecutionRequestStatus::Completed,
            approved_by: Some("daemon".to_string()),
            created_at: fixed_ts(),
            updated_at: fixed_ts(),
            payload_json: json!({
                "execution_result": {
                    "order_status": "accepted",
                    "client_order_id": "req-daemon-accepted_000001_1"
                }
            }),
        }),
    };

    let line = format_execution_daemon_summary(&summary);

    assert!(line.contains("request=req-daemon-accepted"));
    assert!(line.contains("status=completed"));
    assert!(line.contains("semantics=request_completed_order_non_terminal"));
    assert!(
        line.contains("result=order_status=accepted client_order_id=req-daemon-accepted_000001_1")
    );
}

#[test]
fn test_format_strategy_request_row_includes_execution_timestamp_diagnostics() {
    let row = crate::execution::models::ExecutionRequestRecord {
        request_id: "req-row-executed".to_string(),
        signal_id: "signal-row-executed".to_string(),
        target_mode: "paper".to_string(),
        target_account: "default".to_string(),
        request_status: crate::execution::models::ExecutionRequestStatus::Completed,
        approved_by: Some("daemon".to_string()),
        created_at: fixed_ts(),
        updated_at: fixed_ts(),
        payload_json: json!({
            "execution_result": {
                "order_status": "filled",
                "client_order_id": "req-row-executed_000001_1",
                "executed_at": "2026-03-17T09:31:00Z"
            }
        }),
    };

    let line = format_strategy_request_row(&row);

    assert!(line.contains("status=completed"));
    assert!(line.contains("result=order_status=filled client_order_id=req-row-executed_000001_1 executed_at=2026-03-17T09:31:00Z"));
}

#[test]
fn test_format_strategy_request_row_appends_compact_diag_for_gate_failures() {
    let row = crate::execution::models::ExecutionRequestRecord {
        request_id: "req-row-diag".to_string(),
        signal_id: "signal-row-diag".to_string(),
        target_mode: "qmt_live".to_string(),
        target_account: "default".to_string(),
        request_status: crate::execution::models::ExecutionRequestStatus::Failed,
        approved_by: Some("cli".to_string()),
        created_at: fixed_ts(),
        updated_at: fixed_ts(),
        payload_json: json!({
            "execution_error": {
                "message": "QMT 实盘下单被拒绝: bridge qmt.mode=preview_only，要求 bridge qmt.mode=live"
            },
            "execution_diagnostics": {
                "code": "bridge_qmt_mode_not_live"
            }
        }),
    };

    let line = format_strategy_request_row(&row);

    assert!(line.contains("status=failed"));
    assert!(line.contains("diag=bridge_qmt_mode_not_live"));
}

#[test]
fn test_format_strategy_request_row_includes_cancellation_timestamp_diagnostics() {
    let row = crate::execution::models::ExecutionRequestRecord {
        request_id: "req-row-canceled".to_string(),
        signal_id: "signal-row-canceled".to_string(),
        target_mode: "paper".to_string(),
        target_account: "default".to_string(),
        request_status: crate::execution::models::ExecutionRequestStatus::Canceled,
        approved_by: Some("cli".to_string()),
        created_at: fixed_ts(),
        updated_at: fixed_ts(),
        payload_json: json!({
            "cancellation": {
                "reason": "manual cancel",
                "canceled_at": "2026-03-17T09:33:00Z"
            }
        }),
    };

    let line = format_strategy_request_row(&row);

    assert!(line.contains("status=canceled"));
    assert!(line.contains("result=reason=manual cancel canceled_at=2026-03-17T09:33:00Z"));
}

#[test]
fn test_format_execution_daemon_summary_appends_compact_diag_for_gate_failures() {
    let summary = crate::execution::daemon::ExecutionDaemonIterationSummary {
        claimed: 1,
        completed: 0,
        failed: 1,
        request: Some(crate::execution::models::ExecutionRequestRecord {
            request_id: "req-daemon-diag".to_string(),
            signal_id: "signal-daemon-diag".to_string(),
            target_mode: "qmt_live".to_string(),
            target_account: "default".to_string(),
            request_status: crate::execution::models::ExecutionRequestStatus::Failed,
            approved_by: Some("cli".to_string()),
            created_at: fixed_ts(),
            updated_at: fixed_ts(),
            payload_json: json!({
                "execution_error": {
                    "message": "QMT 实盘下单被拒绝: bridge qmt.mode=preview_only，要求 bridge qmt.mode=live"
                },
                "execution_diagnostics": {
                    "code": "bridge_qmt_mode_not_live"
                }
            }),
        }),
    };

    let line = format_execution_daemon_summary(&summary);

    assert!(line.contains("request=req-daemon-diag"));
    assert!(line.contains("status=failed"));
    assert!(line.contains("diag=bridge_qmt_mode_not_live"));
}

#[test]
fn test_format_execution_daemon_summary_includes_failure_timestamp_diagnostics() {
    let summary = crate::execution::daemon::ExecutionDaemonIterationSummary {
        claimed: 1,
        completed: 0,
        failed: 1,
        request: Some(crate::execution::models::ExecutionRequestRecord {
            request_id: "req-daemon-failed-ts".to_string(),
            signal_id: "signal-daemon-failed-ts".to_string(),
            target_mode: "live".to_string(),
            target_account: "default".to_string(),
            request_status: crate::execution::models::ExecutionRequestStatus::Failed,
            approved_by: Some("cli".to_string()),
            created_at: fixed_ts(),
            updated_at: fixed_ts(),
            payload_json: json!({
                "execution_error": {
                    "message": "execution daemon live 模式尚未实现；如需真实 QMT 提交，请将 request target_mode 设为 qmt_live，并确保 bridge qmt.mode=live，然后走 execution bridge qmt-live 路径",
                    "failed_at": "2026-03-17T09:32:00Z"
                }
            }),
        }),
    };

    let line = format_execution_daemon_summary(&summary);

    assert!(line.contains("request=req-daemon-failed-ts"));
    assert!(line.contains("status=failed"));
    assert!(line.contains("execution daemon live 模式尚未实现"));
    assert!(line.contains("qmt_live"));
    assert!(line.contains("qmt.mode=live"));
    assert!(line.contains("execution bridge qmt-live"));
    assert!(line.contains("failed_at=2026-03-17T09:32:00Z"));
}

#[test]
fn test_format_execution_daemon_summary_includes_failure_reason() {
    let summary = crate::execution::daemon::ExecutionDaemonIterationSummary {
        claimed: 1,
        completed: 0,
        failed: 1,
        request: Some(crate::execution::models::ExecutionRequestRecord {
            request_id: "req-daemon-failed".to_string(),
            signal_id: "signal-daemon-failed".to_string(),
            target_mode: "live".to_string(),
            target_account: "default".to_string(),
            request_status: crate::execution::models::ExecutionRequestStatus::Failed,
            approved_by: Some("cli".to_string()),
            created_at: fixed_ts(),
            updated_at: fixed_ts(),
            payload_json: json!({
                "execution_error": {
                    "message": "execution daemon live 模式尚未实现；如需真实 QMT 提交，请将 request target_mode 设为 qmt_live，并确保 bridge qmt.mode=live，然后走 execution bridge qmt-live 路径"
                }
            }),
        }),
    };

    let line = format_execution_daemon_summary(&summary);

    assert!(line.contains("request=req-daemon-failed"));
    assert!(line.contains("status=failed"));
    assert!(line.contains("execution daemon live 模式尚未实现"));
    assert!(line.contains("qmt_live"));
    assert!(line.contains("execution bridge qmt-live"));
}

#[test]
fn test_format_strategy_signal_row_includes_source_metadata() {
    let row = crate::execution::models::StrategySignalRecord {
        signal_id: "signal-1".to_string(),
        strategy_instance_id: "ma_fast_5_slow_20".to_string(),
        strategy_name: "ma_cross".to_string(),
        symbol: "000001".to_string(),
        timeframe: "1d".to_string(),
        bar_end: fixed_ts(),
        signal_value: "buy".to_string(),
        signal_status: crate::execution::models::SignalStatus::New,
        approval_status: crate::execution::models::ApprovalStatus::Pending,
        run_id: "run-1".to_string(),
        metadata_json: json!({
            "bar_source_id": "clickhouse-storage",
            "bar_source_fallback": false
        }),
        created_at: fixed_ts(),
        updated_at: fixed_ts(),
    };

    let line = format_strategy_signal_row(&row);

    assert!(line.contains("signal-1"));
    assert!(line.contains("bar_end=2026-03-17T09:30:00Z"));
    assert!(line.contains("source=clickhouse-storage"));
    assert!(line.contains("fallback=false"));
}

#[tokio::test]
async fn test_execute_screener_preset_list_returns_supported_presets() {
    let output = execute_screener_command_with_loader(
        ScreenerCommands::PresetList,
        FakeLoader::default(),
        WatchlistStorage::new("/tmp/unused-screener-watchlist.json"),
    )
    .await
    .unwrap();

    match output {
        ScreenerCommandOutput::PresetList(presets) => {
            let names: Vec<&str> = presets.iter().map(|item| item.name).collect();
            assert_eq!(
                names,
                vec![
                    "close_above_ma",
                    "close_below_ma",
                    "rsi_gte",
                    "rsi_lte",
                    "volume_ratio_gte",
                ]
            );
        }
        ScreenerCommandOutput::Rows(_) => panic!("expected preset list output"),
    }
}

#[tokio::test]
async fn test_execute_screener_run_with_codes_returns_rows() {
    let loader = FakeLoader {
        data: HashMap::from([
            (
                "000001".to_string(),
                vec![
                    make_kline("000001", 1, dec!(10), 100),
                    make_kline("000001", 2, dec!(10), 100),
                    make_kline("000001", 3, dec!(10), 100),
                    make_kline("000001", 4, dec!(11), 100),
                    make_kline("000001", 5, dec!(12), 100),
                ],
            ),
            (
                "000002".to_string(),
                vec![
                    make_kline("000002", 1, dec!(10), 100),
                    make_kline("000002", 2, dec!(10), 100),
                    make_kline("000002", 3, dec!(10), 100),
                    make_kline("000002", 4, dec!(12), 100),
                    make_kline("000002", 5, dec!(15), 100),
                ],
            ),
        ]),
    };

    let output = execute_screener_command_with_loader(
        ScreenerCommands::Run {
            codes: Some("000001,000002".to_string()),
            watchlist: false,
            group: None,
            preset: vec!["close_above_ma:period=3".to_string()],
            limit: Some(1),
            sort_by: Some("score".to_string()),
        },
        loader,
        WatchlistStorage::new("/tmp/unused-screener-watchlist.json"),
    )
    .await
    .unwrap();

    match output {
        ScreenerCommandOutput::Rows(rows) => {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].code, "000002");
            assert!(rows[0].matched);
        }
        ScreenerCommandOutput::PresetList(_) => panic!("expected rows output"),
    }
}

#[tokio::test]
async fn test_execute_screener_run_with_watchlist_group_uses_watchlist_storage() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("watchlist.json");
    let storage = WatchlistStorage::new(&path);
    let service = WatchlistService::default();
    let mut store = storage.load_or_create().unwrap();
    service
        .create_group(&mut store, "core", Utc::now())
        .unwrap();
    service
        .add(&mut store, "000001", Some("core"), Utc::now())
        .unwrap();
    service.add(&mut store, "000002", None, Utc::now()).unwrap();
    storage.save(&store).unwrap();

    let loader = FakeLoader {
        data: HashMap::from([(
            "000001".to_string(),
            vec![
                make_kline("000001", 1, dec!(10), 100),
                make_kline("000001", 2, dec!(10), 100),
                make_kline("000001", 3, dec!(10), 100),
                make_kline("000001", 4, dec!(11), 100),
                make_kline("000001", 5, dec!(12), 100),
            ],
        )]),
    };

    let output = execute_screener_command_with_loader(
        ScreenerCommands::Run {
            codes: None,
            watchlist: true,
            group: Some("core".to_string()),
            preset: vec!["close_above_ma:period=3".to_string()],
            limit: None,
            sort_by: None,
        },
        loader,
        storage,
    )
    .await
    .unwrap();

    match output {
        ScreenerCommandOutput::Rows(rows) => {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].code, "000001");
        }
        ScreenerCommandOutput::PresetList(_) => panic!("expected rows output"),
    }
}

#[tokio::test]
async fn test_execute_screener_run_rejects_invalid_preset() {
    let err = execute_screener_command_with_loader(
        ScreenerCommands::Run {
            codes: Some("000001".to_string()),
            watchlist: false,
            group: None,
            preset: vec!["unknown_rule:value=1".to_string()],
            limit: None,
            sort_by: None,
        },
        FakeLoader::default(),
        WatchlistStorage::new("/tmp/unused-screener-watchlist.json"),
    )
    .await
    .unwrap_err();

    assert!(matches!(err, QuantixError::Other(_)));
    assert!(err.to_string().contains("未知的 preset"));
}
