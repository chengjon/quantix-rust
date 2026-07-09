use super::*;

pub(crate) fn execute_monitor_service_command(
    cmd: MonitorServiceCommands,
) -> Result<MonitorCommandOutput> {
    let runtime = CliRuntime::load();
    let store = JsonMonitorServiceConfigStore::with_default_path()?;
    let service_config = match store.load() {
        Ok(config) => config,
        Err(QuantixError::Config(_)) if matches!(cmd, MonitorServiceCommands::Status) => {
            return Ok(MonitorCommandOutput::ServiceStatus(
                build_unconfigured_monitor_service_status_summary(),
            ));
        }
        Err(other) => return Err(other),
    };
    let installer = MonitorUserServiceInstaller::new(runtime, service_config);
    execute_monitor_service_command_with_installer(cmd, &installer)
}

pub(crate) fn execute_monitor_service_command_with_installer<I>(
    cmd: MonitorServiceCommands,
    installer: &I,
) -> Result<MonitorCommandOutput>
where
    I: MonitorServiceInstallerOps,
{
    match cmd {
        MonitorServiceCommands::Install => {
            installer.install()?;
            Ok(MonitorCommandOutput::ServiceMessage(
                "monitor service installed".to_string(),
            ))
        }
        MonitorServiceCommands::Uninstall => {
            installer.uninstall()?;
            Ok(MonitorCommandOutput::ServiceMessage(
                "monitor service uninstalled".to_string(),
            ))
        }
        MonitorServiceCommands::Start => {
            installer.start()?;
            Ok(MonitorCommandOutput::ServiceMessage(
                "monitor service started".to_string(),
            ))
        }
        MonitorServiceCommands::Stop => {
            installer.stop()?;
            Ok(MonitorCommandOutput::ServiceMessage(
                "monitor service stopped".to_string(),
            ))
        }
        MonitorServiceCommands::Status => Ok(MonitorCommandOutput::ServiceStatus(
            installer.status_summary()?,
        )),
        MonitorServiceCommands::Enable => {
            installer.enable()?;
            Ok(MonitorCommandOutput::ServiceMessage(
                "monitor service enabled".to_string(),
            ))
        }
        MonitorServiceCommands::Disable => {
            installer.disable()?;
            Ok(MonitorCommandOutput::ServiceMessage(
                "monitor service disabled".to_string(),
            ))
        }
    }
}
