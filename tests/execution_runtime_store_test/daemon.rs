use super::helpers::*;

#[tokio::test]
async fn daemon_checkpoint_upsert_overwrites_existing_row_for_same_stream() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("runtime.db");
    let store = StrategyRuntimeStore::new(&path).await.unwrap();

    let first = sample_daemon_checkpoint("run-a");
    let mut second = sample_daemon_checkpoint("run-b");
    second.last_processed_bar = Some(fixed_ts() + chrono::Duration::days(1));
    second.updated_at = fixed_ts() + chrono::Duration::minutes(10);

    store.upsert_daemon_checkpoint(&first).await.unwrap();
    store.upsert_daemon_checkpoint(&second).await.unwrap();

    let saved = store
        .find_daemon_checkpoint("ma_fast_5_slow_20", "000001", "1d")
        .await
        .unwrap()
        .unwrap();

    assert_eq!(saved.last_run_id.as_deref(), Some("run-b"));
    assert_eq!(saved.last_processed_bar, second.last_processed_bar);
}
