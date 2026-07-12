use super::helpers::*;

#[tokio::test]
async fn bootstrap_creates_phase29a_schema() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("runtime.db");

    let store = StrategyRuntimeStore::new(&path).await.unwrap();

    assert!(store.has_table("strategy_runs").await.unwrap());
    assert!(store.has_table("signal_events").await.unwrap());
    assert!(store.has_table("orders").await.unwrap());
    assert!(store.has_table("order_events").await.unwrap());
    assert!(store.has_table("runner_checkpoints").await.unwrap());
    assert!(store.has_table("signals").await.unwrap());
    assert!(store.has_table("execution_requests").await.unwrap());
    assert!(
        store
            .has_table("strategy_daemon_checkpoints")
            .await
            .unwrap()
    );
    assert!(store.has_table("mock_live_orders").await.unwrap());
}

#[tokio::test]
async fn insert_run_rejects_duplicate_dedupe_key() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("runtime.db");
    let store = StrategyRuntimeStore::new(&path).await.unwrap();
    let bar_end = fixed_ts();

    let first = sample_run("000001", bar_end);
    let second = sample_run("000001", bar_end);

    store.insert_run(&first).await.unwrap();
    let err = store.insert_run(&second).await.unwrap_err();

    assert!(err.to_string().contains("strategy_runs"));
}

#[tokio::test]
async fn insert_order_rejects_duplicate_client_order_id() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("runtime.db");
    let store = StrategyRuntimeStore::new(&path).await.unwrap();
    let run = sample_run("000001", fixed_ts());
    store.insert_run(&run).await.unwrap();

    let first = sample_order(&run.run_id, "run_000001_1");
    let second = sample_order(&run.run_id, "run_000001_1");

    store.insert_order(&first).await.unwrap();
    let err = store.insert_order(&second).await.unwrap_err();

    assert!(err.to_string().contains("client_order_id"));
}
