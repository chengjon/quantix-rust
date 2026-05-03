use super::*;
use crate::execution::models::OrderRecord;
use crate::execution::request_diagnostics::{
    diagnostics_code, diagnostics_semantics, should_show_compact_diag,
};

pub(crate) async fn execute_strategy_signal_list(
    approval_status: Option<&str>,
    signal_status: Option<&str>,
) -> Result<()> {
    let runtime = CliRuntime::load();
    let runtime_store = StrategyRuntimeStore::new(runtime.strategy_runtime_db_path).await?;
    let rows =
        execute_strategy_signal_list_with_store(&runtime_store, approval_status, signal_status)
            .await?;

    for row in rows {
        println!("{}", format_strategy_signal_row(&row));
    }

    Ok(())
}

pub(crate) fn format_strategy_signal_row(row: &StrategySignalRecord) -> String {
    let source_id = row
        .metadata_json
        .get("bar_source_id")
        .and_then(|value| value.as_str())
        .unwrap_or("unknown");
    let fallback = row
        .metadata_json
        .get("bar_source_fallback")
        .and_then(|value| value.as_bool())
        .map(|value| value.to_string())
        .unwrap_or_else(|| "unknown".to_string());

    format!(
        "{} {} {} {} {} {} bar_end={} source={} fallback={}",
        row.signal_id,
        row.strategy_instance_id,
        row.symbol,
        row.signal_value,
        row.signal_status.as_str(),
        row.approval_status.as_str(),
        row.bar_end.format("%Y-%m-%dT%H:%M:%SZ"),
        source_id,
        fallback
    )
}

pub(crate) async fn execute_strategy_signal_list_with_store(
    store: &StrategyRuntimeStore,
    approval_status: Option<&str>,
    signal_status: Option<&str>,
) -> Result<Vec<StrategySignalRecord>> {
    let approval_filter = approval_status.map(parse_approval_status).transpose()?;
    let signal_filter = signal_status.map(parse_signal_status).transpose()?;

    let rows = store.list_signals().await?;
    Ok(rows
        .into_iter()
        .filter(|row| approval_filter.is_none_or(|status| row.approval_status == status))
        .filter(|row| signal_filter.is_none_or(|status| row.signal_status == status))
        .collect())
}

pub(crate) async fn execute_strategy_signal_approve(
    signal_id: &str,
    target_mode: &str,
    target_account: &str,
) -> Result<()> {
    let runtime = CliRuntime::load();
    let runtime_store = StrategyRuntimeStore::new(runtime.strategy_runtime_db_path).await?;
    let request = execute_strategy_signal_approve_with_store(
        &runtime_store,
        signal_id,
        target_mode,
        target_account,
    )
    .await?;
    println!("{}", format_strategy_approval_result(&request));
    Ok(())
}

pub(crate) async fn execute_strategy_signal_approve_with_store(
    store: &StrategyRuntimeStore,
    signal_id: &str,
    target_mode: &str,
    target_account: &str,
) -> Result<ExecutionRequestRecord> {
    match target_mode {
        "paper" | "mock_live" | "qmt_live" => {}
        "live" => {
            return Err(QuantixError::Unsupported(format!(
                "strategy signal approve 暂不支持 target_mode=live；如需真实 QMT 提交，请改用 target_mode=qmt_live，然后走 {QMT_LIVE_BRIDGE_COMMAND} 路径，并确保 {QMT_LIVE_BRIDGE_MODE_REQUIREMENT}"
            )));
        }
        other => {
            return Err(QuantixError::Unsupported(format!(
                "strategy signal approve 不支持 target_mode={other}"
            )));
        }
    }

    store
        .approve_signal_and_create_request(signal_id, target_mode, target_account, Some("cli"))
        .await
}

pub(crate) async fn execute_strategy_signal_reject(
    signal_id: &str,
    reason: Option<&str>,
) -> Result<()> {
    let runtime = CliRuntime::load();
    let runtime_store = StrategyRuntimeStore::new(runtime.strategy_runtime_db_path).await?;
    let signal =
        execute_strategy_signal_reject_with_store(&runtime_store, signal_id, reason).await?;
    println!("{}", format_strategy_rejection_result(&signal));
    Ok(())
}

pub(crate) async fn execute_strategy_signal_reject_with_store(
    store: &StrategyRuntimeStore,
    signal_id: &str,
    reason: Option<&str>,
) -> Result<StrategySignalRecord> {
    store.reject_signal(signal_id, reason).await?;
    store
        .get_signal(signal_id)
        .await?
        .ok_or_else(|| QuantixError::Other(format!("signal 不存在: {signal_id}")))
}

pub(crate) async fn execute_strategy_request_list(
    status: Option<&str>,
    target_mode: Option<&str>,
    target_account: Option<&str>,
    limit: usize,
    stats: bool,
) -> Result<()> {
    let runtime = CliRuntime::load();
    let runtime_store = StrategyRuntimeStore::new(runtime.strategy_runtime_db_path).await?;
    let rows = execute_strategy_request_list_with_store(&runtime_store, status).await?;

    let mut filtered: Vec<_> = rows
        .into_iter()
        .filter(|row| {
            let mode_match = target_mode.is_none_or(|m| row.target_mode == m);
            let account_match = target_account.is_none_or(|a| row.target_account == a);
            mode_match && account_match
        })
        .collect();

    if stats {
        let total = filtered.len();
        let pending = filtered
            .iter()
            .filter(|r| r.request_status == ExecutionRequestStatus::Pending)
            .count();
        let in_progress = filtered
            .iter()
            .filter(|r| r.request_status == ExecutionRequestStatus::InProgress)
            .count();
        let completed = filtered
            .iter()
            .filter(|r| r.request_status == ExecutionRequestStatus::Completed)
            .count();
        let failed = filtered
            .iter()
            .filter(|r| r.request_status == ExecutionRequestStatus::Failed)
            .count();
        let canceled = filtered
            .iter()
            .filter(|r| r.request_status == ExecutionRequestStatus::Canceled)
            .count();

        println!("=== Execution Request Statistics ===");
        println!(
            "Total: {} | Pending: {} | InProgress: {} | Completed: {} | Failed: {} | Canceled: {}",
            total, pending, in_progress, completed, failed, canceled
        );
        println!();
    }

    filtered.truncate(limit);

    for row in filtered {
        let related_order = find_related_order_for_request(&runtime_store, &row).await?;
        println!(
            "{}",
            format_strategy_request_row_with_related_order(&row, related_order.as_ref())
        );
    }

    Ok(())
}

pub(crate) async fn execute_strategy_request_show(request_id: &str, verbose: bool) -> Result<()> {
    let runtime = CliRuntime::load();
    let store = StrategyRuntimeStore::new(runtime.strategy_runtime_db_path).await?;

    let request = store
        .get_execution_request(request_id)
        .await?
        .ok_or_else(|| QuantixError::Other(format!("request 不存在: {request_id}")))?;

    let related_order = find_related_order_for_request(&store, &request).await?;
    println!(
        "{}",
        format_strategy_request_detail_with_related_order(&request, related_order.as_ref(), verbose)
    );

    if let Some(order) = related_order.as_ref() {
        println!();
        println!("=== Related Order ===");
        println!("order_id: {}", order.order_id);
        println!("symbol: {}", order.symbol);
        println!("status: {}", order.status.as_str());
        println!(
            "filled: {}/{}",
            order.filled_quantity, order.requested_quantity
        );
        if let Some(avg_price) = order.avg_fill_price {
            println!("avg_fill_price: {}", avg_price);
        }
    }

    Ok(())
}

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
        if let Some(order_terminality) = diagnostics.get("order_terminality").and_then(|v| v.as_str())
        {
            lines.push(format!("order_terminality: {}", order_terminality));
        }
        if let Some(summary) = diagnostics.get("summary").and_then(|v| v.as_str()) {
            lines.push(format!("summary: {}", summary));
        }
        if let Some(operator_action) = diagnostics.get("operator_action").and_then(|v| v.as_str())
        {
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

pub(crate) async fn execute_strategy_request_execute(request_id: &str) -> Result<()> {
    let runtime = CliRuntime::load();
    let runtime_store = StrategyRuntimeStore::new(runtime.strategy_runtime_db_path).await?;
    let request = execute_strategy_request_execute_with_components(
        &runtime_store,
        request_id,
        create_trade_store(),
        create_risk_store(),
    )
    .await?;
    println!("{}", format_strategy_request_row(&request));
    Ok(())
}

pub(crate) async fn execute_strategy_request_execute_with_components<TS>(
    store: &StrategyRuntimeStore,
    request_id: &str,
    trade_store: TS,
    risk_store: JsonRiskStore,
) -> Result<ExecutionRequestRecord>
where
    TS: PaperTradeStore + Clone,
{
    crate::execution::daemon::execute_request_by_id_with_components(
        store,
        request_id,
        trade_store,
        risk_store,
    )
    .await
}

pub(crate) async fn execute_strategy_request_cancel(
    request_id: &str,
    reason: Option<&str>,
) -> Result<()> {
    let runtime = CliRuntime::load();
    let runtime_store = StrategyRuntimeStore::new(runtime.strategy_runtime_db_path).await?;
    let request =
        execute_strategy_request_cancel_with_store(&runtime_store, request_id, reason).await?;
    println!("{}", format_strategy_request_row(&request));
    Ok(())
}

pub(crate) async fn execute_strategy_request_cancel_with_store(
    store: &StrategyRuntimeStore,
    request_id: &str,
    reason: Option<&str>,
) -> Result<ExecutionRequestRecord> {
    let request = store
        .get_execution_request(request_id)
        .await?
        .ok_or_else(|| QuantixError::Other(format!("request 不存在: {request_id}")))?;
    if request.request_status != ExecutionRequestStatus::Pending {
        return Err(QuantixError::Other(format!(
            "request 不是 pending: {request_id}"
        )));
    }

    let payload_json = merge_execution_request_payload(
        &request.payload_json,
        "cancellation",
        serde_json::json!({
            "canceled_at": Utc::now().to_rfc3339(),
            "reason": reason.unwrap_or("manual cancel"),
        }),
    );
    let updated = store
        .try_cancel_execution_request(&request.request_id, payload_json, Utc::now())
        .await?;
    if !updated {
        return Err(QuantixError::Other(format!(
            "request 状态已变化: {}",
            request.request_id
        )));
    }
    store
        .get_execution_request(&request.request_id)
        .await?
        .ok_or_else(|| QuantixError::Other(format!("request 不存在: {}", request.request_id)))
}

pub(crate) fn format_strategy_approval_result(request: &ExecutionRequestRecord) -> String {
    format!(
        "{} signal={} target={}/{} status={}",
        request.request_id,
        request.signal_id,
        request.target_mode,
        request.target_account,
        request.request_status.as_str()
    )
}

pub(crate) fn format_strategy_rejection_result(signal: &StrategySignalRecord) -> String {
    let reason = signal
        .metadata_json
        .get("rejection_reason")
        .and_then(|value| value.as_str())
        .unwrap_or("-");

    format!(
        "{} signal_status={} approval_status={} reason={}",
        signal.signal_id,
        signal.signal_status.as_str(),
        signal.approval_status.as_str(),
        reason
    )
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
    let qmt_suffix = related_order
        .and_then(qmt_live_compact_summary)
        .map(|summary| format!(" {summary}"))
        .unwrap_or_default();

    format!(
        "{} signal={} target={}/{} status={}{}{}{} created_at={}",
        row.request_id,
        row.signal_id,
        row.target_mode,
        row.target_account,
        row.request_status.as_str(),
        semantics,
        diag,
        format!("{result}{qmt_suffix}"),
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
    let result = format_execution_request_result(&request.payload_json);
    format!(
        "execution daemon consumed request={} status={}{}{}{}",
        request.request_id,
        request.request_status.as_str(),
        semantics,
        diag,
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

async fn find_related_order_for_request(
    store: &StrategyRuntimeStore,
    request: &ExecutionRequestRecord,
) -> Result<Option<OrderRecord>> {
    let Some(client_order_id) = request
        .payload_json
        .get("execution_result")
        .and_then(|value| value.get("client_order_id"))
        .and_then(|value| value.as_str())
    else {
        return Ok(None);
    };

    store.find_order_by_client_order_id(client_order_id).await
}

fn qmt_live_compact_summary(order: &OrderRecord) -> Option<String> {
    if order.adapter != "qmt_live" {
        return None;
    }

    let qmt_live = order.payload_json.get("qmt_live");
    let task_id = qmt_live
        .and_then(|value| value.get("task_identity"))
        .and_then(|value| value.get("task_id"))
        .and_then(|value| value.as_str())
        .filter(|value| !value.trim().is_empty());
    let latest_status = qmt_live
        .and_then(|value| value.get("last_query"))
        .and_then(|value| value.get("latest_status"))
        .and_then(|value| value.as_str());
    let last_action = qmt_live
        .and_then(|value| value.get("reconciliation"))
        .and_then(|value| value.get("last_action"))
        .and_then(|value| value.as_str());

    let mut parts = Vec::new();
    match task_id {
        Some(task_id) => parts.push(format!("qmt_task_id={task_id}")),
        None => parts.push("qmt_recovery=unavailable".to_string()),
    }
    if let Some(latest_status) = latest_status {
        parts.push(format!("qmt_latest_status={latest_status}"));
    }
    if let Some(last_action) = last_action {
        parts.push(format!("qmt_last_action={last_action}"));
    }

    Some(parts.join(" "))
}

fn append_qmt_live_recovery_detail(lines: &mut Vec<String>, related_order: Option<&OrderRecord>) {
    let Some(order) = related_order else {
        return;
    };
    if order.adapter != "qmt_live" {
        return;
    }

    let qmt_live = order.payload_json.get("qmt_live");
    let task_identity = qmt_live.and_then(|value| value.get("task_identity"));
    let task_id = task_identity
        .and_then(|value| value.get("task_id"))
        .and_then(|value| value.as_str())
        .filter(|value| !value.trim().is_empty());
    let last_query = qmt_live.and_then(|value| value.get("last_query"));
    let reconciliation = qmt_live.and_then(|value| value.get("reconciliation"));

    lines.push(String::new());
    lines.push("=== QMT Live Recovery ===".to_string());
    match task_id {
        Some(task_id) => lines.push(format!("task_id: {task_id}")),
        None => lines.push("automatic_reconciliation: unavailable".to_string()),
    }
    if let Some(client_order_id) = task_identity
        .and_then(|value| value.get("client_order_id"))
        .and_then(|value| value.as_str())
    {
        lines.push(format!("client_order_id: {client_order_id}"));
    }
    if let Some(local_submission_id) = task_identity
        .and_then(|value| value.get("local_submission_id"))
        .and_then(|value| value.as_str())
    {
        lines.push(format!("local_submission_id: {local_submission_id}"));
    }
    if let Some(external_order_id) = task_identity
        .and_then(|value| value.get("external_order_id"))
        .and_then(|value| value.as_str())
    {
        lines.push(format!("external_order_id: {external_order_id}"));
    }
    if let Some(last_query) = last_query {
        if let Some(latest_status) = last_query.get("latest_status").and_then(|value| value.as_str()) {
            lines.push(format!("latest_status: {latest_status}"));
        }
        if let Some(filled_quantity) = last_query
            .get("filled_quantity")
            .and_then(|value| value.as_i64())
        {
            lines.push(format!("filled_quantity: {filled_quantity}"));
        }
        if let Some(avg_fill_price) = last_query
            .get("avg_fill_price")
            .and_then(|value| value.as_str())
        {
            lines.push(format!("avg_fill_price: {avg_fill_price}"));
        }
        if let Some(broker_event_type) = last_query
            .get("broker_event_type")
            .and_then(|value| value.as_str())
        {
            lines.push(format!("broker_event_type: {broker_event_type}"));
        }
        if let Some(rejection_reason) = last_query
            .get("rejection_reason")
            .and_then(|value| value.as_str())
        {
            lines.push(format!("rejection_reason: {rejection_reason}"));
        }
        if let Some(updated_at) = last_query.get("updated_at").and_then(|value| value.as_str()) {
            lines.push(format!("last_query_updated_at: {updated_at}"));
        }
    }
    if let Some(reconciliation) = reconciliation {
        if let Some(last_action) = reconciliation
            .get("last_action")
            .and_then(|value| value.as_str())
        {
            lines.push(format!("last_action: {last_action}"));
        }
        if let Some(last_error) = reconciliation
            .get("last_error")
            .and_then(|value| value.as_str())
        {
            lines.push(format!("last_error: {last_error}"));
        }
        if let Some(last_attempt_at) = reconciliation
            .get("last_attempt_at")
            .and_then(|value| value.as_str())
        {
            lines.push(format!("last_attempt_at: {last_attempt_at}"));
        }
    }
}

pub(crate) async fn execute_strategy_request_list_with_store(
    store: &StrategyRuntimeStore,
    status: Option<&str>,
) -> Result<Vec<ExecutionRequestRecord>> {
    let status_filter = status.map(parse_execution_request_status).transpose()?;
    store.list_execution_requests(status_filter).await
}

pub(crate) fn parse_approval_status(value: &str) -> Result<ApprovalStatus> {
    ApprovalStatus::from_str(value)
        .ok_or_else(|| QuantixError::Other(format!("未知 approval_status: {value}")))
}

pub(crate) fn parse_signal_status(value: &str) -> Result<SignalStatus> {
    SignalStatus::from_str(value)
        .ok_or_else(|| QuantixError::Other(format!("未知 signal_status: {value}")))
}

pub(crate) fn parse_execution_request_status(value: &str) -> Result<ExecutionRequestStatus> {
    ExecutionRequestStatus::from_str(value)
        .ok_or_else(|| QuantixError::Other(format!("未知 request_status: {value}")))
}

pub(crate) fn merge_execution_request_payload(
    original: &serde_json::Value,
    key: &str,
    value: serde_json::Value,
) -> serde_json::Value {
    let mut payload = match original {
        serde_json::Value::Object(map) => serde_json::Value::Object(map.clone()),
        _ => serde_json::json!({}),
    };
    payload[key] = value;
    payload
}
