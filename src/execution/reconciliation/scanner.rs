use super::*;
use crate::core::Result;

impl OpenOrderScanner {
    /// 构造扫描器，stale 阈值默认 1h、unknown 超时默认 5min。
    pub fn new(store: StrategyRuntimeStore) -> Self {
        Self {
            store,
            stale_order_threshold_seconds: 3600, // 1 hour
            unknown_timeout_seconds: 300,        // 5 minutes
        }
    }

    /// 构造扫描器并自定义 stale 与 unknown 阈值（秒）。
    pub fn with_thresholds(
        store: StrategyRuntimeStore,
        stale_threshold_seconds: i64,
        unknown_timeout_seconds: i64,
    ) -> Self {
        Self {
            store,
            stale_order_threshold_seconds: stale_threshold_seconds,
            unknown_timeout_seconds,
        }
    }

    /// 列出所有未终结（非终态）订单，透传 store.list_open_orders。
    pub async fn list_open_orders(&self) -> Result<Vec<OrderRecord>> {
        self.store.list_open_orders().await
    }

    /// 列出处于 Unknown 状态的订单（可能需要恢复）。
    pub async fn list_unknown_orders(&self) -> Result<Vec<OrderRecord>> {
        let open_orders = self.list_open_orders().await?;
        Ok(open_orders
            .into_iter()
            .filter(|o| o.status == OrderStatus::Unknown)
            .collect())
    }

    /// 列出超过 stale 阈值的挂单（按 created_at 计算 age）。
    pub async fn list_stale_orders(&self) -> Result<Vec<OrderRecord>> {
        let open_orders = self.list_open_orders().await?;
        let now = Utc::now();
        let threshold = chrono::Duration::seconds(self.stale_order_threshold_seconds);

        Ok(open_orders
            .into_iter()
            .filter(|o| {
                let age = now - o.created_at;
                age > threshold
            })
            .collect())
    }

    /// 汇总挂单：按状态计数、stale/unknown 计数，并透出当前阈值。
    pub async fn get_open_order_summary(&self) -> Result<OpenOrderSummary> {
        let open_orders = self.list_open_orders().await?;
        let now = Utc::now();
        let stale_threshold = chrono::Duration::seconds(self.stale_order_threshold_seconds);

        let mut by_status: HashMap<String, usize> = HashMap::new();
        let mut stale_count = 0;
        let mut unknown_count = 0;

        for order in &open_orders {
            *by_status
                .entry(order.status.as_str().to_string())
                .or_insert(0) += 1;

            if order.status == OrderStatus::Unknown {
                unknown_count += 1;
            }

            let age = now - order.created_at;
            if age > stale_threshold {
                stale_count += 1;
            }
        }

        Ok(OpenOrderSummary {
            total_open: open_orders.len(),
            by_status,
            stale_count,
            unknown_count,
            stale_threshold_seconds: self.stale_order_threshold_seconds,
            unknown_timeout_seconds: self.unknown_timeout_seconds,
        })
    }
}
