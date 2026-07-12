use crate::cli::handlers::shared_support::{create_risk_store, create_trade_store};
use crate::core::{CliRuntime, QuantixError, Result};
use crate::execution::daemon::execute_request_by_id_with_components;
use crate::execution::models::{ExecutionRequestRecord, ExecutionRequestStatus};
use crate::execution::runtime_store::StrategyRuntimeStore;
use crate::risk::JsonRiskStore;
use crate::trade::PaperTradeStore;
use chrono::Utc;

use super::format::{
    format_strategy_request_detail_with_related_order, format_strategy_request_row,
    format_strategy_request_row_with_related_order,
};
use super::qmt::find_related_order_for_request;
use super::store::{execute_strategy_request_list_with_store, merge_execution_request_payload};

pub(crate) async fn execute_strategy_request_list(
    status: Option<&str>,
    target_mode: Option<&str>,
    target_account: Option<&str>,
    limit: usize,
    stats: bool,
) -> Result<()> {
    let runtime = CliRuntime::load();
    let runtime_store = StrategyRuntimeStore::new(runtime.strategy_runtime_db_path).await?;
    let rows = execute_strategy_request_list_with_store(&runtime_store, status).await?;

    let mut filtered: Vec<_> = rows
        .into_iter()
        .filter(|row| {
            let mode_match = target_mode.is_none_or(|m| row.target_mode == m);
            let account_match = target_account.is_none_or(|a| row.target_account == a);
            mode_match && account_match
        })
        .collect();

    if stats {
        let total = filtered.len();
        let pending = filtered
            .iter()
            .filter(|r| r.request_status == ExecutionRequestStatus::Pending)
            .count();
        let in_progress = filtered
            .iter()
            .filter(|r| r.request_status == ExecutionRequestStatus::InProgress)
            .count();
        let completed = filtered
            .iter()
            .filter(|r| r.request_status == ExecutionRequestStatus::Completed)
            .count();
        let failed = filtered
            .iter()
            .filter(|r| r.request_status == ExecutionRequestStatus::Failed)
            .count();
        let canceled = filtered
            .iter()
            .filter(|r| r.request_status == ExecutionRequestStatus::Canceled)
            .count();

        println!("=== Execution Request Statistics ===");
        println!(
            "Total: {} | Pending: {} | InProgress: {} | Completed: {} | Failed: {} | Canceled: {}",
            total, pending, in_progress, completed, failed, canceled
        );
        println!();
    }

    filtered.truncate(limit);

    for row in filtered {
        let related_order = find_related_order_for_request(&runtime_store, &row).await?;
        println!(
            "{}",
            format_strategy_request_row_with_related_order(&row, related_order.as_ref())
        );
    }

    Ok(())
}

pub(crate) async fn execute_strategy_request_show(request_id: &str, verbose: bool) -> Result<()> {
    let runtime = CliRuntime::load();
    let store = StrategyRuntimeStore::new(runtime.strategy_runtime_db_path).await?;

    let request = store
        .get_execution_request(request_id)
        .await?
        .ok_or_else(|| QuantixError::Other(format!("request 不存在: {request_id}")))?;

    let related_order = find_related_order_for_request(&store, &request).await?;
    println!(
        "{}",
        format_strategy_request_detail_with_related_order(
            &request,
            related_order.as_ref(),
            verbose
        )
    );

    if let Some(order) = related_order.as_ref() {
        println!();
        println!("=== Related Order ===");
        println!("order_id: {}", order.order_id);
        println!("symbol: {}", order.symbol);
        println!("status: {}", order.status.as_str());
        println!(
            "filled: {}/{}",
            order.filled_quantity, order.requested_quantity
        );
        if let Some(avg_price) = order.avg_fill_price {
            println!("avg_fill_price: {}", avg_price);
        }
    }

    Ok(())
}

pub(crate) async fn execute_strategy_request_execute(request_id: &str) -> Result<()> {
    let runtime = CliRuntime::load();
    let runtime_store = StrategyRuntimeStore::new(runtime.strategy_runtime_db_path).await?;
    let request = execute_strategy_request_execute_with_components(
        &runtime_store,
        request_id,
        create_trade_store(),
        create_risk_store(),
    )
    .await?;
    println!("{}", format_strategy_request_row(&request));
    Ok(())
}

pub(crate) async fn execute_strategy_request_execute_with_components<TS>(
    store: &StrategyRuntimeStore,
    request_id: &str,
    trade_store: TS,
    risk_store: JsonRiskStore,
) -> Result<ExecutionRequestRecord>
where
    TS: PaperTradeStore + Clone,
{
    execute_request_by_id_with_components(store, request_id, trade_store, risk_store).await
}

pub(crate) async fn execute_strategy_request_cancel(
    request_id: &str,
    reason: Option<&str>,
) -> Result<()> {
    let runtime = CliRuntime::load();
    let runtime_store = StrategyRuntimeStore::new(runtime.strategy_runtime_db_path).await?;
    let request =
        execute_strategy_request_cancel_with_store(&runtime_store, request_id, reason).await?;
    println!("{}", format_strategy_request_row(&request));
    Ok(())
}

pub(crate) async fn execute_strategy_request_cancel_with_store(
    store: &StrategyRuntimeStore,
    request_id: &str,
    reason: Option<&str>,
) -> Result<ExecutionRequestRecord> {
    let request = store
        .get_execution_request(request_id)
        .await?
        .ok_or_else(|| QuantixError::Other(format!("request 不存在: {request_id}")))?;
    if request.request_status != ExecutionRequestStatus::Pending {
        return Err(QuantixError::Other(format!(
            "request 不是 pending: {request_id}"
        )));
    }

    let payload_json = merge_execution_request_payload(
        &request.payload_json,
        "cancellation",
        serde_json::json!({
            "canceled_at": Utc::now().to_rfc3339(),
            "reason": reason.unwrap_or("manual cancel"),
        }),
    );
    let updated = store
        .try_cancel_execution_request(&request.request_id, payload_json, Utc::now())
        .await?;
    if !updated {
        return Err(QuantixError::Other(format!(
            "request 状态已变化: {}",
            request.request_id
        )));
    }
    store
        .get_execution_request(&request.request_id)
        .await?
        .ok_or_else(|| QuantixError::Other(format!("request 不存在: {}", request.request_id)))
}
