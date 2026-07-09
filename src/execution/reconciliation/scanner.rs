use super::*;
use crate::core::Result;

impl OpenOrderScanner {
    /// Create a new open order scanner
    pub fn new(store: StrategyRuntimeStore) -> Self {
        Self {
            store,
            stale_order_threshold_seconds: 3600, // 1 hour
            unknown_timeout_seconds: 300,        // 5 minutes
        }
    }

    /// Create with custom thresholds
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

    /// List all open orders (orders that are not in terminal state)
    pub async fn list_open_orders(&self) -> Result<Vec<OrderRecord>> {
        self.store.list_open_orders().await
    }

    /// List orders in Unknown state that may need recovery
    pub async fn list_unknown_orders(&self) -> Result<Vec<OrderRecord>> {
        let open_orders = self.list_open_orders().await?;
        Ok(open_orders
            .into_iter()
            .filter(|o| o.status == OrderStatus::Unknown)
            .collect())
    }

    /// List stale orders (open orders older than threshold)
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

    /// Get summary of open orders by status
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
