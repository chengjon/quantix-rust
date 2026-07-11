//! Bridge QMT status/preview/live execute handlers.

use super::*;
pub(crate) async fn execute_execution_bridge_status(checklist: bool) -> Result<()> {
    let bridge_client = create_bridge_client()?;
    let qmt_live_capabilities = QmtLiveExecutionAdapter::new(bridge_client.clone()).capabilities();
    let capabilities_result = bridge_client.capabilities().await;
    let bridge_error = capabilities_result.as_ref().err().map(ToString::to_string);
    let kill_switch_state = if checklist {
        Some(JsonKillSwitchStore::with_default_path()?.load_or_default()?)
    } else {
        None
    };
    let preflight_report = build_qmt_live_preflight_report(
        capabilities_result.as_ref().ok(),
        bridge_error.as_deref(),
        qmt_live_capabilities,
        kill_switch_state.as_ref(),
    );

    let capabilities = match capabilities_result {
        Ok(capabilities) => capabilities,
        Err(err) if checklist => {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "bridge_status_error": err.to_string(),
                    "qmt_live_preflight": qmt_live_preflight_report_json(&preflight_report)
                }))?
            );
            println!();
            println!("{}", format_qmt_live_preflight_report(&preflight_report));
            return Ok(());
        }
        Err(err) => {
            return Err(QuantixError::Other(format!(
                "bridge status 查询失败: {err}"
            )));
        }
    };

    let mut status_payload = serde_json::json!({
            "tdx": {
                "enabled": capabilities.tdx.enabled,
                "supports": capabilities.tdx.supports
            },
            "qmt": {
                "enabled": capabilities.qmt.enabled,
                "mode": capabilities.qmt.mode,
                "supports": capabilities.qmt.supports
            }
    });
    if checklist {
        status_payload["qmt_live_preflight"] = qmt_live_preflight_report_json(&preflight_report);
    }

    println!("{}", serde_json::to_string_pretty(&status_payload)?);
    if checklist {
        println!();
        println!(
            "{}",
            format_qmt_promotion_checklist(&capabilities, qmt_live_capabilities)
        );
        println!();
        println!("{}", format_qmt_live_preflight_report(&preflight_report));
    }
    Ok(())
}

pub(crate) async fn execute_execution_bridge_qmt_preview(request_id: &str) -> Result<()> {
    let runtime = CliRuntime::load();
    let runtime_store = StrategyRuntimeStore::new(runtime.strategy_runtime_db_path).await?;
    let request = runtime_store
        .get_execution_request(request_id)
        .await?
        .ok_or_else(|| QuantixError::Other(format!("request 不存在: {request_id}")))?;
    if request.target_mode != "qmt_live" {
        return Err(QuantixError::Unsupported(format!(
            "execution bridge qmt-preview 只支持 target_mode=qmt_live 的 request；当前 request target_mode={}。如需预览 QMT 提交流程，请先创建 qmt_live request；通用 target_mode=live 仍未实现",
            request.target_mode
        )));
    }

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
            "broker_payload": preview.broker_payload,
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
    let kill_switch_store = JsonKillSwitchStore::with_default_path()?;
    execute_execution_bridge_qmt_live_with_runtime_store_and_kill_switch(
        &runtime_store,
        &kill_switch_store,
        request_id,
        skip_confirm,
    )
    .await
}

pub(crate) async fn execute_execution_bridge_qmt_live_with_runtime_store_and_kill_switch(
    runtime_store: &StrategyRuntimeStore,
    kill_switch_store: &JsonKillSwitchStore,
    request_id: &str,
    skip_confirm: bool,
) -> Result<()> {
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

    if let Some(state) =
        load_blocking_kill_switch_state(kill_switch_store, request.target_mode.as_str())?
    {
        let blocked_at = Utc::now();
        let err = QuantixError::Other(format_execution_kill_switch_block_message(
            request.target_mode.as_str(),
            &state,
        ));
        let payload_json = merge_execution_request_payload(
            &request.payload_json,
            "execution_error",
            serde_json::json!({
                "failed_at": blocked_at.to_rfc3339(),
                "message": err.to_string(),
                "adapter": request.target_mode.as_str(),
            }),
        );
        let payload_json = merge_execution_request_payload(
            &payload_json,
            "kill_switch",
            build_kill_switch_payload(&state, request.target_mode.as_str(), blocked_at),
        );
        let payload_json = merge_execution_request_payload(
            &payload_json,
            "execution_diagnostics",
            build_kill_switch_blocked_diagnostics(request.target_mode.as_str()),
        );
        let updated = runtime_store
            .try_fail_pending_execution_request(&request.request_id, payload_json, blocked_at)
            .await?;
        if !updated {
            return Err(QuantixError::Other(format!(
                "request 状态已变化: {}",
                request.request_id
            )));
        }
        return Err(err);
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
    let side = OrderSide::from_str(side)
        .ok_or_else(|| QuantixError::Other(format!("request 包含无效 side: {side}")))?;
    let order_type = OrderType::from_str(order_type)
        .ok_or_else(|| QuantixError::Other(format!("request 包含无效 order_type: {order_type}")))?;
    let requested_price = Decimal::from_str(price).map_err(|err| {
        QuantixError::Other(format!("request 包含无效 requested_price: {price}, {err}"))
    })?;
    let signal = runtime_store
        .get_signal(&request.signal_id)
        .await?
        .ok_or_else(|| QuantixError::Other(format!("signal 不存在: {}", request.signal_id)))?;

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
    if let Err(gate_error) =
        crate::execution::qmt_live_gate::check_bridge_qmt_live_mode(&client).await
    {
        let failed_at = Utc::now();
        let error = gate_error.to_quantix_error();
        let diagnostics = match &gate_error {
            QmtLiveGateFailure::CapabilityCheckFailed(_) => {
                build_bridge_qmt_capability_check_failed_diagnostics(&error.to_string())
            }
            QmtLiveGateFailure::CapabilityDisabled => {
                build_bridge_qmt_capability_disabled_diagnostics()
            }
            QmtLiveGateFailure::ModeNotLive { observed_mode } => {
                build_bridge_qmt_mode_not_live_diagnostics(observed_mode)
            }
            QmtLiveGateFailure::MissingOrderSubmitSupport => {
                build_bridge_qmt_order_submit_capability_missing_diagnostics()
            }
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
        let payload_json =
            merge_execution_request_payload(&payload_json, "execution_diagnostics", diagnostics);
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

    let submit_service = QmtTaskSubmitService::new(client.clone(), 1, 30_000)
        .map_err(|err| QuantixError::Other(err.to_string()))?;
    let order_request = AdapterOrderRequest {
        client_order_id: request.request_id.clone(),
        symbol: normalize_symbol_for_bridge(symbol),
        side,
        quantity,
        price: requested_price,
    };

    let receipt = match submit_service.submit_order(&order_request).await {
        Ok(receipt) => receipt,
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
    let client_order_id = request.request_id.clone();
    let task_id = receipt.task_id.clone();
    let qmt_live_metadata = QmtLiveRuntimeMetadata {
        task_identity: Some(QmtLiveTaskIdentity {
            task_id: task_id.clone(),
            client_order_id: client_order_id.clone(),
            local_submission_id: receipt.local_submission_id.clone(),
            external_order_id: None,
        }),
        last_query: None,
        reconciliation: None,
    };
    match runtime_store
        .find_order_by_client_order_id(&request.request_id)
        .await?
    {
        Some(existing_order) => {
            let updated = runtime_store
                .try_update_order_qmt_live_metadata(
                    &existing_order,
                    &qmt_live_metadata,
                    finished_at,
                )
                .await?;
            if !updated {
                return Err(QuantixError::Other(format!(
                    "related order qmt_live metadata 更新失败: {}",
                    existing_order.order_id
                )));
            }
        }
        None => {
            let related_order = OrderRecord {
                order_id: client_order_id.clone(),
                client_order_id: client_order_id.clone(),
                run_id: signal.run_id,
                symbol: symbol.to_string(),
                side,
                order_type,
                requested_quantity: quantity,
                requested_price: order_request.price,
                filled_quantity: 0,
                remaining_quantity: quantity,
                avg_fill_price: None,
                status: OrderStatus::PendingSubmit,
                adapter: "qmt_live".to_string(),
                created_at: finished_at,
                updated_at: finished_at,
                last_transition_at: finished_at,
                version: 1,
                payload_json: serde_json::json!({
                    "qmt_live": qmt_live_metadata
                }),
            };
            runtime_store.insert_order(&related_order).await?;
        }
    }

    let payload_json = merge_execution_request_payload(
        &start_payload,
        "execution_result",
        serde_json::json!({
            "executed_at": finished_at.to_rfc3339(),
            "client_order_id": client_order_id,
            "order_status": OrderStatus::PendingSubmit.as_str(),
            "adapter": "qmt_live",
            "adapter_order_id": task_id.clone(),
            "filled_quantity": 0,
            "avg_fill_price": serde_json::Value::Null,
            "rejection_reason": serde_json::Value::Null,
        }),
    );
    let payload_json = merge_execution_request_payload(
        &payload_json,
        "execution_diagnostics",
        build_completion_diagnostics(Some(OrderStatus::PendingSubmit.as_str())),
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
    println!("✓ 订单提交任务已受理");
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "request_id": request_id,
            "adapter_order_id": task_id,
            "latest_status": OrderStatus::PendingSubmit.as_str(),
            "local_submission_id": receipt.local_submission_id,
            "bridge_contract_version": receipt.bridge_contract_version,
            "source_name": receipt.source_name,
        }))?
    );

    println!();
    println!("提示: request 可能会显示为 completed。");
    println!("      这只表示执行层已完成提交，不代表订单已经终态。");
    println!("      订单初始状态通常仍为 pending_submit，请继续跟踪后续收敛。");
    println!();
    println!(
        "查看 request 与后续收敛状态: quantix strategy request show {} --verbose",
        request_id
    );

    Ok(())
}
