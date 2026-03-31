use std::path::Path;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::Row;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use crate::core::Result;
use crate::execution::models::{
    ExecutionRequestRecord, ExecutionRequestStatus, MockLiveOrderState, OrderEventRecord,
    OrderRecord, OrderStatus, RunnerCheckpointRecord, SignalEventRecord, SignalStatus,
    StrategyDaemonCheckpointRecord, StrategyRunRecord, StrategyRunStatus, StrategySignalRecord,
};
use super::runtime_store_rows::{row_to_run, row_to_signal};

#[derive(Debug, Clone)]
pub struct StrategyRuntimeStore {
    pool: SqlitePool,
}

impl StrategyRuntimeStore {
    pub async fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await?;
        let store = Self { pool };
        store.ensure_schema().await?;
        Ok(store)
    }

    async fn ensure_schema(&self) -> Result<()> {
        super::runtime_store_schema::ensure_runtime_store_schema(&self.pool).await
    }

    pub async fn has_table(&self, table_name: &str) -> Result<bool> {
        let exists = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(1) FROM sqlite_master WHERE type = 'table' AND name = ?",
        )
        .bind(table_name)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists > 0)
    }

    pub async fn insert_run(&self, run: &StrategyRunRecord) -> Result<()> {
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
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn find_run_by_dedupe_key(
        &self,
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
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_run).transpose()
    }

    pub async fn insert_signal_event(&self, event: &SignalEventRecord) -> Result<()> {
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
        .execute(&self.pool)
        .await?;

        Ok(())
    }

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

    pub async fn insert_order(&self, order: &OrderRecord) -> Result<()> {
        super::runtime_store_orders::insert_order(&self.pool, order).await
    }

    pub async fn insert_order_event(&self, event: &OrderEventRecord) -> Result<()> {
        super::runtime_store_orders::insert_order_event(&self.pool, event).await
    }

    pub async fn find_order_by_client_order_id(
        &self,
        client_order_id: &str,
    ) -> Result<Option<OrderRecord>> {
        super::runtime_store_orders::find_order_by_client_order_id(&self.pool, client_order_id)
            .await
    }

    pub async fn find_first_order_for_run(&self, run_id: &str) -> Result<Option<OrderRecord>> {
        super::runtime_store_orders::find_first_order_for_run(&self.pool, run_id).await
    }

    pub async fn update_run_status(
        &self,
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
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_order(
        &self,
        order_id: &str,
        status: OrderStatus,
        filled_quantity: i64,
        avg_fill_price: Option<Decimal>,
        updated_at: DateTime<Utc>,
    ) -> Result<()> {
        super::runtime_store_orders::update_order(
            &self.pool,
            order_id,
            status,
            filled_quantity,
            avg_fill_price,
            updated_at,
        )
        .await
    }

    pub async fn insert_mock_live_order_state(
        &self,
        order_id: &str,
        adapter_order_id: Option<&str>,
        state: &MockLiveOrderState,
    ) -> Result<()> {
        super::runtime_store_orders::insert_mock_live_order_state(
            &self.pool,
            order_id,
            adapter_order_id,
            state,
        )
        .await
    }

    pub async fn get_mock_live_order_state(
        &self,
        order_id: &str,
    ) -> Result<Option<MockLiveOrderState>> {
        super::runtime_store_orders::get_mock_live_order_state(&self.pool, order_id).await
    }

    pub async fn list_recoverable_mock_live_orders(&self) -> Result<Vec<OrderRecord>> {
        super::runtime_store_orders::list_recoverable_mock_live_orders(&self.pool).await
    }

    pub async fn update_mock_live_order_state(
        &self,
        order_id: &str,
        adapter_order_id: Option<&str>,
        state: &MockLiveOrderState,
    ) -> Result<()> {
        super::runtime_store_orders::update_mock_live_order_state(
            &self.pool,
            order_id,
            adapter_order_id,
            state,
        )
        .await
    }

    pub async fn try_update_order_with_version(
        &self,
        order_id: &str,
        expected_version: i64,
        status: OrderStatus,
        filled_quantity: i64,
        remaining_quantity: i64,
        avg_fill_price: Option<Decimal>,
        updated_at: DateTime<Utc>,
    ) -> Result<bool> {
        super::runtime_store_orders::try_update_order_with_version(
            &self.pool,
            order_id,
            expected_version,
            status,
            filled_quantity,
            remaining_quantity,
            avg_fill_price,
            updated_at,
        )
        .await
    }

    pub async fn upsert_checkpoint(&self, checkpoint: &RunnerCheckpointRecord) -> Result<()> {
        super::runtime_store_checkpoints::upsert_checkpoint(&self.pool, checkpoint).await
    }

    pub async fn load_checkpoint(
        &self,
        strategy_name: &str,
        mode: &str,
        symbol: &str,
        timeframe: &str,
    ) -> Result<Option<RunnerCheckpointRecord>> {
        super::runtime_store_checkpoints::load_checkpoint(
            &self.pool,
            strategy_name,
            mode,
            symbol,
            timeframe,
        )
        .await
    }

    pub async fn list_order_events(&self, order_id: &str) -> Result<Vec<OrderEventRecord>> {
        super::runtime_store_orders::list_order_events(&self.pool, order_id).await
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

        row.map(row_to_signal).transpose()
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

        rows.into_iter().map(row_to_signal).collect()
    }

    pub async fn approve_signal_and_create_request(
        &self,
        signal_id: &str,
        target_mode: &str,
        target_account: &str,
        approved_by: Option<&str>,
    ) -> Result<ExecutionRequestRecord> {
        super::runtime_store_requests::approve_signal_and_create_request(
            &self.pool,
            signal_id,
            target_mode,
            target_account,
            approved_by,
        )
        .await
    }

    pub async fn reject_signal(&self, signal_id: &str, reason: Option<&str>) -> Result<()> {
        super::runtime_store_requests::reject_signal(&self.pool, signal_id, reason).await
    }

    pub async fn list_execution_requests(
        &self,
        status: Option<ExecutionRequestStatus>,
    ) -> Result<Vec<ExecutionRequestRecord>> {
        super::runtime_store_requests::list_execution_requests(&self.pool, status).await
    }

    pub async fn get_execution_request_by_signal_id(
        &self,
        signal_id: &str,
    ) -> Result<Option<ExecutionRequestRecord>> {
        super::runtime_store_requests::get_execution_request_by_signal_id(&self.pool, signal_id)
            .await
    }

    pub async fn get_execution_request(
        &self,
        request_id: &str,
    ) -> Result<Option<ExecutionRequestRecord>> {
        super::runtime_store_requests::get_execution_request(&self.pool, request_id).await
    }

    pub async fn try_complete_execution_request(
        &self,
        request_id: &str,
        payload_json: serde_json::Value,
        updated_at: DateTime<Utc>,
    ) -> Result<bool> {
        super::runtime_store_requests::try_complete_execution_request(
            &self.pool,
            request_id,
            payload_json,
            updated_at,
        )
        .await
    }

    pub async fn try_fail_execution_request(
        &self,
        request_id: &str,
        payload_json: serde_json::Value,
        updated_at: DateTime<Utc>,
    ) -> Result<bool> {
        super::runtime_store_requests::try_fail_execution_request(
            &self.pool,
            request_id,
            payload_json,
            updated_at,
        )
        .await
    }

    pub async fn try_cancel_execution_request(
        &self,
        request_id: &str,
        payload_json: serde_json::Value,
        updated_at: DateTime<Utc>,
    ) -> Result<bool> {
        super::runtime_store_requests::try_cancel_execution_request(
            &self.pool,
            request_id,
            payload_json,
            updated_at,
        )
        .await
    }

    pub async fn try_start_execution_request(
        &self,
        request_id: &str,
        payload_json: serde_json::Value,
        updated_at: DateTime<Utc>,
    ) -> Result<bool> {
        super::runtime_store_requests::try_start_execution_request(
            &self.pool,
            request_id,
            payload_json,
            updated_at,
        )
        .await
    }

    pub async fn find_next_pending_execution_request(
        &self,
    ) -> Result<Option<ExecutionRequestRecord>> {
        super::runtime_store_requests::find_next_pending_execution_request(&self.pool).await
    }

    pub async fn supersede_previous_signals_and_cancel_pending_requests(
        &self,
        strategy_instance_id: &str,
        symbol: &str,
        timeframe: &str,
        current_signal_id: &str,
        current_bar_end: DateTime<Utc>,
    ) -> Result<usize> {
        super::runtime_store_requests::supersede_previous_signals_and_cancel_pending_requests(
            &self.pool,
            strategy_instance_id,
            symbol,
            timeframe,
            current_signal_id,
            current_bar_end,
        )
        .await
    }

    pub async fn upsert_daemon_checkpoint(
        &self,
        checkpoint: &StrategyDaemonCheckpointRecord,
    ) -> Result<()> {
        super::runtime_store_checkpoints::upsert_daemon_checkpoint(&self.pool, checkpoint).await
    }

    pub async fn find_daemon_checkpoint(
        &self,
        strategy_instance_id: &str,
        symbol: &str,
        timeframe: &str,
    ) -> Result<Option<StrategyDaemonCheckpointRecord>> {
        super::runtime_store_checkpoints::find_daemon_checkpoint(
            &self.pool,
            strategy_instance_id,
            symbol,
            timeframe,
        )
        .await
    }

    pub async fn count_runs(&self) -> Result<i64> {
        self.count_table_rows("strategy_runs").await
    }

    pub async fn count_orders(&self) -> Result<i64> {
        self.count_table_rows("orders").await
    }

    pub async fn count_signal_events(&self) -> Result<i64> {
        self.count_table_rows("signal_events").await
    }

    pub async fn count_signals(&self) -> Result<i64> {
        self.count_table_rows("signals").await
    }

    pub async fn record_daemon_signal_run(
        &self,
        run: &StrategyRunRecord,
        signal: &StrategySignalRecord,
        checkpoint: &StrategyDaemonCheckpointRecord,
    ) -> Result<usize> {
        let mut tx = self.pool.begin().await?;
        let now = Utc::now().to_rfc3339();

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
        .execute(&mut *tx)
        .await?;

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
        .execute(&mut *tx)
        .await?;

        let candidate_rows = sqlx::query(
            r#"
SELECT signal_id
FROM signals
WHERE strategy_instance_id = ?
  AND symbol = ?
  AND timeframe = ?
  AND signal_id <> ?
  AND signal_status = ?
  AND bar_end < ?
"#,
        )
        .bind(&signal.strategy_instance_id)
        .bind(&signal.symbol)
        .bind(&signal.timeframe)
        .bind(&signal.signal_id)
        .bind(SignalStatus::New.as_str())
        .bind(signal.bar_end.to_rfc3339())
        .fetch_all(&mut *tx)
        .await?;

        let signal_ids: Vec<String> = candidate_rows
            .into_iter()
            .map(|row| row.try_get::<String, _>("signal_id"))
            .collect::<std::result::Result<Vec<_>, _>>()?;

        for signal_id in &signal_ids {
            sqlx::query(
                r#"
UPDATE signals
SET signal_status = ?, updated_at = ?
WHERE signal_id = ?
"#,
            )
            .bind(SignalStatus::Superseded.as_str())
            .bind(&now)
            .bind(signal_id)
            .execute(&mut *tx)
            .await?;

            sqlx::query(
                r#"
UPDATE execution_requests
SET request_status = ?, updated_at = ?
WHERE signal_id = ? AND request_status = ?
"#,
            )
            .bind(ExecutionRequestStatus::Canceled.as_str())
            .bind(&now)
            .bind(signal_id)
            .bind(ExecutionRequestStatus::Pending.as_str())
            .execute(&mut *tx)
            .await?;
        }

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
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(signal_ids.len())
    }

    async fn count_table_rows(&self, table_name: &str) -> Result<i64> {
        let sql = format!("SELECT COUNT(1) FROM {table_name}");
        Ok(sqlx::query_scalar::<_, i64>(&sql)
            .fetch_one(&self.pool)
            .await?)
    }

    /// List all open orders (orders not in terminal state)
    pub async fn list_open_orders(&self) -> Result<Vec<OrderRecord>> {
        super::runtime_store_orders::list_open_orders(&self.pool).await
    }
}
