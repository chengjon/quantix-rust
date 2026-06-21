use async_trait::async_trait;
use rust_decimal::Decimal;
use thiserror::Error;

use crate::execution::models::{FillDetails, OrderSide, OrderStatus};

#[derive(Debug, Clone, PartialEq)]
pub struct AdapterOrderRequest {
    pub client_order_id: String,
    pub symbol: String,
    pub side: OrderSide,
    pub quantity: i64,
    pub price: Decimal,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrderInitialResponse {
    pub adapter_order_id: String,
    pub latest_status: OrderStatus,
    pub filled_quantity: i64,
    pub avg_fill_price: Option<Decimal>,
    pub fill_details: Option<FillDetails>,
    pub rejection_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrderQueryResponse {
    pub adapter_order_id: String,
    pub latest_status: OrderStatus,
    pub filled_quantity: i64,
    pub avg_fill_price: Option<Decimal>,
    pub fill_details: Option<FillDetails>,
    pub rejection_reason: Option<String>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AdapterError {
    #[error("execution adapter 暂不支持: {0}")]
    Unsupported(String),

    #[error("execution adapter 执行失败: {0}")]
    Execution(String),

    #[error("execution adapter 网络错误: {0}")]
    Network(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionChannel {
    PaperImmediate,
    MockLive,
    QmtLive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionStatusSource {
    LocalImmediateAccounting,
    LocalSimulatedLifecycle,
    Broker,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionFillSource {
    LocalImmediateAccounting,
    LocalSimulatedMatcher,
    Broker,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionCancelSemantics {
    AlreadyFilledOnly,
    LocalLifecycle,
    Broker,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutionCapabilities {
    pub channel: ExecutionChannel,
    pub status_source: ExecutionStatusSource,
    pub fill_source: ExecutionFillSource,
    pub relies_on_broker_api: bool,
    pub supports_pending_order_lifecycle: bool,
    pub supports_partial_fill: bool,
    pub cancel_semantics: ExecutionCancelSemantics,
}

#[async_trait]
pub trait ExecutionAdapter: Send + Sync {
    fn adapter_name(&self) -> &'static str;

    fn capabilities(&self) -> ExecutionCapabilities;

    async fn submit_order(
        &self,
        request: AdapterOrderRequest,
    ) -> std::result::Result<OrderInitialResponse, AdapterError>;

    async fn query_order(
        &self,
        order_id: &str,
    ) -> std::result::Result<OrderQueryResponse, AdapterError>;

    async fn cancel_order(&self, order_id: &str) -> std::result::Result<(), AdapterError>;
}
