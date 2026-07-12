//! Mock-live order state (in-memory simulation persisted to the
//! `mock_live_orders` table): insert/recover/update plus the recoverable-order
//! listing used by daemon startup.

use super::*;

impl StrategyRuntimeStore {
    /// 写入某 order 的 mock-live 状态（含 adapter 端 order_id 可选），用于在进程重启后重建模拟行情。
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

    /// 读取某 order 的 mock-live 状态；无记录时返回 `None`。
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

    /// 列出所有 mock-live 状态仍在途（Submitted/Accepted/PartiallyFilled/Unknown/PendingCancel）的订单，按 updated_at 升序。
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

    /// 用 adapter_order_id 与 state 覆盖 mock-live 状态；order 不存在时静默无影响。
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
}
