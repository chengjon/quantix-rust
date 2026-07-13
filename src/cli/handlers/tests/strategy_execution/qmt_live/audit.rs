use super::super::strategy_helpers::{fixed_ts, sample_run, sample_signal};
use super::*;
use rust_decimal_macros::dec;

#[tokio::test]
async fn test_build_execution_bridge_qmt_audit_output_includes_redacted_evidence() {
    let dir = tempdir().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let run = sample_run("000001", fixed_ts());
    runtime_store.insert_run(&run).await.unwrap();

    let signal = sample_signal(&run.run_id, "signal-qmt-live-audit", fixed_ts());
    runtime_store.insert_signal(&signal).await.unwrap();

    let request = runtime_store
        .approve_signal_and_create_request(
            "signal-qmt-live-audit",
            "qmt_live",
            "6222020202021234567",
            Some("cli"),
        )
        .await
        .unwrap();

    let request_payload = merge_execution_request_payload(
        &request.payload_json,
        "execution_result",
        serde_json::json!({
            "executed_at": fixed_ts().to_rfc3339(),
            "client_order_id": request.request_id.clone(),
            "order_status": "pending_submit",
            "adapter": "qmt_live",
            "adapter_order_id": "task-audit-1",
            "filled_quantity": 0,
            "avg_fill_price": null,
            "rejection_reason": null,
            "bridge_contract_version": "miniqmt.v1"
        }),
    );
    let request_payload = merge_execution_request_payload(
        &request_payload,
        "execution_diagnostics",
        serde_json::json!({
            "qmt_live_failure_category": "broker_unknown_state"
        }),
    );
    runtime_store
        .update_execution_request_status(
            &request.request_id,
            ExecutionRequestStatus::Completed,
            request_payload,
            fixed_ts(),
        )
        .await
        .unwrap();

    let order = OrderRecord {
        order_id: request.request_id.clone(),
        client_order_id: request.request_id.clone(),
        run_id: run.run_id,
        symbol: "000001".to_string(),
        side: OrderSide::Buy,
        order_type: OrderType::Limit,
        requested_quantity: 800,
        requested_price: dec!(12.34),
        filled_quantity: 0,
        remaining_quantity: 800,
        avg_fill_price: None,
        status: OrderStatus::PendingSubmit,
        adapter: "qmt_live".to_string(),
        created_at: fixed_ts(),
        updated_at: fixed_ts(),
        last_transition_at: fixed_ts(),
        version: 1,
        payload_json: serde_json::json!({
            "qmt_live": {
                "bridge_contract_version": "miniqmt.v1",
                "task_identity": {
                    "task_id": "task-audit-1",
                    "client_order_id": request.request_id.clone(),
                    "local_submission_id": "local-audit-1",
                    "external_order_id": "broker-audit-1"
                },
                "last_query": {
                    "latest_status": "accepted",
                    "filled_quantity": 0,
                    "avg_fill_price": null,
                    "broker_event_type": "acknowledgement",
                    "rejection_reason": null,
                    "updated_at": fixed_ts().to_rfc3339()
                },
                "reconciliation": {
                    "last_action": "manual_intervention",
                    "last_error": "broker state ambiguous",
                    "last_attempt_at": fixed_ts().to_rfc3339()
                }
            }
        }),
    };
    runtime_store.insert_order(&order).await.unwrap();

    let output = build_execution_bridge_qmt_audit_output(
        &runtime_store,
        QmtAuditLookup::Task("task-audit-1".to_string()),
    )
    .await
    .unwrap();

    assert_eq!(output["lookup"]["type"], "task_id");
    assert_eq!(output["lookup"]["value"], "task-audit-1");
    assert_eq!(output["request"]["request_id"], request.request_id);
    assert_eq!(output["request"]["target_mode"], "qmt_live");
    assert_eq!(
        output["request"]["redacted_account_label"],
        "**************4567"
    );
    assert_eq!(
        output["request"]["target_account_raw"],
        serde_json::Value::Null
    );
    assert_eq!(output["order"]["symbol"], "000001");
    assert_eq!(output["order"]["side"], "buy");
    assert_eq!(output["order"]["quantity"], 800);
    assert_eq!(output["order"]["order_type"], "limit");
    assert_eq!(output["order"]["price_intent"], "12.34");
    assert_eq!(output["qmt_live"]["local_submission_id"], "local-audit-1");
    assert_eq!(output["qmt_live"]["client_order_id"], request.request_id);
    assert_eq!(output["qmt_live"]["task_id"], "task-audit-1");
    assert_eq!(output["qmt_live"]["external_order_id"], "broker-audit-1");
    assert_eq!(output["qmt_live"]["bridge_contract_version"], "miniqmt.v1");
    assert_eq!(
        output["qmt_live"]["qmt_live_error_category"],
        "broker_unknown_state"
    );
    assert_eq!(
        output["qmt_live"]["reconciliation_decision"],
        "manual_intervention"
    );
    assert_eq!(output["qmt_live"]["manual_intervention_marker"], true);

    let local_submission_output = build_execution_bridge_qmt_audit_output(
        &runtime_store,
        QmtAuditLookup::LocalSubmission("local-audit-1".to_string()),
    )
    .await
    .unwrap();
    assert_eq!(
        local_submission_output["lookup"]["type"],
        "local_submission_id"
    );
    assert_eq!(
        local_submission_output["request"]["request_id"],
        request.request_id
    );
}

struct QmtManualCaseSpec<'a> {
    suffix: &'a str,
    failure_category: Option<&'a str>,
    external_order_id: Option<&'a str>,
    reconciliation_action: Option<&'a str>,
    reconciliation_error: Option<&'a str>,
}

async fn insert_qmt_manual_intervention_case(
    runtime_store: &StrategyRuntimeStore,
    run_id: &str,
    spec: QmtManualCaseSpec<'_>,
) -> String {
    let signal_id = format!("signal-qmt-manual-{}", spec.suffix);
    let bar_offset = spec.suffix.bytes().map(i64::from).sum::<i64>();
    let signal = sample_signal(
        run_id,
        &signal_id,
        fixed_ts() + chrono::Duration::seconds(bar_offset),
    );
    runtime_store.insert_signal(&signal).await.unwrap();

    let request = runtime_store
        .approve_signal_and_create_request(&signal_id, "qmt_live", "manual-account", Some("cli"))
        .await
        .unwrap();
    let task_id = format!("task-{}", spec.suffix);
    let local_submission_id = format!("local-{}", spec.suffix);

    let mut request_payload = merge_execution_request_payload(
        &request.payload_json,
        "execution_result",
        serde_json::json!({
            "executed_at": fixed_ts().to_rfc3339(),
            "client_order_id": request.request_id.clone(),
            "order_status": "pending_submit",
            "adapter": "qmt_live",
            "adapter_order_id": task_id,
            "filled_quantity": 0,
            "avg_fill_price": null,
            "rejection_reason": null,
            "bridge_contract_version": "miniqmt.v1"
        }),
    );
    if let Some(failure_category) = spec.failure_category {
        request_payload = merge_execution_request_payload(
            &request_payload,
            "execution_diagnostics",
            serde_json::json!({
                "qmt_live_failure_category": failure_category
            }),
        );
    }
    runtime_store
        .update_execution_request_status(
            &request.request_id,
            ExecutionRequestStatus::Completed,
            request_payload,
            fixed_ts(),
        )
        .await
        .unwrap();

    let external_order_id = spec
        .external_order_id
        .map(|value| serde_json::json!(value))
        .unwrap_or(serde_json::Value::Null);
    let reconciliation = match (spec.reconciliation_action, spec.reconciliation_error) {
        (Some(last_action), last_error) => serde_json::json!({
            "last_action": last_action,
            "last_error": last_error,
            "last_attempt_at": fixed_ts().to_rfc3339()
        }),
        _ => serde_json::Value::Null,
    };

    let order = OrderRecord {
        order_id: format!("order-{}", spec.suffix),
        client_order_id: request.request_id.clone(),
        run_id: run_id.to_string(),
        symbol: "000001".to_string(),
        side: OrderSide::Buy,
        order_type: OrderType::Limit,
        requested_quantity: 800,
        requested_price: dec!(12.34),
        filled_quantity: 0,
        remaining_quantity: 800,
        avg_fill_price: None,
        status: OrderStatus::PendingSubmit,
        adapter: "qmt_live".to_string(),
        created_at: fixed_ts(),
        updated_at: fixed_ts(),
        last_transition_at: fixed_ts(),
        version: 1,
        payload_json: serde_json::json!({
            "qmt_live": {
                "bridge_contract_version": "miniqmt.v1",
                "task_identity": {
                    "task_id": task_id,
                    "client_order_id": request.request_id.clone(),
                    "local_submission_id": local_submission_id,
                    "external_order_id": external_order_id
                },
                "last_query": {
                    "latest_status": "accepted",
                    "filled_quantity": 0,
                    "avg_fill_price": null,
                    "broker_event_type": "acknowledgement",
                    "rejection_reason": null,
                    "updated_at": fixed_ts().to_rfc3339()
                },
                "reconciliation": reconciliation
            }
        }),
    };
    runtime_store.insert_order(&order).await.unwrap();

    request.request_id
}

#[tokio::test]
async fn test_build_execution_bridge_qmt_manual_intervention_report_lists_and_shows_unresolved_cases()
 {
    let dir = tempdir().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let run = sample_run("000001", fixed_ts());
    runtime_store.insert_run(&run).await.unwrap();

    let identity_request_id = insert_qmt_manual_intervention_case(
        &runtime_store,
        &run.run_id,
        QmtManualCaseSpec {
            suffix: "identity-mismatch",
            failure_category: Some("manual_intervention_required"),
            external_order_id: Some("broker-identity"),
            reconciliation_action: Some("manual_intervention"),
            reconciliation_error: Some("task identity mismatch between local and broker payload"),
        },
    )
    .await;
    insert_qmt_manual_intervention_case(
        &runtime_store,
        &run.run_id,
        QmtManualCaseSpec {
            suffix: "broker-unknown",
            failure_category: Some("broker_unknown_state"),
            external_order_id: Some("broker-unknown"),
            reconciliation_action: Some("manual_intervention"),
            reconciliation_error: Some("broker state ambiguous"),
        },
    )
    .await;
    insert_qmt_manual_intervention_case(
        &runtime_store,
        &run.run_id,
        QmtManualCaseSpec {
            suffix: "missing-external",
            failure_category: Some("manual_intervention_required"),
            external_order_id: None,
            reconciliation_action: Some("manual_intervention"),
            reconciliation_error: Some("bridge task completed without external_order_id"),
        },
    )
    .await;
    insert_qmt_manual_intervention_case(
        &runtime_store,
        &run.run_id,
        QmtManualCaseSpec {
            suffix: "preserved-local",
            failure_category: None,
            external_order_id: Some("broker-preserved"),
            reconciliation_action: Some("preserved_local_state"),
            reconciliation_error: Some("reconciliation preserved local qmt_live state"),
        },
    )
    .await;
    insert_qmt_manual_intervention_case(
        &runtime_store,
        &run.run_id,
        QmtManualCaseSpec {
            suffix: "bridge-failure",
            failure_category: Some("bridge_failure"),
            external_order_id: Some("broker-bridge"),
            reconciliation_action: Some("manual_intervention"),
            reconciliation_error: Some("bridge failure requires operator review"),
        },
    )
    .await;

    let before_requests = runtime_store.list_execution_requests(None).await.unwrap();
    let before_orders = runtime_store.list_orders().await.unwrap();

    let output = build_execution_bridge_qmt_manual_interventions_list_output(&runtime_store)
        .await
        .unwrap();
    assert_eq!(output["count"], 5);
    assert_eq!(output["mutates_runtime"], false);
    let cases = output["manual_interventions"].as_array().unwrap();
    let mut categories = cases
        .iter()
        .map(|case| case["category"].as_str().unwrap())
        .collect::<Vec<_>>();
    categories.sort_unstable();
    assert_eq!(
        categories,
        vec![
            "bridge_failure_requires_operator_review",
            "broker_unknown_state",
            "identity_mismatch",
            "missing_external_order_id_after_bridge_task_completion",
            "reconciliation_preserved_local_state",
        ]
    );
    let guidance = cases[0]["operator_guidance"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|value| value.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(guidance.contains("miniQMT same-day orders"));
    assert!(guidance.contains("task ID"));
    assert!(guidance.contains("avoid resubmission"));
    assert!(cases.iter().all(|case| case["status"] == "unresolved"));
    assert!(
        cases
            .iter()
            .all(|case| case["target_account_raw"] == serde_json::Value::Null)
    );

    let show = build_execution_bridge_qmt_manual_intervention_show_output(
        &runtime_store,
        QmtAuditLookup::Task("task-identity-mismatch".to_string()),
    )
    .await
    .unwrap();
    assert_eq!(show["manual_intervention"]["category"], "identity_mismatch");
    assert_eq!(
        show["manual_intervention"]["request_id"],
        identity_request_id
    );
    assert_eq!(
        show["audit"]["qmt_live"]["task_id"],
        "task-identity-mismatch"
    );

    assert_eq!(
        runtime_store.list_execution_requests(None).await.unwrap(),
        before_requests
    );
    assert_eq!(runtime_store.list_orders().await.unwrap(), before_orders);
}
