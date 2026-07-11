//! Bridge QMT cancel/account/positions/asset execute handlers.

use super::*;
pub(crate) async fn execute_execution_bridge_qmt_cancel_with_client(
    client: &BridgeHttpClient,
    order_id: &str,
) -> Result<QmtCancelCommandResult> {
    let submit_service = create_qmt_task_submit_service(client)?;
    let (cancel_order_id, resolved_from_task_result) =
        match submit_service.query_task_result_by_task_id(order_id).await {
            Ok(result) => match result.external_order_id {
                Some(external_order_id) => (external_order_id, true),
                None => (order_id.to_string(), false),
            },
            Err(error) if should_fallback_from_task_result_lookup(&error) => {
                (order_id.to_string(), false)
            }
            Err(error) => return Err(QuantixError::Other(error.to_string())),
        };

    let response = client
        .qmt_cancel_order(&cancel_order_id)
        .await
        .map_err(|err| QuantixError::Other(err.to_string()))?;

    Ok(QmtCancelCommandResult {
        requested_order_id: order_id.to_string(),
        cancel_order_id,
        resolved_from_task_result,
        response,
    })
}

pub(crate) async fn execute_execution_bridge_qmt_cancel(order_id: &str) -> Result<()> {
    println!("⚠️  确认撤销订单: {}", order_id);
    println!("    输入 'YES' 确认撤单，其他任意键取消:");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    if input.trim() != "YES" {
        println!("已取消撤单");
        return Ok(());
    }

    let client = create_bridge_client()?;
    let result = execute_execution_bridge_qmt_cancel_with_client(&client, order_id).await?;

    if result.response.success {
        if result.resolved_from_task_result && result.cancel_order_id != result.requested_order_id {
            println!(
                "✓ 撤单成功: {} (from task_id {})",
                result.response.order_id, result.requested_order_id
            );
        } else {
            println!("✓ 撤单成功: {}", result.response.order_id);
        }
    } else {
        println!(
            "✗ 撤单失败: {}",
            result
                .response
                .error_message
                .unwrap_or_else(|| "未知错误".to_string())
        );
    }

    Ok(())
}

pub(crate) async fn execute_execution_bridge_qmt_account() -> Result<()> {
    let client = create_bridge_client()?;
    let response = client
        .qmt_account_status()
        .await
        .map_err(|e| QuantixError::Other(e.to_string()))?;

    println!("账户状态");
    println!("─────────────────────────────────────");
    println!("  适配器:      {}", response.adapter);
    println!("  模式:        {}", response.mode);
    println!("  SDK 可用:   {}", response.sdk_available);
    println!(
        "  连接状态:    {}",
        if response.connected {
            "已连接"
        } else {
            "未连接"
        }
    );
    if let Some(account) = response.account_masked {
        println!("  账户:        {}", account);
    }

    Ok(())
}

pub(crate) async fn execute_execution_bridge_qmt_positions() -> Result<()> {
    let client = create_bridge_client()?;
    let positions = client
        .qmt_positions()
        .await
        .map_err(|e| QuantixError::Other(e.to_string()))?;

    if positions.is_empty() {
        println!("当前无持仓");
        return Ok(());
    }

    println!("持仓列表");
    println!("─────────────────────────────────────────────────────────────────────────────────");
    println!(
        "{:<12} {:<10} {:<12} {:<12} {:<12}",
        "股票代码", "持仓", "可用", "成本价", "市值"
    );
    println!("{}", "-".repeat(76));

    for pos in positions {
        println!(
            "{:<12} {:<10} {:<12} {:<12} {:<12}",
            pos.symbol,
            pos.volume,
            pos.available,
            pos.cost_price.as_deref().unwrap_or("-"),
            pos.market_value.as_deref().unwrap_or("-")
        );
    }

    Ok(())
}

pub(crate) async fn execute_execution_bridge_qmt_asset() -> Result<()> {
    let client = create_bridge_client()?;
    let asset = client
        .qmt_asset()
        .await
        .map_err(|e| QuantixError::Other(e.to_string()))?;

    println!("资产信息");
    println!("─────────────────────────────────────");
    println!("  总资产:      {}", asset.total_asset);
    println!("  可用现金:    {}", asset.cash);
    println!("  持仓市值:    {}", asset.market_value);
    println!("  账户 ID:    {}", asset.account_id);

    Ok(())
}

pub(crate) fn print_execution_daemon_summary(summary: &ExecutionDaemonIterationSummary) {
    println!("{}", format_execution_daemon_summary(summary));
}
