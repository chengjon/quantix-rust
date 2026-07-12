use crate::cli::handlers::strategy_handler::is_non_terminal_order_status;
use crate::execution::daemon::ExecutionDaemonIterationSummary;
use crate::execution::models::{ExecutionRequestRecord, ExecutionRequestStatus, OrderRecord};
use crate::execution::request_diagnostics::{
    diagnostics_code, diagnostics_semantics, should_show_compact_diag,
};

use super::qmt::{append_qmt_live_recovery_detail, qmt_live_compact_summary};

#[allow(dead_code)]
pub(crate) fn format_strategy_request_detail(
    request: &ExecutionRequestRecord,
    verbose: bool,
) -> String {
    format_strategy_request_detail_with_related_order(request, None, verbose)
}

pub(crate) fn format_strategy_request_detail_with_related_order(
    request: &ExecutionRequestRecord,
    related_order: Option<&OrderRecord>,
    verbose: bool,
) -> String {
    let mut lines = vec![
        "=== Execution Request Detail ===".to_string(),
        format!("request_id: {}", request.request_id),
        format!("signal_id: {}", request.signal_id),
        format!("target_mode: {}", request.target_mode),
        format!("target_account: {}", request.target_account),
        format!("status: {}", request.request_status.as_str()),
        format!("request_status: {}", request.request_status.as_str()),
        format!(
            "approved_by: {}",
            request.approved_by.as_deref().unwrap_or("-")
        ),
        format!(
            "created_at: {}",
            request.created_at.format("%Y-%m-%dT%H:%M:%SZ")
        ),
        format!(
            "updated_at: {}",
            request.updated_at.format("%Y-%m-%dT%H:%M:%SZ")
        ),
    ];

    if let Some(snapshot) = request.payload_json.get("execution_snapshot") {
        lines.push(String::new());
        lines.push("=== Execution Snapshot ===".to_string());
        if let Some(symbol) = snapshot.get("symbol").and_then(|v| v.as_str()) {
            lines.push(format!("symbol: {}", symbol));
        }
        if let Some(signal_value) = snapshot.get("signal_value").and_then(|v| v.as_str()) {
            lines.push(format!("signal: {}", signal_value));
        }
        if let Some(intent) = snapshot.get("order_intent") {
            if let Some(side) = intent.get("side").and_then(|v| v.as_str()) {
                lines.push(format!("side: {}", side));
            }
            if let Some(qty) = intent.get("requested_quantity").and_then(|v| v.as_i64()) {
                lines.push(format!("quantity: {}", qty));
            }
            if let Some(price) = intent.get("requested_price").and_then(|v| v.as_str()) {
                lines.push(format!("price: {}", price));
            }
        }
    }

    if let Some(result) = request.payload_json.get("execution_result") {
        lines.push(String::new());
        lines.push("=== Execution Result ===".to_string());
        if let Some(run_id) = result.get("run_id").and_then(|v| v.as_str()) {
            lines.push(format!("run_id: {}", run_id));
        }
        if let Some(client_order_id) = result.get("client_order_id").and_then(|v| v.as_str()) {
            lines.push(format!("client_order_id: {}", client_order_id));
        }
        if let Some(order_status) = result.get("order_status").and_then(|v| v.as_str()) {
            lines.push(format!("order_status: {}", order_status));
            if request.request_status == ExecutionRequestStatus::Completed
                && is_non_terminal_order_status(order_status)
            {
                lines.push(format!(
                    "status_note: request completed only means execution layer finished; order remains {order_status}"
                ));
            }
        }
        if let Some(executed_at) = result.get("executed_at").and_then(|v| v.as_str()) {
            lines.push(format!("executed_at: {}", executed_at));
        }
    }

    if let Some(error) = request.payload_json.get("execution_error") {
        lines.push(String::new());
        lines.push("=== Execution Error ===".to_string());
        if let Some(message) = error.get("message").and_then(|v| v.as_str()) {
            lines.push(format!("message: {}", message));
        }
        if let Some(failed_at) = error.get("failed_at").and_then(|v| v.as_str()) {
            lines.push(format!("failed_at: {}", failed_at));
        }
    }

    if let Some(diagnostics) = request.payload_json.get("execution_diagnostics") {
        lines.push(String::new());
        lines.push("=== Execution Diagnostics ===".to_string());
        if let Some(code) = diagnostics.get("code").and_then(|v| v.as_str()) {
            lines.push(format!("code: {}", code));
        }
        if let Some(category) = diagnostics.get("category").and_then(|v| v.as_str()) {
            lines.push(format!("category: {}", category));
        }
        if let Some(stage) = diagnostics.get("stage").and_then(|v| v.as_str()) {
            lines.push(format!("stage: {}", stage));
        }
        if let Some(semantics) = diagnostics.get("semantics").and_then(|v| v.as_str()) {
            lines.push(format!("semantics: {}", semantics));
        }
        if let Some(order_terminality) = diagnostics
            .get("order_terminality")
            .and_then(|v| v.as_str())
        {
            lines.push(format!("order_terminality: {}", order_terminality));
        }
        if let Some(summary) = diagnostics.get("summary").and_then(|v| v.as_str()) {
            lines.push(format!("summary: {}", summary));
        }
        if let Some(operator_action) = diagnostics.get("operator_action").and_then(|v| v.as_str()) {
            lines.push(format!("operator_action: {}", operator_action));
        }
        if let Some(hint_command) = diagnostics.get("hint_command").and_then(|v| v.as_str()) {
            lines.push(format!("hint_command: {}", hint_command));
        }
    }

    if let Some(cancellation) = request.payload_json.get("cancellation") {
        lines.push(String::new());
        lines.push("=== Cancellation ===".to_string());
        if let Some(reason) = cancellation.get("reason").and_then(|v| v.as_str()) {
            lines.push(format!("reason: {}", reason));
        }
        if let Some(canceled_at) = cancellation.get("canceled_at").and_then(|v| v.as_str()) {
            lines.push(format!("canceled_at: {}", canceled_at));
        }
    }

    append_qmt_live_recovery_detail(&mut lines, related_order);

    if verbose {
        lines.push(String::new());
        lines.push("=== Full Payload (verbose) ===".to_string());
        lines.push(
            serde_json::to_string_pretty(&request.payload_json)
                .unwrap_or_else(|_| "<serialize error>".to_string()),
        );
    }

    lines.join("\n")
}

pub(crate) fn format_strategy_request_row(row: &ExecutionRequestRecord) -> String {
    format_strategy_request_row_with_related_order(row, None)
}

pub(crate) fn format_strategy_request_row_with_related_order(
    row: &ExecutionRequestRecord,
    related_order: Option<&OrderRecord>,
) -> String {
    let result = format_execution_request_result(&row.payload_json);
    let semantics = compact_semantics_suffix(&row.payload_json, row.request_status);
    let diag = compact_diag_suffix(&row.payload_json);
    let guidance = compact_operator_guidance_suffix(&row.payload_json);
    let qmt_suffix = related_order
        .and_then(qmt_live_compact_summary)
        .map(|summary| format!(" {summary}"))
        .unwrap_or_default();

    format!(
        "{} signal={} target={}/{} status={}{}{}{}{}{} created_at={}",
        row.request_id,
        row.signal_id,
        row.target_mode,
        row.target_account,
        row.request_status.as_str(),
        semantics,
        diag,
        guidance,
        result,
        qmt_suffix,
        row.created_at.format("%Y-%m-%dT%H:%M:%SZ")
    )
}

pub(crate) fn format_execution_daemon_summary(summary: &ExecutionDaemonIterationSummary) -> String {
    if summary.claimed == 0 {
        return "execution daemon 未找到 pending request".to_string();
    }

    let Some(request) = summary.request.as_ref() else {
        return "execution daemon consumed request=<unknown> status=unknown".to_string();
    };

    let semantics = compact_semantics_suffix(&request.payload_json, request.request_status);
    let diag = compact_diag_suffix(&request.payload_json);
    let guidance = compact_operator_guidance_suffix(&request.payload_json);
    let result = format_execution_request_result(&request.payload_json);
    format!(
        "execution daemon consumed request={} status={}{}{}{}{}",
        request.request_id,
        request.request_status.as_str(),
        semantics,
        diag,
        guidance,
        result
    )
}

fn compact_semantics_suffix(
    payload_json: &serde_json::Value,
    request_status: ExecutionRequestStatus,
) -> String {
    diagnostics_semantics(payload_json)
        .map(|semantics| format!(" semantics={semantics}"))
        .or_else(|| {
            payload_json
                .get("execution_result")
                .and_then(|value| value.get("order_status"))
                .and_then(|value| value.as_str())
                .filter(|order_status| {
                    request_status == ExecutionRequestStatus::Completed
                        && is_non_terminal_order_status(order_status)
                })
                .map(|_| " semantics=request_completed_order_non_terminal".to_string())
        })
        .unwrap_or_default()
}

fn compact_diag_suffix(payload_json: &serde_json::Value) -> String {
    diagnostics_code(payload_json)
        .filter(|code| should_show_compact_diag(code))
        .map(|code| format!(" diag={code}"))
        .unwrap_or_default()
}

fn compact_operator_guidance_suffix(payload_json: &serde_json::Value) -> String {
    if diagnostics_code(payload_json)
        .filter(|code| should_show_compact_diag(code))
        .is_none()
    {
        return String::new();
    }

    let Some(diagnostics) = payload_json.get("execution_diagnostics") else {
        return String::new();
    };

    let action = diagnostics
        .get("operator_action")
        .and_then(|value| value.as_str())
        .filter(|value| !value.trim().is_empty() && *value != "none");
    let hint_command = diagnostics
        .get("hint_command")
        .and_then(|value| value.as_str())
        .filter(|value| !value.trim().is_empty());

    let mut suffix = String::new();
    if let Some(action) = action {
        suffix.push_str(&format!(" action={action}"));
    }
    if let Some(hint_command) = hint_command {
        suffix.push_str(&format!(" hint_command={hint_command:?}"));
    }
    suffix
}

pub(crate) fn format_execution_request_result(payload_json: &serde_json::Value) -> String {
    payload_json
        .get("execution_result")
        .and_then(|value| {
            let order_status = value.get("order_status").and_then(|item| item.as_str())?;
            let client_order_id = value
                .get("client_order_id")
                .and_then(|item| item.as_str())
                .unwrap_or("-");
            let mut line = format!(
                " result=order_status={} client_order_id={}",
                order_status, client_order_id
            );
            if let Some(executed_at) = value.get("executed_at").and_then(|item| item.as_str()) {
                line.push_str(&format!(" executed_at={executed_at}"));
            }
            Some(line)
        })
        .or_else(|| {
            payload_json.get("execution_error").and_then(|value| {
                let message = value.get("message").and_then(|item| item.as_str())?;
                let mut line = format!(" result=error={message}");
                if let Some(failed_at) = value.get("failed_at").and_then(|item| item.as_str()) {
                    line.push_str(&format!(" failed_at={failed_at}"));
                }
                Some(line)
            })
        })
        .or_else(|| {
            payload_json.get("cancellation").and_then(|value| {
                let reason = value.get("reason").and_then(|item| item.as_str())?;
                let mut line = format!(" result=reason={reason}");
                if let Some(canceled_at) = value.get("canceled_at").and_then(|item| item.as_str()) {
                    line.push_str(&format!(" canceled_at={canceled_at}"));
                }
                Some(line)
            })
        })
        .unwrap_or_default()
}
