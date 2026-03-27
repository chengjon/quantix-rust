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

use async_trait::async_trait;
use rust_decimal::Decimal;
use tracing::{info, warn, error};

use crate::bridge::client::BridgeHttpClient;
use crate::bridge::models::{BridgeQmtOrderRequest, BridgeQmtOrderResponse};
use crate::execution::adapter::{
    AdapterError, AdapterOrderRequest, ExecutionAdapter, OrderInitialResponse, OrderQueryResponse,
};
use crate::execution::models::{FillDetails, OrderSide, OrderStatus};

/// QMT Live Execution Adapter
///
/// Implements real order execution via the Windows bridge.
/// Orders submitted through this adapter will be executed on the broker.
#[derive(Debug, Clone)]
pub struct QmtLiveExecutionAdapter {
    client: BridgeHttpClient,
    adapter_name: &'static str,
}

impl QmtLiveExecutionAdapter {
    /// Create a new QMT live execution adapter
    pub fn new(client: BridgeHttpClient) -> Self {
        Self {
            client,
            adapter_name: "qmt_live",
        }
    }

    /// Create with custom adapter name (for testing)
    pub fn with_name(client: BridgeHttpClient, name: &'static str) -> Self {
        Self {
            client,
            adapter_name: name,
        }
    }

    /// Convert order side to bridge format
    fn side_to_bridge(side: &OrderSide) -> &'static str {
        match side {
            OrderSide::Buy => "buy",
            OrderSide::Sell => "sell",
        }
    }

    /// Convert order type to bridge format
    fn order_type_to_bridge(price: &Decimal) -> &'static str {
        // If price is zero, it's a market order
        if price.is_zero() {
            "market"
        } else {
            "limit"
        }
    }

    /// Parse bridge response status to OrderStatus
    fn parse_status(status: &str) -> OrderStatus {
        match status.to_lowercase().as_str() {
            "pending_submit" => OrderStatus::PendingSubmit,
            "submitted" => OrderStatus::Submitted,
            "accepted" => OrderStatus::Accepted,
            "partially_filled" => OrderStatus::PartiallyFilled,
            "filled" => OrderStatus::Filled,
            "canceled" | "cancelled" => OrderStatus::Canceled,
            "rejected" => OrderStatus::Rejected,
            _ => OrderStatus::Unknown,
        }
    }

    /// Convert bridge response to OrderInitialResponse
    fn response_to_initial(response: BridgeQmtOrderResponse) -> Result<OrderInitialResponse, AdapterError> {
        let avg_price = response.avg_fill_price
            .as_ref()
            .and_then(|p| p.parse::<Decimal>().ok());

        let fill_details = response.fill_details.as_ref().map(|d| FillDetails {
            fill_id: d.get("fill_id")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            fill_quantity: response.filled_quantity,
            fill_price: avg_price.unwrap_or(Decimal::ZERO),
            last_fill_price: d.get("last_fill_price")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse().ok())
                .unwrap_or(Decimal::ZERO),
            last_fill_quantity: d.get("last_fill_quantity")
                .and_then(|v| v.as_i64())
                .unwrap_or(0),
            total_fills: d.get("total_fills")
                .and_then(|v| v.as_i64())
                .unwrap_or(0),
            commission: d.get("commission")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse().ok())
                .unwrap_or(Decimal::ZERO),
            fees: d.get("fees")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse().ok())
                .unwrap_or(Decimal::ZERO),
            venue: d.get("venue")
                .and_then(|v| v.as_str())
                .unwrap_or("qmt")
                .to_string(),
            broker_fill_id: d.get("broker_fill_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        });

        Ok(OrderInitialResponse {
            adapter_order_id: response.adapter_order_id,
            latest_status: Self::parse_status(&response.latest_status),
            filled_quantity: response.filled_quantity,
            avg_fill_price: avg_price,
            fill_details,
            rejection_reason: response.rejection_reason,
        })
    }
}

#[async_trait]
impl ExecutionAdapter for QmtLiveExecutionAdapter {
    fn adapter_name(&self) -> &'static str {
        self.adapter_name
    }

    async fn submit_order(
        &self,
        request: AdapterOrderRequest,
    ) -> Result<OrderInitialResponse, AdapterError> {
        info!(
            adapter = self.adapter_name,
            symbol = %request.symbol,
            side = ?request.side,
            quantity = request.quantity,
            price = %request.price,
            "Submitting LIVE order to QMT"
        );

        // Build bridge request
        let bridge_request = BridgeQmtOrderRequest {
            request_id: uuid::Uuid::new_v4().to_string(),
            client_order_id: request.client_order_id.clone(),
            symbol: request.symbol.clone(),
            side: Self::side_to_bridge(&request.side).to_string(),
            quantity: request.quantity,
            price: request.price.to_string(),
            order_type: Self::order_type_to_bridge(&request.price).to_string(),
            strategy_name: None,
            order_remark: None,
            snapshot_metadata: None,
        };

        // Submit to bridge
        match self.client.qmt_submit_order(&bridge_request).await {
            Ok(response) => {
                let status = Self::parse_status(&response.latest_status);

                if status == OrderStatus::Rejected {
                    warn!(
                        adapter = self.adapter_name,
                        symbol = %request.symbol,
                        reason = ?response.rejection_reason,
                        "Order rejected by QMT"
                    );
                } else {
                    info!(
                        adapter = self.adapter_name,
                        order_id = %response.adapter_order_id,
                        status = ?status,
                        "Order submitted successfully"
                    );
                }

                Self::response_to_initial(response)
            }
            Err(e) => {
                error!(
                    adapter = self.adapter_name,
                    symbol = %request.symbol,
                    error = %e,
                    "Failed to submit order"
                );
                Err(AdapterError::Execution(format!("Bridge error: {}", e)))
            }
        }
    }

    async fn query_order(
        &self,
        order_id: &str,
    ) -> Result<OrderQueryResponse, AdapterError> {
        info!(
            adapter = self.adapter_name,
            order_id = %order_id,
            "Querying order status"
        );

        match self.client.qmt_query_order(order_id).await {
            Ok(response) => {
                let avg_price = response.avg_fill_price
                    .as_ref()
                    .and_then(|p| p.parse::<Decimal>().ok());

                let fill_details = response.fill_details.map(|d| FillDetails {
                    fill_id: d.get("fill_id")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0),
                    fill_quantity: response.filled_quantity,
                    fill_price: avg_price.unwrap_or(Decimal::ZERO),
                    last_fill_price: d.get("last_fill_price")
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(Decimal::ZERO),
                    last_fill_quantity: d.get("last_fill_quantity")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0),
                    total_fills: d.get("total_fills")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0),
                    commission: d.get("commission")
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(Decimal::ZERO),
                    fees: d.get("fees")
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(Decimal::ZERO),
                    venue: d.get("venue")
                        .and_then(|v| v.as_str())
                        .unwrap_or("qmt")
                        .to_string(),
                    broker_fill_id: d.get("broker_fill_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                });

                Ok(OrderQueryResponse {
                    adapter_order_id: response.adapter_order_id,
                    latest_status: Self::parse_status(&response.latest_status),
                    filled_quantity: response.filled_quantity,
                    avg_fill_price: avg_price,
                    fill_details,
                })
            }
            Err(e) => {
                error!(
                    adapter = self.adapter_name,
                    order_id = %order_id,
                    error = %e,
                    "Failed to query order"
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
                        response.error_message
                            .unwrap_or_else(|| "Cancel failed".to_string())
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
        assert_eq!(QmtLiveExecutionAdapter::side_to_bridge(&OrderSide::Buy), "buy");
        assert_eq!(QmtLiveExecutionAdapter::side_to_bridge(&OrderSide::Sell), "sell");
    }

    #[test]
    fn test_parse_status() {
        assert_eq!(QmtLiveExecutionAdapter::parse_status("submitted"), OrderStatus::Submitted);
        assert_eq!(QmtLiveExecutionAdapter::parse_status("filled"), OrderStatus::Filled);
        assert_eq!(QmtLiveExecutionAdapter::parse_status("rejected"), OrderStatus::Rejected);
        assert_eq!(QmtLiveExecutionAdapter::parse_status("unknown_value"), OrderStatus::Unknown);
    }

    #[test]
    fn test_order_type_to_bridge() {
        let limit_price = Decimal::new(1050, 2); // 10.50
        let market_price = Decimal::ZERO;

        assert_eq!(QmtLiveExecutionAdapter::order_type_to_bridge(&limit_price), "limit");
        assert_eq!(QmtLiveExecutionAdapter::order_type_to_bridge(&market_price), "market");
    }
}
