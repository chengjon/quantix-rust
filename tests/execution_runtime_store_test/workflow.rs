use super::helpers::*;

#[tokio::test]
async fn superseding_signal_cancels_pending_execution_request() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("runtime.db");
    let store = StrategyRuntimeStore::new(&path).await.unwrap();
    let run = sample_run("000001", fixed_ts());
    store.insert_run(&run).await.unwrap();

    let old_signal = sample_signal(&run.run_id, "signal-old", fixed_ts());
    store.insert_signal(&old_signal).await.unwrap();
    store
        .approve_signal_and_create_request("signal-old", "paper", "default", Some("cli-user"))
        .await
        .unwrap();

    let new_signal = sample_signal(
        &run.run_id,
        "signal-new",
        fixed_ts() + chrono::Duration::days(1),
    );
    store.insert_signal(&new_signal).await.unwrap();

    let superseded = store
        .supersede_previous_signals_and_cancel_pending_requests(
            "ma_fast_5_slow_20",
            "000001",
            "1d",
            "signal-new",
            fixed_ts() + chrono::Duration::days(1),
        )
        .await
        .unwrap();

    assert_eq!(superseded, 1);

    let saved_old_signal = store.get_signal("signal-old").await.unwrap().unwrap();
    assert_eq!(saved_old_signal.signal_status, SignalStatus::Superseded);

    let request = store
        .get_execution_request_by_signal_id("signal-old")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(request.request_status, ExecutionRequestStatus::Canceled);
}

#[tokio::test]
async fn superseding_signal_does_not_cancel_in_progress_execution_request() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("runtime.db");
    let store = StrategyRuntimeStore::new(&path).await.unwrap();
    let run = sample_run("000001", fixed_ts());
    store.insert_run(&run).await.unwrap();

    let old_signal = sample_signal(&run.run_id, "signal-old-in-progress", fixed_ts());
    store.insert_signal(&old_signal).await.unwrap();
    let request = store
        .approve_signal_and_create_request(
            "signal-old-in-progress",
            "paper",
            "default",
            Some("cli-user"),
        )
        .await
        .unwrap();
    let claimed = store
        .try_start_execution_request(
            &request.request_id,
            json!({
                "executor": {
                    "type": "daemon"
                }
            }),
            fixed_ts(),
        )
        .await
        .unwrap();
    assert!(claimed);

    let new_signal = sample_signal(
        &run.run_id,
        "signal-newer",
        fixed_ts() + chrono::Duration::days(1),
    );
    store.insert_signal(&new_signal).await.unwrap();

    let superseded = store
        .supersede_previous_signals_and_cancel_pending_requests(
            "ma_fast_5_slow_20",
            "000001",
            "1d",
            "signal-newer",
            fixed_ts() + chrono::Duration::days(1),
        )
        .await
        .unwrap();

    assert_eq!(superseded, 1);

    let saved_request = store
        .get_execution_request(&request.request_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        saved_request.request_status,
        ExecutionRequestStatus::InProgress
    );
}

#[tokio::test]
async fn execution_request_can_transition_from_pending_to_completed_once() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("runtime.db");
    let store = StrategyRuntimeStore::new(&path).await.unwrap();
    let run = sample_run("000001", fixed_ts());
    store.insert_run(&run).await.unwrap();
    let signal = sample_signal(&run.run_id, "signal-request-complete", fixed_ts());
    store.insert_signal(&signal).await.unwrap();

    let request = store
        .approve_signal_and_create_request(
            "signal-request-complete",
            "mock_live",
            "default",
            Some("cli-user"),
        )
        .await
        .unwrap();
    let claimed = store
        .try_start_execution_request(&request.request_id, json!({}), fixed_ts())
        .await
        .unwrap();
    assert!(claimed);

    let completed = store
        .try_complete_execution_request(
            &request.request_id,
            json!({
                "execution_result": {
                    "order_status": "accepted",
                    "client_order_id": "req-order-1",
                }
            }),
            fixed_ts(),
        )
        .await
        .unwrap();
    assert!(completed);

    let repeated = store
        .try_complete_execution_request(&request.request_id, json!({}), fixed_ts())
        .await
        .unwrap();
    assert!(!repeated);

    let saved = store
        .get_execution_request(&request.request_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved.request_status, ExecutionRequestStatus::Completed);
    assert_eq!(
        saved.payload_json["execution_result"]["order_status"],
        "accepted"
    );
}

#[tokio::test]
async fn execution_request_can_transition_from_pending_to_failed_or_canceled() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("runtime.db");
    let store = StrategyRuntimeStore::new(&path).await.unwrap();
    let run = sample_run("000001", fixed_ts());
    store.insert_run(&run).await.unwrap();

    let failed_signal = sample_signal(&run.run_id, "signal-request-failed", fixed_ts());
    store.insert_signal(&failed_signal).await.unwrap();
    let failed_request = store
        .approve_signal_and_create_request(
            "signal-request-failed",
            "paper",
            "default",
            Some("cli-user"),
        )
        .await
        .unwrap();
    let failed_claimed = store
        .try_start_execution_request(&failed_request.request_id, json!({}), fixed_ts())
        .await
        .unwrap();
    assert!(failed_claimed);
    let failed = store
        .try_fail_execution_request(
            &failed_request.request_id,
            json!({
                "execution_error": {
                    "message": "quote lookup failed"
                }
            }),
            fixed_ts(),
        )
        .await
        .unwrap();
    assert!(failed);

    let canceled_signal = sample_signal(
        &run.run_id,
        "signal-request-canceled",
        fixed_ts() + chrono::Duration::days(1),
    );
    store.insert_signal(&canceled_signal).await.unwrap();
    let canceled_request = store
        .approve_signal_and_create_request(
            "signal-request-canceled",
            "paper",
            "default",
            Some("cli-user"),
        )
        .await
        .unwrap();
    let canceled = store
        .try_cancel_execution_request(
            &canceled_request.request_id,
            json!({
                "cancellation": {
                    "reason": "manual cancel"
                }
            }),
            fixed_ts(),
        )
        .await
        .unwrap();
    assert!(canceled);

    let failed_saved = store
        .get_execution_request(&failed_request.request_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(failed_saved.request_status, ExecutionRequestStatus::Failed);
    assert_eq!(
        failed_saved.payload_json["execution_error"]["message"],
        "quote lookup failed"
    );

    let canceled_saved = store
        .get_execution_request(&canceled_request.request_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        canceled_saved.request_status,
        ExecutionRequestStatus::Canceled
    );
    assert_eq!(
        canceled_saved.payload_json["cancellation"]["reason"],
        "manual cancel"
    );
}

#[tokio::test]
async fn execution_request_can_transition_from_pending_to_in_progress_once() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("runtime.db");
    let store = StrategyRuntimeStore::new(&path).await.unwrap();
    let run = sample_run("000001", fixed_ts());
    store.insert_run(&run).await.unwrap();
    let signal = sample_signal(&run.run_id, "signal-request-start", fixed_ts());
    store.insert_signal(&signal).await.unwrap();

    let request = store
        .approve_signal_and_create_request(
            "signal-request-start",
            "paper",
            "default",
            Some("cli-user"),
        )
        .await
        .unwrap();

    let started = store
        .try_start_execution_request(
            &request.request_id,
            json!({
                "executor": {
                    "type": "daemon",
                    "started_at": fixed_ts().to_rfc3339()
                }
            }),
            fixed_ts(),
        )
        .await
        .unwrap();
    assert!(started);

    let repeated = store
        .try_start_execution_request(&request.request_id, json!({}), fixed_ts())
        .await
        .unwrap();
    assert!(!repeated);

    let saved = store
        .get_execution_request(&request.request_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved.request_status, ExecutionRequestStatus::InProgress);
}
