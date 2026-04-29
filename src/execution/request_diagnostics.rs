use serde_json::{json, Value};

pub const EXECUTION_DIAGNOSTICS_KEY: &str = "execution_diagnostics";

pub fn diagnostics_code(payload_json: &Value) -> Option<&str> {
    payload_json
        .get(EXECUTION_DIAGNOSTICS_KEY)
        .and_then(|value| value.get("code"))
        .and_then(|value| value.as_str())
}

pub fn diagnostics_semantics(payload_json: &Value) -> Option<&str> {
    payload_json
        .get(EXECUTION_DIAGNOSTICS_KEY)
        .and_then(|value| value.get("semantics"))
        .and_then(|value| value.as_str())
}

pub fn should_show_compact_diag(code: &str) -> bool {
    !matches!(
        code,
        "request_completed_order_terminal" | "request_completed_order_non_terminal"
    )
}

pub fn build_completion_diagnostics(order_status: Option<&str>) -> Value {
    match order_status {
        Some(
            "pending_submit"
            | "submitted"
            | "accepted"
            | "partially_filled"
            | "pending_cancel"
            | "unknown",
        ) => json!({
            "schema_version": 1,
            "code": "request_completed_order_non_terminal",
            "category": "completion",
            "stage": "complete",
            "semantics": "request_completed_order_non_terminal",
            "order_terminality": "non_terminal",
            "summary": format!(
                "request completed 仅表示执行层已完成；订单仍处于 {}",
                order_status.unwrap_or("unknown")
            ),
            "operator_action": "wait_reconciliation",
            "hint_command": null
        }),
        _ => json!({
            "schema_version": 1,
            "code": "request_completed_order_terminal",
            "category": "completion",
            "stage": "complete",
            "semantics": null,
            "order_terminality": "terminal",
            "summary": "request completed，且当前订单状态已终态",
            "operator_action": "none",
            "hint_command": null
        }),
    }
}

pub fn build_daemon_qmt_live_manual_bridge_required_diagnostics(request_id: &str) -> Value {
    json!({
        "schema_version": 1,
        "code": "daemon_qmt_live_manual_bridge_required",
        "category": "gate",
        "stage": "execute",
        "semantics": null,
        "order_terminality": "unknown",
        "summary": "qmt_live request 不能通过自动执行路径提交",
        "operator_action": "use_manual_qmt_live_bridge",
        "hint_command": format!("quantix execution bridge qmt-live --request-id {request_id}")
    })
}

pub fn build_daemon_live_mode_unsupported_diagnostics() -> Value {
    json!({
        "schema_version": 1,
        "code": "daemon_live_mode_unsupported",
        "category": "mode_boundary",
        "stage": "execute",
        "semantics": null,
        "order_terminality": "unknown",
        "summary": "execution daemon live 模式尚未实现",
        "operator_action": "recreate_as_qmt_live_request",
        "hint_command": "quantix execution bridge status"
    })
}

pub fn build_bridge_qmt_mode_not_live_diagnostics(observed_mode: &str) -> Value {
    json!({
        "schema_version": 1,
        "code": "bridge_qmt_mode_not_live",
        "category": "gate",
        "stage": "execute",
        "semantics": null,
        "order_terminality": "unknown",
        "summary": format!("qmt_live 提交被阻止：bridge qmt.mode={}，要求 live", observed_mode),
        "operator_action": "use_live_bridge_mode",
        "hint_command": "quantix execution bridge status"
    })
}

pub fn build_bridge_qmt_order_submit_capability_missing_diagnostics() -> Value {
    json!({
        "schema_version": 1,
        "code": "bridge_qmt_order_submit_capability_missing",
        "category": "gate",
        "stage": "execute",
        "semantics": null,
        "order_terminality": "unknown",
        "summary": "qmt_live 提交被阻止：bridge 缺少 order_submit 能力",
        "operator_action": "enable_order_submit_capability",
        "hint_command": "quantix execution bridge status"
    })
}

pub fn build_unclassified_execution_error_diagnostics(summary: &str) -> Value {
    json!({
        "schema_version": 1,
        "code": "execution_error_unclassified",
        "category": "execution_error",
        "stage": "execute",
        "semantics": null,
        "order_terminality": "unknown",
        "summary": summary,
        "operator_action": "inspect_error",
        "hint_command": null
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_completion_diagnostics_marks_non_terminal_order() {
        let diagnostics = build_completion_diagnostics(Some("accepted"));

        assert_eq!(
            diagnostics["code"].as_str(),
            Some("request_completed_order_non_terminal")
        );
        assert_eq!(
            diagnostics["semantics"].as_str(),
            Some("request_completed_order_non_terminal")
        );
        assert_eq!(diagnostics["order_terminality"].as_str(), Some("non_terminal"));
        assert_eq!(
            diagnostics["operator_action"].as_str(),
            Some("wait_reconciliation")
        );
    }

    #[test]
    fn test_build_bridge_mode_gate_diagnostics_keeps_status_hint() {
        let diagnostics = build_bridge_qmt_mode_not_live_diagnostics("preview_only");

        assert_eq!(
            diagnostics["code"].as_str(),
            Some("bridge_qmt_mode_not_live")
        );
        assert_eq!(diagnostics["category"].as_str(), Some("gate"));
        assert_eq!(diagnostics["stage"].as_str(), Some("execute"));
        assert!(
            diagnostics["summary"]
                .as_str()
                .unwrap()
                .contains("preview_only")
        );
        assert_eq!(
            diagnostics["hint_command"].as_str(),
            Some("quantix execution bridge status")
        );
    }

    #[test]
    fn test_should_hide_completion_codes_from_compact_diag_suffix() {
        assert!(!should_show_compact_diag("request_completed_order_terminal"));
        assert!(!should_show_compact_diag("request_completed_order_non_terminal"));
        assert!(should_show_compact_diag("bridge_qmt_mode_not_live"));
    }
}
