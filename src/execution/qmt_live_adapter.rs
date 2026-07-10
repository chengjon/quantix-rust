//! QMT Live Execution Adapter - Real order execution via Windows bridge
//!
//! This adapter implements `ExecutionAdapter` for real trading via QMT.
//! It communicates with the Windows-side `quantix-bridge` service.
//!
//! # Safety
//!
//! - Only use when `BRIDGE_QMT_MODE=live` is explicitly configured
//! - Orders submitted through this adapter are REAL and will affect your account
//!
//! # Example
//!
//! ```ignore
//! use quantix::bridge::client::BridgeHttpClient;
//! use quantix::execution::qmt_live_adapter::QmtLiveExecutionAdapter;
//! use quantix::execution::adapter::{ExecutionAdapter, AdapterOrderRequest};
//!
//! let client = BridgeHttpClient::new("http://127.0.0.1:17580".to_string(), Some("api-key".to_string()))?;
//! let adapter = QmtLiveExecutionAdapter::new(client);
//!
//! // Submit a real order
//! let response = adapter.submit_order(AdapterOrderRequest {
//!     client_order_id: "order-001".to_string(),
//!     symbol: "600000.SH".to_string(),
//!     side: OrderSide::Buy,
//!     quantity: 100,
//!     price: Decimal::new(1050, 2), // 10.50
//! }).await?;
//! ```

use std::time::Duration;

use async_trait::async_trait;
use rust_decimal::Decimal;
use tracing::{error, info, warn};

use crate::bridge::client::BridgeHttpClient;
use crate::core::runtime::{DEFAULT_BRIDGE_POLL_INTERVAL_MS, DEFAULT_BRIDGE_POLL_TIMEOUT_MS};
use crate::execution::adapter::{
    AdapterError, AdapterOrderRequest, ExecutionAdapter, ExecutionCancelSemantics,
    ExecutionCapabilities, ExecutionChannel, ExecutionFillSource, ExecutionStatusSource,
    OrderInitialResponse, OrderQueryResponse,
};
use crate::execution::mode_semantics::{QMT_LIVE_CHANNEL, log_execution_mode_risk_notice};
use crate::execution::models::{FillDetails, OrderSide, OrderStatus};
use crate::execution::qmt_live_gate::ensure_bridge_qmt_live_mode;
use crate::execution::qmt_task_submit_service::QmtTaskSubmitService;

/// QMT 实盘执行适配器：经 Windows bridge 执行真实订单，下单即进入 broker。组合 BridgeHttpClient + QmtTaskSubmitService 实现 submit/query/cancel。
#[derive(Debug, Clone)]
pub struct QmtLiveExecutionAdapter {
    client: BridgeHttpClient,
    submit_service: QmtTaskSubmitService,
    adapter_name: &'static str,
}

impl QmtLiveExecutionAdapter {
    /// 构造 qmt_live 适配器：adapter_name=qmt_live、使用默认轮询参数（DEFAULT_BRIDGE_POLL_INTERVAL_MS / DEFAULT_BRIDGE_POLL_TIMEOUT_MS）。
    pub fn new(client: BridgeHttpClient) -> Self {
        Self::with_name_and_polling(
            client,
            "qmt_live",
            DEFAULT_BRIDGE_POLL_INTERVAL_MS,
            DEFAULT_BRIDGE_POLL_TIMEOUT_MS,
        )
    }

    /// 构造 qmt_live 适配器并自定义 adapter_name（主要用于测试场景隔离）。
    pub fn with_name(client: BridgeHttpClient, name: &'static str) -> Self {
        Self::with_name_and_polling(
            client,
            name,
            DEFAULT_BRIDGE_POLL_INTERVAL_MS,
            DEFAULT_BRIDGE_POLL_TIMEOUT_MS,
        )
    }

    /// 构造 qmt_live 适配器并自定义 task 轮询参数（poll_interval_ms / poll_timeout_ms）。
    pub fn with_polling(
        client: BridgeHttpClient,
        poll_interval_ms: u64,
        poll_timeout_ms: u64,
    ) -> Self {
        Self::with_name_and_polling(client, "qmt_live", poll_interval_ms, poll_timeout_ms)
    }

    fn with_name_and_polling(
        client: BridgeHttpClient,
        name: &'static str,
        poll_interval_ms: u64,
        poll_timeout_ms: u64,
    ) -> Self {
        // poll_interval_ms / poll_timeout_ms 为 0 会让 QmtTaskSubmitService::new 失败，
        // 强制将 0 提升为 1ms 以保持 Self 返回类型的契约（绝不 panic）。
        // QmtTaskSubmitService::new 仅在这两个参数为 0 时返回 Err，因此 .max(1) 之后必然 Ok；
        // Err 分支仅做日志并以 1ms/1ms 占位实例兜底，让后续 poll 自行暴露错误。
        let safe_interval = poll_interval_ms.max(1);
        let safe_timeout = poll_timeout_ms.max(1);
        let submit_service = match QmtTaskSubmitService::new(
            client.clone(),
            safe_interval,
            safe_timeout,
        ) {
            Ok(svc) => svc,
            Err(e) => {
                tracing::error!(
                    "QmtTaskSubmitService 构造不可达分支触发 (interval={}, timeout={}): {}",
                    safe_interval,
                    safe_timeout,
                    e
                );
                // 直接构造占位实例；字段已对 crate 内可见。
                QmtTaskSubmitService {
                    client: client.clone(),
                    poll_interval: Duration::from_millis(1),
                    poll_timeout: Duration::from_millis(1),
                }
            }
        };

        Self {
            client,
            submit_service,
            adapter_name: name,
        }
    }

    /// Convert order side to bridge format
    #[allow(dead_code)]
    fn side_to_bridge(side: &OrderSide) -> &'static str {
        match side {
            OrderSide::Buy => "buy",
            OrderSide::Sell => "sell",
        }
    }

    /// Convert order type to bridge format
    #[allow(dead_code)]
    fn order_type_to_bridge(price: &Decimal) -> &'static str {
        // If price is zero, it's a market order
        if price.is_zero() { "market" } else { "limit" }
    }

    fn receipt_to_initial(task_id: String) -> OrderInitialResponse {
        OrderInitialResponse {
            adapter_order_id: task_id,
            latest_status: OrderStatus::PendingSubmit,
            filled_quantity: 0,
            avg_fill_price: None,
            fill_details: None,
            rejection_reason: None,
        }
    }
}

#[async_trait]
impl ExecutionAdapter for QmtLiveExecutionAdapter {
    fn adapter_name(&self) -> &'static str {
        self.adapter_name
    }

    fn capabilities(&self) -> ExecutionCapabilities {
        ExecutionCapabilities {
            channel: ExecutionChannel::QmtLive,
            status_source: ExecutionStatusSource::Broker,
            fill_source: ExecutionFillSource::Broker,
            relies_on_broker_api: true,
            supports_pending_order_lifecycle: true,
            supports_partial_fill: true,
            cancel_semantics: ExecutionCancelSemantics::Broker,
        }
    }

    async fn submit_order(
        &self,
        request: AdapterOrderRequest,
    ) -> Result<OrderInitialResponse, AdapterError> {
        log_execution_mode_risk_notice(QMT_LIVE_CHANNEL);
        info!(
            adapter = self.adapter_name,
            symbol = %request.symbol,
            side = ?request.side,
            quantity = request.quantity,
            price = %request.price,
            "Submitting LIVE order to QMT"
        );

        if let Err(err) = ensure_bridge_qmt_live_mode(&self.client).await {
            return Err(AdapterError::Execution(err.to_string()));
        }

        match self.submit_service.submit_order(&request).await {
            Ok(receipt) => {
                info!(
                    adapter = self.adapter_name,
                    task_id = %receipt.task_id,
                    local_submission_id = %receipt.local_submission_id,
                    "QMT task receipt accepted"
                );
                Ok(Self::receipt_to_initial(receipt.task_id))
            }
            Err(e) => {
                error!(
                    adapter = self.adapter_name,
                    symbol = %request.symbol,
                    error = %e,
                    "Failed to submit task-contract order"
                );
                Err(AdapterError::Execution(format!("Bridge error: {}", e)))
            }
        }
    }

    async fn query_order(&self, order_id: &str) -> Result<OrderQueryResponse, AdapterError> {
        info!(
            adapter = self.adapter_name,
            order_id = %order_id,
            "Querying order status"
        );

        match self
            .submit_service
            .query_task_result_by_task_id(order_id)
            .await
        {
            Ok(result) => Ok(OrderQueryResponse {
                adapter_order_id: result.adapter_order_id,
                latest_status: result.latest_status,
                filled_quantity: result.filled_quantity,
                avg_fill_price: result.avg_fill_price,
                fill_details: None::<FillDetails>,
                rejection_reason: result.rejection_reason,
            }),
            Err(e) => {
                error!(
                    adapter = self.adapter_name,
                    order_id = %order_id,
                    error = %e,
                    "Failed to query task-contract order"
                );
                Err(AdapterError::Execution(format!("Query error: {}", e)))
            }
        }
    }

    async fn cancel_order(&self, order_id: &str) -> Result<(), AdapterError> {
        info!(
            adapter = self.adapter_name,
            order_id = %order_id,
            "Canceling order"
        );

        match self.client.qmt_cancel_order(order_id).await {
            Ok(response) => {
                if response.success {
                    info!(
                        adapter = self.adapter_name,
                        order_id = %order_id,
                        "Order canceled successfully"
                    );
                    Ok(())
                } else {
                    warn!(
                        adapter = self.adapter_name,
                        order_id = %order_id,
                        error = ?response.error_message,
                        "Cancel failed"
                    );
                    Err(AdapterError::Execution(
                        response
                            .error_message
                            .unwrap_or_else(|| "Cancel failed".to_string()),
                    ))
                }
            }
            Err(e) => {
                error!(
                    adapter = self.adapter_name,
                    order_id = %order_id,
                    error = %e,
                    "Failed to cancel order"
                );
                Err(AdapterError::Execution(format!("Cancel error: {}", e)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_side_to_bridge() {
        assert_eq!(
            QmtLiveExecutionAdapter::side_to_bridge(&OrderSide::Buy),
            "buy"
        );
        assert_eq!(
            QmtLiveExecutionAdapter::side_to_bridge(&OrderSide::Sell),
            "sell"
        );
    }

    #[test]
    fn test_parse_status() {
        assert_eq!(
            QmtLiveExecutionAdapter::receipt_to_initial("task-1".to_string()).latest_status,
            OrderStatus::PendingSubmit
        );
    }

    #[test]
    fn test_order_type_to_bridge() {
        let limit_price = Decimal::new(1050, 2); // 10.50
        let market_price = Decimal::ZERO;

        assert_eq!(
            QmtLiveExecutionAdapter::order_type_to_bridge(&limit_price),
            "limit"
        );
        assert_eq!(
            QmtLiveExecutionAdapter::order_type_to_bridge(&market_price),
            "market"
        );
    }
}
