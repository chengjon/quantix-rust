use super::helpers::*;

#[tokio::test]
async fn try_update_order_with_version_updates_and_increments_version() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("runtime.db");
    let store = StrategyRuntimeStore::new(&path).await.unwrap();
    let run = sample_run("000001", fixed_ts());
    store.insert_run(&run).await.unwrap();

    let order = sample_order(&run.run_id, "run_000001_versioned");
    store.insert_order(&order).await.unwrap();

    let updated = store
        .try_update_order_with_version(
            &order.order_id,
            0,
            OrderStatus::Accepted,
            20,
            80,
            Some(dec!(12.50)),
            fixed_ts() + chrono::Duration::minutes(1),
        )
        .await
        .unwrap();

    assert!(updated);

    let saved = store
        .find_order_by_client_order_id("run_000001_versioned")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved.status, OrderStatus::Accepted);
    assert_eq!(saved.filled_quantity, 20);
    assert_eq!(saved.remaining_quantity, 80);
    assert_eq!(saved.version, 1);
}

#[tokio::test]
async fn try_update_order_with_stale_version_is_rejected() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("runtime.db");
    let store = StrategyRuntimeStore::new(&path).await.unwrap();
    let run = sample_run("000001", fixed_ts());
    store.insert_run(&run).await.unwrap();

    let order = sample_order(&run.run_id, "run_000001_stale");
    store.insert_order(&order).await.unwrap();

    let updated = store
        .try_update_order_with_version(
            &order.order_id,
            9,
            OrderStatus::Accepted,
            20,
            80,
            Some(dec!(12.50)),
            fixed_ts() + chrono::Duration::minutes(1),
        )
        .await
        .unwrap();

    assert!(!updated);

    let saved = store
        .find_order_by_client_order_id("run_000001_stale")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved.status, OrderStatus::PendingSubmit);
    assert_eq!(saved.version, 0);
}
