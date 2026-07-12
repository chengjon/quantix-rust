//! Optimistic-lock updates (version-gated writes), QMT live metadata merge,
//! and runner-checkpoint upsert/load.

use super::*;
use crate::execution::models::QmtLiveRuntimeMetadata;

impl StrategyRuntimeStore {
    /// 乐观锁更新：仅当 DB 中 version 等于 expected_version 时才覆盖 status/filled/remaining/avg；成功返回 `true`。
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

    /// 乐观锁更新：仅当 version 匹配时才替换 payload_json；成功返回 `true`。
    pub async fn try_update_order_payload_with_version(
        &self,
        order_id: &str,
        expected_version: i64,
        payload_json: serde_json::Value,
        updated_at: DateTime<Utc>,
    ) -> Result<bool> {
        let result = sqlx::query(
            r#"
UPDATE orders
SET payload_json = ?,
    updated_at = ?,
    version = version + 1
WHERE order_id = ? AND version = ?
"#,
        )
        .bind(serde_json::to_string(&payload_json)?)
        .bind(updated_at.to_rfc3339())
        .bind(order_id)
        .bind(expected_version)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() == 1)
    }

    /// 乐观锁更新：仅当 version 匹配时同时替换 status/filled/remaining/avg 与 payload_json；成功返回 `true`。
    pub async fn try_update_order_state_and_payload_with_version(
        &self,
        order_id: &str,
        expected_version: i64,
        status: OrderStatus,
        filled_quantity: i64,
        remaining_quantity: i64,
        avg_fill_price: Option<Decimal>,
        payload_json: serde_json::Value,
        updated_at: DateTime<Utc>,
    ) -> Result<bool> {
        let result = sqlx::query(
            r#"
UPDATE orders
SET status = ?,
    filled_quantity = ?,
    remaining_quantity = ?,
    avg_fill_price = ?,
    payload_json = ?,
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
        .bind(serde_json::to_string(&payload_json)?)
        .bind(updated_at.to_rfc3339())
        .bind(updated_at.to_rfc3339())
        .bind(order_id)
        .bind(expected_version)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() == 1)
    }

    /// 把 QMT 实盘元数据合并进 order.payload_json.qmt_live（保留既有 external_order_id），并乐观锁写回。
    pub async fn try_update_order_qmt_live_metadata(
        &self,
        order: &OrderRecord,
        metadata: &QmtLiveRuntimeMetadata,
        updated_at: DateTime<Utc>,
    ) -> Result<bool> {
        let mut payload_json = order.payload_json.clone();
        let mut qmt_live_metadata = serde_json::to_value(metadata)?;
        preserve_existing_qmt_live_external_order_id(
            payload_json.get("qmt_live"),
            &mut qmt_live_metadata,
        );
        payload_json["qmt_live"] = qmt_live_metadata;
        self.try_update_order_payload_with_version(
            &order.order_id,
            order.version,
            payload_json,
            updated_at,
        )
        .await
    }

    /// upsert runner_checkpoints：冲突时整体覆盖（除主键外的所有字段），按 (strategy_name, mode, symbol, timeframe) 唯一。
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

    /// 按 (strategy_name, mode, symbol, timeframe) 加载 runner_checkpoints；不存在返回 `None`。
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
}

fn preserve_existing_qmt_live_external_order_id(
    existing: Option<&serde_json::Value>,
    next: &mut serde_json::Value,
) {
    let Some(existing_external_order_id) = existing
        .and_then(|value| value.get("task_identity"))
        .and_then(|value| value.get("external_order_id"))
        .filter(|value| !value.is_null())
        .cloned()
    else {
        return;
    };

    let Some(next_task_identity) = next
        .get_mut("task_identity")
        .and_then(|value| value.as_object_mut())
    else {
        return;
    };

    let has_next_external_order_id = next_task_identity
        .get("external_order_id")
        .is_some_and(|value| !value.is_null());
    if !has_next_external_order_id {
        next_task_identity.insert("external_order_id".to_string(), existing_external_order_id);
    }
}
