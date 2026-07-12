//! QMT manual-intervention list/show output builders and executors.

use crate::cli::handlers::execution_handler::create_bridge_client;
use crate::core::{CliRuntime, QuantixError, Result};
use crate::execution::models::OrderRecord;
use crate::execution::runtime_store::StrategyRuntimeStore;
use std::collections::HashMap;

use crate::execution::models::ExecutionRequestRecord;

use super::audit::{
    QMT_MANUAL_INTERVENTION_OPERATOR_GUIDANCE, QmtAuditLookup,
    build_execution_bridge_qmt_audit_output, i64_path, qmt_identity_value,
    qmt_manual_intervention_category, qmt_reconciliation_field, qmt_request_failure_category,
    qmt_request_result_field, redact_account_label, string_path,
};
use super::output::build_execution_bridge_qmt_query_output;

pub(crate) fn build_qmt_manual_intervention_case(
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

pub(crate) fn qmt_manual_intervention_case_matches_lookup(
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
