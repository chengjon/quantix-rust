//! `StrategyRuntimeStore` — the SQLite-backed runtime store for the
//! execution runtime (runs / signals / orders / requests / checkpoints).
//!
//! Construction, run/signal/checkpoint inserts, and row-to-record decoders
//! live here. Sibling files (`orders.rs`, `requests.rs`, `signals.rs`,
//! `schema.rs`) add additional `impl StrategyRuntimeStore` blocks for their
//! respective tables.

use std::path::Path;

use chrono::{DateTime, Utc};
use sqlx::Row;
use sqlx::sqlite::{SqlitePool, SqliteRow};

use crate::core::{QuantixError, Result};
use crate::execution::models::{
    ExecutionRequestRecord, ExecutionRequestStatus, OrderEventRecord, OrderRecord, OrderSide,
    OrderStatus, OrderType, SignalEventRecord, SignalStatus, StrategyDaemonCheckpointRecord,
    StrategyRunRecord, StrategyRunStatus, StrategySignalRecord,
};

use super::codec::{parse_decimal, parse_timestamp};

/// 策略运行时 SQLite 存储，承载 run/signal/order/request/checkpoint 等表。
#[derive(Debug, Clone)]
pub struct StrategyRuntimeStore {
    pub(crate) pool: SqlitePool,
}

impl StrategyRuntimeStore {
    /// 打开（必要时创建）SQLite 文件并初始化全部运行时表；父目录会按需创建。
    pub async fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let options = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true);
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await?;
        let store = Self { pool };
        store.ensure_schema().await?;
        Ok(store)
    }

    /// 判断指定表名是否存在于当前库；表名直接拼接到 SQL，调用方需保证来源可信。
    pub async fn has_table(&self, table_name: &str) -> Result<bool> {
        let exists = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(1) FROM sqlite_master WHERE type = 'table' AND name = ?",
        )
        .bind(table_name)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists > 0)
    }

    /// 插入一条 strategy_runs 记录（不去重，主键冲突时直接失败）。
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

    /// 按 (strategy_name, mode, symbol, timeframe, bar_end) 五元组查找 run；不存在返回 `None`。
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

    /// 插入一条原始信号事件到 signal_events 表（用于回放/审计）。
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

    /// 把指定 run 的状态与 finished_at 写回；run 不存在时不报错（影响行数=0）。
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

    /// 统计 strategy_runs 表的总行数。
    pub async fn count_runs(&self) -> Result<i64> {
        self.count_table_rows("strategy_runs").await
    }

    /// 统计 orders 表的总行数。
    pub async fn count_orders(&self) -> Result<i64> {
        self.count_table_rows("orders").await
    }

    /// 统计 signal_events 表的总行数。
    pub async fn count_signal_events(&self) -> Result<i64> {
        self.count_table_rows("signal_events").await
    }

    /// 统计 signals 表的总行数。
    pub async fn count_signals(&self) -> Result<i64> {
        self.count_table_rows("signals").await
    }

    /// 守护进程一次完整写入：单事务插入 run/signal，把同 key 的旧 New 信号与 Pending 请求置为 superseded/canceled，并 upsert checkpoint。返回被取代的 signal 数量。
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

    pub(super) async fn count_table_rows(&self, table_name: &str) -> Result<i64> {
        let sql = format!("SELECT COUNT(1) FROM {table_name}");
        Ok(sqlx::query_scalar::<_, i64>(&sql)
            .fetch_one(&self.pool)
            .await?)
    }

    pub(super) fn row_to_order(row: SqliteRow) -> Result<OrderRecord> {
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

    pub(super) fn row_to_order_event(row: SqliteRow) -> Result<OrderEventRecord> {
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

    pub(super) fn row_to_execution_request(row: SqliteRow) -> Result<ExecutionRequestRecord> {
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

    pub(super) fn row_to_daemon_checkpoint(
        row: SqliteRow,
    ) -> Result<StrategyDaemonCheckpointRecord> {
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
