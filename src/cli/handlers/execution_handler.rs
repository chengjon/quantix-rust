use super::*;
use crate::execution::qmt_live_gate::QmtLiveGateFailure;
use crate::execution::request_diagnostics::{
    build_bridge_qmt_mode_not_live_diagnostics,
    build_bridge_qmt_order_submit_capability_missing_diagnostics, build_completion_diagnostics,
    build_unclassified_execution_error_diagnostics,
};

fn create_execution_config_store() -> JsonExecutionConfigStore {
    let runtime = CliRuntime::load();
    JsonExecutionConfigStore::new(runtime.execution_config_path)
}

pub fn create_bridge_client() -> Result<BridgeHttpClient> {
    let runtime = CliRuntime::load();
    BridgeHttpClient::new(runtime.bridge.base_url, runtime.bridge.api_key)
        .map_err(|err| QuantixError::Other(err.to_string()))
}

pub(crate) async fn execute_execution_config_init() -> Result<()> {
    let config = create_execution_config_store().load_or_create()?;
    println!("{}", serde_json::to_string_pretty(&config)?);
    Ok(())
}

pub(crate) async fn execute_execution_config_show() -> Result<()> {
    let config = create_execution_config_store().load_or_create()?;
    println!("{}", serde_json::to_string_pretty(&config)?);
    Ok(())
}

pub(crate) async fn execute_execution_daemon_run(once: bool) -> Result<()> {
    let runtime = CliRuntime::load();
    let runtime_store = StrategyRuntimeStore::new(runtime.strategy_runtime_db_path).await?;
    let config_store = JsonExecutionConfigStore::new(runtime.execution_config_path);
    let config = config_store.load_or_create()?;
    let trade_store = create_trade_store();
    let risk_store = create_risk_store();

    if once {
        let summary =
            consume_next_pending_request_with_components(&runtime_store, trade_store, risk_store)
                .await?;
        print_execution_daemon_summary(&summary);
        return Ok(());
    }

    loop {
        let summary = consume_next_pending_request_with_components(
            &runtime_store,
            trade_store.clone(),
            risk_store.clone(),
        )
        .await?;
        print_execution_daemon_summary(&summary);
        tokio::time::sleep(Duration::from_secs(config.poll_interval_secs)).await;
    }
}

pub(crate) async fn execute_execution_bridge_status() -> Result<()> {
    let capabilities = create_bridge_client()?
        .capabilities()
        .await
        .map_err(|err| QuantixError::Other(format!("bridge status 查询失败: {err}")))?;

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "tdx": {
                "enabled": capabilities.tdx.enabled,
                "supports": capabilities.tdx.supports
            },
            "qmt": {
                "enabled": capabilities.qmt.enabled,
                "mode": capabilities.qmt.mode,
                "supports": capabilities.qmt.supports
            }
        }))?
    );
    Ok(())
}

pub(crate) async fn execute_execution_bridge_qmt_preview(request_id: &str) -> Result<()> {
    let runtime = CliRuntime::load();
    let runtime_store = StrategyRuntimeStore::new(runtime.strategy_runtime_db_path).await?;
    let request = runtime_store
        .get_execution_request(request_id)
        .await?
        .ok_or_else(|| QuantixError::Other(format!("request 不存在: {request_id}")))?;

    let adapter = QmtBridgePreviewAdapter::new(create_bridge_client()?);
    let preview = adapter.preview_request(&request).await?;

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "request_id": request_id,
            "adapter_order_id": preview.adapter_order_id,
            "latest_status": preview.latest_status.as_str(),
            "filled_quantity": preview.filled_quantity,
            "rejection_reason": preview.rejection_reason,
        }))?
    );
    Ok(())
}

pub(crate) async fn execute_execution_bridge_qmt_live(
    request_id: &str,
    skip_confirm: bool,
) -> Result<()> {
    let runtime = CliRuntime::load();
    let runtime_store = StrategyRuntimeStore::new(runtime.strategy_runtime_db_path).await?;
    let request = runtime_store
        .get_execution_request(request_id)
        .await?
        .ok_or_else(|| QuantixError::Other(format!("request 不存在: {request_id}")))?;
    if request.request_status != ExecutionRequestStatus::Pending {
        return Err(QuantixError::Other(format!(
            "request 不是 pending: {request_id}"
        )));
    }
    if request.target_mode != "qmt_live" {
        return Err(QuantixError::Other(format!(
            "request target_mode 不是 qmt_live: {}",
            request.target_mode
        )));
    }

    // 从 payload_json 提取订单信息
    let snapshot = request
        .payload_json
        .get("execution_snapshot")
        .ok_or_else(|| QuantixError::Other("request 缺少 execution_snapshot".to_string()))?;
    let order_intent = snapshot
        .get("order_intent")
        .ok_or_else(|| QuantixError::Other("request 缺少 order_intent".to_string()))?;

    let symbol = snapshot
        .get("symbol")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let side = order_intent
        .get("side")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let quantity = order_intent
        .get("requested_quantity")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let price = order_intent
        .get("requested_price")
        .and_then(|v| v.as_str())
        .unwrap_or("0");
    let strategy_name = snapshot
        .get("strategy_name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    // 显示订单信息
    println!("═══════════════════════════════════════════════════════════════════");
    println!("⚠️  实盘下单确认");
    println!("═══════════════════════════════════════════════════════════════════");
    println!();
    println!("  股票代码:    {}", symbol);
    println!("  买卖方向:    {}", side);
    println!("  数量:        {} 股", quantity);
    println!("  价格:        {}", price);
    println!("  策略名称:    {}", strategy_name);
    println!();

    // 确认提示
    if !skip_confirm {
        println!("⚠️  警告: 这将提交真实订单到券商账户!");
        println!("    输入 'YES' 确认下单，其他任意键取消:");

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if input.trim() != "YES" {
            println!("已取消下单");
            return Ok(());
        }
    }

    // 构建订单请求
    let order_type = order_intent
        .get("order_type")
        .and_then(|v| v.as_str())
        .unwrap_or("limit");

    let started_at = Utc::now();
    let start_payload = merge_execution_request_payload(
        &request.payload_json,
        "executor",
        serde_json::json!({
            "type": "manual_qmt_live",
            "started_at": started_at.to_rfc3339(),
        }),
    );
    let claimed = runtime_store
        .try_start_execution_request(&request.request_id, start_payload.clone(), started_at)
        .await?;
    if !claimed {
        return Err(QuantixError::Other(format!(
            "request 状态已变化: {}",
            request.request_id
        )));
    }

    // 提交订单
    let client = create_bridge_client()?;
    if let Err(gate_error) = crate::execution::qmt_live_gate::check_bridge_qmt_live_mode(&client).await
    {
        let failed_at = Utc::now();
        let error = gate_error.to_quantix_error();
        let diagnostics = match &gate_error {
            QmtLiveGateFailure::ModeNotLive { observed_mode } => {
                build_bridge_qmt_mode_not_live_diagnostics(observed_mode)
            }
            QmtLiveGateFailure::MissingOrderSubmitSupport => {
                build_bridge_qmt_order_submit_capability_missing_diagnostics()
            }
            _ => build_unclassified_execution_error_diagnostics(&error.to_string()),
        };
        let payload_json = merge_execution_request_payload(
            &start_payload,
            "execution_error",
            serde_json::json!({
                "failed_at": failed_at.to_rfc3339(),
                "message": error.to_string(),
                "adapter": "qmt_live",
            }),
        );
        let payload_json = merge_execution_request_payload(
            &payload_json,
            "execution_diagnostics",
            diagnostics,
        );
        let updated = runtime_store
            .try_fail_execution_request(&request.request_id, payload_json, failed_at)
            .await?;
        if !updated {
            return Err(QuantixError::Other(format!(
                "request 状态已变化: {}",
                request.request_id
            )));
        }
        return Err(error);
    }

    let order_request = crate::bridge::models::BridgeQmtOrderRequest {
        request_id: request_id.to_string(),
        client_order_id: request.request_id.clone(),
        symbol: normalize_symbol_for_bridge(symbol),
        side: side.to_lowercase(),
        quantity,
        price: price.to_string(),
        order_type: order_type.to_string(),
        strategy_name: Some(strategy_name.to_string()),
        order_remark: Some("quantix-cli".to_string()),
        snapshot_metadata: None,
    };

    let response = match client.qmt_submit_order(&order_request).await {
        Ok(response) => response,
        Err(err) => {
            let failed_at = Utc::now();
            let payload_json = merge_execution_request_payload(
                &start_payload,
                "execution_error",
                serde_json::json!({
                    "failed_at": failed_at.to_rfc3339(),
                    "message": err.to_string(),
                    "adapter": "qmt_live",
                }),
            );
            let payload_json = merge_execution_request_payload(
                &payload_json,
                "execution_diagnostics",
                build_unclassified_execution_error_diagnostics(&err.to_string()),
            );
            let updated = runtime_store
                .try_fail_execution_request(&request.request_id, payload_json, failed_at)
                .await?;
            if !updated {
                return Err(QuantixError::Other(format!(
                    "request 状态已变化: {}",
                    request.request_id
                )));
            }
            return Err(QuantixError::Other(err.to_string()));
        }
    };

    let finished_at = Utc::now();
    let payload_json = merge_execution_request_payload(
        &start_payload,
        "execution_result",
        serde_json::json!({
            "executed_at": finished_at.to_rfc3339(),
            "client_order_id": request.request_id,
            "order_status": response.latest_status,
            "adapter": "qmt_live",
            "adapter_order_id": response.adapter_order_id,
            "filled_quantity": response.filled_quantity,
            "avg_fill_price": response.avg_fill_price,
            "rejection_reason": response.rejection_reason,
        }),
    );
    let payload_json = merge_execution_request_payload(
        &payload_json,
        "execution_diagnostics",
        build_completion_diagnostics(Some(response.latest_status.as_str())),
    );
    let updated = runtime_store
        .try_complete_execution_request(&request.request_id, payload_json, finished_at)
        .await?;
    if !updated {
        return Err(QuantixError::Other(format!(
            "request 状态已变化: {}",
            request.request_id
        )));
    }

    println!();
    println!("✓ 订单已提交");
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "request_id": request_id,
            "adapter_order_id": response.adapter_order_id,
            "latest_status": response.latest_status,
            "filled_quantity": response.filled_quantity,
            "avg_fill_price": response.avg_fill_price,
            "rejection_reason": response.rejection_reason,
        }))?
    );

    if response.latest_status == "rejected" {
        println!();
    } else {
        println!();
        println!(
            "查询订单状态: quantix execution bridge qmt-query --order-id {}",
            response.adapter_order_id
        );
    }

    Ok(())
}

pub(crate) fn normalize_symbol_for_bridge(symbol: &str) -> String {
    if symbol.contains('.') {
        return symbol.to_string();
    }
    if symbol.starts_with('6') {
        format!("{symbol}.SH")
    } else {
        format!("{symbol}.SZ")
    }
}

pub(crate) async fn execute_execution_bridge_qmt_query(order_id: &str) -> Result<()> {
    let client = create_bridge_client()?;
    let response = client
        .qmt_query_order(order_id)
        .await
        .map_err(|e| QuantixError::Other(e.to_string()))?;

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "adapter_order_id": response.adapter_order_id,
            "latest_status": response.latest_status,
            "filled_quantity": response.filled_quantity,
            "avg_fill_price": response.avg_fill_price,
        }))?
    );

    Ok(())
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
    let response = client
        .qmt_cancel_order(order_id)
        .await
        .map_err(|e| QuantixError::Other(e.to_string()))?;

    if response.success {
        println!("✓ 撤单成功: {}", response.order_id);
    } else {
        println!(
            "✗ 撤单失败: {}",
            response
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
