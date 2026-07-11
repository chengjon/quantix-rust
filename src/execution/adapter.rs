use async_trait::async_trait;
use rust_decimal::Decimal;
use thiserror::Error;

use crate::execution::models::{FillDetails, OrderSide, OrderStatus};

/// 适配器下单请求：client_order_id 客户端单号、symbol 标的、side 方向、quantity 数量、price 价格。
#[derive(Debug, Clone, PartialEq)]
pub struct AdapterOrderRequest {
    pub client_order_id: String,
    pub symbol: String,
    pub side: OrderSide,
    pub quantity: i64,
    pub price: Decimal,
}

/// 下单初始响应：adapter_order_id 适配器单号、latest_status 最新状态、filled_quantity 已成交量、avg_fill_price 可选均价、fill_details 可选明细、rejection_reason 可选拒单原因。
#[derive(Debug, Clone, PartialEq)]
pub struct OrderInitialResponse {
    pub adapter_order_id: String,
    pub latest_status: OrderStatus,
    pub filled_quantity: i64,
    pub avg_fill_price: Option<Decimal>,
    pub fill_details: Option<FillDetails>,
    pub rejection_reason: Option<String>,
}

/// 订单查询响应：结构与 OrderInitialResponse 一致，用于轮询已提交订单的最新状态。
#[derive(Debug, Clone, PartialEq)]
pub struct OrderQueryResponse {
    pub adapter_order_id: String,
    pub latest_status: OrderStatus,
    pub filled_quantity: i64,
    pub avg_fill_price: Option<Decimal>,
    pub fill_details: Option<FillDetails>,
    pub rejection_reason: Option<String>,
}

/// 适配器错误：Unsupported 方法不支持、Execution 执行失败、Network 网络错误。区别于 broker 业务失败。
#[derive(Debug, Error, PartialEq, Eq)]
pub enum AdapterError {
    #[error("execution adapter 暂不支持: {0}")]
    Unsupported(String),

    #[error("execution adapter 执行失败: {0}")]
    Execution(String),

    #[error("execution adapter 网络错误: {0}")]
    Network(String),
}

/// 执行通道：PaperImmediate 纸面即时记账、MockLive 模拟实盘生命周期、QmtLive QMT 实盘。入库用 as_str() 字符串形式存储。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionChannel {
    PaperImmediate,
    MockLive,
    QmtLive,
}

impl ExecutionChannel {
    /// 返回该执行通道的稳定字符串标识（"paper_immediate" / "mock_live" / "qmt_live"），用于入库与日志。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PaperImmediate => "paper_immediate",
            Self::MockLive => "mock_live",
            Self::QmtLive => "qmt_live",
        }
    }
}

/// 状态来源：LocalImmediateAccounting 本地即时记账、LocalSimulatedLifecycle 本地模拟生命周期、Broker 来自 broker。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionStatusSource {
    LocalImmediateAccounting,
    LocalSimulatedLifecycle,
    Broker,
}

impl ExecutionStatusSource {
    /// 返回状态来源的稳定字符串标识（"local_immediate_accounting" / "local_simulated_lifecycle" / "broker"）。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LocalImmediateAccounting => "local_immediate_accounting",
            Self::LocalSimulatedLifecycle => "local_simulated_lifecycle",
            Self::Broker => "broker",
        }
    }
}

/// 成交来源：LocalImmediateAccounting 本地即时记账、LocalSimulatedMatcher 本地模拟撮合、Broker 来自 broker。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionFillSource {
    LocalImmediateAccounting,
    LocalSimulatedMatcher,
    Broker,
}

impl ExecutionFillSource {
    /// 返回成交来源的稳定字符串标识（"local_immediate_accounting" / "local_simulated_matcher" / "broker"）。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LocalImmediateAccounting => "local_immediate_accounting",
            Self::LocalSimulatedMatcher => "local_simulated_matcher",
            Self::Broker => "broker",
        }
    }
}

/// 撤单语义：AlreadyFilledOnly 仅允许撤已成交、LocalLifecycle 本地生命周期撤单、Broker 委托 broker 撤单。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionCancelSemantics {
    AlreadyFilledOnly,
    LocalLifecycle,
    Broker,
}

impl ExecutionCancelSemantics {
    /// 返回撤单语义的稳定字符串标识（"already_filled_only" / "local_lifecycle" / "broker"）。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AlreadyFilledOnly => "already_filled_only",
            Self::LocalLifecycle => "local_lifecycle",
            Self::Broker => "broker",
        }
    }
}

/// 执行适配器能力描述：channel 通道、status_source 状态来源、fill_source 成交来源、relies_on_broker_api 是否依赖 broker API、supports_pending_order_lifecycle 是否支持挂单生命周期、supports_partial_fill 是否支持部分成交、cancel_semantics 撤单语义。
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

/// 执行适配器 trait：抽象 paper/mock/qmt 三种通道的下单/查询/撤单能力。实现方需声明 adapter_name 与 capabilities，并实现三个异步方法。
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
