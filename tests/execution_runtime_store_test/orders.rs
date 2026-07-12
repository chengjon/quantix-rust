use super::helpers::*;

#[tokio::test]
async fn checkpoint_upsert_overwrites_existing_row_for_same_stream() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("runtime.db");
    let store = StrategyRuntimeStore::new(&path).await.unwrap();

    let first = RunnerCheckpointRecord {
        checkpoint_id: Uuid::new_v4().to_string(),
        strategy_name: "ma_cross".to_string(),
        mode: "paper".to_string(),
        symbol: "000001".to_string(),
        timeframe: "1d".to_string(),
        last_processed_bar: Some(fixed_ts()),
        last_run_id: Some("run-a".to_string()),
        state_json: json!({"step": 1}),
        updated_at: fixed_ts(),
    };
    let second = RunnerCheckpointRecord {
        checkpoint_id: Uuid::new_v4().to_string(),
        strategy_name: "ma_cross".to_string(),
        mode: "paper".to_string(),
        symbol: "000001".to_string(),
        timeframe: "1d".to_string(),
        last_processed_bar: Some(fixed_ts() + chrono::Duration::days(1)),
        last_run_id: Some("run-b".to_string()),
        state_json: json!({"step": 2}),
        updated_at: fixed_ts() + chrono::Duration::minutes(5),
    };

    store.upsert_checkpoint(&first).await.unwrap();
    store.upsert_checkpoint(&second).await.unwrap();

    let saved = store
        .load_checkpoint("ma_cross", "paper", "000001", "1d")
        .await
        .unwrap()
        .unwrap();

    assert_eq!(saved.last_run_id.as_deref(), Some("run-b"));
    assert_eq!(saved.state_json, json!({"step": 2}));
}

#[tokio::test]
async fn order_events_round_trip_against_existing_order() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("runtime.db");
    let store = StrategyRuntimeStore::new(&path).await.unwrap();
    let run = sample_run("000001", fixed_ts());
    store.insert_run(&run).await.unwrap();

    let order = sample_order(&run.run_id, "run_000001_1");
    store.insert_order(&order).await.unwrap();

    let event = OrderEventRecord {
        event_id: Uuid::new_v4().to_string(),
        order_id: order.order_id.clone(),
        client_order_id: order.client_order_id.clone(),
        event_type: "submitted".to_string(),
        event_time: fixed_ts(),
        details_json: json!({"status": "submitted"}),
    };

    store.insert_order_event(&event).await.unwrap();

    let events = store.list_order_events(&order.order_id).await.unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, "submitted");
}

#[tokio::test]
async fn insert_order_round_trips_extended_lifecycle_fields() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("runtime.db");
    let store = StrategyRuntimeStore::new(&path).await.unwrap();
    let run = sample_run("000001", fixed_ts());
    store.insert_run(&run).await.unwrap();

    let order = sample_order(&run.run_id, "run_000001_extended");
    store.insert_order(&order).await.unwrap();

    let saved = store
        .find_order_by_client_order_id("run_000001_extended")
        .await
        .unwrap()
        .unwrap();

    assert_eq!(saved.remaining_quantity, 100);
    assert_eq!(saved.last_transition_at, fixed_ts());
    assert_eq!(saved.version, 0);
}

#[tokio::test]
async fn mock_live_order_state_round_trips_through_store() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("runtime.db");
    let store = StrategyRuntimeStore::new(&path).await.unwrap();
    let run = sample_run("000001", fixed_ts());
    store.insert_run(&run).await.unwrap();

    let order = sample_order(&run.run_id, "run_000001_mock_live");
    store.insert_order(&order).await.unwrap();

    let state = MockLiveOrderState {
        cancel_requested: true,
        unknown_retries: 2,
        ..Default::default()
    };

    store
        .insert_mock_live_order_state(&order.order_id, Some("adapter-1"), &state)
        .await
        .unwrap();

    let saved = store
        .get_mock_live_order_state(&order.order_id)
        .await
        .unwrap()
        .unwrap();

    assert!(saved.cancel_requested);
    assert_eq!(saved.unknown_retries, 2);
}
