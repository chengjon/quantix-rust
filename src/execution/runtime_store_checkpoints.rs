use sqlx::sqlite::SqlitePool;

use crate::core::Result;
use crate::execution::models::{RunnerCheckpointRecord, StrategyDaemonCheckpointRecord};

use super::runtime_store_rows::{row_to_checkpoint, row_to_daemon_checkpoint};

pub(crate) async fn upsert_checkpoint(
    pool: &SqlitePool,
    checkpoint: &RunnerCheckpointRecord,
) -> Result<()> {
    sqlx::query(
        r#"
INSERT INTO runner_checkpoints (
    checkpoint_id,
    strategy_name,
    mode,
    symbol,
    timeframe,
    last_processed_bar,
    last_run_id,
    state_json,
    updated_at
) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
ON CONFLICT(strategy_name, mode, symbol, timeframe) DO UPDATE SET
    checkpoint_id = excluded.checkpoint_id,
    last_processed_bar = excluded.last_processed_bar,
    last_run_id = excluded.last_run_id,
    state_json = excluded.state_json,
    updated_at = excluded.updated_at
"#,
    )
    .bind(&checkpoint.checkpoint_id)
    .bind(&checkpoint.strategy_name)
    .bind(&checkpoint.mode)
    .bind(&checkpoint.symbol)
    .bind(&checkpoint.timeframe)
    .bind(checkpoint.last_processed_bar.map(|value| value.to_rfc3339()))
    .bind(&checkpoint.last_run_id)
    .bind(serde_json::to_string(&checkpoint.state_json)?)
    .bind(checkpoint.updated_at.to_rfc3339())
    .execute(pool)
    .await?;

    Ok(())
}

pub(crate) async fn load_checkpoint(
    pool: &SqlitePool,
    strategy_name: &str,
    mode: &str,
    symbol: &str,
    timeframe: &str,
) -> Result<Option<RunnerCheckpointRecord>> {
    let row = sqlx::query(
        r#"
SELECT
    checkpoint_id,
    strategy_name,
    mode,
    symbol,
    timeframe,
    last_processed_bar,
    last_run_id,
    state_json,
    updated_at
FROM runner_checkpoints
WHERE strategy_name = ? AND mode = ? AND symbol = ? AND timeframe = ?
"#,
    )
    .bind(strategy_name)
    .bind(mode)
    .bind(symbol)
    .bind(timeframe)
    .fetch_optional(pool)
    .await?;

    row.map(row_to_checkpoint).transpose()
}

pub(crate) async fn upsert_daemon_checkpoint(
    pool: &SqlitePool,
    checkpoint: &StrategyDaemonCheckpointRecord,
) -> Result<()> {
    sqlx::query(
        r#"
INSERT INTO strategy_daemon_checkpoints (
    checkpoint_id,
    strategy_instance_id,
    strategy_name,
    symbol,
    timeframe,
    last_processed_bar,
    last_run_id,
    state_json,
    updated_at
) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
ON CONFLICT(strategy_instance_id, symbol, timeframe) DO UPDATE SET
    checkpoint_id = excluded.checkpoint_id,
    strategy_name = excluded.strategy_name,
    last_processed_bar = excluded.last_processed_bar,
    last_run_id = excluded.last_run_id,
    state_json = excluded.state_json,
    updated_at = excluded.updated_at
"#,
    )
    .bind(&checkpoint.checkpoint_id)
    .bind(&checkpoint.strategy_instance_id)
    .bind(&checkpoint.strategy_name)
    .bind(&checkpoint.symbol)
    .bind(&checkpoint.timeframe)
    .bind(checkpoint.last_processed_bar.map(|value| value.to_rfc3339()))
    .bind(&checkpoint.last_run_id)
    .bind(serde_json::to_string(&checkpoint.state_json)?)
    .bind(checkpoint.updated_at.to_rfc3339())
    .execute(pool)
    .await?;

    Ok(())
}

pub(crate) async fn find_daemon_checkpoint(
    pool: &SqlitePool,
    strategy_instance_id: &str,
    symbol: &str,
    timeframe: &str,
) -> Result<Option<StrategyDaemonCheckpointRecord>> {
    let row = sqlx::query(
        r#"
SELECT
    checkpoint_id,
    strategy_instance_id,
    strategy_name,
    symbol,
    timeframe,
    last_processed_bar,
    last_run_id,
    state_json,
    updated_at
FROM strategy_daemon_checkpoints
WHERE strategy_instance_id = ? AND symbol = ? AND timeframe = ?
"#,
    )
    .bind(strategy_instance_id)
    .bind(symbol)
    .bind(timeframe)
    .fetch_optional(pool)
    .await?;

    row.map(row_to_daemon_checkpoint).transpose()
}
