use sqlx::Row;

use crate::core::Result;

use super::StrategyRuntimeStore;

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

fn schema_statements() -> [&'static str; 22] {
    [
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
    ]
}

impl StrategyRuntimeStore {
    pub(super) async fn ensure_schema(&self) -> Result<()> {
        for statement in schema_statements() {
            sqlx::query(statement).execute(&self.pool).await?;
        }
        sqlx::query(CREATE_EXECUTION_REQUESTS_TARGET_STATUS_INDEX_SQL)
            .execute(&self.pool)
            .await?;
        sqlx::query(CREATE_STRATEGY_DAEMON_CHECKPOINTS_TABLE_SQL)
            .execute(&self.pool)
            .await?;
        sqlx::query(CREATE_STRATEGY_DAEMON_CHECKPOINTS_UNIQUE_INDEX_SQL)
            .execute(&self.pool)
            .await?;

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
}
