use async_trait::async_trait;
use chrono::Utc;

use crate::core::{QuantixError, Result};
use crate::execution::kernel::{ExecutionKernel, FillDeltaApplier, RiskDecision, RiskEvaluator};
use crate::execution::mock_live::{MockLiveExecutionAdapter, SystemMockLiveClock};
use crate::execution::models::{
    ExecutionRequestRecord, ExecutionRequestStatus, FillDeltaContext, FillDeltaResult, OrderIntent,
    OrderSide,
};
use crate::execution::paper::PaperExecutionAdapter;
use crate::execution::qmt_live_gate::{QMT_LIVE_BRIDGE_COMMAND, QMT_LIVE_BRIDGE_MODE_REQUIREMENT};
use crate::execution::request_diagnostics::{
    build_completion_diagnostics, build_daemon_live_mode_unsupported_diagnostics,
    build_daemon_qmt_live_manual_bridge_required_diagnostics,
    build_kill_switch_blocked_diagnostics, build_unclassified_execution_error_diagnostics,
};
use crate::execution::runtime_store::StrategyRuntimeStore;
use crate::risk::JsonRiskStore;
use crate::risk::service::RuntimeJsonRiskServices;
use crate::safety::{
    JsonKillSwitchStore, build_kill_switch_payload, format_execution_kill_switch_block_message,
    load_blocking_kill_switch_state,
};
use crate::trade::{PaperTradeStore, TradeOrderRequest, TradeService};

mod helpers;

use helpers::{
    build_prepared_request_from_execution_request, build_projected_buy_impact,
    build_risk_account_snapshot, decimal_to_f64, load_initialized_trade_account,
    merge_execution_request_payload, remap_trade_request_error, sync_risk_from_trade_store,
};

/// execution daemon 单轮迭代摘要：claimed 本轮领取的 pending 数、completed 完成数、failed 失败数、request 本轮处理的首个请求（便于排查）。
#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionDaemonIterationSummary {
    pub claimed: usize,
    pub completed: usize,
    pub failed: usize,
    pub request: Option<ExecutionRequestRecord>,
}

#[derive(Debug, Clone)]
struct RequestFillDeltaBridge<TradeStore> {
    trade_service: TradeService<TradeStore>,
}

impl<TradeStore> RequestFillDeltaBridge<TradeStore>
where
    TradeStore: PaperTradeStore,
{
    fn new(trade_store: TradeStore) -> Self {
        Self {
            trade_service: TradeService::new(trade_store),
        }
    }
}

#[async_trait]
impl<TradeStore> FillDeltaApplier for RequestFillDeltaBridge<TradeStore>
where
    TradeStore: PaperTradeStore,
{
    async fn apply_fill_delta(&self, ctx: FillDeltaContext) -> Result<FillDeltaResult> {
        if ctx.new_filled_quantity <= ctx.old_filled_quantity {
            return Ok(FillDeltaResult {
                applied: false,
                delta_quantity: 0,
                trade_record_id: None,
            });
        }

        let fill_details = ctx.fill_details.ok_or_else(|| {
            QuantixError::Other("execution daemon 增量成交缺少 fill_details".to_string())
        })?;

        let request = TradeOrderRequest::new(
            ctx.symbol.clone(),
            decimal_to_f64(fill_details.fill_price, "execution daemon")?,
            fill_details.fill_quantity,
        )
        .map_err(|err| remap_trade_request_error(err, "execution daemon"))?;

        let record = match ctx.side {
            OrderSide::Buy => self.trade_service.buy(request, ctx.event_time).await?,
            OrderSide::Sell => self.trade_service.sell(request, ctx.event_time).await?,
        };

        Ok(FillDeltaResult {
            applied: true,
            delta_quantity: fill_details.fill_quantity,
            trade_record_id: Some(record.id),
        })
    }
}

#[derive(Debug, Clone)]
struct RequestRuntimeRiskBridge<TradeStore> {
    trade_store: TradeStore,
    risk_services: RuntimeJsonRiskServices,
}

impl<TradeStore> RequestRuntimeRiskBridge<TradeStore> {
    fn new(trade_store: TradeStore, risk_services: RuntimeJsonRiskServices) -> Self {
        Self {
            trade_store,
            risk_services,
        }
    }
}

#[async_trait]
impl<TradeStore> RiskEvaluator for RequestRuntimeRiskBridge<TradeStore>
where
    TradeStore: PaperTradeStore,
{
    async fn evaluate(&self, intent: OrderIntent) -> Result<RiskDecision> {
        if intent.side == OrderSide::Sell {
            return Ok(RiskDecision::Allow);
        }

        let account = load_initialized_trade_account(&self.trade_store).await?;
        let snapshot = build_risk_account_snapshot(&account);
        let request = TradeOrderRequest::new(
            intent.symbol.clone(),
            decimal_to_f64(intent.requested_price, "execution daemon")?,
            intent.requested_quantity,
        )
        .map_err(|err| remap_trade_request_error(err, "execution daemon"))?;
        let projected_buy = build_projected_buy_impact(&account, &request);
        let risk_service = self.risk_services.buy_checks().await?;

        match risk_service
            .check_buy(&snapshot, &projected_buy, Utc::now())
            .await
        {
            Ok(()) => Ok(RiskDecision::Allow),
            Err(QuantixError::Other(reason)) => Ok(RiskDecision::Reject { reason }),
            Err(other) => Err(other),
        }
    }

    async fn sync_after_fill(&self) -> Result<()> {
        sync_risk_from_trade_store(&self.trade_store, self.risk_services.base()).await
    }
}

pub async fn consume_next_pending_request_with_components<TS>(
    store: &StrategyRuntimeStore,
    trade_store: TS,
    risk_store: JsonRiskStore,
) -> Result<ExecutionDaemonIterationSummary>
where
    TS: PaperTradeStore + Clone,
{
    let Some(request) = store.find_next_pending_execution_request().await? else {
        return Ok(ExecutionDaemonIterationSummary {
            claimed: 0,
            completed: 0,
            failed: 0,
            request: None,
        });
    };

    match execute_request_by_id_with_components(store, &request.request_id, trade_store, risk_store)
        .await
    {
        Ok(saved_request) => Ok(ExecutionDaemonIterationSummary {
            claimed: 1,
            completed: 1,
            failed: 0,
            request: Some(saved_request),
        }),
        Err(err) => {
            if let Some(saved_request) = store
                .get_execution_request(&request.request_id)
                .await?
                .filter(|saved| saved.request_status == ExecutionRequestStatus::Failed)
            {
                Ok(ExecutionDaemonIterationSummary {
                    claimed: 1,
                    completed: 0,
                    failed: 1,
                    request: Some(saved_request),
                })
            } else {
                Err(err)
            }
        }
    }
}

pub async fn execute_request_by_id_with_components<TS>(
    store: &StrategyRuntimeStore,
    request_id: &str,
    trade_store: TS,
    risk_store: JsonRiskStore,
) -> Result<ExecutionRequestRecord>
where
    TS: PaperTradeStore + Clone,
{
    let kill_switch_store = JsonKillSwitchStore::with_default_path()?;
    execute_request_by_id_with_components_and_kill_switch(
        store,
        &kill_switch_store,
        request_id,
        trade_store,
        risk_store,
    )
    .await
}

pub async fn execute_request_by_id_with_components_and_kill_switch<TS>(
    store: &StrategyRuntimeStore,
    kill_switch_store: &JsonKillSwitchStore,
    request_id: &str,
    trade_store: TS,
    risk_store: JsonRiskStore,
) -> Result<ExecutionRequestRecord>
where
    TS: PaperTradeStore + Clone,
{
    let request = store
        .get_execution_request(request_id)
        .await?
        .ok_or_else(|| QuantixError::Other(format!("request 不存在: {request_id}")))?;
    if request.request_status != ExecutionRequestStatus::Pending {
        return Err(QuantixError::Other(format!(
            "request 不是 pending: {request_id}"
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
        let updated = store
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

    let now = Utc::now();
    let start_payload = merge_execution_request_payload(
        &request.payload_json,
        "executor",
        serde_json::json!({
            "type": "manual_or_daemon",
            "started_at": now.to_rfc3339(),
        }),
    );
    let claimed = store
        .try_start_execution_request(&request.request_id, start_payload, now)
        .await?;
    if !claimed {
        return Err(QuantixError::Other(format!(
            "request 状态已变化: {}",
            request.request_id
        )));
    }

    let prepared = build_prepared_request_from_execution_request(&request)?;
    let risk = RequestRuntimeRiskBridge::new(
        trade_store.clone(),
        RuntimeJsonRiskServices::new(risk_store),
    );

    let execution_result = match request.target_mode.as_str() {
        "paper" => {
            let adapter = PaperExecutionAdapter::new(TradeService::new(trade_store));
            let kernel = ExecutionKernel::new(store.clone(), adapter, risk);
            kernel.execute_request(prepared).await
        }
        "mock_live" => {
            let adapter = MockLiveExecutionAdapter::new(store.clone(), SystemMockLiveClock);
            let fill_delta = RequestFillDeltaBridge::new(trade_store.clone());
            let kernel = ExecutionKernel::with_fill_delta(store.clone(), adapter, fill_delta, risk);
            kernel.execute_request(prepared).await
        }
        "qmt_live" => Err(QuantixError::Other(format!(
            "qmt_live request 不能通过自动执行路径提交，请使用 {QMT_LIVE_BRIDGE_COMMAND} --request-id {}，并确保 {QMT_LIVE_BRIDGE_MODE_REQUIREMENT}",
            request.request_id
        ))),
        "live" => Err(QuantixError::Unsupported(format!(
            "execution daemon live 模式尚未实现；如需真实 QMT 提交，请将 request target_mode 设为 qmt_live，并确保 {QMT_LIVE_BRIDGE_MODE_REQUIREMENT}，然后走 {QMT_LIVE_BRIDGE_COMMAND} 路径"
        ))),
        other => Err(QuantixError::Unsupported(format!(
            "execution daemon {other} 模式尚未实现"
        ))),
    };

    let finished_at = Utc::now();
    match execution_result {
        Ok(result) => {
            let order_status = result.order_status.map(|status| status.as_str());
            let payload_json = merge_execution_request_payload(
                &request.payload_json,
                "execution_result",
                serde_json::json!({
                    "executed_at": finished_at.to_rfc3339(),
                    "run_id": result.run_id,
                    "client_order_id": result.client_order_id,
                    "order_status": order_status,
                    "adapter": request.target_mode,
                }),
            );
            let payload_json = merge_execution_request_payload(
                &payload_json,
                "execution_diagnostics",
                build_completion_diagnostics(order_status),
            );
            let updated = store
                .try_complete_execution_request(&request.request_id, payload_json, finished_at)
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
                .ok_or_else(|| {
                    QuantixError::Other(format!("request 不存在: {}", request.request_id))
                })
        }
        Err(err) => {
            let payload_json = merge_execution_request_payload(
                &request.payload_json,
                "execution_error",
                serde_json::json!({
                    "failed_at": finished_at.to_rfc3339(),
                    "message": err.to_string(),
                }),
            );
            let diagnostics = match request.target_mode.as_str() {
                "qmt_live" => {
                    build_daemon_qmt_live_manual_bridge_required_diagnostics(&request.request_id)
                }
                "live" => build_daemon_live_mode_unsupported_diagnostics(),
                _ => build_unclassified_execution_error_diagnostics(&err.to_string()),
            };
            let payload_json = merge_execution_request_payload(
                &payload_json,
                "execution_diagnostics",
                diagnostics,
            );
            let updated = store
                .try_fail_execution_request(&request.request_id, payload_json, finished_at)
                .await?;
            if !updated {
                return Err(QuantixError::Other(format!(
                    "request 状态已变化: {}",
                    request.request_id
                )));
            }
            Err(err)
        }
    }
}
