use sqlx::Row;
use sqlx::sqlite::SqliteRow;

use crate::core::{QuantixError, Result};
use crate::execution::models::{
    ApprovalStatus, ExecutionRequestRecord, ExecutionRequestStatus, OrderEventRecord, OrderRecord,
    OrderSide, OrderStatus, OrderType, RunnerCheckpointRecord, SignalStatus,
    StrategyDaemonCheckpointRecord, StrategyRunRecord, StrategyRunStatus, StrategySignalRecord,
};

use super::runtime_store_codec::{parse_decimal, parse_timestamp};

pub(crate) fn row_to_order(row: SqliteRow) -> Result<OrderRecord> {
    let side: String = row.try_get("side")?;
    let order_type: String = row.try_get("order_type")?;
    let requested_price: String = row.try_get("requested_price")?;
    let avg_fill_price: Option<String> = row.try_get("avg_fill_price")?;
    let status: String = row.try_get("status")?;
    let created_at: String = row.try_get("created_at")?;
    let updated_at: String = row.try_get("updated_at")?;
    let last_transition_at: String = row.try_get("last_transition_at")?;
    let payload_json: String = row.try_get("payload_json")?;

    Ok(OrderRecord {
        order_id: row.try_get("order_id")?,
        client_order_id: row.try_get("client_order_id")?,
        run_id: row.try_get("run_id")?,
        symbol: row.try_get("symbol")?,
        side: OrderSide::from_str(&side)
            .ok_or_else(|| QuantixError::DataParse(format!("invalid order side: {side}")))?,
        order_type: OrderType::from_str(&order_type)
            .ok_or_else(|| QuantixError::DataParse(format!("invalid order type: {order_type}")))?,
        requested_quantity: row.try_get("requested_quantity")?,
        requested_price: parse_decimal(&requested_price)?,
        filled_quantity: row.try_get("filled_quantity")?,
        remaining_quantity: row.try_get("remaining_quantity")?,
        avg_fill_price: avg_fill_price.as_deref().map(parse_decimal).transpose()?,
        status: OrderStatus::from_str(&status)
            .ok_or_else(|| QuantixError::DataParse(format!("invalid order status: {status}")))?,
        adapter: row.try_get("adapter")?,
        created_at: parse_timestamp(&created_at)?,
        updated_at: parse_timestamp(&updated_at)?,
        last_transition_at: parse_timestamp(&last_transition_at)?,
        version: row.try_get("version")?,
        payload_json: serde_json::from_str(&payload_json)?,
    })
}

pub(crate) fn row_to_run(row: SqliteRow) -> Result<StrategyRunRecord> {
    let status: String = row.try_get("status")?;
    let bar_end: String = row.try_get("bar_end")?;
    let started_at: String = row.try_get("started_at")?;
    let finished_at: Option<String> = row.try_get("finished_at")?;
    let metadata_json: String = row.try_get("metadata_json")?;

    Ok(StrategyRunRecord {
        run_id: row.try_get("run_id")?,
        strategy_name: row.try_get("strategy_name")?,
        mode: row.try_get("mode")?,
        trigger: row.try_get("trigger_type")?,
        status: StrategyRunStatus::from_str(&status).ok_or_else(|| {
            QuantixError::DataParse(format!("invalid strategy run status: {status}"))
        })?,
        symbol: row.try_get("symbol")?,
        timeframe: row.try_get("timeframe")?,
        bar_end: parse_timestamp(&bar_end)?,
        started_at: parse_timestamp(&started_at)?,
        finished_at: finished_at.as_deref().map(parse_timestamp).transpose()?,
        metadata_json: serde_json::from_str(&metadata_json)?,
    })
}

pub(crate) fn row_to_checkpoint(row: SqliteRow) -> Result<RunnerCheckpointRecord> {
    let last_processed_bar: Option<String> = row.try_get("last_processed_bar")?;
    let state_json: String = row.try_get("state_json")?;
    let updated_at: String = row.try_get("updated_at")?;

    Ok(RunnerCheckpointRecord {
        checkpoint_id: row.try_get("checkpoint_id")?,
        strategy_name: row.try_get("strategy_name")?,
        mode: row.try_get("mode")?,
        symbol: row.try_get("symbol")?,
        timeframe: row.try_get("timeframe")?,
        last_processed_bar: last_processed_bar
            .as_deref()
            .map(parse_timestamp)
            .transpose()?,
        last_run_id: row.try_get("last_run_id")?,
        state_json: serde_json::from_str(&state_json)?,
        updated_at: parse_timestamp(&updated_at)?,
    })
}

pub(crate) fn row_to_order_event(row: SqliteRow) -> Result<OrderEventRecord> {
    let event_time: String = row.try_get("event_time")?;
    let details_json: String = row.try_get("details_json")?;

    Ok(OrderEventRecord {
        event_id: row.try_get("event_id")?,
        order_id: row.try_get("order_id")?,
        client_order_id: row.try_get("client_order_id")?,
        event_type: row.try_get("event_type")?,
        event_time: parse_timestamp(&event_time)?,
        details_json: serde_json::from_str(&details_json)?,
    })
}

pub(crate) fn row_to_signal(row: SqliteRow) -> Result<StrategySignalRecord> {
    let bar_end: String = row.try_get("bar_end")?;
    let signal_status: String = row.try_get("signal_status")?;
    let approval_status: String = row.try_get("approval_status")?;
    let metadata_json: String = row.try_get("metadata_json")?;
    let created_at: String = row.try_get("created_at")?;
    let updated_at: String = row.try_get("updated_at")?;

    Ok(StrategySignalRecord {
        signal_id: row.try_get("signal_id")?,
        strategy_instance_id: row.try_get("strategy_instance_id")?,
        strategy_name: row.try_get("strategy_name")?,
        symbol: row.try_get("symbol")?,
        timeframe: row.try_get("timeframe")?,
        bar_end: parse_timestamp(&bar_end)?,
        signal_value: row.try_get("signal_value")?,
        signal_status: SignalStatus::from_str(&signal_status)
            .ok_or_else(|| QuantixError::DataParse(format!("invalid signal status: {signal_status}")))?,
        approval_status: ApprovalStatus::from_str(&approval_status)
            .ok_or_else(|| QuantixError::DataParse(format!("invalid approval status: {approval_status}")))?,
        run_id: row.try_get("run_id")?,
        metadata_json: serde_json::from_str(&metadata_json)?,
        created_at: parse_timestamp(&created_at)?,
        updated_at: parse_timestamp(&updated_at)?,
    })
}

pub(crate) fn row_to_execution_request(row: SqliteRow) -> Result<ExecutionRequestRecord> {
    let request_status: String = row.try_get("request_status")?;
    let created_at: String = row.try_get("created_at")?;
    let updated_at: String = row.try_get("updated_at")?;
    let payload_json: String = row.try_get("payload_json")?;

    Ok(ExecutionRequestRecord {
        request_id: row.try_get("request_id")?,
        signal_id: row.try_get("signal_id")?,
        target_mode: row.try_get("target_mode")?,
        target_account: row.try_get("target_account")?,
        request_status: ExecutionRequestStatus::from_str(&request_status)
            .ok_or_else(|| QuantixError::DataParse(format!("invalid request status: {request_status}")))?,
        approved_by: row.try_get("approved_by")?,
        created_at: parse_timestamp(&created_at)?,
        updated_at: parse_timestamp(&updated_at)?,
        payload_json: serde_json::from_str(&payload_json)?,
    })
}

pub(crate) fn row_to_daemon_checkpoint(row: SqliteRow) -> Result<StrategyDaemonCheckpointRecord> {
    let last_processed_bar: Option<String> = row.try_get("last_processed_bar")?;
    let state_json: String = row.try_get("state_json")?;
    let updated_at: String = row.try_get("updated_at")?;

    Ok(StrategyDaemonCheckpointRecord {
        checkpoint_id: row.try_get("checkpoint_id")?,
        strategy_instance_id: row.try_get("strategy_instance_id")?,
        strategy_name: row.try_get("strategy_name")?,
        symbol: row.try_get("symbol")?,
        timeframe: row.try_get("timeframe")?,
        last_processed_bar: last_processed_bar
            .as_deref()
            .map(parse_timestamp)
            .transpose()?,
        last_run_id: row.try_get("last_run_id")?,
        state_json: serde_json::from_str(&state_json)?,
        updated_at: parse_timestamp(&updated_at)?,
    })
}
