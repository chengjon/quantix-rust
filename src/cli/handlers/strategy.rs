use super::*;

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
