use super::strategy_helpers::{FakeLoader, make_kline};
use super::*;
use rust_decimal_macros::dec;

fn test_execute_strategy_config_init_creates_default_file() {
    let dir = tempdir().unwrap();
    let store =
        crate::strategy::JsonStrategyConfigStore::new(dir.path().join("strategy-config.json"));

    let config = execute_strategy_config_init_to_store(&store).unwrap();

    assert_eq!(config.check_interval_secs, 60);
    assert!(dir.path().join("strategy-config.json").exists());
}

#[test]
fn test_execute_strategy_config_show_returns_saved_config() {
    let dir = tempdir().unwrap();
    let store =
        crate::strategy::JsonStrategyConfigStore::new(dir.path().join("strategy-config.json"));
    let expected = store.load_or_create().unwrap();

    let shown = execute_strategy_config_show_from_store(&store).unwrap();

    assert_eq!(shown, expected);
}

#[test]
fn test_execute_strategy_service_config_show_reports_not_configured_when_missing() {
    let dir = tempdir().unwrap();
    let store = crate::strategy::JsonStrategyServiceConfigStore::new(
        dir.path().join("strategy-service.json"),
    );

    let shown = execute_strategy_service_config_command_with_store(
        StrategyServiceConfigCommands::Show,
        &store,
    )
    .unwrap();

    assert!(shown.is_none());
}

#[test]
fn test_execute_strategy_service_config_set_persists_values() {
    let dir = tempdir().unwrap();
    let binary_path = dir.path().join("quantix");
    std::fs::write(&binary_path, "#!/bin/sh\nexit 0\n").unwrap();
    let mut perms = std::fs::metadata(&binary_path).unwrap().permissions();
    use std::os::unix::fs::PermissionsExt;
    perms.set_mode(0o755);
    std::fs::set_permissions(&binary_path, perms).unwrap();

    let store = crate::strategy::JsonStrategyServiceConfigStore::new(
        dir.path().join("strategy-service.json"),
    );

    let shown = execute_strategy_service_config_command_with_store(
        StrategyServiceConfigCommands::Set {
            quantix_bin: binary_path.display().to_string(),
            env_file: Some("/tmp/strategy.env".to_string()),
        },
        &store,
    )
    .unwrap()
    .unwrap();

    assert_eq!(shown.quantix_bin_path, binary_path);
    assert_eq!(
        shown.environment_file_path,
        Some(std::path::PathBuf::from("/tmp/strategy.env"))
    );

    let saved = store.load().unwrap();
    assert_eq!(saved.quantix_bin_path, binary_path);
    assert_eq!(
        saved.environment_file_path,
        Some(std::path::PathBuf::from("/tmp/strategy.env"))
    );
}

#[derive(Default)]
struct FakeStrategyServiceInstaller {
    status_output: Option<String>,
}

impl StrategyServiceInstallerOps for FakeStrategyServiceInstaller {
    fn install(&self) -> Result<()> {
        Ok(())
    }

    fn uninstall(&self) -> Result<()> {
        Ok(())
    }

    fn start(&self) -> Result<()> {
        Ok(())
    }

    fn stop(&self) -> Result<()> {
        Ok(())
    }

    fn enable(&self) -> Result<()> {
        Ok(())
    }

    fn disable(&self) -> Result<()> {
        Ok(())
    }

    fn status(&self) -> Result<String> {
        Ok(self
            .status_output
            .clone()
            .unwrap_or_else(|| "installed: yes".to_string()))
    }

    fn status_summary(&self) -> Result<StrategyServiceStatusSummary> {
        Ok(StrategyServiceStatusSummary {
            installed: true,
            enabled: false,
            active: "inactive".to_string(),
            unit_path: std::path::PathBuf::from("~/.config/systemd/user/quantix-strategy.service"),
            wrapper_path: std::path::PathBuf::from("~/.local/bin/quantix-strategy-run"),
            quantix_bin_path: std::path::PathBuf::from("/bin/echo"),
            environment_file_path: None,
            raw_status: None,
        })
    }
}

#[test]
fn test_execute_strategy_service_install_returns_message() {
    let message = execute_strategy_service_command_with_installer(
        StrategyServiceCommands::Install,
        &FakeStrategyServiceInstaller::default(),
    )
    .unwrap();

    assert_eq!(message, "strategy service installed");
}

#[test]
fn test_execute_strategy_service_status_returns_status_text() {
    let message = execute_strategy_service_command_with_installer(
        StrategyServiceCommands::Status,
        &FakeStrategyServiceInstaller {
            status_output: Some("installed: yes\nenabled: no".to_string()),
        },
    )
    .unwrap();

    assert!(message.contains("installed: yes"));
    assert!(message.contains("enabled: no"));
}

#[tokio::test]
async fn test_execute_strategy_daemon_once_bootstraps_and_then_emits_signal() {
    let dir = tempdir().unwrap();
    let config_store =
        crate::strategy::JsonStrategyConfigStore::new(dir.path().join("strategy-config.json"));
    config_store.load_or_create().unwrap();
    let runtime_store = StrategyRuntimeStore::new(dir.path().join("runtime.db"))
        .await
        .unwrap();
    let mut loader = FakeLoader::default();
    loader.data.insert(
        "000001".to_string(),
        vec![
            make_kline("000001", 1, dec!(10), 1000),
            make_kline("000001", 2, dec!(10), 1000),
            make_kline("000001", 3, dec!(10), 1000),
            make_kline("000001", 4, dec!(9), 1000),
            make_kline("000001", 5, dec!(9), 1000),
            make_kline("000001", 6, dec!(20), 1000),
        ],
    );

    let first = execute_strategy_daemon_run_once_with_components(
        loader.clone(),
        &config_store,
        &runtime_store,
    )
    .await
    .unwrap();
    assert!(first.is_none());
    assert_eq!(runtime_store.count_signals().await.unwrap(), 0);

    loader
        .data
        .get_mut("000001")
        .unwrap()
        .push(make_kline("000001", 7, dec!(21), 1000));

    let second =
        execute_strategy_daemon_run_once_with_components(loader, &config_store, &runtime_store)
            .await
            .unwrap();
    assert_eq!(
        second.map(|signal| signal.metadata_json["bar_source_id"].clone()),
        Some(json!("test-primary"))
    );
    assert_eq!(runtime_store.count_signals().await.unwrap(), 1);
}
