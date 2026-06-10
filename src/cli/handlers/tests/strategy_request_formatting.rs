use super::strategy_helpers::fixed_ts;
use super::*;
use rust_decimal_macros::dec;

#[allow(dead_code)]
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
    assert!(!line.contains("action=wait_reconciliation"));
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
                "hint_command": "quantix execution qmt status --checklist"
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
    assert!(detail.contains("hint_command: quantix execution qmt status --checklist"));
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
    assert!(!line.contains("action=wait_reconciliation"));
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
                "code": "bridge_qmt_mode_not_live",
                "operator_action": "use_live_bridge_mode",
                "hint_command": "quantix execution qmt status --checklist"
            }
        }),
    };

    let line = format_strategy_request_row(&row);

    assert!(line.contains("status=failed"));
    assert!(line.contains("diag=bridge_qmt_mode_not_live"));
    assert!(line.contains("action=use_live_bridge_mode"));
    assert!(line.contains(r#"hint_command="quantix execution qmt status --checklist""#));
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
                    "code": "bridge_qmt_mode_not_live",
                    "operator_action": "use_live_bridge_mode",
                    "hint_command": "quantix execution qmt status --checklist"
                }
            }),
        }),
    };

    let line = format_execution_daemon_summary(&summary);

    assert!(line.contains("request=req-daemon-diag"));
    assert!(line.contains("status=failed"));
    assert!(line.contains("diag=bridge_qmt_mode_not_live"));
    assert!(line.contains("action=use_live_bridge_mode"));
    assert!(line.contains(r#"hint_command="quantix execution qmt status --checklist""#));
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
