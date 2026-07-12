use super::helpers::*;

#[tokio::test]
async fn qmt_live_runtime_metadata_update_preserves_unrelated_payload_keys() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("runtime.db");
    let store = StrategyRuntimeStore::new(&path).await.unwrap();
    let run = sample_run("000001", fixed_ts());
    store.insert_run(&run).await.unwrap();

    let mut order = sample_order(&run.run_id, "run_qmt_live_metadata");
    order.adapter = "qmt_live".to_string();
    order.payload_json = json!({
        "reason": "ma_cross_buy",
        "nested": { "keep": true }
    });
    store.insert_order(&order).await.unwrap();

    let metadata = QmtLiveRuntimeMetadata {
        task_identity: Some(QmtLiveTaskIdentity {
            task_id: "task-1".to_string(),
            client_order_id: "run_qmt_live_metadata".to_string(),
            local_submission_id: "local-1".to_string(),
            external_order_id: None,
        }),
        last_query: None,
        reconciliation: None,
    };

    let updated = store
        .try_update_order_qmt_live_metadata(
            &order,
            &metadata,
            fixed_ts() + chrono::Duration::minutes(1),
        )
        .await
        .unwrap();

    assert!(updated);

    let saved = store
        .find_order_by_client_order_id("run_qmt_live_metadata")
        .await
        .unwrap()
        .unwrap();

    assert_eq!(saved.payload_json["reason"], "ma_cross_buy");
    assert_eq!(saved.payload_json["nested"]["keep"], true);
    assert_eq!(
        saved.payload_json["qmt_live"]["task_identity"]["task_id"],
        "task-1"
    );
}

#[tokio::test]
async fn qmt_live_runtime_metadata_update_preserves_existing_external_order_id_when_new_metadata_has_none()
 {
    let dir = tempdir().unwrap();
    let path = dir.path().join("runtime.db");
    let store = StrategyRuntimeStore::new(&path).await.unwrap();
    let run = sample_run("000001", fixed_ts());
    store.insert_run(&run).await.unwrap();

    let mut order = sample_order(&run.run_id, "run_qmt_live_identity_preserve");
    order.adapter = "qmt_live".to_string();
    order.payload_json = json!({
        "qmt_live": {
            "task_identity": {
                "task_id": "task-1",
                "client_order_id": "run_qmt_live_identity_preserve",
                "local_submission_id": "local-1",
                "external_order_id": "broker-ack-1"
            }
        }
    });
    store.insert_order(&order).await.unwrap();

    let metadata = QmtLiveRuntimeMetadata {
        task_identity: Some(QmtLiveTaskIdentity {
            task_id: "task-1".to_string(),
            client_order_id: "run_qmt_live_identity_preserve".to_string(),
            local_submission_id: "local-1".to_string(),
            external_order_id: None,
        }),
        last_query: None,
        reconciliation: None,
    };

    let updated = store
        .try_update_order_qmt_live_metadata(
            &order,
            &metadata,
            fixed_ts() + chrono::Duration::minutes(1),
        )
        .await
        .unwrap();

    assert!(updated);

    let saved = store
        .find_order_by_client_order_id("run_qmt_live_identity_preserve")
        .await
        .unwrap()
        .unwrap();

    assert_eq!(
        saved.payload_json["qmt_live"]["task_identity"]["external_order_id"],
        "broker-ack-1"
    );
}

#[test]
fn qmt_live_task_identity_deserializes_missing_fields_as_compatibility_defaults() {
    let identity: QmtLiveTaskIdentity = serde_json::from_value(serde_json::json!({
        "task_id": "task-compat",
        "client_order_id": "client-compat"
    }))
    .unwrap();

    assert_eq!(identity.task_id, "task-compat");
    assert_eq!(identity.client_order_id, "client-compat");
    assert_eq!(identity.local_submission_id, "");
    assert_eq!(identity.external_order_id, None);
}
