use super::*;

pub(super) fn create_strategy_config_store() -> JsonStrategyConfigStore {
    let runtime = CliRuntime::load();
    JsonStrategyConfigStore::new(runtime.strategy_config_path)
}

pub(super) async fn execute_strategy_config_init() -> Result<()> {
    let config = execute_strategy_config_init_to_store(&create_strategy_config_store())?;
    println!("{}", serde_json::to_string_pretty(&config)?);
    Ok(())
}

pub(super) fn execute_strategy_config_init_to_store(
    store: &JsonStrategyConfigStore,
) -> Result<StrategyDaemonConfig> {
    store.load_or_create()
}

pub(super) async fn execute_strategy_config_show() -> Result<()> {
    let config = execute_strategy_config_show_from_store(&create_strategy_config_store())?;
    println!("{}", serde_json::to_string_pretty(&config)?);
    Ok(())
}

pub(super) fn execute_strategy_config_show_from_store(
    store: &JsonStrategyConfigStore,
) -> Result<StrategyDaemonConfig> {
    store.load_or_create()
}

pub(crate) async fn run_strategy_command_impl(cmd: StrategyCommands) -> Result<()> {
    match cmd {
        StrategyCommands::Run { name, mode, code } => {
            run_strategy(name, mode, code).await?;
        }
        StrategyCommands::List => {
            list_strategies().await?;
        }
        StrategyCommands::Show { name } => {
            show_strategy(name).await?;
        }
        StrategyCommands::Config(subcommand) => match subcommand {
            StrategyConfigCommands::Init => {
                execute_strategy_config_init().await?;
            }
            StrategyConfigCommands::Show => {
                execute_strategy_config_show().await?;
            }
        },
        StrategyCommands::Daemon(subcommand) => match subcommand {
            StrategyDaemonCommands::Run { once } => {
                execute_strategy_daemon_run(once).await?;
            }
        },
        StrategyCommands::Signal(subcommand) => match subcommand {
            StrategySignalCommands::List {
                approval_status,
                signal_status,
                ..
            } => {
                execute_strategy_signal_list(approval_status.as_deref(), signal_status.as_deref())
                    .await?;
            }
            StrategySignalCommands::Approve {
                signal_id,
                target_mode,
                target_account,
            } => {
                execute_strategy_signal_approve(&signal_id, &target_mode, &target_account).await?;
            }
            StrategySignalCommands::Reject { signal_id, reason } => {
                execute_strategy_signal_reject(&signal_id, reason.as_deref()).await?;
            }
        },
        StrategyCommands::Request(subcommand) => match subcommand {
            StrategyRequestCommands::List {
                status,
                target_mode,
                target_account,
                limit,
                stats,
            } => {
                execute_strategy_request_list(
                    status.as_deref(),
                    target_mode.as_deref(),
                    target_account.as_deref(),
                    limit,
                    stats,
                )
                .await?;
            }
            StrategyRequestCommands::Show {
                request_id,
                verbose,
            } => {
                execute_strategy_request_show(&request_id, verbose).await?;
            }
            StrategyRequestCommands::Execute { request_id } => {
                execute_strategy_request_execute(&request_id).await?;
            }
            StrategyRequestCommands::Cancel { request_id, reason } => {
                execute_strategy_request_cancel(&request_id, reason.as_deref()).await?;
            }
        },
        StrategyCommands::Service(subcommand) => match subcommand {
            StrategyServiceCommands::Install => {
                execute_strategy_service_command(StrategyServiceCommands::Install)?;
            }
            StrategyServiceCommands::Uninstall => {
                execute_strategy_service_command(StrategyServiceCommands::Uninstall)?;
            }
            StrategyServiceCommands::Start => {
                execute_strategy_service_command(StrategyServiceCommands::Start)?;
            }
            StrategyServiceCommands::Stop => {
                execute_strategy_service_command(StrategyServiceCommands::Stop)?;
            }
            StrategyServiceCommands::Status => {
                execute_strategy_service_command(StrategyServiceCommands::Status)?;
            }
            StrategyServiceCommands::Enable => {
                execute_strategy_service_command(StrategyServiceCommands::Enable)?;
            }
            StrategyServiceCommands::Disable => {
                execute_strategy_service_command(StrategyServiceCommands::Disable)?;
            }
        },
        StrategyCommands::ServiceConfig(subcommand) => match subcommand {
            StrategyServiceConfigCommands::Show => {
                let output = execute_strategy_service_config_command_with_store(
                    StrategyServiceConfigCommands::Show,
                    &JsonStrategyServiceConfigStore::with_default_path()?,
                )?;
                print_strategy_service_config_output(output)?;
            }
            StrategyServiceConfigCommands::Set {
                quantix_bin,
                env_file,
            } => {
                let output = execute_strategy_service_config_command_with_store(
                    StrategyServiceConfigCommands::Set {
                        quantix_bin,
                        env_file,
                    },
                    &JsonStrategyServiceConfigStore::with_default_path()?,
                )?;
                print_strategy_service_config_output(output)?;
            }
        },
    }

    Ok(())
}

pub(super) fn execute_strategy_service_config_command_with_store(
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

pub(super) fn print_strategy_service_config_output(
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

pub(super) trait StrategyServiceInstallerOps {
    fn install(&self) -> Result<()>;
    fn uninstall(&self) -> Result<()>;
    fn start(&self) -> Result<()>;
    fn stop(&self) -> Result<()>;
    fn enable(&self) -> Result<()>;
    fn disable(&self) -> Result<()>;
    fn status(&self) -> Result<String>;
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

pub(super) fn execute_strategy_service_command(cmd: StrategyServiceCommands) -> Result<()> {
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

pub(super) fn execute_strategy_service_command_with_installer<I>(
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
