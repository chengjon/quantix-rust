use std::path::Path;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::Row;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions, SqliteRow};
use uuid::Uuid;

use crate::core::{QuantixError, Result};
use crate::execution::models::{
    ApprovalStatus, ExecutionPolicy, ExecutionRequestRecord, ExecutionRequestStatus,
    MockLiveOrderState, OrderEventRecord, OrderRecord, OrderSide, OrderStatus, OrderType,
    RunnerCheckpointRecord, SignalEnvelope, SignalEventRecord, SignalStatus,
    StrategyDaemonCheckpointRecord, StrategyRunRecord, StrategyRunStatus, StrategySignalRecord,
    translate_signal,
};
use crate::strategy::trait_def::Signal;

const CREATE_STRATEGY_RUNS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS strategy_runs (
    run_id TEXT PRIMARY KEY,
    strategy_name TEXT NOT NULL,
    mode TEXT NOT NULL,
    trigger_type TEXT NOT NULL,
    status TEXT NOT NULL,
    symbol TEXT NOT NULL,
    timeframe TEXT NOT NULL,
    bar_end TEXT NOT NULL,
    started_at TEXT NOT NULL,
    finished_at TEXT,
    metadata_json TEXT NOT NULL
);
"#;

const CREATE_STRATEGY_RUNS_DEDUPE_INDEX_SQL: &str = r#"
CREATE UNIQUE INDEX IF NOT EXISTS idx_strategy_runs_dedupe
ON strategy_runs(strategy_name, mode, symbol, timeframe, bar_end);
"#;

const CREATE_SIGNAL_EVENTS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS signal_events (
    event_id TEXT PRIMARY KEY,
    run_id TEXT NOT NULL,
    strategy_name TEXT NOT NULL,
    symbol TEXT NOT NULL,
    signal TEXT NOT NULL,
    ts TEXT NOT NULL,
    payload_json TEXT NOT NULL
);
"#;

const CREATE_SIGNAL_EVENTS_RUN_INDEX_SQL: &str = r#"
CREATE INDEX IF NOT EXISTS idx_signal_events_run_id
ON signal_events(run_id);
"#;

const CREATE_SIGNAL_EVENTS_SYMBOL_TS_INDEX_SQL: &str = r#"
CREATE INDEX IF NOT EXISTS idx_signal_events_symbol_ts
ON signal_events(symbol, ts);
"#;

const CREATE_ORDERS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS orders (
    order_id TEXT PRIMARY KEY,
    client_order_id TEXT NOT NULL UNIQUE,
    run_id TEXT NOT NULL,
    symbol TEXT NOT NULL,
    side TEXT NOT NULL,
    order_type TEXT NOT NULL,
    requested_quantity INTEGER NOT NULL,
    requested_price TEXT NOT NULL,
    filled_quantity INTEGER NOT NULL,
    remaining_quantity INTEGER NOT NULL DEFAULT 0,
    avg_fill_price TEXT,
    status TEXT NOT NULL,
    adapter TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    last_transition_at TEXT NOT NULL DEFAULT '',
    version INTEGER NOT NULL DEFAULT 0,
    payload_json TEXT NOT NULL
);
"#;

const CREATE_ORDERS_RUN_INDEX_SQL: &str = r#"
CREATE INDEX IF NOT EXISTS idx_orders_run_id
ON orders(run_id);
"#;

const CREATE_ORDERS_SYMBOL_STATUS_INDEX_SQL: &str = r#"
CREATE INDEX IF NOT EXISTS idx_orders_symbol_status
ON orders(symbol, status);
"#;

const CREATE_ORDER_EVENTS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS order_events (
    event_id TEXT PRIMARY KEY,
    order_id TEXT NOT NULL,
    client_order_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    event_time TEXT NOT NULL,
    details_json TEXT NOT NULL
);
"#;

const CREATE_ORDER_EVENTS_ORDER_INDEX_SQL: &str = r#"
CREATE INDEX IF NOT EXISTS idx_order_events_order_id
ON order_events(order_id);
"#;

const CREATE_ORDER_EVENTS_CLIENT_TIME_INDEX_SQL: &str = r#"
CREATE INDEX IF NOT EXISTS idx_order_events_client_time
ON order_events(client_order_id, event_time);
"#;

const CREATE_MOCK_LIVE_ORDERS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS mock_live_orders (
    order_id TEXT PRIMARY KEY,
    adapter_order_id TEXT,
    state_json TEXT NOT NULL
);
"#;

const CREATE_RUNNER_CHECKPOINTS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS runner_checkpoints (
    checkpoint_id TEXT PRIMARY KEY,
    strategy_name TEXT NOT NULL,
    mode TEXT NOT NULL,
    symbol TEXT NOT NULL,
    timeframe TEXT NOT NULL,
    last_processed_bar TEXT,
    last_run_id TEXT,
    state_json TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
"#;

const CREATE_RUNNER_CHECKPOINTS_UNIQUE_INDEX_SQL: &str = r#"
CREATE UNIQUE INDEX IF NOT EXISTS idx_runner_checkpoints_stream
ON runner_checkpoints(strategy_name, mode, symbol, timeframe);
"#;

const CREATE_SIGNALS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS signals (
    signal_id TEXT PRIMARY KEY,
    strategy_instance_id TEXT NOT NULL,
    strategy_name TEXT NOT NULL,
    symbol TEXT NOT NULL,
    timeframe TEXT NOT NULL,
    bar_end TEXT NOT NULL,
    signal_value TEXT NOT NULL,
    signal_status TEXT NOT NULL,
    approval_status TEXT NOT NULL,
    run_id TEXT NOT NULL,
    metadata_json TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
"#;

const CREATE_SIGNALS_UNIQUE_INDEX_SQL: &str = r#"
CREATE UNIQUE INDEX IF NOT EXISTS idx_signals_stream_bar
ON signals(strategy_instance_id, symbol, timeframe, bar_end);
"#;

const CREATE_SIGNALS_SYMBOL_BAR_INDEX_SQL: &str = r#"
CREATE INDEX IF NOT EXISTS idx_signals_symbol_bar
ON signals(symbol, bar_end);
"#;

const CREATE_SIGNALS_APPROVAL_INDEX_SQL: &str = r#"
CREATE INDEX IF NOT EXISTS idx_signals_approval
ON signals(approval_status);
"#;

const CREATE_SIGNALS_INSTANCE_APPROVAL_INDEX_SQL: &str = r#"
CREATE INDEX IF NOT EXISTS idx_signals_instance_approval
ON signals(strategy_instance_id, approval_status);
"#;

const CREATE_SIGNALS_INSTANCE_STATUS_INDEX_SQL: &str = r#"
CREATE INDEX IF NOT EXISTS idx_signals_instance_status
ON signals(strategy_instance_id, signal_status);
"#;

const CREATE_EXECUTION_REQUESTS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS execution_requests (
    request_id TEXT PRIMARY KEY,
    signal_id TEXT NOT NULL UNIQUE,
    target_mode TEXT NOT NULL,
    target_account TEXT NOT NULL,
    request_status TEXT NOT NULL,
    approved_by TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    payload_json TEXT NOT NULL
);
"#;

const CREATE_EXECUTION_REQUESTS_STATUS_INDEX_SQL: &str = r#"
CREATE INDEX IF NOT EXISTS idx_execution_requests_status
ON execution_requests(request_status);
"#;

const CREATE_EXECUTION_REQUESTS_TARGET_STATUS_INDEX_SQL: &str = r#"
CREATE INDEX IF NOT EXISTS idx_execution_requests_target_status
ON execution_requests(target_mode, request_status);
"#;

const CREATE_STRATEGY_DAEMON_CHECKPOINTS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS strategy_daemon_checkpoints (
    checkpoint_id TEXT PRIMARY KEY,
    strategy_instance_id TEXT NOT NULL,
    strategy_name TEXT NOT NULL,
    symbol TEXT NOT NULL,
    timeframe TEXT NOT NULL,
    last_processed_bar TEXT,
    last_run_id TEXT,
    state_json TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
"#;

const CREATE_STRATEGY_DAEMON_CHECKPOINTS_UNIQUE_INDEX_SQL: &str = r#"
CREATE UNIQUE INDEX IF NOT EXISTS idx_strategy_daemon_checkpoints_stream
ON strategy_daemon_checkpoints(strategy_instance_id, symbol, timeframe);
"#;

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
        for statement in [
            CREATE_STRATEGY_RUNS_TABLE_SQL,
            CREATE_STRATEGY_RUNS_DEDUPE_INDEX_SQL,
            CREATE_SIGNAL_EVENTS_TABLE_SQL,
            CREATE_SIGNAL_EVENTS_RUN_INDEX_SQL,
            CREATE_SIGNAL_EVENTS_SYMBOL_TS_INDEX_SQL,
            CREATE_ORDERS_TABLE_SQL,
            CREATE_ORDERS_RUN_INDEX_SQL,
            CREATE_ORDERS_SYMBOL_STATUS_INDEX_SQL,
            CREATE_ORDER_EVENTS_TABLE_SQL,
            CREATE_ORDER_EVENTS_ORDER_INDEX_SQL,
            CREATE_ORDER_EVENTS_CLIENT_TIME_INDEX_SQL,
            CREATE_MOCK_LIVE_ORDERS_TABLE_SQL,
            CREATE_RUNNER_CHECKPOINTS_TABLE_SQL,
            CREATE_RUNNER_CHECKPOINTS_UNIQUE_INDEX_SQL,
            CREATE_SIGNALS_TABLE_SQL,
            CREATE_SIGNALS_UNIQUE_INDEX_SQL,
            CREATE_SIGNALS_SYMBOL_BAR_INDEX_SQL,
            CREATE_SIGNALS_APPROVAL_INDEX_SQL,
            CREATE_SIGNALS_INSTANCE_APPROVAL_INDEX_SQL,
            CREATE_SIGNALS_INSTANCE_STATUS_INDEX_SQL,
            CREATE_EXECUTION_REQUESTS_TABLE_SQL,
            CREATE_EXECUTION_REQUESTS_STATUS_INDEX_SQL,
            CREATE_EXECUTION_REQUESTS_TARGET_STATUS_INDEX_SQL,
            CREATE_STRATEGY_DAEMON_CHECKPOINTS_TABLE_SQL,
            CREATE_STRATEGY_DAEMON_CHECKPOINTS_UNIQUE_INDEX_SQL,
        ] {
            sqlx::query(statement).execute(&self.pool).await?;
        }

        self.ensure_orders_schema_extensions().await?;

        Ok(())
    }

    async fn ensure_orders_schema_extensions(&self) -> Result<()> {
        self.ensure_column_exists(
            "orders",
            "remaining_quantity",
            "ALTER TABLE orders ADD COLUMN remaining_quantity INTEGER NOT NULL DEFAULT 0",
        )
        .await?;
        self.ensure_column_exists(
            "orders",
            "last_transition_at",
            "ALTER TABLE orders ADD COLUMN last_transition_at TEXT NOT NULL DEFAULT ''",
        )
        .await?;
        self.ensure_column_exists(
            "orders",
            "version",
            "ALTER TABLE orders ADD COLUMN version INTEGER NOT NULL DEFAULT 0",
        )
        .await?;
        sqlx::query(
            r#"
UPDATE orders
SET remaining_quantity = MAX(requested_quantity - filled_quantity, 0)
"#,
        )
        .execute(&self.pool)
        .await?;
        sqlx::query(
            r#"
UPDATE orders
SET last_transition_at = updated_at
WHERE last_transition_at = ''
"#,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn ensure_column_exists(
        &self,
        table_name: &str,
        column_name: &str,
        alter_sql: &str,
    ) -> Result<()> {
        let pragma_sql = format!("PRAGMA table_info({table_name})");
        let rows = sqlx::query(&pragma_sql).fetch_all(&self.pool).await?;
        let has_column = rows.iter().any(|row| {
            row.try_get::<String, _>("name")
                .map(|name| name == column_name)
                .unwrap_or(false)
        });
        if !has_column {
            sqlx::query(alter_sql).execute(&self.pool).await?;
        }
        Ok(())
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

        row.map(Self::row_to_run).transpose()
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
        sqlx::query(
            r#"
INSERT INTO orders (
    order_id,
    client_order_id,
    run_id,
    symbol,
    side,
    order_type,
    requested_quantity,
    requested_price,
    filled_quantity,
    remaining_quantity,
    avg_fill_price,
    status,
    adapter,
    created_at,
    updated_at,
    last_transition_at,
    version,
    payload_json
) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
"#,
        )
        .bind(&order.order_id)
        .bind(&order.client_order_id)
        .bind(&order.run_id)
        .bind(&order.symbol)
        .bind(order.side.as_str())
        .bind(order.order_type.as_str())
        .bind(order.requested_quantity)
        .bind(order.requested_price.to_string())
        .bind(order.filled_quantity)
        .bind(order.remaining_quantity)
        .bind(order.avg_fill_price.map(|value| value.to_string()))
        .bind(order.status.as_str())
        .bind(&order.adapter)
        .bind(order.created_at.to_rfc3339())
        .bind(order.updated_at.to_rfc3339())
        .bind(order.last_transition_at.to_rfc3339())
        .bind(order.version)
        .bind(serde_json::to_string(&order.payload_json)?)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn insert_order_event(&self, event: &OrderEventRecord) -> Result<()> {
        sqlx::query(
            r#"
INSERT INTO order_events (
    event_id,
    order_id,
    client_order_id,
    event_type,
    event_time,
    details_json
) VALUES (?, ?, ?, ?, ?, ?)
"#,
        )
        .bind(&event.event_id)
        .bind(&event.order_id)
        .bind(&event.client_order_id)
        .bind(&event.event_type)
        .bind(event.event_time.to_rfc3339())
        .bind(serde_json::to_string(&event.details_json)?)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn find_order_by_client_order_id(
        &self,
        client_order_id: &str,
    ) -> Result<Option<OrderRecord>> {
        let row = sqlx::query(
            r#"
SELECT
    order_id,
    client_order_id,
    run_id,
    symbol,
    side,
    order_type,
    requested_quantity,
    requested_price,
    filled_quantity,
    remaining_quantity,
    avg_fill_price,
    status,
    adapter,
    created_at,
    updated_at,
    last_transition_at,
    version,
    payload_json
FROM orders
WHERE client_order_id = ?
"#,
        )
        .bind(client_order_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(Self::row_to_order).transpose()
    }

    pub async fn find_first_order_for_run(&self, run_id: &str) -> Result<Option<OrderRecord>> {
        let row = sqlx::query(
            r#"
SELECT
    order_id,
    client_order_id,
    run_id,
    symbol,
    side,
    order_type,
    requested_quantity,
    requested_price,
    filled_quantity,
    remaining_quantity,
    avg_fill_price,
    status,
    adapter,
    created_at,
    updated_at,
    last_transition_at,
    version,
    payload_json
FROM orders
WHERE run_id = ?
ORDER BY created_at ASC, order_id ASC
LIMIT 1
"#,
        )
        .bind(run_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(Self::row_to_order).transpose()
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
        sqlx::query(
            r#"
UPDATE orders
SET status = ?,
    filled_quantity = ?,
    remaining_quantity = MAX(requested_quantity - ?, 0),
    avg_fill_price = ?,
    updated_at = ?,
    last_transition_at = ?,
    version = version + 1
WHERE order_id = ?
"#,
        )
        .bind(status.as_str())
        .bind(filled_quantity)
        .bind(filled_quantity)
        .bind(avg_fill_price.map(|value| value.to_string()))
        .bind(updated_at.to_rfc3339())
        .bind(updated_at.to_rfc3339())
        .bind(order_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn insert_mock_live_order_state(
        &self,
        order_id: &str,
        adapter_order_id: Option<&str>,
        state: &MockLiveOrderState,
    ) -> Result<()> {
        sqlx::query(
            r#"
INSERT INTO mock_live_orders (
    order_id,
    adapter_order_id,
    state_json
) VALUES (?, ?, ?)
"#,
        )
        .bind(order_id)
        .bind(adapter_order_id)
        .bind(serde_json::to_string(state)?)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_mock_live_order_state(
        &self,
        order_id: &str,
    ) -> Result<Option<MockLiveOrderState>> {
        let row = sqlx::query(
            r#"
SELECT state_json
FROM mock_live_orders
WHERE order_id = ?
"#,
        )
        .bind(order_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|row| -> Result<MockLiveOrderState> {
            let state_json: String = row.try_get("state_json")?;
            Ok(serde_json::from_str(&state_json)?)
        })
        .transpose()
    }

    pub async fn list_recoverable_mock_live_orders(&self) -> Result<Vec<OrderRecord>> {
        let rows = sqlx::query(
            r#"
SELECT
    o.order_id,
    o.client_order_id,
    o.run_id,
    o.symbol,
    o.side,
    o.order_type,
    o.requested_quantity,
    o.requested_price,
    o.filled_quantity,
    o.remaining_quantity,
    o.avg_fill_price,
    o.status,
    o.adapter,
    o.created_at,
    o.updated_at,
    o.last_transition_at,
    o.version,
    o.payload_json
FROM orders o
INNER JOIN mock_live_orders m ON m.order_id = o.order_id
WHERE o.status IN (?, ?, ?, ?, ?)
ORDER BY o.updated_at ASC, o.order_id ASC
"#,
        )
        .bind(OrderStatus::Submitted.as_str())
        .bind(OrderStatus::Accepted.as_str())
        .bind(OrderStatus::PartiallyFilled.as_str())
        .bind(OrderStatus::Unknown.as_str())
        .bind(OrderStatus::PendingCancel.as_str())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(Self::row_to_order).collect()
    }

    pub async fn update_mock_live_order_state(
        &self,
        order_id: &str,
        adapter_order_id: Option<&str>,
        state: &MockLiveOrderState,
    ) -> Result<()> {
        sqlx::query(
            r#"
UPDATE mock_live_orders
SET adapter_order_id = ?, state_json = ?
WHERE order_id = ?
"#,
        )
        .bind(adapter_order_id)
        .bind(serde_json::to_string(state)?)
        .bind(order_id)
        .execute(&self.pool)
        .await?;

        Ok(())
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
        let result = sqlx::query(
            r#"
UPDATE orders
SET status = ?,
    filled_quantity = ?,
    remaining_quantity = ?,
    avg_fill_price = ?,
    updated_at = ?,
    last_transition_at = ?,
    version = version + 1
WHERE order_id = ? AND version = ?
"#,
        )
        .bind(status.as_str())
        .bind(filled_quantity)
        .bind(remaining_quantity)
        .bind(avg_fill_price.map(|value| value.to_string()))
        .bind(updated_at.to_rfc3339())
        .bind(updated_at.to_rfc3339())
        .bind(order_id)
        .bind(expected_version)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() == 1)
    }

    pub async fn upsert_checkpoint(&self, checkpoint: &RunnerCheckpointRecord) -> Result<()> {
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

    pub async fn load_checkpoint(
        &self,
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
        .fetch_optional(&self.pool)
        .await?;

        row.map(Self::row_to_checkpoint).transpose()
    }

    pub async fn list_order_events(&self, order_id: &str) -> Result<Vec<OrderEventRecord>> {
        let rows = sqlx::query(
            r#"
SELECT
    event_id,
    order_id,
    client_order_id,
    event_type,
    event_time,
    details_json
FROM order_events
WHERE order_id = ?
ORDER BY event_time ASC, event_id ASC
"#,
        )
        .bind(order_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(Self::row_to_order_event).collect()
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

    pub async fn approve_signal_and_create_request(
        &self,
        signal_id: &str,
        target_mode: &str,
        target_account: &str,
        approved_by: Option<&str>,
    ) -> Result<ExecutionRequestRecord> {
        let mut tx = self.pool.begin().await?;
        let now = Utc::now();

        let signal_row = sqlx::query(
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
WHERE signal_id = ? AND signal_status = ? AND approval_status = ?
"#,
        )
        .bind(signal_id)
        .bind(SignalStatus::New.as_str())
        .bind(ApprovalStatus::Pending.as_str())
        .fetch_optional(&mut *tx)
        .await?;

        let Some(signal_row) = signal_row else {
            return Err(QuantixError::Other(format!("signal 不可审批: {signal_id}")));
        };
        let signal = Self::row_to_signal(signal_row)?;

        let update = sqlx::query(
            r#"
UPDATE signals
SET approval_status = ?, updated_at = ?
WHERE signal_id = ? AND signal_status = ? AND approval_status = ?
"#,
        )
        .bind(ApprovalStatus::Approved.as_str())
        .bind(now.to_rfc3339())
        .bind(signal_id)
        .bind(SignalStatus::New.as_str())
        .bind(ApprovalStatus::Pending.as_str())
        .execute(&mut *tx)
        .await?;

        if update.rows_affected() != 1 {
            return Err(QuantixError::Other(format!("signal 不可审批: {signal_id}")));
        }

        let record = ExecutionRequestRecord {
            request_id: Uuid::new_v4().to_string(),
            signal_id: signal_id.to_string(),
            target_mode: target_mode.to_string(),
            target_account: target_account.to_string(),
            request_status: ExecutionRequestStatus::Pending,
            approved_by: approved_by.map(str::to_string),
            created_at: now,
            updated_at: now,
            payload_json: serde_json::json!({
                "execution_snapshot": build_execution_snapshot(&signal)?,
            }),
        };

        sqlx::query(
            r#"
INSERT INTO execution_requests (
    request_id,
    signal_id,
    target_mode,
    target_account,
    request_status,
    approved_by,
    created_at,
    updated_at,
    payload_json
) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
"#,
        )
        .bind(&record.request_id)
        .bind(&record.signal_id)
        .bind(&record.target_mode)
        .bind(&record.target_account)
        .bind(record.request_status.as_str())
        .bind(&record.approved_by)
        .bind(record.created_at.to_rfc3339())
        .bind(record.updated_at.to_rfc3339())
        .bind(serde_json::to_string(&record.payload_json)?)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(record)
    }

    pub async fn reject_signal(&self, signal_id: &str, reason: Option<&str>) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
SELECT metadata_json
FROM signals
WHERE signal_id = ? AND signal_status = ? AND approval_status = ?
"#,
        )
        .bind(signal_id)
        .bind(SignalStatus::New.as_str())
        .bind(ApprovalStatus::Pending.as_str())
        .fetch_optional(&mut *tx)
        .await?;

        let Some(row) = row else {
            return Err(QuantixError::Other(format!("signal 不可拒绝: {signal_id}")));
        };

        let metadata_json: String = row.try_get("metadata_json")?;
        let mut metadata: serde_json::Value = serde_json::from_str(&metadata_json)?;
        if let Some(reason) = reason {
            metadata["rejection_reason"] = serde_json::Value::String(reason.to_string());
        }

        sqlx::query(
            r#"
UPDATE signals
SET approval_status = ?, metadata_json = ?, updated_at = ?
WHERE signal_id = ?
"#,
        )
        .bind(ApprovalStatus::Rejected.as_str())
        .bind(serde_json::to_string(&metadata)?)
        .bind(Utc::now().to_rfc3339())
        .bind(signal_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn list_execution_requests(
        &self,
        status: Option<ExecutionRequestStatus>,
    ) -> Result<Vec<ExecutionRequestRecord>> {
        let rows = if let Some(status) = status {
            sqlx::query(
                r#"
SELECT
    request_id,
    signal_id,
    target_mode,
    target_account,
    request_status,
    approved_by,
    created_at,
    updated_at,
    payload_json
FROM execution_requests
WHERE request_status = ?
ORDER BY created_at ASC, request_id ASC
"#,
            )
            .bind(status.as_str())
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query(
                r#"
SELECT
    request_id,
    signal_id,
    target_mode,
    target_account,
    request_status,
    approved_by,
    created_at,
    updated_at,
    payload_json
FROM execution_requests
ORDER BY created_at ASC, request_id ASC
"#,
            )
            .fetch_all(&self.pool)
            .await?
        };

        rows.into_iter()
            .map(Self::row_to_execution_request)
            .collect()
    }

    pub async fn get_execution_request_by_signal_id(
        &self,
        signal_id: &str,
    ) -> Result<Option<ExecutionRequestRecord>> {
        let row = sqlx::query(
            r#"
SELECT
    request_id,
    signal_id,
    target_mode,
    target_account,
    request_status,
    approved_by,
    created_at,
    updated_at,
    payload_json
FROM execution_requests
WHERE signal_id = ?
"#,
        )
        .bind(signal_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(Self::row_to_execution_request).transpose()
    }

    pub async fn get_execution_request(
        &self,
        request_id: &str,
    ) -> Result<Option<ExecutionRequestRecord>> {
        let row = sqlx::query(
            r#"
SELECT
    request_id,
    signal_id,
    target_mode,
    target_account,
    request_status,
    approved_by,
    created_at,
    updated_at,
    payload_json
FROM execution_requests
WHERE request_id = ?
"#,
        )
        .bind(request_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(Self::row_to_execution_request).transpose()
    }

    pub async fn try_complete_execution_request(
        &self,
        request_id: &str,
        payload_json: serde_json::Value,
        updated_at: DateTime<Utc>,
    ) -> Result<bool> {
        self.try_update_execution_request_status(
            request_id,
            ExecutionRequestStatus::InProgress,
            ExecutionRequestStatus::Completed,
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
        self.try_update_execution_request_status(
            request_id,
            ExecutionRequestStatus::InProgress,
            ExecutionRequestStatus::Failed,
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
        self.try_update_execution_request_status(
            request_id,
            ExecutionRequestStatus::Pending,
            ExecutionRequestStatus::Canceled,
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
        self.try_update_execution_request_status(
            request_id,
            ExecutionRequestStatus::Pending,
            ExecutionRequestStatus::InProgress,
            payload_json,
            updated_at,
        )
        .await
    }

    async fn try_update_execution_request_status(
        &self,
        request_id: &str,
        expected_status: ExecutionRequestStatus,
        target_status: ExecutionRequestStatus,
        payload_json: serde_json::Value,
        updated_at: DateTime<Utc>,
    ) -> Result<bool> {
        let result = sqlx::query(
            r#"
UPDATE execution_requests
SET request_status = ?, updated_at = ?, payload_json = ?
WHERE request_id = ? AND request_status = ?
"#,
        )
        .bind(target_status.as_str())
        .bind(updated_at.to_rfc3339())
        .bind(serde_json::to_string(&payload_json)?)
        .bind(request_id)
        .bind(expected_status.as_str())
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() == 1)
    }

    pub async fn find_next_pending_execution_request(
        &self,
    ) -> Result<Option<ExecutionRequestRecord>> {
        let row = sqlx::query(
            r#"
SELECT
    request_id,
    signal_id,
    target_mode,
    target_account,
    request_status,
    approved_by,
    created_at,
    updated_at,
    payload_json
FROM execution_requests
WHERE request_status = ?
ORDER BY created_at ASC, request_id ASC
LIMIT 1
"#,
        )
        .bind(ExecutionRequestStatus::Pending.as_str())
        .fetch_optional(&self.pool)
        .await?;

        row.map(Self::row_to_execution_request).transpose()
    }

    pub async fn supersede_previous_signals_and_cancel_pending_requests(
        &self,
        strategy_instance_id: &str,
        symbol: &str,
        timeframe: &str,
        current_signal_id: &str,
        current_bar_end: DateTime<Utc>,
    ) -> Result<usize> {
        let mut tx = self.pool.begin().await?;
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
        .bind(strategy_instance_id)
        .bind(symbol)
        .bind(timeframe)
        .bind(current_signal_id)
        .bind(SignalStatus::New.as_str())
        .bind(current_bar_end.to_rfc3339())
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
            .bind(Utc::now().to_rfc3339())
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
            .bind(Utc::now().to_rfc3339())
            .bind(signal_id)
            .bind(ExecutionRequestStatus::Pending.as_str())
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(signal_ids.len())
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

    fn row_to_order(row: SqliteRow) -> Result<OrderRecord> {
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
            order_type: OrderType::from_str(&order_type).ok_or_else(|| {
                QuantixError::DataParse(format!("invalid order type: {order_type}"))
            })?,
            requested_quantity: row.try_get("requested_quantity")?,
            requested_price: parse_decimal(&requested_price)?,
            filled_quantity: row.try_get("filled_quantity")?,
            remaining_quantity: row.try_get("remaining_quantity")?,
            avg_fill_price: avg_fill_price.as_deref().map(parse_decimal).transpose()?,
            status: OrderStatus::from_str(&status).ok_or_else(|| {
                QuantixError::DataParse(format!("invalid order status: {status}"))
            })?,
            adapter: row.try_get("adapter")?,
            created_at: parse_timestamp(&created_at)?,
            updated_at: parse_timestamp(&updated_at)?,
            last_transition_at: parse_timestamp(&last_transition_at)?,
            version: row.try_get("version")?,
            payload_json: serde_json::from_str(&payload_json)?,
        })
    }

    fn row_to_run(row: SqliteRow) -> Result<StrategyRunRecord> {
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

    fn row_to_checkpoint(row: SqliteRow) -> Result<RunnerCheckpointRecord> {
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

    fn row_to_order_event(row: SqliteRow) -> Result<OrderEventRecord> {
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

    fn row_to_signal(row: SqliteRow) -> Result<StrategySignalRecord> {
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

    fn row_to_execution_request(row: SqliteRow) -> Result<ExecutionRequestRecord> {
        let request_status: String = row.try_get("request_status")?;
        let created_at: String = row.try_get("created_at")?;
        let updated_at: String = row.try_get("updated_at")?;
        let payload_json: String = row.try_get("payload_json")?;

        Ok(ExecutionRequestRecord {
            request_id: row.try_get("request_id")?,
            signal_id: row.try_get("signal_id")?,
            target_mode: row.try_get("target_mode")?,
            target_account: row.try_get("target_account")?,
            request_status: ExecutionRequestStatus::from_str(&request_status).ok_or_else(|| {
                QuantixError::DataParse(format!("invalid request status: {request_status}"))
            })?,
            approved_by: row.try_get("approved_by")?,
            created_at: parse_timestamp(&created_at)?,
            updated_at: parse_timestamp(&updated_at)?,
            payload_json: serde_json::from_str(&payload_json)?,
        })
    }

    fn row_to_daemon_checkpoint(row: SqliteRow) -> Result<StrategyDaemonCheckpointRecord> {
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
}

fn build_execution_snapshot(signal: &StrategySignalRecord) -> Result<serde_json::Value> {
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

fn parse_execution_policy_from_metadata(
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

fn parse_signal_value(value: &str) -> Option<Signal> {
    match value {
        "buy" => Some(Signal::Buy),
        "sell" => Some(Signal::Sell),
        "hold" => Some(Signal::Hold),
        _ => None,
    }
}

fn parse_timestamp(value: &str) -> Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .map(|ts| ts.with_timezone(&Utc))
        .map_err(|err| QuantixError::DataParse(format!("invalid RFC3339 timestamp {value}: {err}")))
}

fn parse_decimal(value: &str) -> Result<Decimal> {
    Decimal::from_str_exact(value)
        .map_err(|err| QuantixError::DataParse(format!("invalid decimal {value}: {err}")))
}
