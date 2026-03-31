use chrono::{DateTime, Utc};
use sqlx::sqlite::SqlitePool;

use crate::core::Result;
use crate::execution::models::{StrategyRunRecord, StrategyRunStatus};

use super::runtime_store_rows::row_to_run;

pub(crate) async fn insert_run(pool: &SqlitePool, run: &StrategyRunRecord) -> Result<()> {
    sqlx::query(
        r#"
INSERT INTO strategy_runs (
    run_id,
    strategy_name,
    mode,
    trigger_type,
    status,
    symbol,
    timeframe,
    bar_end,
    started_at,
    finished_at,
    metadata_json
) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
"#,
    )
    .bind(&run.run_id)
    .bind(&run.strategy_name)
    .bind(&run.mode)
    .bind(&run.trigger)
    .bind(run.status.as_str())
    .bind(&run.symbol)
    .bind(&run.timeframe)
    .bind(run.bar_end.to_rfc3339())
    .bind(run.started_at.to_rfc3339())
    .bind(run.finished_at.map(|value| value.to_rfc3339()))
    .bind(serde_json::to_string(&run.metadata_json)?)
    .execute(pool)
    .await?;

    Ok(())
}

pub(crate) async fn find_run_by_dedupe_key(
    pool: &SqlitePool,
    strategy_name: &str,
    mode: &str,
    symbol: &str,
    timeframe: &str,
    bar_end: DateTime<Utc>,
) -> Result<Option<StrategyRunRecord>> {
    let row = sqlx::query(
        r#"
SELECT
    run_id,
    strategy_name,
    mode,
    trigger_type,
    status,
    symbol,
    timeframe,
    bar_end,
    started_at,
    finished_at,
    metadata_json
FROM strategy_runs
WHERE strategy_name = ? AND mode = ? AND symbol = ? AND timeframe = ? AND bar_end = ?
"#,
    )
    .bind(strategy_name)
    .bind(mode)
    .bind(symbol)
    .bind(timeframe)
    .bind(bar_end.to_rfc3339())
    .fetch_optional(pool)
    .await?;

    row.map(row_to_run).transpose()
}

pub(crate) async fn update_run_status(
    pool: &SqlitePool,
    run_id: &str,
    status: StrategyRunStatus,
    finished_at: Option<DateTime<Utc>>,
) -> Result<()> {
    sqlx::query(
        r#"
UPDATE strategy_runs
SET status = ?, finished_at = ?
WHERE run_id = ?
"#,
    )
    .bind(status.as_str())
    .bind(finished_at.map(|value| value.to_rfc3339()))
    .bind(run_id)
    .execute(pool)
    .await?;

    Ok(())
}
