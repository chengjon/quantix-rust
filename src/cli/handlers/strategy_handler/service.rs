use super::*;

pub(crate) fn execute_strategy_service_config_command_with_store(
    cmd: StrategyServiceConfigCommands,
    store: &JsonStrategyServiceConfigStore,
) -> Result<Option<StrategyServiceConfig>> {
    match cmd {
        StrategyServiceConfigCommands::Show => match store.load() {
            Ok(config) => Ok(Some(config)),
            Err(QuantixError::Config(_)) => Ok(None),
            Err(other) => Err(other),
        },
        StrategyServiceConfigCommands::Set {
            quantix_bin,
            env_file,
        } => {
            let config = StrategyServiceConfig {
                quantix_bin_path: quantix_bin.into(),
                environment_file_path: env_file.map(Into::into),
            };
            JsonStrategyServiceConfigStore::validate(&config)?;
            store.save(&config)?;
            Ok(Some(config))
        }
    }
}

pub(crate) fn print_strategy_service_config_output(
    config: Option<StrategyServiceConfig>,
) -> Result<()> {
    match config {
        Some(config) => {
            println!("{}", serde_json::to_string_pretty(&config)?);
        }
        None => {
            println!(
                "strategy service 未配置，请先运行 strategy service-config set --quantix-bin /abs/path/to/quantix"
            );
        }
    }

    Ok(())
}

pub(crate) trait StrategyServiceInstallerOps {
    fn install(&self) -> Result<()>;
    fn uninstall(&self) -> Result<()>;
    fn start(&self) -> Result<()>;
    fn stop(&self) -> Result<()>;
    fn enable(&self) -> Result<()>;
    fn disable(&self) -> Result<()>;
    fn status(&self) -> Result<String>;
    #[allow(dead_code)]
    fn status_summary(&self) -> Result<StrategyServiceStatusSummary>;
}

impl StrategyServiceInstallerOps for StrategyUserServiceInstaller {
    fn install(&self) -> Result<()> {
        StrategyUserServiceInstaller::install(self)
    }

    fn uninstall(&self) -> Result<()> {
        StrategyUserServiceInstaller::uninstall(self)
    }

    fn start(&self) -> Result<()> {
        StrategyUserServiceInstaller::start(self)
    }

    fn stop(&self) -> Result<()> {
        StrategyUserServiceInstaller::stop(self)
    }

    fn enable(&self) -> Result<()> {
        StrategyUserServiceInstaller::enable(self)
    }

    fn disable(&self) -> Result<()> {
        StrategyUserServiceInstaller::disable(self)
    }

    fn status(&self) -> Result<String> {
        StrategyUserServiceInstaller::status(self)
    }

    fn status_summary(&self) -> Result<StrategyServiceStatusSummary> {
        StrategyUserServiceInstaller::status_summary(self)
    }
}

pub(crate) fn execute_strategy_service_command(cmd: StrategyServiceCommands) -> Result<()> {
    let runtime = CliRuntime::load();
    let store = JsonStrategyServiceConfigStore::with_default_path()?;
    let service_config = match store.load() {
        Ok(config) => config,
        Err(QuantixError::Config(_)) => {
            return Err(QuantixError::Other(
                "strategy service 未配置，请先运行 strategy service-config set --quantix-bin /abs/path/to/quantix".to_string(),
            ));
        }
        Err(other) => return Err(other),
    };
    let installer = StrategyUserServiceInstaller::new(runtime, service_config);
    let message = execute_strategy_service_command_with_installer(cmd, &installer)?;
    println!("{}", message);
    Ok(())
}

pub(crate) fn execute_strategy_service_command_with_installer<I>(
    cmd: StrategyServiceCommands,
    installer: &I,
) -> Result<String>
where
    I: StrategyServiceInstallerOps,
{
    match cmd {
        StrategyServiceCommands::Install => {
            installer.install()?;
            Ok("strategy service installed".to_string())
        }
        StrategyServiceCommands::Uninstall => {
            installer.uninstall()?;
            Ok("strategy service uninstalled".to_string())
        }
        StrategyServiceCommands::Start => {
            installer.start()?;
            Ok("strategy service started".to_string())
        }
        StrategyServiceCommands::Stop => {
            installer.stop()?;
            Ok("strategy service stopped".to_string())
        }
        StrategyServiceCommands::Enable => {
            installer.enable()?;
            Ok("strategy service enabled".to_string())
        }
        StrategyServiceCommands::Disable => {
            installer.disable()?;
            Ok("strategy service disabled".to_string())
        }
        StrategyServiceCommands::Status => installer.status(),
    }
}
