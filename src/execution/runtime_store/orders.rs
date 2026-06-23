#![allow(clippy::too_many_arguments)]

use super::*;
use crate::execution::models::QmtLiveRuntimeMetadata;

impl StrategyRuntimeStore {
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

    pub async fn list_orders(&self) -> Result<Vec<OrderRecord>> {
        let rows = sqlx::query(
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
ORDER BY created_at DESC
"#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut orders = Vec::new();
        for row in rows {
            orders.push(Self::row_to_order(row)?);
        }
        Ok(orders)
    }

    pub async fn list_open_orders(&self) -> Result<Vec<OrderRecord>> {
        let rows = sqlx::query(
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
WHERE status NOT IN ('filled', 'canceled', 'rejected')
ORDER BY created_at DESC
"#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut orders = Vec::new();
        for row in rows {
            orders.push(Self::row_to_order(row)?);
        }
        Ok(orders)
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
