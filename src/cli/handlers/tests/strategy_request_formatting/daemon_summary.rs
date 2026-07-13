use super::strategy_helpers::fixed_ts;
use super::*;

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
