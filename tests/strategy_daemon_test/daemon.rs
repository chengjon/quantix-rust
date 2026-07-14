use super::*;
use quantix_cli::execution::models::{ApprovalStatus, ExecutionRequestStatus};
use quantix_cli::strategy::StrategySignalDaemon;

#[tokio::test]
async fn daemon_bootstraps_without_backfilling_historical_signals() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("strategy").join("config.json");
    let runtime_db_path = dir.path().join("strategy").join("runtime.db");
    let config_store = JsonStrategyConfigStore::new(&config_path);
    config_store.load_or_create().unwrap();

    let store = StrategyRuntimeStore::new(&runtime_db_path).await.unwrap();
    let loader = FakeBarLoader::default();
    loader.set_bars(
        "000001",
        vec![
            kline(1, 10),
            kline(2, 10),
            kline(3, 10),
            kline(4, 9),
            kline(5, 9),
            kline(6, 20),
        ],
    );

    let mut daemon =
        StrategySignalDaemon::new(loader.clone(), store.clone(), config_store).unwrap();
    daemon.run_once().await.unwrap();

    assert_eq!(store.count_runs().await.unwrap(), 0);
    assert_eq!(store.count_signals().await.unwrap(), 0);

    let checkpoint = store
        .find_daemon_checkpoint("ma_fast_5_slow_20", "000001", "1d")
        .await
        .unwrap()
        .unwrap();
    assert!(checkpoint.last_processed_bar.is_some());
}

#[tokio::test]
async fn daemon_skips_when_no_new_bar_exists() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("strategy").join("config.json");
    let runtime_db_path = dir.path().join("strategy").join("runtime.db");
    let config_store = JsonStrategyConfigStore::new(&config_path);
    config_store.load_or_create().unwrap();

    let store = StrategyRuntimeStore::new(&runtime_db_path).await.unwrap();
    let loader = FakeBarLoader::default();
    let bars = vec![
        kline(1, 10),
        kline(2, 10),
        kline(3, 10),
        kline(4, 9),
        kline(5, 9),
        kline(6, 20),
    ];
    loader.set_bars("000001", bars.clone());

    let mut daemon =
        StrategySignalDaemon::new(loader.clone(), store.clone(), config_store).unwrap();
    daemon.run_once().await.unwrap();
    daemon.run_once().await.unwrap();

    assert_eq!(store.count_runs().await.unwrap(), 0);
    assert_eq!(store.count_signals().await.unwrap(), 0);
}

#[tokio::test]
async fn daemon_writes_run_signal_and_checkpoint_when_new_bar_arrives() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("strategy").join("config.json");
    let runtime_db_path = dir.path().join("strategy").join("runtime.db");
    let config_store = JsonStrategyConfigStore::new(&config_path);
    config_store.load_or_create().unwrap();

    let store = StrategyRuntimeStore::new(&runtime_db_path).await.unwrap();
    let loader = FakeBarLoader::default();
    loader.set_bars(
        "000001",
        vec![
            kline(1, 10),
            kline(2, 10),
            kline(3, 10),
            kline(4, 9),
            kline(5, 9),
            kline(6, 20),
        ],
    );

    let mut daemon =
        StrategySignalDaemon::new(loader.clone(), store.clone(), config_store).unwrap();
    daemon.run_once().await.unwrap();

    loader.set_bars(
        "000001",
        vec![
            kline(1, 10),
            kline(2, 10),
            kline(3, 10),
            kline(4, 9),
            kline(5, 9),
            kline(6, 20),
            kline(7, 21),
        ],
    );

    daemon.run_once().await.unwrap();

    assert_eq!(store.count_runs().await.unwrap(), 1);
    assert_eq!(store.count_signals().await.unwrap(), 1);

    let signal = store
        .list_signals()
        .await
        .unwrap()
        .into_iter()
        .find(|row| row.strategy_instance_id == "ma_fast_5_slow_20")
        .unwrap();
    assert_eq!(signal.metadata_json["bar_source_id"], "test-primary");
    assert_eq!(signal.metadata_json["bar_source_fallback"], false);
    assert_eq!(signal.metadata_json["market_price"], "21");
    assert_eq!(signal.metadata_json["signal_value"], signal.signal_value);
    assert_eq!(
        signal.metadata_json["execution_policy"]["fixed_cash_per_buy"],
        "10000"
    );
    assert_eq!(signal.metadata_json["execution_policy"]["slippage_bps"], 0);

    let checkpoint = store
        .find_daemon_checkpoint("ma_fast_5_slow_20", "000001", "1d")
        .await
        .unwrap()
        .unwrap();
    assert!(checkpoint.last_run_id.is_some());
}

#[tokio::test]
async fn daemon_hot_reloads_config_and_bootstraps_new_strategy_instance() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("strategy").join("config.json");
    let runtime_db_path = dir.path().join("strategy").join("runtime.db");
    let config_store = JsonStrategyConfigStore::new(&config_path);
    let mut config = config_store.load_or_create().unwrap();

    let store = StrategyRuntimeStore::new(&runtime_db_path).await.unwrap();
    let loader = FakeBarLoader::default();
    loader.set_bars(
        "000001",
        vec![
            kline(1, 10),
            kline(2, 10),
            kline(3, 10),
            kline(4, 9),
            kline(5, 9),
            kline(6, 20),
        ],
    );

    let mut daemon =
        StrategySignalDaemon::new(loader.clone(), store.clone(), config_store.clone()).unwrap();
    daemon.run_once().await.unwrap();

    config.stocks[0]
        .strategies
        .push(quantix_cli::strategy::ConfiguredStrategyInstance {
            id: "ma_fast_2_slow_3".to_string(),
            name: "ma_cross".to_string(),
            enabled: true,
            params: serde_json::json!({"fast": 2, "slow": 3}),
        });
    config_store.save(&config).unwrap();

    daemon.run_once().await.unwrap();

    assert!(
        store
            .find_daemon_checkpoint("ma_fast_2_slow_3", "000001", "1d")
            .await
            .unwrap()
            .is_some()
    );
}

#[tokio::test]
async fn auto_approval_manual_leaves_signal_pending() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("strategy").join("config.json");
    let runtime_db_path = dir.path().join("strategy").join("runtime.db");
    let execution_config_path = dir.path().join("execution").join("config.json");
    let config_store = JsonStrategyConfigStore::new(&config_path);
    config_store.load_or_create().unwrap();
    let execution_config_store = JsonExecutionConfigStore::new(&execution_config_path);
    execution_config_store.load_or_create().unwrap();

    let store = StrategyRuntimeStore::new(&runtime_db_path).await.unwrap();
    let loader = FakeBarLoader::default();
    loader.set_bars(
        "000001",
        vec![
            kline(1, 10),
            kline(2, 10),
            kline(3, 10),
            kline(4, 9),
            kline(5, 9),
            kline(6, 20),
            kline(7, 21),
        ],
    );

    let mut daemon = StrategySignalDaemon::with_execution_config_store(
        loader.clone(),
        store.clone(),
        config_store,
        execution_config_store,
    )
    .unwrap();
    daemon.run_once().await.unwrap();
    loader.set_bars(
        "000001",
        vec![
            kline(1, 10),
            kline(2, 10),
            kline(3, 10),
            kline(4, 9),
            kline(5, 9),
            kline(6, 20),
            kline(7, 21),
            kline(8, 22),
        ],
    );
    daemon.run_once().await.unwrap();

    let signal = store
        .list_signals()
        .await
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    assert_eq!(signal.approval_status, ApprovalStatus::Pending);
    assert!(
        store
            .list_execution_requests(None)
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn auto_approval_always_creates_pending_request() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("strategy").join("config.json");
    let runtime_db_path = dir.path().join("strategy").join("runtime.db");
    let execution_config_path = dir.path().join("execution").join("config.json");
    let config_store = JsonStrategyConfigStore::new(&config_path);
    config_store.load_or_create().unwrap();
    let execution_config_store = JsonExecutionConfigStore::new(&execution_config_path);
    let mut execution_config = execution_config_store.load_or_create().unwrap();
    execution_config.auto_approval.mode = AutoApprovalMode::Always;
    execution_config_store.save(&execution_config).unwrap();

    let store = StrategyRuntimeStore::new(&runtime_db_path).await.unwrap();
    let loader = FakeBarLoader::default();
    loader.set_bars(
        "000001",
        vec![
            kline(1, 10),
            kline(2, 10),
            kline(3, 10),
            kline(4, 9),
            kline(5, 9),
            kline(6, 20),
            kline(7, 21),
        ],
    );

    let mut daemon = StrategySignalDaemon::with_execution_config_store(
        loader.clone(),
        store.clone(),
        config_store,
        execution_config_store,
    )
    .unwrap();
    daemon.run_once().await.unwrap();
    loader.set_bars(
        "000001",
        vec![
            kline(1, 10),
            kline(2, 10),
            kline(3, 10),
            kline(4, 9),
            kline(5, 9),
            kline(6, 20),
            kline(7, 21),
            kline(8, 22),
        ],
    );
    daemon.run_once().await.unwrap();

    let signal = store
        .list_signals()
        .await
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    assert_eq!(signal.approval_status, ApprovalStatus::Approved);

    let requests = store.list_execution_requests(None).await.unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].request_status, ExecutionRequestStatus::Pending);
    assert_eq!(requests[0].target_mode, "paper");
    assert_eq!(requests[0].target_account, "default");
}
