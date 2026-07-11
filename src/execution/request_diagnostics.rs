use crate::execution::qmt_live_gate::QMT_LIVE_SUBMIT_SUPPORT_REQUIREMENT;
use serde_json::{Value, json};

pub const EXECUTION_DIAGNOSTICS_KEY: &str = "execution_diagnostics";
pub const QMT_LIVE_DIAGNOSTIC_SOURCE: &str = "qmt_live_gate";
pub const QMT_LIVE_FAILURE_CATEGORY_CAPABILITY_CHECK_FAILED: &str = "capability_check_failed";
pub const QMT_LIVE_FAILURE_CATEGORY_MISSING_REQUIRED_CAPABILITY: &str =
    "missing_required_capability";
pub const QMT_LIVE_CAPABILITIES_ENDPOINT_REQUIREMENT: &str =
    "bridge /api/v1/capabilities returns qmt capability metadata";

/// 从 request.payload_json 中读取 `execution_diagnostics.code`；不存在返回 `None`。
pub fn diagnostics_code(payload_json: &Value) -> Option<&str> {
    payload_json
        .get(EXECUTION_DIAGNOSTICS_KEY)
        .and_then(|value| value.get("code"))
        .and_then(|value| value.as_str())
}

/// 从 request.payload_json 中读取 `execution_diagnostics.semantics`；不存在或显式为 null 时返回 `None`。
pub fn diagnostics_semantics(payload_json: &Value) -> Option<&str> {
    payload_json
        .get(EXECUTION_DIAGNOSTICS_KEY)
        .and_then(|value| value.get("semantics"))
        .and_then(|value| value.as_str())
}

/// 判断指定 diagnostics code 是否需要在 compact 视图中显示；`request_completed_*` 两个表示常规终态，默认隐藏。
pub fn should_show_compact_diag(code: &str) -> bool {
    !matches!(
        code,
        "request_completed_order_terminal" | "request_completed_order_non_terminal"
    )
}

/// 根据 order_status 生成 request 完成时的诊断：未终态返回 `request_completed_order_non_terminal`，否则返回 terminal 变体。
pub fn build_completion_diagnostics(order_status: Option<&str>) -> Value {
    match order_status {
        Some(
            "pending_submit" | "submitted" | "accepted" | "partially_filled" | "pending_cancel"
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

/// 生成 `daemon_qmt_live_manual_bridge_required` 诊断，提示用 `quantix execution bridge qmt-live` 手动提交指定 request。
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

/// 生成 `daemon_live_mode_unsupported` 诊断，提示 daemon 的 live 模式尚未实现、需改用 qmt_live。
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

/// 生成 `kill_switch_blocked` 诊断，summary 携带 target_mode，hint 指向 `safety kill-switch status`。
pub fn build_kill_switch_blocked_diagnostics(target_mode: &str) -> Value {
    json!({
        "schema_version": 1,
        "code": "kill_switch_blocked",
        "category": "gate",
        "stage": "execute",
        "semantics": null,
        "order_terminality": "unknown",
        "summary": format!("{target_mode} execution 被 kill switch 阻止"),
        "operator_action": "disable_kill_switch_or_use_paper",
        "hint_command": "quantix safety kill-switch status"
    })
}

/// 生成 `bridge_qmt_mode_not_live` 诊断，summary 携带观察到的 mode 值（如 preview_only）。
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

/// 生成 `bridge_qmt_capability_disabled` 诊断，提示在 bridge 端启用 qmt capability。
pub fn build_bridge_qmt_capability_disabled_diagnostics() -> Value {
    json!({
        "schema_version": 1,
        "code": "bridge_qmt_capability_disabled",
        "category": "gate",
        "stage": "execute",
        "semantics": null,
        "order_terminality": "unknown",
        "summary": "qmt_live 提交被阻止：bridge qmt capability 未启用",
        "operator_action": "enable_qmt_capability",
        "hint_command": "quantix execution bridge status"
    })
}

/// 生成 `bridge_qmt_order_submit_capability_missing` 诊断，附 qmt_live_gate 来源与 compatibility 要求。
pub fn build_bridge_qmt_order_submit_capability_missing_diagnostics() -> Value {
    json!({
        "schema_version": 1,
        "code": "bridge_qmt_order_submit_capability_missing",
        "category": "gate",
        "stage": "execute",
        "semantics": null,
        "order_terminality": "unknown",
        "diagnostic_source": QMT_LIVE_DIAGNOSTIC_SOURCE,
        "qmt_live_failure_category": QMT_LIVE_FAILURE_CATEGORY_MISSING_REQUIRED_CAPABILITY,
        "compatibility_requirement": QMT_LIVE_SUBMIT_SUPPORT_REQUIREMENT,
        "summary": "qmt_live 提交被阻止：bridge 缺少 order_submit 能力",
        "operator_action": "enable_order_submit_capability",
        "hint_command": "quantix execution bridge status"
    })
}

/// 生成 `bridge_qmt_capability_check_failed` 诊断；summary 描述具体失败原因（如 503/超时）。
pub fn build_bridge_qmt_capability_check_failed_diagnostics(summary: &str) -> Value {
    json!({
        "schema_version": 1,
        "code": "bridge_qmt_capability_check_failed",
        "category": "gate",
        "stage": "execute",
        "semantics": null,
        "order_terminality": "unknown",
        "diagnostic_source": QMT_LIVE_DIAGNOSTIC_SOURCE,
        "qmt_live_failure_category": QMT_LIVE_FAILURE_CATEGORY_CAPABILITY_CHECK_FAILED,
        "compatibility_requirement": QMT_LIVE_CAPABILITIES_ENDPOINT_REQUIREMENT,
        "summary": summary,
        "operator_action": "inspect_bridge_status",
        "hint_command": "quantix execution bridge status"
    })
}

/// 兜底生成 `execution_error_unclassified` 诊断；调用方已无法归类失败原因时使用，summary 携带原始错误。
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
        assert_eq!(
            diagnostics["order_terminality"].as_str(),
            Some("non_terminal")
        );
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
    fn test_build_bridge_capability_disabled_diagnostics_keeps_status_hint() {
        let diagnostics = build_bridge_qmt_capability_disabled_diagnostics();

        assert_eq!(
            diagnostics["code"].as_str(),
            Some("bridge_qmt_capability_disabled")
        );
        assert_eq!(diagnostics["category"].as_str(), Some("gate"));
        assert_eq!(
            diagnostics["operator_action"].as_str(),
            Some("enable_qmt_capability")
        );
        assert_eq!(
            diagnostics["hint_command"].as_str(),
            Some("quantix execution bridge status")
        );
    }

    #[test]
    fn test_build_bridge_capability_check_failed_diagnostics_keeps_status_hint() {
        let diagnostics =
            build_bridge_qmt_capability_check_failed_diagnostics("QMT 实盘能力检查失败: 503");

        assert_eq!(
            diagnostics["code"].as_str(),
            Some("bridge_qmt_capability_check_failed")
        );
        assert_eq!(diagnostics["category"].as_str(), Some("gate"));
        assert_eq!(
            diagnostics["summary"].as_str(),
            Some("QMT 实盘能力检查失败: 503")
        );
        assert_eq!(
            diagnostics["hint_command"].as_str(),
            Some("quantix execution bridge status")
        );
    }

    #[test]
    fn test_build_bridge_qmt_diagnostics_surface_structured_gate_metadata() {
        let capability_failed =
            build_bridge_qmt_capability_check_failed_diagnostics("QMT 实盘能力检查失败: 503");

        assert_eq!(
            capability_failed["diagnostic_source"].as_str(),
            Some("qmt_live_gate")
        );
        assert_eq!(
            capability_failed["qmt_live_failure_category"].as_str(),
            Some("capability_check_failed")
        );
        assert_eq!(
            capability_failed["compatibility_requirement"].as_str(),
            Some("bridge /api/v1/capabilities returns qmt capability metadata")
        );

        let missing_order_submit = build_bridge_qmt_order_submit_capability_missing_diagnostics();

        assert_eq!(
            missing_order_submit["diagnostic_source"].as_str(),
            Some("qmt_live_gate")
        );
        assert_eq!(
            missing_order_submit["qmt_live_failure_category"].as_str(),
            Some("missing_required_capability")
        );
        assert_eq!(
            missing_order_submit["compatibility_requirement"].as_str(),
            Some("bridge qmt.supports includes order_submit")
        );
    }

    #[test]
    fn test_build_kill_switch_blocked_diagnostics_keeps_kill_switch_hint() {
        let diagnostics = build_kill_switch_blocked_diagnostics("qmt_live");

        assert_eq!(diagnostics["code"].as_str(), Some("kill_switch_blocked"));
        assert_eq!(diagnostics["category"].as_str(), Some("gate"));
        assert!(
            diagnostics["summary"]
                .as_str()
                .unwrap()
                .contains("qmt_live")
        );
        assert_eq!(
            diagnostics["hint_command"].as_str(),
            Some("quantix safety kill-switch status")
        );
    }

    #[test]
    fn test_should_hide_completion_codes_from_compact_diag_suffix() {
        assert!(!should_show_compact_diag(
            "request_completed_order_terminal"
        ));
        assert!(!should_show_compact_diag(
            "request_completed_order_non_terminal"
        ));
        assert!(should_show_compact_diag("bridge_qmt_mode_not_live"));
    }
}
