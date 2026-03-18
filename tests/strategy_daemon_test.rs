use chrono::NaiveDate;
use quantix_cli::data::models::{AdjustType, Kline};
use quantix_cli::execution::runtime_store::StrategyRuntimeStore;
use quantix_cli::strategy::daemon::StrategyBarLoadTelemetry;
use quantix_cli::strategy::runtime::StrategyBarLoader;
use quantix_cli::strategy::trait_def::Signal;
use quantix_cli::strategy::{
    BootstrapPolicy, FallbackStrategyBarLoader, JsonStrategyConfigStore, StrategyBarLoadSource,
    StrategyRegistry, StrategySignalDaemon,
};
use rust_decimal_macros::dec;
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};
use tempfile::tempdir;

#[test]
fn strategy_config_store_load_or_create_persists_default_latest_only_config() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("strategy").join("config.json");

    let store = JsonStrategyConfigStore::new(&path);
    let config = store.load_or_create().unwrap();

    assert_eq!(config.check_interval_secs, 60);
    assert_eq!(config.bootstrap_policy, BootstrapPolicy::LatestOnly);
    assert_eq!(config.stocks.len(), 1);
    assert_eq!(config.stocks[0].code, "000001");
    assert!(config.stocks[0].enabled);
    assert_eq!(config.stocks[0].strategies.len(), 1);
    assert_eq!(config.stocks[0].strategies[0].id, "ma_fast_5_slow_20");
    assert_eq!(config.stocks[0].strategies[0].name, "ma_cross");
    assert!(config.stocks[0].strategies[0].enabled);
    assert_eq!(config.stocks[0].strategies[0].params["fast"], 5);
    assert_eq!(config.stocks[0].strategies[0].params["slow"], 20);

    let saved = std::fs::read_to_string(&path).unwrap();
    assert!(saved.contains("\"bootstrap_policy\": \"latest_only\""));
}

fn kline(day: u32, close: i64) -> Kline {
    Kline {
        code: "000001".to_string(),
        date: NaiveDate::from_ymd_opt(2026, 3, day).unwrap(),
        open: dec!(1),
        high: dec!(1),
        low: dec!(1),
        close: rust_decimal::Decimal::from(close),
        volume: 100,
        amount: None,
        adjust_type: AdjustType::None,
    }
}

#[derive(Clone, Default)]
struct FakeBarLoader {
    bars: Arc<Mutex<HashMap<String, Vec<Kline>>>>,
}

#[async_trait::async_trait]
impl StrategyBarLoader for FakeBarLoader {
    async fn load_daily_bars(
        &self,
        code: &str,
        limit: usize,
    ) -> quantix_cli::core::Result<Vec<Kline>> {
        let bars = self
            .bars
            .lock()
            .unwrap()
            .get(code)
            .cloned()
            .unwrap_or_default();
        if bars.len() > limit {
            Ok(bars[bars.len() - limit..].to_vec())
        } else {
            Ok(bars)
        }
    }
}

impl FakeBarLoader {
    fn set_bars(&self, code: &str, bars: Vec<Kline>) {
        self.bars.lock().unwrap().insert(code.to_string(), bars);
    }
}

impl StrategyBarLoadTelemetry for FakeBarLoader {
    fn last_source(&self) -> Option<StrategyBarLoadSource> {
        Some(StrategyBarLoadSource {
            source_id: "test-primary".to_string(),
            fallback_used: false,
        })
    }
}

#[derive(Clone)]
struct ErrorBarLoader;

#[async_trait::async_trait]
impl StrategyBarLoader for ErrorBarLoader {
    async fn load_daily_bars(
        &self,
        _code: &str,
        _limit: usize,
    ) -> quantix_cli::core::Result<Vec<Kline>> {
        Err(quantix_cli::core::QuantixError::DataSource(
            "primary loader failed".to_string(),
        ))
    }
}

fn write_day_file(root: &std::path::Path, market: &str, code: &str, closes: &[u32]) {
    let dir = root.join("vipdoc").join(market).join("lday");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("{}{}.day", market, code));
    let mut bytes = Vec::new();

    for (index, close) in closes.iter().enumerate() {
        let date = 20260301u32 + index as u32;
        bytes.extend_from_slice(&date.to_le_bytes());
        bytes.extend_from_slice(&close.to_le_bytes());
        bytes.extend_from_slice(&close.to_le_bytes());
        bytes.extend_from_slice(&close.to_le_bytes());
        bytes.extend_from_slice(&close.to_le_bytes());
        bytes.extend_from_slice(&(1000.0f32).to_le_bytes());
        bytes.extend_from_slice(&(100u32).to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes());
    }

    fs::write(path, bytes).unwrap();
}

#[test]
fn strategy_registry_resolves_multiple_ma_cross_instances_with_different_params() {
    let config = JsonStrategyConfigStore::new("/tmp/unused")
        .load_or_create()
        .unwrap();
    let registry = StrategyRegistry::new();

    let fast = registry.build(&config.stocks[0].strategies[0]).unwrap();
    let slow = registry
        .build(&quantix_cli::strategy::ConfiguredStrategyInstance {
            id: "ma_fast_2_slow_3".to_string(),
            name: "ma_cross".to_string(),
            enabled: true,
            params: serde_json::json!({"fast": 2, "slow": 3}),
        })
        .unwrap();

    assert_eq!(fast.lookback_required(), 20);
    assert_eq!(slow.lookback_required(), 3);
}

#[test]
fn strategy_registry_evaluator_returns_latest_signal_envelope() {
    let registry = StrategyRegistry::new();
    let evaluator = registry
        .build(&quantix_cli::strategy::ConfiguredStrategyInstance {
            id: "ma_fast_2_slow_3".to_string(),
            name: "ma_cross".to_string(),
            enabled: true,
            params: serde_json::json!({"fast": 2, "slow": 3}),
        })
        .unwrap();

    let bars = vec![
        kline(1, 10),
        kline(2, 10),
        kline(3, 10),
        kline(4, 9),
        kline(5, 9),
        kline(6, 20),
    ];

    let envelope = evaluator.evaluate(&bars).unwrap();
    assert_eq!(envelope.signal, Signal::Buy);
}

#[test]
fn strategy_registry_rejects_unknown_strategy_names() {
    let registry = StrategyRegistry::new();
    let result = registry.build(&quantix_cli::strategy::ConfiguredStrategyInstance {
        id: "unknown-1".to_string(),
        name: "unknown_strategy".to_string(),
        enabled: true,
        params: serde_json::json!({}),
    });

    let error = match result {
        Ok(_) => panic!("expected unknown strategy to fail"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("unknown_strategy"));
}

#[tokio::test]
async fn fallback_loader_prefers_primary_rows_when_available() {
    let root = tempdir().unwrap();
    write_day_file(root.path(), "sh", "000001", &[800, 900, 1000]);

    let primary = FakeBarLoader::default();
    primary.set_bars("000001", vec![kline(1, 10), kline(2, 11)]);

    let loader = FallbackStrategyBarLoader::new(primary, Some(root.path().to_path_buf()));
    let rows = loader.load_daily_bars("000001", 10).await.unwrap();

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].close, dec!(10));
    assert_eq!(rows[1].close, dec!(11));
    assert_eq!(
        loader.last_source(),
        Some(StrategyBarLoadSource {
            source_id: "primary".to_string(),
            fallback_used: false,
        })
    );
}

#[tokio::test]
async fn fallback_loader_uses_tdx_when_primary_returns_empty() {
    let root = tempdir().unwrap();
    write_day_file(root.path(), "sh", "000001", &[1200, 1300, 1400]);

    let loader =
        FallbackStrategyBarLoader::new(FakeBarLoader::default(), Some(root.path().to_path_buf()));
    let rows = loader.load_daily_bars("000001", 10).await.unwrap();

    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].close, dec!(12));
    assert_eq!(rows[2].close, dec!(14));
    assert_eq!(
        loader.last_source(),
        Some(StrategyBarLoadSource {
            source_id: "tdx-day-file".to_string(),
            fallback_used: true,
        })
    );
}

#[tokio::test]
async fn fallback_loader_uses_preferred_market_to_resolve_ambiguous_code() {
    let root = tempdir().unwrap();
    write_day_file(root.path(), "sh", "000001", &[800, 900]);
    write_day_file(root.path(), "sz", "000001", &[1200, 1300]);

    let loader = FallbackStrategyBarLoader::with_options(
        FakeBarLoader::default(),
        "primary",
        Some(root.path().to_path_buf()),
        Some("sz".to_string()),
    );
    let rows = loader.load_daily_bars("000001", 10).await.unwrap();

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].close, dec!(12));
    assert_eq!(rows[1].close, dec!(13));
}

#[tokio::test]
async fn fallback_loader_uses_tdx_when_primary_errors() {
    let root = tempdir().unwrap();
    write_day_file(root.path(), "sh", "000001", &[1500, 1600]);

    let loader = FallbackStrategyBarLoader::new(ErrorBarLoader, Some(root.path().to_path_buf()));
    let rows = loader.load_daily_bars("000001", 10).await.unwrap();

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].close, dec!(15));
    assert_eq!(rows[1].close, dec!(16));
}

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
