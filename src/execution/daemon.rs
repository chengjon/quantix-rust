use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

use crate::core::{QuantixError, Result};
use crate::execution::kernel::{
    ExecutionKernel, FillDeltaApplier, PreparedExecutionRequest, RiskDecision, RiskEvaluator,
};
use crate::execution::mock_live::{MockLiveExecutionAdapter, SystemMockLiveClock};
use crate::execution::models::{
    ExecutionRequestRecord, ExecutionRequestStatus, FillDeltaContext, FillDeltaResult, OrderIntent,
    OrderSide, OrderType,
};
use crate::execution::paper::PaperExecutionAdapter;
use crate::execution::runtime_store::StrategyRuntimeStore;
use crate::risk::{RiskAccountSnapshot, RiskService, RiskStore};
use crate::trade::{PaperTradeAccount, PaperTradeStore, TradeOrderRequest, TradeService};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionDaemonIterationSummary {
    pub claimed: usize,
    pub completed: usize,
    pub failed: usize,
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
struct RequestRiskBridge<TradeStore, RiskStoreImpl> {
    trade_store: TradeStore,
    risk_service: RiskService<RiskStoreImpl>,
}

impl<TradeStore, RiskStoreImpl> RequestRiskBridge<TradeStore, RiskStoreImpl> {
    fn new(trade_store: TradeStore, risk_service: RiskService<RiskStoreImpl>) -> Self {
        Self {
            trade_store,
            risk_service,
        }
    }
}

#[async_trait]
impl<TradeStore, RiskStoreImpl> RiskEvaluator for RequestRiskBridge<TradeStore, RiskStoreImpl>
where
    TradeStore: PaperTradeStore,
    RiskStoreImpl: RiskStore,
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

        match self
            .risk_service
            .check_buy(&snapshot, &projected_buy, Utc::now())
            .await
        {
            Ok(()) => Ok(RiskDecision::Allow),
            Err(QuantixError::Other(reason)) => Ok(RiskDecision::Reject { reason }),
            Err(other) => Err(other),
        }
    }

    async fn sync_after_fill(&self) -> Result<()> {
        sync_risk_from_trade_store(&self.trade_store, &self.risk_service).await
    }
}

pub async fn consume_next_pending_request_with_components<TS, RS>(
    store: &StrategyRuntimeStore,
    trade_store: TS,
    risk_store: RS,
) -> Result<ExecutionDaemonIterationSummary>
where
    TS: PaperTradeStore + Clone,
    RS: RiskStore + Clone,
{
    let Some(request) = store.find_next_pending_execution_request().await? else {
        return Ok(ExecutionDaemonIterationSummary {
            claimed: 0,
            completed: 0,
            failed: 0,
        });
    };

    match execute_request_by_id_with_components(store, &request.request_id, trade_store, risk_store)
        .await
    {
        Ok(_) => Ok(ExecutionDaemonIterationSummary {
            claimed: 1,
            completed: 1,
            failed: 0,
        }),
        Err(err) => {
            if store
                .get_execution_request(&request.request_id)
                .await?
                .is_some_and(|saved| saved.request_status == ExecutionRequestStatus::Failed)
            {
                Ok(ExecutionDaemonIterationSummary {
                    claimed: 1,
                    completed: 0,
                    failed: 1,
                })
            } else {
                Err(err)
            }
        }
    }
}

pub async fn execute_request_by_id_with_components<TS, RS>(
    store: &StrategyRuntimeStore,
    request_id: &str,
    trade_store: TS,
    risk_store: RS,
) -> Result<ExecutionRequestRecord>
where
    TS: PaperTradeStore + Clone,
    RS: RiskStore + Clone,
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
    let risk_service = RiskService::new(risk_store.clone());
    let risk = RequestRiskBridge::new(trade_store.clone(), risk_service);

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
        "live" => Err(QuantixError::Unsupported(
            "execution daemon live 模式尚未实现".to_string(),
        )),
        other => Err(QuantixError::Unsupported(format!(
            "execution daemon {other} 模式尚未实现"
        ))),
    };

    let finished_at = Utc::now();
    match execution_result {
        Ok(result) => {
            let payload_json = merge_execution_request_payload(
                &request.payload_json,
                "execution_result",
                serde_json::json!({
                    "executed_at": finished_at.to_rfc3339(),
                    "run_id": result.run_id,
                    "client_order_id": result.client_order_id,
                    "order_status": result.order_status.map(|status| status.as_str()),
                    "adapter": request.target_mode,
                }),
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

fn merge_execution_request_payload(
    original: &serde_json::Value,
    key: &str,
    value: serde_json::Value,
) -> serde_json::Value {
    let mut payload = match original {
        serde_json::Value::Object(map) => serde_json::Value::Object(map.clone()),
        _ => serde_json::json!({}),
    };
    payload[key] = value;
    payload
}

fn build_prepared_request_from_execution_request(
    request: &ExecutionRequestRecord,
) -> Result<PreparedExecutionRequest> {
    let snapshot = request
        .payload_json
        .get("execution_snapshot")
        .ok_or_else(|| {
            QuantixError::Other(format!(
                "request 缺少 execution_snapshot: {}",
                request.request_id
            ))
        })?;
    let order_intent = snapshot.get("order_intent").ok_or_else(|| {
        QuantixError::Other(format!("request 缺少 order_intent: {}", request.request_id))
    })?;

    let side = order_intent
        .get("side")
        .and_then(|value| value.as_str())
        .and_then(OrderSide::from_str)
        .ok_or_else(|| {
            QuantixError::Other(format!(
                "request order_intent.side 无效: {}",
                request.request_id
            ))
        })?;
    let order_type = order_intent
        .get("order_type")
        .and_then(|value| value.as_str())
        .and_then(OrderType::from_str)
        .ok_or_else(|| {
            QuantixError::Other(format!(
                "request order_intent.order_type 无效: {}",
                request.request_id
            ))
        })?;
    let requested_price = order_intent
        .get("requested_price")
        .and_then(|value| value.as_str())
        .ok_or_else(|| {
            QuantixError::Other(format!(
                "request order_intent.requested_price 缺失: {}",
                request.request_id
            ))
        })
        .and_then(|value| {
            Decimal::from_str_exact(value).map_err(|_| {
                QuantixError::Other(format!(
                    "request order_intent.requested_price 无效: {}",
                    request.request_id
                ))
            })
        })?;
    let signal = snapshot
        .get("signal_value")
        .and_then(|value| value.as_str())
        .and_then(|value| match value {
            "buy" => Some(crate::strategy::trait_def::Signal::Buy),
            "sell" => Some(crate::strategy::trait_def::Signal::Sell),
            "hold" => Some(crate::strategy::trait_def::Signal::Hold),
            _ => None,
        })
        .ok_or_else(|| {
            QuantixError::Other(format!("request signal_value 无效: {}", request.request_id))
        })?;
    let bar_end = snapshot
        .get("bar_end")
        .and_then(|value| value.as_str())
        .ok_or_else(|| QuantixError::Other(format!("request bar_end 缺失: {}", request.request_id)))
        .and_then(|value| {
            DateTime::parse_from_rfc3339(value)
                .map(|ts| ts.with_timezone(&Utc))
                .map_err(|_| {
                    QuantixError::Other(format!("request bar_end 无效: {}", request.request_id))
                })
        })?;
    let symbol = snapshot
        .get("symbol")
        .and_then(|value| value.as_str())
        .ok_or_else(|| QuantixError::Other(format!("request symbol 缺失: {}", request.request_id)))?
        .to_string();

    Ok(PreparedExecutionRequest {
        run_id: uuid::Uuid::new_v4().to_string(),
        strategy_name: snapshot
            .get("strategy_name")
            .and_then(|value| value.as_str())
            .unwrap_or("ma_cross")
            .to_string(),
        mode: request.target_mode.clone(),
        trigger: "request".to_string(),
        symbol: symbol.clone(),
        timeframe: snapshot
            .get("timeframe")
            .and_then(|value| value.as_str())
            .unwrap_or("1d")
            .to_string(),
        bar_end,
        signal,
        signal_payload_json: serde_json::json!({
            "request_id": request.request_id,
            "signal_id": request.signal_id,
        }),
        intent: OrderIntent {
            symbol: symbol.clone(),
            side,
            requested_quantity: order_intent
                .get("requested_quantity")
                .and_then(|value| value.as_i64())
                .ok_or_else(|| {
                    QuantixError::Other(format!(
                        "request requested_quantity 缺失: {}",
                        request.request_id
                    ))
                })?,
            requested_price,
            order_type,
            reason: order_intent
                .get("reason")
                .and_then(|value| value.as_str())
                .unwrap_or("signal_request")
                .to_string(),
            policy_snapshot_json: order_intent
                .get("policy_snapshot")
                .cloned()
                .unwrap_or(serde_json::json!({})),
        },
        client_order_id: format!("{}_{}_1", request.request_id, symbol),
    })
}

async fn load_initialized_trade_account<TradeStore>(
    trade_store: &TradeStore,
) -> Result<PaperTradeAccount>
where
    TradeStore: PaperTradeStore,
{
    let state = trade_store.load_state().await?.ok_or_else(|| {
        QuantixError::Other("trade account 尚未初始化，请先运行 trade init".to_string())
    })?;
    state.account.ok_or_else(|| {
        QuantixError::Other("trade account 尚未初始化，请先运行 trade init".to_string())
    })
}

fn build_risk_account_snapshot(account: &PaperTradeAccount) -> RiskAccountSnapshot {
    let positions: Vec<(String, Decimal)> = account
        .positions
        .values()
        .map(|position| {
            (
                position.code.clone(),
                Decimal::from(position.volume) * position.last_trade_price,
            )
        })
        .collect();
    let position_value = positions
        .iter()
        .fold(Decimal::ZERO, |acc, (_, value)| acc + *value);

    RiskAccountSnapshot::new(
        account.account_id.clone(),
        account.available_cash + position_value,
        positions,
    )
}

fn build_projected_buy_impact(
    account: &PaperTradeAccount,
    request: &TradeOrderRequest,
) -> crate::risk::ProjectedBuyImpact {
    let current_position_value = account
        .positions
        .get(&request.code)
        .map(|position| Decimal::from(position.volume) * position.last_trade_price)
        .unwrap_or(Decimal::ZERO);

    crate::risk::ProjectedBuyImpact::new(
        request.code.clone(),
        current_position_value + request.price * Decimal::from(request.volume),
        build_risk_account_snapshot(account).total_assets,
    )
}

async fn sync_risk_from_trade_store<TradeStore, RiskStoreImpl>(
    trade_store: &TradeStore,
    risk_service: &RiskService<RiskStoreImpl>,
) -> Result<()>
where
    TradeStore: PaperTradeStore,
    RiskStoreImpl: RiskStore,
{
    let account = load_initialized_trade_account(trade_store).await?;
    let snapshot = build_risk_account_snapshot(&account);
    risk_service
        .sync_after_trade_snapshot(&snapshot, Utc::now())
        .await?;
    Ok(())
}

fn decimal_to_f64(value: Decimal, command_name: &str) -> Result<f64> {
    use rust_decimal::prelude::ToPrimitive;

    value
        .to_f64()
        .ok_or_else(|| QuantixError::Other(format!("{command_name} 无法将价格 {value} 转换为 f64")))
}

fn remap_trade_request_error(err: QuantixError, command_name: &str) -> QuantixError {
    match err {
        QuantixError::Other(message) => QuantixError::Other(format!("{command_name} {message}")),
        other => other,
    }
}
