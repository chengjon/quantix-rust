use quantix_cli::strategy::{
    BootstrapPolicy, JsonStrategyConfigStore, JsonStrategyServiceConfigStore,
    StrategyServiceConfig,
};
use std::path::PathBuf;
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

#[test]
fn strategy_service_config_validate_requires_absolute_existing_executable() {
    let dir = tempdir().unwrap();
    let bin_path = dir.path().join("quantix");
    std::fs::write(&bin_path, "#!/bin/sh\nexit 0\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&bin_path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&bin_path, perms).unwrap();
    }

    let config = StrategyServiceConfig {
        quantix_bin_path: bin_path.clone(),
        environment_file_path: None,
    };

    JsonStrategyServiceConfigStore::validate(&config).unwrap();

    let relative = StrategyServiceConfig {
        quantix_bin_path: PathBuf::from("quantix"),
        environment_file_path: None,
    };
    let err = JsonStrategyServiceConfigStore::validate(&relative).unwrap_err();
    assert!(err.to_string().contains("绝对路径"));
}
