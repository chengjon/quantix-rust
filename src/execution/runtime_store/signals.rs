use super::*;

impl StrategyRuntimeStore {
    pub async fn insert_signal(&self, signal: &StrategySignalRecord) -> Result<()> {
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
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_signal(&self, signal_id: &str) -> Result<Option<StrategySignalRecord>> {
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
        .fetch_optional(&self.pool)
        .await?;

        row.map(Self::row_to_signal).transpose()
    }

    pub async fn list_signals(&self) -> Result<Vec<StrategySignalRecord>> {
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
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(Self::row_to_signal).collect()
    }

    pub async fn upsert_daemon_checkpoint(
        &self,
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
        .bind(
            checkpoint
                .last_processed_bar
                .map(|value| value.to_rfc3339()),
        )
        .bind(&checkpoint.last_run_id)
        .bind(serde_json::to_string(&checkpoint.state_json)?)
        .bind(checkpoint.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn find_daemon_checkpoint(
        &self,
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
        .fetch_optional(&self.pool)
        .await?;

        row.map(Self::row_to_daemon_checkpoint).transpose()
    }

    pub(super) fn row_to_signal(row: SqliteRow) -> Result<StrategySignalRecord> {
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
            signal_status: SignalStatus::from_str(&signal_status).ok_or_else(|| {
                QuantixError::DataParse(format!("invalid signal status: {signal_status}"))
            })?,
            approval_status: ApprovalStatus::from_str(&approval_status).ok_or_else(|| {
                QuantixError::DataParse(format!("invalid approval status: {approval_status}"))
            })?,
            run_id: row.try_get("run_id")?,
            metadata_json: serde_json::from_str(&metadata_json)?,
            created_at: parse_timestamp(&created_at)?,
            updated_at: parse_timestamp(&updated_at)?,
        })
    }

    pub(super) fn row_to_run(row: SqliteRow) -> Result<StrategyRunRecord> {
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

    pub(super) fn row_to_checkpoint(row: SqliteRow) -> Result<RunnerCheckpointRecord> {
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
}
