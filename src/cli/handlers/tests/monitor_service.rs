use super::*;

#[test]
fn test_execute_monitor_service_config_show_returns_saved_binary_path() {
    let dir = tempdir().unwrap();
    let store = JsonMonitorServiceConfigStore::new(dir.path().join("service.json"));
    store
        .save(&MonitorServiceConfig {
            quantix_bin_path: "/bin/echo".into(),
        })
        .unwrap();

    let output = execute_monitor_service_config_command_with_store(
        MonitorServiceConfigCommands::Show,
        &store,
    )
    .unwrap();

    match output {
        MonitorCommandOutput::ServiceConfig(config) => {
            assert_eq!(
                config.quantix_bin_path,
                std::path::PathBuf::from("/bin/echo")
            );
        }
        other => panic!("unexpected output: {:?}", other),
    }
}

#[test]
fn test_execute_monitor_service_config_show_reports_not_configured_when_missing() {
    let dir = tempdir().unwrap();
    let store = JsonMonitorServiceConfigStore::new(dir.path().join("service.json"));

    let output = execute_monitor_service_config_command_with_store(
        MonitorServiceConfigCommands::Show,
        &store,
    )
    .unwrap();

    match output {
        MonitorCommandOutput::ServiceMessage(message) => {
            assert!(message.contains("未配置"));
        }
        other => panic!("unexpected output: {:?}", other),
    }
}

#[test]
fn test_execute_monitor_service_config_set_persists_binary_path() {
    let dir = tempdir().unwrap();
    let store = JsonMonitorServiceConfigStore::new(dir.path().join("service.json"));

    let output = execute_monitor_service_config_command_with_store(
        MonitorServiceConfigCommands::Set {
            quantix_bin: "/bin/echo".to_string(),
        },
        &store,
    )
    .unwrap();

    match output {
        MonitorCommandOutput::ServiceConfig(config) => {
            assert_eq!(
                config.quantix_bin_path,
                std::path::PathBuf::from("/bin/echo")
            );
        }
        other => panic!("unexpected output: {:?}", other),
    }

    let saved = store.load().unwrap();
    assert_eq!(
        saved.quantix_bin_path,
        std::path::PathBuf::from("/bin/echo")
    );
}

#[test]
fn test_execute_monitor_service_config_set_rejects_invalid_binary_path() {
    let dir = tempdir().unwrap();
    let store = JsonMonitorServiceConfigStore::new(dir.path().join("service.json"));

    let err = execute_monitor_service_config_command_with_store(
        MonitorServiceConfigCommands::Set {
            quantix_bin: "relative/path".to_string(),
        },
        &store,
    )
    .unwrap_err();

    assert!(err.to_string().contains("绝对"));
}

#[derive(Debug, Clone, Default)]
struct FakeMonitorServiceInstaller {
    status_summary: Option<MonitorServiceStatusSummary>,
    status_error: Option<String>,
    uninstall_error: Option<String>,
}

fn sample_service_status_summary() -> MonitorServiceStatusSummary {
    MonitorServiceStatusSummary {
        installed: true,
        enabled: false,
        active: "inactive".to_string(),
        unit_path: std::path::PathBuf::from("~/.config/systemd/user/quantix-monitor.service"),
        wrapper_path: std::path::PathBuf::from("~/.local/bin/quantix-monitor-run"),
        quantix_bin_path: std::path::PathBuf::from("/bin/echo"),
        raw_status: None,
    }
}

#[test]
fn test_execute_monitor_service_status_returns_summary() {
    let installer = FakeMonitorServiceInstaller {
        status_summary: Some(sample_service_status_summary()),
        ..Default::default()
    };

    let output =
        execute_monitor_service_command_with_installer(MonitorServiceCommands::Status, &installer)
            .unwrap();

    match output {
        MonitorCommandOutput::ServiceStatus(summary) => {
            assert!(summary.installed);
            assert_eq!(summary.active, "inactive");
        }
        other => panic!("unexpected output: {:?}", other),
    }
}

#[test]
fn test_execute_monitor_service_uninstall_surfaces_stop_first_error() {
    let installer = FakeMonitorServiceInstaller {
        uninstall_error: Some(
            "monitor service 仍在运行，请先执行 monitor service stop".to_string(),
        ),
        ..Default::default()
    };

    let err = execute_monitor_service_command_with_installer(
        MonitorServiceCommands::Uninstall,
        &installer,
    )
    .unwrap_err();

    assert!(err.to_string().contains("monitor service stop"));
}

#[test]
fn test_build_unconfigured_monitor_service_status_summary_marks_unconfigured() {
    let summary = build_unconfigured_monitor_service_status_summary();

    assert!(!summary.installed);
    assert!(!summary.enabled);
    assert_eq!(summary.active, "unconfigured");
    assert_eq!(
        summary.quantix_bin_path,
        std::path::PathBuf::from("<unconfigured>")
    );
}

impl MonitorServiceInstallerOps for FakeMonitorServiceInstaller {
    fn install(&self) -> Result<()> {
        Ok(())
    }

    fn uninstall(&self) -> Result<()> {
        match &self.uninstall_error {
            Some(message) => Err(QuantixError::Other(message.clone())),
            None => Ok(()),
        }
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
        match &self.status_error {
            Some(message) => Err(QuantixError::Other(message.clone())),
            None => Ok("status-text".to_string()),
        }
    }

    fn status_summary(&self) -> Result<MonitorServiceStatusSummary> {
        match (&self.status_summary, &self.status_error) {
            (_, Some(message)) => Err(QuantixError::Other(message.clone())),
            (Some(summary), None) => Ok(summary.clone()),
            (None, None) => Err(QuantixError::Other("missing status summary".to_string())),
        }
    }
}
