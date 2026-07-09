//! Execution command dispatcher and handlers.
//!
//! Responsibilities are split across child modules; the parent keeps the
//! command type imports, the dispatcher ([execute_execution_command]), and
//! re-exports child handlers via glob so callers can keep using
//! `crate::cli::handlers::execution_handler::<name>` paths unchanged.

use super::*;
use crate::bridge::client::BridgeHttpClient;
use crate::bridge::error::BridgeError;
use crate::core::{CliRuntime, QuantixError, Result};
use crate::execution::adapter::{
    AdapterOrderRequest, ExecutionAdapter, ExecutionCancelSemantics, ExecutionCapabilities,
    ExecutionChannel, ExecutionFillSource, ExecutionStatusSource,
};
use crate::execution::config::JsonExecutionConfigStore;
use crate::execution::daemon::{
    ExecutionDaemonIterationSummary, consume_next_pending_request_with_components,
};
use crate::execution::models::{
    ExecutionRequestRecord, ExecutionRequestStatus, OrderRecord, OrderSide, OrderStatus, OrderType,
    QmtLiveRuntimeMetadata, QmtLiveTaskIdentity,
};
use crate::execution::qmt_bridge::QmtBridgePreviewAdapter;
use crate::execution::qmt_live_adapter::QmtLiveExecutionAdapter;
use crate::execution::qmt_live_gate::QmtLiveGateFailure;
use crate::execution::qmt_task_submit_service::QmtTaskSubmitService;
use crate::execution::request_diagnostics::{
    build_bridge_qmt_capability_check_failed_diagnostics,
    build_bridge_qmt_capability_disabled_diagnostics, build_bridge_qmt_mode_not_live_diagnostics,
    build_bridge_qmt_order_submit_capability_missing_diagnostics, build_completion_diagnostics,
    build_kill_switch_blocked_diagnostics, build_unclassified_execution_error_diagnostics,
};
use crate::execution::runtime_store::StrategyRuntimeStore;
use crate::safety::{
    JsonKillSwitchStore, KillSwitchState, build_kill_switch_payload,
    format_execution_kill_switch_block_message, load_blocking_kill_switch_state,
};
use chrono::Utc;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::str::FromStr;

mod account;
mod config;
mod live;
mod promotion;
mod query;

pub(crate) use account::*;
pub(crate) use config::*;
pub(crate) use live::*;
pub(crate) use promotion::*;
pub(crate) use query::*;

pub(crate) async fn execute_execution_command(cmd: ExecutionCommands) -> Result<()> {
    match cmd {
        ExecutionCommands::Config(subcommand) => match subcommand {
            ExecutionConfigCommands::Init => {
                execute_execution_config_init().await?;
            }
            ExecutionConfigCommands::Show => {
                execute_execution_config_show().await?;
            }
        },
        ExecutionCommands::Daemon(subcommand) => match subcommand {
            ExecutionDaemonCommands::Run { once } => {
                execute_execution_daemon_run(once).await?;
            }
        },
        ExecutionCommands::Bridge(subcommand) => match subcommand {
            ExecutionBridgeCommands::Status { checklist } => {
                execute_execution_bridge_status(checklist).await?;
            }
            ExecutionBridgeCommands::QmtPreview { request_id } => {
                execute_execution_bridge_qmt_preview(&request_id).await?;
            }
            ExecutionBridgeCommands::QmtLive { request_id, yes } => {
                execute_execution_bridge_qmt_live(&request_id, yes).await?;
            }
            ExecutionBridgeCommands::QmtQuery { order_id } => {
                execute_execution_bridge_qmt_query(&order_id).await?;
            }
            ExecutionBridgeCommands::QmtAudit {
                request_id,
                task_id,
                local_submission_id,
            } => {
                let lookup = QmtAuditLookup::from_cli(request_id, task_id, local_submission_id)?;
                execute_execution_bridge_qmt_audit(lookup).await?;
            }
            ExecutionBridgeCommands::QmtCancel { order_id } => {
                execute_execution_bridge_qmt_cancel(&order_id).await?;
            }
            ExecutionBridgeCommands::QmtAccount => {
                execute_execution_bridge_qmt_account().await?;
            }
            ExecutionBridgeCommands::QmtPositions => {
                execute_execution_bridge_qmt_positions().await?;
            }
            ExecutionBridgeCommands::QmtAsset => {
                execute_execution_bridge_qmt_asset().await?;
            }
        },
        ExecutionCommands::Qmt(subcommand) => match subcommand {
            ExecutionQmtCommands::Status { checklist } => {
                execute_execution_bridge_status(checklist).await?;
            }
            ExecutionQmtCommands::Preview { request_id } => {
                execute_execution_bridge_qmt_preview(&request_id).await?;
            }
            ExecutionQmtCommands::Live { request_id, yes } => {
                execute_execution_bridge_qmt_live(&request_id, yes).await?;
            }
            ExecutionQmtCommands::Query { order_id } => {
                execute_execution_bridge_qmt_query(&order_id).await?;
            }
            ExecutionQmtCommands::Audit {
                request_id,
                task_id,
                local_submission_id,
            } => {
                let lookup = QmtAuditLookup::from_cli(request_id, task_id, local_submission_id)?;
                execute_execution_bridge_qmt_audit(lookup).await?;
            }
            ExecutionQmtCommands::ManualInterventions {
                action,
                request_id,
                task_id,
                local_submission_id,
            } => {
                execute_execution_bridge_qmt_manual_interventions(
                    &action,
                    request_id,
                    task_id,
                    local_submission_id,
                )
                .await?;
            }
            ExecutionQmtCommands::Cancel { order_id } => {
                execute_execution_bridge_qmt_cancel(&order_id).await?;
            }
            ExecutionQmtCommands::Account => {
                execute_execution_bridge_qmt_account().await?;
            }
            ExecutionQmtCommands::Positions => {
                execute_execution_bridge_qmt_positions().await?;
            }
            ExecutionQmtCommands::Asset => {
                execute_execution_bridge_qmt_asset().await?;
            }
        },
    }

    Ok(())
}
