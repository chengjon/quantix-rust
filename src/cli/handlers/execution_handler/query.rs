//! Bridge QMT query/audit/manual intervention builders and executors.

use super::*;
pub(crate) fn normalize_symbol_for_bridge(symbol: &str) -> String {
    if symbol.contains('.') {
        return symbol.to_string();
    }
    if symbol.starts_with('6') {
        format!("{symbol}.SH")
    } else {
        format!("{symbol}.SZ")
    }
}

#[derive(Debug, Clone)]
pub(crate) struct QmtCancelCommandResult {
    pub requested_order_id: String,
    pub cancel_order_id: String,
    pub resolved_from_task_result: bool,
    pub response: crate::bridge::models::BridgeQmtCancelResponse,
}

pub(crate) fn create_qmt_task_submit_service(
    client: &BridgeHttpClient,
) -> Result<QmtTaskSubmitService> {
    QmtTaskSubmitService::new(client.clone(), 1, 30_000)
        .map_err(|err| QuantixError::Other(err.to_string()))
}

pub(crate) fn should_fallback_from_task_result_lookup(error: &BridgeError) -> bool {
    matches!(
        error,
        BridgeError::Http(_) | BridgeError::UnsupportedMethod(_)
    )
}

pub(crate) async fn build_execution_bridge_qmt_query_output(
    client: &BridgeHttpClient,
    order_id: &str,
) -> Result<serde_json::Value> {
    let submit_service = create_qmt_task_submit_service(client)?;

    match submit_service.query_task_result_by_task_id(order_id).await {
        Ok(result) => Ok(serde_json::json!({
            "query_mode": "task_result",
            "adapter_order_id": result.adapter_order_id,
            "latest_status": result.latest_status.as_str(),
            "filled_quantity": result.filled_quantity,
            "avg_fill_price": result.avg_fill_price.map(|value| value.to_string()),
            "rejection_reason": result.rejection_reason,
            "broker_event_type": result.broker_event_type.map(|value| format!("{value:?}")),
            "external_order_id": result.external_order_id,
            "client_order_id": result.client_order_id,
            "local_submission_id": result.local_submission_id,
            "source_name": result.source_name,
        })),
        Err(error) if should_fallback_from_task_result_lookup(&error) => {
            let response = client
                .qmt_query_order(order_id)
                .await
                .map_err(|err| QuantixError::Other(err.to_string()))?;
            Ok(serde_json::json!({
                "query_mode": "legacy_order",
                "adapter_order_id": response.adapter_order_id,
                "latest_status": response.latest_status,
                "filled_quantity": response.filled_quantity,
                "avg_fill_price": response.avg_fill_price,
            }))
        }
        Err(error) => Err(QuantixError::Other(error.to_string())),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum QmtAuditLookup {
    Request(String),
    Task(String),
    LocalSubmission(String),
}

impl QmtAuditLookup {
    pub(crate) fn from_cli(
        request_id: Option<String>,
        task_id: Option<String>,
        local_submission_id: Option<String>,
    ) -> Result<Self> {
        let request_id = normalize_qmt_audit_lookup_value(request_id);
        let task_id = normalize_qmt_audit_lookup_value(task_id);
        let local_submission_id = normalize_qmt_audit_lookup_value(local_submission_id);

        match (request_id, task_id, local_submission_id) {
            (Some(request_id), None, None) => Ok(Self::Request(request_id)),
            (None, Some(task_id), None) => Ok(Self::Task(task_id)),
            (None, None, Some(local_submission_id)) => Ok(Self::LocalSubmission(local_submission_id)),
            _ => Err(QuantixError::Other(
                "execution qmt audit requires exactly one of --request-id, --task-id, or --local-submission-id".to_string(),
            )),
        }
    }

    fn lookup_type(&self) -> &'static str {
        match self {
            Self::Request(_) => "request_id",
            Self::Task(_) => "task_id",
            Self::LocalSubmission(_) => "local_submission_id",
        }
    }

    fn value(&self) -> &str {
        match self {
            Self::Request(value) | Self::Task(value) | Self::LocalSubmission(value) => value,
        }
    }
}

fn normalize_qmt_audit_lookup_value(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim().to_string();
        (!trimmed.is_empty()).then_some(trimmed)
    })
}

fn value_as_audit_string(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(value) if !value.is_empty() => Some(value.clone()),
        serde_json::Value::Number(value) => Some(value.to_string()),
        serde_json::Value::Bool(value) => Some(value.to_string()),
        _ => None,
    }
}

fn string_path(value: &serde_json::Value, path: &[&str]) -> Option<String> {
    let mut current = value;
    for segment in path {
        current = current.get(*segment)?;
    }
    value_as_audit_string(current)
}

fn i64_path(value: &serde_json::Value, path: &[&str]) -> Option<i64> {
    let mut current = value;
    for segment in path {
        current = current.get(*segment)?;
    }
    current.as_i64()
}

fn qmt_order_identity_field<'a>(order: &'a OrderRecord, field: &str) -> Option<&'a str> {
    order
        .payload_json
        .get("qmt_live")?
        .get("task_identity")?
        .get(field)?
        .as_str()
        .filter(|value| !value.is_empty())
}

fn qmt_order_metadata_field(order: &OrderRecord, path: &[&str]) -> Option<String> {
    let qmt_live = order.payload_json.get("qmt_live")?;
    string_path(qmt_live, path)
}

fn redact_account_label(value: &str) -> String {
    let trimmed = value.trim();
    let char_count = trimmed.chars().count();
    let looks_raw_account =
        char_count >= 8 && trimmed.chars().all(|character| character.is_ascii_digit());

    if !looks_raw_account {
        return trimmed.to_string();
    }

    let tail_len = 4.min(char_count);
    let tail_start = char_count.saturating_sub(tail_len);
    let tail: String = trimmed.chars().skip(tail_start).collect();
    let mask_len = tail_start.min(14);
    format!("{}{}", "*".repeat(mask_len), tail)
}

async fn resolve_qmt_audit_record(
    runtime_store: &StrategyRuntimeStore,
    lookup: &QmtAuditLookup,
) -> Result<(ExecutionRequestRecord, Option<OrderRecord>)> {
    match lookup {
        QmtAuditLookup::Request(request_id) => {
            let request = runtime_store
                .get_execution_request(request_id)
                .await?
                .ok_or_else(|| QuantixError::Other(format!("request 不存在: {request_id}")))?;
            let order = runtime_store
                .find_order_by_client_order_id(&request.request_id)
                .await?;
            Ok((request, order))
        }
        QmtAuditLookup::Task(task_id) => {
            let orders = runtime_store.list_orders().await?;
            if let Some(order) = orders
                .into_iter()
                .find(|order| qmt_order_identity_field(order, "task_id") == Some(task_id.as_str()))
            {
                let request = runtime_store
                    .get_execution_request(&order.client_order_id)
                    .await?
                    .ok_or_else(|| {
                        QuantixError::Other(format!(
                            "qmt_live task_id={task_id} 对应 request 不存在: {}",
                            order.client_order_id
                        ))
                    })?;
                return Ok((request, Some(order)));
            }

            let requests = runtime_store.list_execution_requests(None).await?;
            let request = requests
                .into_iter()
                .find(|request| {
                    string_path(
                        &request.payload_json,
                        &["execution_result", "adapter_order_id"],
                    )
                    .as_deref()
                        == Some(task_id.as_str())
                })
                .ok_or_else(|| {
                    QuantixError::Other(format!("qmt_live task_id 不存在: {task_id}"))
                })?;
            let order = runtime_store
                .find_order_by_client_order_id(&request.request_id)
                .await?;
            Ok((request, order))
        }
        QmtAuditLookup::LocalSubmission(local_submission_id) => {
            let orders = runtime_store.list_orders().await?;
            let order = orders
                .into_iter()
                .find(|order| {
                    qmt_order_identity_field(order, "local_submission_id")
                        == Some(local_submission_id.as_str())
                })
                .ok_or_else(|| {
                    QuantixError::Other(format!(
                        "qmt_live local_submission_id 不存在: {local_submission_id}"
                    ))
                })?;
            let request = runtime_store
                .get_execution_request(&order.client_order_id)
                .await?
                .ok_or_else(|| {
                    QuantixError::Other(format!(
                        "qmt_live local_submission_id={local_submission_id} 对应 request 不存在: {}",
                        order.client_order_id
                    ))
                })?;
            Ok((request, Some(order)))
        }
    }
}

pub(crate) async fn build_execution_bridge_qmt_audit_output(
    runtime_store: &StrategyRuntimeStore,
    lookup: QmtAuditLookup,
) -> Result<serde_json::Value> {
    let (request, order) = resolve_qmt_audit_record(runtime_store, &lookup).await?;
    let order = order.as_ref();
    let request_payload = &request.payload_json;

    let symbol = order
        .map(|order| order.symbol.clone())
        .or_else(|| string_path(request_payload, &["execution_snapshot", "symbol"]));
    let side = order
        .map(|order| order.side.as_str().to_string())
        .or_else(|| {
            string_path(
                request_payload,
                &["execution_snapshot", "order_intent", "side"],
            )
        });
    let quantity = order.map(|order| order.requested_quantity).or_else(|| {
        i64_path(
            request_payload,
            &["execution_snapshot", "order_intent", "requested_quantity"],
        )
    });
    let order_type = order
        .map(|order| order.order_type.as_str().to_string())
        .or_else(|| {
            string_path(
                request_payload,
                &["execution_snapshot", "order_intent", "order_type"],
            )
        });
    let price_intent = order
        .map(|order| order.requested_price.to_string())
        .or_else(|| {
            string_path(
                request_payload,
                &["execution_snapshot", "order_intent", "requested_price"],
            )
        });

    let local_submission_id = order
        .and_then(|order| qmt_order_identity_field(order, "local_submission_id"))
        .map(str::to_string);
    let client_order_id = order
        .map(|order| order.client_order_id.clone())
        .or_else(|| string_path(request_payload, &["execution_result", "client_order_id"]));
    let task_id = order
        .and_then(|order| qmt_order_identity_field(order, "task_id"))
        .map(str::to_string)
        .or_else(|| string_path(request_payload, &["execution_result", "adapter_order_id"]));
    let external_order_id = order
        .and_then(|order| qmt_order_identity_field(order, "external_order_id"))
        .map(str::to_string)
        .or_else(|| string_path(request_payload, &["execution_result", "external_order_id"]));
    let bridge_contract_version = order
        .and_then(|order| qmt_order_metadata_field(order, &["bridge_contract_version"]))
        .or_else(|| {
            string_path(
                request_payload,
                &["execution_result", "bridge_contract_version"],
            )
        });
    let qmt_live_error_category = string_path(
        request_payload,
        &["execution_diagnostics", "qmt_live_failure_category"],
    );
    let reconciliation_decision =
        order.and_then(|order| qmt_order_metadata_field(order, &["reconciliation", "last_action"]));
    let manual_intervention_marker =
        reconciliation_decision.as_deref() == Some("manual_intervention");

    Ok(serde_json::json!({
        "lookup": {
            "type": lookup.lookup_type(),
            "value": lookup.value(),
        },
        "request": {
            "request_id": request.request_id.as_str(),
            "target_mode": request.target_mode.as_str(),
            "redacted_account_label": redact_account_label(&request.target_account),
            "target_account_raw": serde_json::Value::Null,
        },
        "order": {
            "symbol": symbol,
            "side": side,
            "quantity": quantity,
            "order_type": order_type,
            "price_intent": price_intent,
        },
        "qmt_live": {
            "local_submission_id": local_submission_id,
            "client_order_id": client_order_id,
            "task_id": task_id,
            "external_order_id": external_order_id,
            "bridge_contract_version": bridge_contract_version,
            "qmt_live_error_category": qmt_live_error_category,
            "reconciliation_decision": reconciliation_decision,
            "manual_intervention_marker": manual_intervention_marker,
        },
    }))
}

const QMT_MANUAL_INTERVENTION_OPERATOR_GUIDANCE: [&str; 3] = [
    "Inspect miniQMT same-day orders before taking action.",
    "Compare task ID, client order ID, local submission ID, and external order ID.",
    "avoid resubmission until the ambiguous state is resolved.",
];

fn qmt_request_result_field(request: &ExecutionRequestRecord, field: &str) -> Option<String> {
    string_path(&request.payload_json, &["execution_result", field])
}

fn qmt_request_failure_category(request: &ExecutionRequestRecord) -> Option<String> {
    string_path(
        &request.payload_json,
        &["execution_diagnostics", "qmt_live_failure_category"],
    )
}

fn qmt_identity_value(
    request: &ExecutionRequestRecord,
    order: Option<&OrderRecord>,
    identity_field: &str,
    request_result_field: &str,
) -> Option<String> {
    order
        .and_then(|order| qmt_order_identity_field(order, identity_field))
        .map(str::to_string)
        .or_else(|| qmt_request_result_field(request, request_result_field))
}

fn qmt_reconciliation_field(order: Option<&OrderRecord>, field: &str) -> Option<String> {
    order.and_then(|order| qmt_order_metadata_field(order, &["reconciliation", field]))
}

fn qmt_manual_intervention_category(
    request: &ExecutionRequestRecord,
    order: Option<&OrderRecord>,
) -> Option<&'static str> {
    if request.target_mode != "qmt_live"
        && order
            .map(|order| order.adapter.as_str() != "qmt_live")
            .unwrap_or(true)
    {
        return None;
    }

    let failure_category = qmt_request_failure_category(request);
    let reconciliation_action = qmt_reconciliation_field(order, "last_action");
    let reconciliation_error = qmt_reconciliation_field(order, "last_error");
    let error_lower = reconciliation_error
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let task_id = qmt_identity_value(request, order, "task_id", "adapter_order_id");
    let external_order_id =
        qmt_identity_value(request, order, "external_order_id", "external_order_id");

    if failure_category.as_deref() == Some("broker_unknown_state") {
        return Some("broker_unknown_state");
    }

    if failure_category.as_deref() == Some("bridge_failure")
        || error_lower.contains("bridge failure")
    {
        return Some("bridge_failure_requires_operator_review");
    }

    if reconciliation_action.as_deref() == Some("preserved_local_state")
        || error_lower.contains("preserved local")
    {
        return Some("reconciliation_preserved_local_state");
    }

    if task_id.is_some()
        && external_order_id.is_none()
        && (failure_category.as_deref() == Some("manual_intervention_required")
            || reconciliation_action.as_deref() == Some("manual_intervention")
            || error_lower.contains("external_order_id"))
    {
        return Some("missing_external_order_id_after_bridge_task_completion");
    }

    if failure_category.as_deref() == Some("manual_intervention_required")
        && error_lower.contains("identity")
    {
        return Some("identity_mismatch");
    }

    None
}

fn build_qmt_manual_intervention_case(
    request: &ExecutionRequestRecord,
    order: Option<&OrderRecord>,
) -> Option<serde_json::Value> {
    let category = qmt_manual_intervention_category(request, order)?;
    let request_payload = &request.payload_json;

    let symbol = order
        .map(|order| order.symbol.clone())
        .or_else(|| string_path(request_payload, &["execution_snapshot", "symbol"]));
    let side = order
        .map(|order| order.side.as_str().to_string())
        .or_else(|| {
            string_path(
                request_payload,
                &["execution_snapshot", "order_intent", "side"],
            )
        });
    let quantity = order.map(|order| order.requested_quantity).or_else(|| {
        i64_path(
            request_payload,
            &["execution_snapshot", "order_intent", "requested_quantity"],
        )
    });
    let client_order_id = order
        .map(|order| order.client_order_id.clone())
        .or_else(|| qmt_request_result_field(request, "client_order_id"));
    let task_id = qmt_identity_value(request, order, "task_id", "adapter_order_id");
    let local_submission_id =
        qmt_identity_value(request, order, "local_submission_id", "local_submission_id");
    let external_order_id =
        qmt_identity_value(request, order, "external_order_id", "external_order_id");
    let qmt_live_error_category = qmt_request_failure_category(request);
    let reconciliation_decision = qmt_reconciliation_field(order, "last_action");
    let reconciliation_error = qmt_reconciliation_field(order, "last_error");

    Some(serde_json::json!({
        "category": category,
        "status": "unresolved",
        "request_id": request.request_id.as_str(),
        "target_mode": request.target_mode.as_str(),
        "redacted_account_label": redact_account_label(&request.target_account),
        "target_account_raw": serde_json::Value::Null,
        "symbol": symbol,
        "side": side,
        "quantity": quantity,
        "task_id": task_id,
        "client_order_id": client_order_id,
        "local_submission_id": local_submission_id,
        "external_order_id": external_order_id,
        "qmt_live_error_category": qmt_live_error_category,
        "reconciliation_decision": reconciliation_decision,
        "reconciliation_error": reconciliation_error,
        "operator_guidance": QMT_MANUAL_INTERVENTION_OPERATOR_GUIDANCE,
    }))
}

pub(crate) async fn build_execution_bridge_qmt_manual_interventions_list_output(
    runtime_store: &StrategyRuntimeStore,
) -> Result<serde_json::Value> {
    let requests = runtime_store.list_execution_requests(None).await?;
    let orders = runtime_store.list_orders().await?;
    let orders_by_client_id: HashMap<&str, &OrderRecord> = orders
        .iter()
        .map(|order| (order.client_order_id.as_str(), order))
        .collect();

    let mut cases = requests
        .iter()
        .filter_map(|request| {
            build_qmt_manual_intervention_case(
                request,
                orders_by_client_id
                    .get(request.request_id.as_str())
                    .copied(),
            )
        })
        .collect::<Vec<_>>();

    cases.sort_by(|left, right| {
        let left_key = (
            left.get("category")
                .and_then(|value| value.as_str())
                .unwrap_or_default(),
            left.get("request_id")
                .and_then(|value| value.as_str())
                .unwrap_or_default(),
        );
        let right_key = (
            right
                .get("category")
                .and_then(|value| value.as_str())
                .unwrap_or_default(),
            right
                .get("request_id")
                .and_then(|value| value.as_str())
                .unwrap_or_default(),
        );
        left_key.cmp(&right_key)
    });

    Ok(serde_json::json!({
        "count": cases.len(),
        "mutates_runtime": false,
        "manual_interventions": cases,
    }))
}

fn qmt_manual_intervention_case_matches_lookup(
    case: &serde_json::Value,
    lookup: &QmtAuditLookup,
) -> bool {
    match lookup {
        QmtAuditLookup::Request(request_id) => {
            case.get("request_id").and_then(|value| value.as_str()) == Some(request_id.as_str())
        }
        QmtAuditLookup::Task(task_id) => {
            case.get("task_id").and_then(|value| value.as_str()) == Some(task_id.as_str())
        }
        QmtAuditLookup::LocalSubmission(local_submission_id) => {
            case.get("local_submission_id")
                .and_then(|value| value.as_str())
                == Some(local_submission_id.as_str())
        }
    }
}

pub(crate) async fn build_execution_bridge_qmt_manual_intervention_show_output(
    runtime_store: &StrategyRuntimeStore,
    lookup: QmtAuditLookup,
) -> Result<serde_json::Value> {
    let list_output =
        build_execution_bridge_qmt_manual_interventions_list_output(runtime_store).await?;
    let manual_intervention = list_output
        .get("manual_interventions")
        .and_then(|value| value.as_array())
        .and_then(|cases| {
            cases
                .iter()
                .find(|case| qmt_manual_intervention_case_matches_lookup(case, &lookup))
                .cloned()
        })
        .ok_or_else(|| {
            QuantixError::Other(format!(
                "qmt_live manual intervention 不存在: {}={}",
                lookup.lookup_type(),
                lookup.value()
            ))
        })?;
    let audit = build_execution_bridge_qmt_audit_output(runtime_store, lookup).await?;

    Ok(serde_json::json!({
        "mutates_runtime": false,
        "manual_intervention": manual_intervention,
        "audit": audit,
    }))
}

pub(crate) async fn execute_execution_bridge_qmt_audit(lookup: QmtAuditLookup) -> Result<()> {
    let runtime = CliRuntime::load();
    let runtime_store = StrategyRuntimeStore::new(runtime.strategy_runtime_db_path).await?;
    let output = build_execution_bridge_qmt_audit_output(&runtime_store, lookup).await?;

    println!("{}", serde_json::to_string_pretty(&output)?);

    Ok(())
}

pub(crate) async fn execute_execution_bridge_qmt_manual_interventions(
    action: &str,
    request_id: Option<String>,
    task_id: Option<String>,
    local_submission_id: Option<String>,
) -> Result<()> {
    let runtime = CliRuntime::load();
    let runtime_store = StrategyRuntimeStore::new(runtime.strategy_runtime_db_path).await?;
    let output = match action.trim() {
        "list" => {
            if request_id.is_some() || task_id.is_some() || local_submission_id.is_some() {
                return Err(QuantixError::Other(
                    "execution qmt manual-interventions list does not accept lookup flags"
                        .to_string(),
                ));
            }
            build_execution_bridge_qmt_manual_interventions_list_output(&runtime_store).await?
        }
        "show" => {
            let lookup = QmtAuditLookup::from_cli(request_id, task_id, local_submission_id)?;
            build_execution_bridge_qmt_manual_intervention_show_output(&runtime_store, lookup)
                .await?
        }
        other => {
            return Err(QuantixError::Other(format!(
                "unsupported qmt_live manual-interventions action: {other}; expected list or show"
            )));
        }
    };

    println!("{}", serde_json::to_string_pretty(&output)?);

    Ok(())
}

pub(crate) async fn execute_execution_bridge_qmt_query(order_id: &str) -> Result<()> {
    let client = create_bridge_client()?;
    let output = build_execution_bridge_qmt_query_output(&client, order_id).await?;

    println!("{}", serde_json::to_string_pretty(&output)?);

    Ok(())
}
