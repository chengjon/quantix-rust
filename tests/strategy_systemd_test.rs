use std::path::PathBuf;

use quantix_cli::strategy::{JsonStrategyServiceConfigStore, StrategyServiceConfig};
use tempfile::tempdir;

#[test]
fn strategy_service_config_store_save_and_load_preserves_optional_env_file() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("strategy").join("service.json");
    let binary_path = dir.path().join("bin").join("quantix");
    std::fs::create_dir_all(binary_path.parent().unwrap()).unwrap();
    std::fs::write(&binary_path, "#!/bin/sh\nexit 0\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&binary_path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&binary_path, perms).unwrap();
    }

    let store = JsonStrategyServiceConfigStore::new(&path);
    let config = StrategyServiceConfig {
        quantix_bin_path: binary_path.clone(),
        environment_file_path: Some(PathBuf::from("/tmp/quantix-strategy.env")),
    };

    store.save(&config).unwrap();
    let loaded = store.load().unwrap();

    assert_eq!(loaded, config);
}

#[test]
fn strategy_service_config_validate_rejects_non_absolute_binary_path() {
    let config = StrategyServiceConfig {
        quantix_bin_path: PathBuf::from("target/debug/quantix"),
        environment_file_path: None,
    };

    let error = JsonStrategyServiceConfigStore::validate(&config).unwrap_err();
    assert!(error.to_string().contains("必须是绝对路径"));
}
