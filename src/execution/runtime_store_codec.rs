use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

use crate::core::{QuantixError, Result};
use crate::execution::models::{ExecutionPolicy, SignalEnvelope, StrategySignalRecord, translate_signal};
use crate::strategy::trait_def::Signal;

pub(crate) fn build_execution_snapshot(signal: &StrategySignalRecord) -> Result<serde_json::Value> {
    let market_price = signal
        .metadata_json
        .get("market_price")
        .and_then(|value| value.as_str())
        .map(parse_decimal)
        .transpose()?;
    let execution_policy = parse_execution_policy_from_metadata(&signal.metadata_json)?;
    let held_volume = signal
        .metadata_json
        .get("held_volume")
        .and_then(|value| value.as_i64());

    let order_intent = match (
        parse_signal_value(&signal.signal_value),
        market_price,
        execution_policy.clone(),
    ) {
        (Some(signal_value), Some(market_price), Some(policy)) => {
            let envelope = SignalEnvelope::new(signal_value);
            translate_signal(
                &envelope,
                &signal.symbol,
                market_price,
                held_volume,
                &policy,
            )
            .ok()
            .flatten()
            .map(order_intent_to_json)
            .unwrap_or(serde_json::Value::Null)
        }
        _ => serde_json::Value::Null,
    };

    Ok(serde_json::json!({
        "strategy_name": signal.strategy_name.clone(),
        "strategy_instance_id": signal.strategy_instance_id.clone(),
        "symbol": signal.symbol.clone(),
        "timeframe": signal.timeframe.clone(),
        "bar_end": signal.bar_end.to_rfc3339(),
        "signal_value": signal.signal_value.clone(),
        "market_price": signal.metadata_json.get("market_price").cloned().unwrap_or(serde_json::Value::Null),
        "execution_policy": execution_policy_to_json(execution_policy),
        "bar_source_id": signal.metadata_json.get("bar_source_id").cloned().unwrap_or(serde_json::Value::Null),
        "bar_source_fallback": signal.metadata_json.get("bar_source_fallback").cloned().unwrap_or(serde_json::Value::Null),
        "held_volume": held_volume,
        "order_intent": order_intent,
    }))
}

pub(crate) fn parse_execution_policy_from_metadata(
    metadata_json: &serde_json::Value,
) -> Result<Option<ExecutionPolicy>> {
    let Some(policy_json) = metadata_json.get("execution_policy") else {
        return Ok(None);
    };

    let fixed_cash = policy_json
        .get("fixed_cash_per_buy")
        .and_then(|value| value.as_str())
        .map(parse_decimal)
        .transpose()?;
    let slippage_bps = policy_json
        .get("slippage_bps")
        .and_then(|value| value.as_u64())
        .map(|value| value as u32)
        .unwrap_or(0);

    Ok(fixed_cash.map(|fixed_cash_per_buy| ExecutionPolicy {
        fixed_cash_per_buy,
        slippage_bps,
    }))
}

fn execution_policy_to_json(policy: Option<ExecutionPolicy>) -> serde_json::Value {
    match policy {
        Some(policy) => serde_json::json!({
            "fixed_cash_per_buy": policy.fixed_cash_per_buy.to_string(),
            "slippage_bps": policy.slippage_bps,
        }),
        None => serde_json::Value::Null,
    }
}

fn order_intent_to_json(intent: crate::execution::models::OrderIntent) -> serde_json::Value {
    serde_json::json!({
        "side": intent.side.as_str(),
        "requested_quantity": intent.requested_quantity,
        "requested_price": intent.requested_price.to_string(),
        "order_type": intent.order_type.as_str(),
        "reason": intent.reason,
        "policy_snapshot": intent.policy_snapshot_json,
    })
}

pub(crate) fn parse_signal_value(value: &str) -> Option<Signal> {
    match value {
        "buy" => Some(Signal::Buy),
        "sell" => Some(Signal::Sell),
        "hold" => Some(Signal::Hold),
        _ => None,
    }
}

pub(crate) fn parse_timestamp(value: &str) -> Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .map(|ts| ts.with_timezone(&Utc))
        .map_err(|err| QuantixError::DataParse(format!("invalid RFC3339 timestamp {value}: {err}")))
}

pub(crate) fn parse_decimal(value: &str) -> Result<Decimal> {
    Decimal::from_str_exact(value)
        .map_err(|err| QuantixError::DataParse(format!("invalid decimal {value}: {err}")))
}
