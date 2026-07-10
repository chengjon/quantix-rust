use serde::{Deserialize, Serialize};

/// bridge /capabilities 端点的通用能力描述段：enabled 是否启用、supports 支持方法名列表。
#[derive(Debug, Clone, Deserialize)]
pub struct BridgeCapabilitySection {
    pub enabled: bool,
    #[serde(default)]
    pub supports: Vec<String>,
}

/// bridge /capabilities 端点的 QMT 专属能力段：enabled 是否启用、mode 模式（paper/live）、supports 支持方法名列表。
#[derive(Debug, Clone, Deserialize)]
pub struct BridgeQmtCapabilitySection {
    pub enabled: bool,
    pub mode: String,
    #[serde(default)]
    pub supports: Vec<String>,
}

/// bridge /capabilities 顶层响应：tdx + qmt 两段能力描述，由 BridgeClient 探测可用 provider。
#[derive(Debug, Clone, Deserialize)]
pub struct BridgeCapabilitiesResponse {
    pub tdx: BridgeCapabilitySection,
    pub qmt: BridgeQmtCapabilitySection,
}

/// 单条行情快照 payload：symbol/name、last 最新价、bid/ask 买卖盘、开高低/前收、volume 成交量、turnover 成交额、timestamp 时间字符串、source 数据源标识。
#[derive(Debug, Clone, Deserialize)]
pub struct BridgeQuotePayload {
    pub symbol: String,
    pub name: String,
    pub last: f64,
    pub bid: f64,
    pub ask: f64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub pre_close: f64,
    pub volume: i64,
    pub turnover: f64,
    pub timestamp: String,
    pub source: String,
}

/// bridge 行情查询响应：quotes 多标的快照列表。
#[derive(Debug, Clone, Deserialize)]
pub struct BridgeQuotesResponse {
    pub quotes: Vec<BridgeQuotePayload>,
}

/// bridge K 线单根 bar：datetime 时间字符串、开高低收、volume 成交量、turnover 成交额。
#[derive(Debug, Clone, Deserialize)]
pub struct BridgeKlineBarPayload {
    pub datetime: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: i64,
    pub turnover: f64,
}

/// bridge K 线查询响应：symbol/period 标的与周期、bars K 线列表、source 数据源标识。
#[derive(Debug, Clone, Deserialize)]
pub struct BridgeKlineResponse {
    pub symbol: String,
    pub period: String,
    pub bars: Vec<BridgeKlineBarPayload>,
    pub source: String,
}

/// QMT 实盘下单预览请求：request_id 幂等键、client_order_id 客户端单号、symbol/side/quantity/price/order_type 标的与订单参数、snapshot_metadata 上下文快照。
#[derive(Debug, Clone, serde::Serialize)]
pub struct BridgeQmtPreviewRequest {
    pub request_id: String,
    pub client_order_id: String,
    pub symbol: String,
    pub side: String,
    pub quantity: i64,
    pub price: String,
    pub order_type: String,
    pub snapshot_metadata: serde_json::Value,
}

/// QMT 实盘下单预览响应：adapter_order_id 适配器单号、latest_status 最新状态、filled_quantity 已成交量、avg_fill_price 均价、fill_details 成交明细、rejection_reason 拒单原因、broker_payload 原始回包。
#[derive(Debug, Clone, Deserialize)]
pub struct BridgeQmtPreviewResponse {
    pub adapter_order_id: String,
    pub latest_status: String,
    pub filled_quantity: i64,
    pub avg_fill_price: Option<String>,
    pub fill_details: Option<serde_json::Value>,
    pub rejection_reason: Option<String>,
    pub broker_payload: serde_json::Value,
}

/// bridge task/execute 请求：provider 方法前缀、method 具体方法名、params 任务参数体。
#[derive(Debug, Clone, Serialize)]
pub struct BridgeTaskExecuteRequest {
    pub provider: String,
    pub method: String,
    pub params: BridgeTaskExecuteParams,
}

/// bridge task/execute 参数体：request_id/client_order_id/local_submission_id 三元幂等键、symbol/side/quantity/price/order_type 订单参数、可选 strategy_name/order_remark/snapshot_metadata 上下文。
#[derive(Debug, Clone, Serialize)]
pub struct BridgeTaskExecuteParams {
    pub request_id: String,
    pub client_order_id: String,
    pub local_submission_id: String,
    pub symbol: String,
    pub side: String,
    pub quantity: i64,
    pub price: String,
    pub order_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_remark: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot_metadata: Option<serde_json::Value>,
}

/// bridge task/execute 同步回执：task_id 任务 ID、status 生命周期状态、receipt_timestamp 回执时间、bridge_contract_version 契约版本、source_name 数据源名称。
#[derive(Debug, Clone, Deserialize)]
pub struct BridgeTaskExecuteReceipt {
    pub task_id: String,
    pub status: BridgeTaskLifecycleStatus,
    pub receipt_timestamp: String,
    pub bridge_contract_version: String,
    pub source_name: String,
}

/// bridge task/result 响应：task_id 任务 ID、status 生命周期状态、bridge_contract_version 契约版本、result 可选结果 payload（任务完成才有）。
#[derive(Debug, Clone, Deserialize)]
pub struct BridgeTaskResultResponse {
    pub task_id: String,
    pub status: BridgeTaskLifecycleStatus,
    pub bridge_contract_version: String,
    pub result: Option<BridgeTaskResultPayload>,
}

/// bridge task/result 结果 payload：client_order_id/local_submission_id 本地单号、account_scope 账户范围、event_id/occurred_at 事件 ID 与时间、source_name 数据源、broker_event_type 可选 broker 事件类型、external_order_id 可选 broker 单号、reason_code/reason_detail 可选失败码与说明、evidence_ref 可选证据引用。
#[derive(Debug, Clone, Deserialize)]
pub struct BridgeTaskResultPayload {
    pub client_order_id: String,
    pub local_submission_id: String,
    pub account_scope: String,
    pub event_id: String,
    pub occurred_at: String,
    pub source_name: String,
    pub broker_event_type: Option<BridgeBrokerEventType>,
    pub external_order_id: Option<String>,
    pub reason_code: Option<BridgeFailureCode>,
    pub reason_detail: Option<String>,
    pub evidence_ref: Option<String>,
}

/// bridge 任务生命周期状态：Pending 已派发未完成、Completed 成功完成、Failed 失败、BridgeTaskAccepted bridge 已受理但未派发。入库为 snake_case 字符串。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BridgeTaskLifecycleStatus {
    Pending,
    Completed,
    Failed,
    BridgeTaskAccepted,
}

/// bridge 立即结果状态：Pending 待定（任务还未完成）、BrokerResult broker 已返回结果、BridgeFailure bridge 自身失败。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BridgeImmediateOutcomeStatus {
    Pending,
    BrokerResult,
    BridgeFailure,
}

/// bridge 失败码：超时/不可达/鉴权失败/契约版本不支持/方法不支持/结果无效/身份不匹配 7 类 infra 失败，区别于 broker 业务失败。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BridgeFailureCode {
    LiveBridgeTimeout,
    LiveBridgeUnavailable,
    LiveBridgeAuthFailed,
    LiveBridgeUnsupportedContractVersion,
    LiveBridgeUnsupportedMethod,
    LiveBridgeInvalidResult,
    LiveBridgeIdentityMismatch,
}

/// broker 事件类型：Acknowledgement 受理确认、Reject 拒单、Execution 成交回报。入库为 snake_case 字符串。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BridgeBrokerEventType {
    Acknowledgement,
    Reject,
    Execution,
}

// ============ Live Order Models ============

/// QMT 实盘下单请求：request_id 幂等键、client_order_id 客户端单号、symbol/side/quantity/price/order_type 订单参数、可选 strategy_name/order_remark/snapshot_metadata 上下文。
#[derive(Debug, Clone, serde::Serialize)]
pub struct BridgeQmtOrderRequest {
    pub request_id: String,
    pub client_order_id: String,
    pub symbol: String,
    pub side: String,
    pub quantity: i64,
    pub price: String,
    pub order_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_remark: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot_metadata: Option<serde_json::Value>,
}

/// QMT 实盘下单响应：adapter_order_id 适配器单号、latest_status 最新状态、filled_quantity 已成交量、avg_fill_price 均价、fill_details 成交明细、rejection_reason 拒单原因、broker_payload 原始回包。
#[derive(Debug, Clone, Deserialize)]
pub struct BridgeQmtOrderResponse {
    pub adapter_order_id: String,
    pub latest_status: String,
    pub filled_quantity: i64,
    pub avg_fill_price: Option<String>,
    pub fill_details: Option<serde_json::Value>,
    pub rejection_reason: Option<String>,
    pub broker_payload: Option<serde_json::Value>,
}

/// QMT 实盘订单查询响应：adapter_order_id/latest_status/filled_quantity/avg_fill_price/fill_details，不含拒单原因（查询时订单已接受）。
#[derive(Debug, Clone, Deserialize)]
pub struct BridgeQmtOrderQueryResponse {
    pub adapter_order_id: String,
    pub latest_status: String,
    pub filled_quantity: i64,
    pub avg_fill_price: Option<String>,
    pub fill_details: Option<serde_json::Value>,
}

/// QMT 实盘撤单响应：success 是否成功、order_id 撤销的订单 ID、error_message 可选失败原因。
#[derive(Debug, Clone, Deserialize)]
pub struct BridgeQmtCancelResponse {
    pub success: bool,
    pub order_id: String,
    pub error_message: Option<String>,
}

/// QMT 账户状态响应：adapter 适配器名、mode 模式（paper/live）、sdk_available SDK 是否可用、connected 是否已连接、account_masked 脱敏账号。
#[derive(Debug, Clone, Deserialize)]
pub struct BridgeQmtAccountStatusResponse {
    pub adapter: String,
    pub mode: String,
    pub sdk_available: bool,
    pub connected: bool,
    pub account_masked: Option<String>,
}

/// QMT 单只持仓：symbol 标的、name 名称、volume 总持仓、available 可卖持仓、cost_price 成本价、market_value 市值。
#[derive(Debug, Clone, Deserialize)]
pub struct BridgeQmtPosition {
    pub symbol: String,
    pub name: Option<String>,
    pub volume: i64,
    pub available: i64,
    pub cost_price: Option<String>,
    pub market_value: Option<String>,
}

/// QMT 账户资产：total_asset 总资产、cash 现金、market_value 持仓市值、account_id 账户 ID。
#[derive(Debug, Clone, Deserialize)]
pub struct BridgeQmtAsset {
    pub total_asset: String,
    pub cash: String,
    pub market_value: String,
    pub account_id: String,
}
