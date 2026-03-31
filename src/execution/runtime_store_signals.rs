use sqlx::sqlite::SqlitePool;

use crate::core::Result;
use crate::execution::models::{SignalEventRecord, StrategySignalRecord};

use super::runtime_store_rows::row_to_signal;

pub(crate) async fn insert_signal_event(
    pool: &SqlitePool,
    event: &SignalEventRecord,
) -> Result<()> {
    sqlx::query(
        r#"
INSERT INTO signal_events (
    event_id,
    run_id,
    strategy_name,
    symbol,
    signal,
    ts,
    payload_json
) VALUES (?, ?, ?, ?, ?, ?, ?)
"#,
    )
    .bind(&event.event_id)
    .bind(&event.run_id)
    .bind(&event.strategy_name)
    .bind(&event.symbol)
    .bind(&event.signal)
    .bind(event.ts.to_rfc3339())
    .bind(serde_json::to_string(&event.payload_json)?)
    .execute(pool)
    .await?;

    Ok(())
}

pub(crate) async fn insert_signal(pool: &SqlitePool, signal: &StrategySignalRecord) -> Result<()> {
    sqlx::query(
        r#"
INSERT INTO signals (
    signal_id,
    strategy_instance_id,
    strategy_name,
    symbol,
    timeframe,
    bar_end,
    signal_value,
    signal_status,
    approval_status,
    run_id,
    metadata_json,
    created_at,
    updated_at
) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
"#,
    )
    .bind(&signal.signal_id)
    .bind(&signal.strategy_instance_id)
    .bind(&signal.strategy_name)
    .bind(&signal.symbol)
    .bind(&signal.timeframe)
    .bind(signal.bar_end.to_rfc3339())
    .bind(&signal.signal_value)
    .bind(signal.signal_status.as_str())
    .bind(signal.approval_status.as_str())
    .bind(&signal.run_id)
    .bind(serde_json::to_string(&signal.metadata_json)?)
    .bind(signal.created_at.to_rfc3339())
    .bind(signal.updated_at.to_rfc3339())
    .execute(pool)
    .await?;

    Ok(())
}

pub(crate) async fn get_signal(
    pool: &SqlitePool,
    signal_id: &str,
) -> Result<Option<StrategySignalRecord>> {
    let row = sqlx::query(
        r#"
SELECT
    signal_id,
    strategy_instance_id,
    strategy_name,
    symbol,
    timeframe,
    bar_end,
    signal_value,
    signal_status,
    approval_status,
    run_id,
    metadata_json,
    created_at,
    updated_at
FROM signals
WHERE signal_id = ?
"#,
    )
    .bind(signal_id)
    .fetch_optional(pool)
    .await?;

    row.map(row_to_signal).transpose()
}

pub(crate) async fn list_signals(pool: &SqlitePool) -> Result<Vec<StrategySignalRecord>> {
    let rows = sqlx::query(
        r#"
SELECT
    signal_id,
    strategy_instance_id,
    strategy_name,
    symbol,
    timeframe,
    bar_end,
    signal_value,
    signal_status,
    approval_status,
    run_id,
    metadata_json,
    created_at,
    updated_at
FROM signals
ORDER BY created_at DESC, signal_id DESC
"#,
    )
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(row_to_signal).collect()
}
