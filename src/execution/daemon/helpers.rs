use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

use crate::core::{QuantixError, Result};
use crate::execution::kernel::PreparedExecutionRequest;
use crate::execution::models::{ExecutionRequestRecord, OrderIntent, OrderSide, OrderType};
use crate::risk::{RiskAccountSnapshot, RiskService, RiskStore};
use crate::trade::{PaperTradeAccount, PaperTradeStore, TradeOrderRequest};

pub(super) fn merge_execution_request_payload(
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

pub(super) fn build_prepared_request_from_execution_request(
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
            "buy" => Some(crate::core::signal::Signal::Buy),
            "sell" => Some(crate::core::signal::Signal::Sell),
            "hold" => Some(crate::core::signal::Signal::Hold),
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

pub(super) async fn load_initialized_trade_account<TradeStore>(
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

pub(super) fn build_risk_account_snapshot(account: &PaperTradeAccount) -> RiskAccountSnapshot {
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

pub(super) fn build_projected_buy_impact(
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

pub(super) async fn sync_risk_from_trade_store<TradeStore, RiskStoreImpl>(
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

pub(super) fn decimal_to_f64(value: Decimal, command_name: &str) -> Result<f64> {
    use rust_decimal::prelude::ToPrimitive;

    value
        .to_f64()
        .ok_or_else(|| QuantixError::Other(format!("{command_name} 无法将价格 {value} 转换为 f64")))
}

pub(super) fn remap_trade_request_error(err: QuantixError, command_name: &str) -> QuantixError {
    match err {
        QuantixError::Other(message) => QuantixError::Other(format!("{command_name} {message}")),
        other => other,
    }
}
