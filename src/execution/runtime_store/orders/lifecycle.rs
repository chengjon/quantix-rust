//! Order insert and read operations (single-row find + full/event listings).

use super::*;

impl StrategyRuntimeStore {
    /// 插入一条 order 记录；主键冲突直接失败。
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

    /// 插入一条 order_events 流水记录（用于审计/状态机回放）。
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

    /// 按 client_order_id 查找订单；不存在返回 `None`。
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

    /// 返回某个 run 下最早一笔订单（按 created_at, order_id 升序），无则 `None`。
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

    /// 用 status/filled/avg 价覆盖订单；remaining_quantity 由 SQL 端重新计算，version 自增。order 不存在时静默无影响。
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

    /// 列出某 order 的全部事件流水，按 (event_time, event_id) 升序。
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

    /// 列出全部订单，按 created_at 倒序返回。
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

    /// 列出未完结订单（status 不在 filled/canceled/rejected 中），按 created_at 倒序返回。
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
