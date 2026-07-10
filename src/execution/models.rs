#![allow(clippy::should_implement_trait)]

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::signal::Signal;
use crate::core::{QuantixError, Result};

mod mock_live;
pub use mock_live::{MockLiveFaultInjection, MockLiveFillStep, MockLiveOrderState};

/// 策略单次运行的状态：Running 运行中、Success 成功完成、Failed 失败。入库用 as_str() 字符串形式存储。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrategyRunStatus {
    Running,
    Success,
    Failed,
}

impl StrategyRunStatus {
    /// 返回标识串，与 from_str 互逆。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Success => "success",
            Self::Failed => "failed",
        }
    }

    /// 从字符串解析，未匹配返回 None。
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "running" => Some(Self::Running),
            "success" => Some(Self::Success),
            "failed" => Some(Self::Failed),
            _ => None,
        }
    }
}

/// 订单全生命周期状态：PendingSubmit→Submitted→Accepted→（PartiallyFilled）→Filled，或 PendingCancel→Canceled，或 Rejected；Unknown 兜底用于 broker 返回未知字符串。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    PendingSubmit,
    Submitted,
    Accepted,
    PartiallyFilled,
    PendingCancel,
    Filled,
    Canceled,
    Rejected,
    Unknown,
}

impl OrderStatus {
    /// 返回标识串，与 from_str 互逆。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PendingSubmit => "pending_submit",
            Self::Submitted => "submitted",
            Self::Accepted => "accepted",
            Self::PartiallyFilled => "partially_filled",
            Self::PendingCancel => "pending_cancel",
            Self::Filled => "filled",
            Self::Canceled => "canceled",
            Self::Rejected => "rejected",
            Self::Unknown => "unknown",
        }
    }

    /// 从字符串解析，未匹配返回 None。
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "pending_submit" => Some(Self::PendingSubmit),
            "submitted" => Some(Self::Submitted),
            "accepted" => Some(Self::Accepted),
            "partially_filled" => Some(Self::PartiallyFilled),
            "pending_cancel" => Some(Self::PendingCancel),
            "filled" => Some(Self::Filled),
            "canceled" => Some(Self::Canceled),
            "rejected" => Some(Self::Rejected),
            "unknown" => Some(Self::Unknown),
            _ => None,
        }
    }
}

/// 信号状态：New 新建、Superseded 被更新版本替代、Expired 已过期不再可执行。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalStatus {
    New,
    Superseded,
    Expired,
}

impl SignalStatus {
    /// 返回标识串，与 from_str 互逆。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Superseded => "superseded",
            Self::Expired => "expired",
        }
    }

    /// 从字符串解析，未匹配返回 None。
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "new" => Some(Self::New),
            "superseded" => Some(Self::Superseded),
            "expired" => Some(Self::Expired),
            _ => None,
        }
    }
}

/// 信号审批状态：Pending 待审批、Approved 已批准可执行、Rejected 已拒绝、AutoApproved 自动批准（无需人工）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
}

impl ApprovalStatus {
    /// 返回标识串，与 from_str 互逆。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
        }
    }

    /// 从字符串解析，未匹配返回 None。
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "pending" => Some(Self::Pending),
            "approved" => Some(Self::Approved),
            "rejected" => Some(Self::Rejected),
            _ => None,
        }
    }
}

/// 执行请求状态：Pending 待处理、InProgress 已派发执行中、Completed 已完成、Failed 执行失败、Canceled 已撤销。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionRequestStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Canceled,
}

impl ExecutionRequestStatus {
    /// 返回标识串，与 from_str 互逆。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Canceled => "canceled",
        }
    }

    /// 从字符串解析，未匹配返回 None。
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "pending" => Some(Self::Pending),
            "in_progress" => Some(Self::InProgress),
            "completed" => Some(Self::Completed),
            "failed" => Some(Self::Failed),
            "canceled" => Some(Self::Canceled),
            _ => None,
        }
    }
}

/// 订单方向：Buy 买入、Sell 卖出。入库用 as_str() 字符串形式存储。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderSide {
    Buy,
    Sell,
}

impl OrderSide {
    /// 返回标识串，与 from_str 互逆。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Buy => "buy",
            Self::Sell => "sell",
        }
    }

    /// 从字符串解析，未匹配返回 None。
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "buy" => Some(Self::Buy),
            "sell" => Some(Self::Sell),
            _ => None,
        }
    }
}

/// 订单类型：Market 市价单（按当前盘口撮合）、Limit 限价单（按指定价格挂单）。入库用 as_str() 字符串形式存储。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderType {
    Market,
    Limit,
}

impl OrderType {
    /// 返回标识串，与 from_str 互逆。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Market => "market",
            Self::Limit => "limit",
        }
    }

    /// 从字符串解析，未匹配返回 None。
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "market" => Some(Self::Market),
            "limit" => Some(Self::Limit),
            _ => None,
        }
    }
}

/// 策略单次运行的入库记录：run_id 主键、策略名/模式/触发器、状态、标的/周期/bar_end、起止时间与 metadata JSON。
#[derive(Debug, Clone, PartialEq)]
pub struct StrategyRunRecord {
    pub run_id: String,
    pub strategy_name: String,
    pub mode: String,
    pub trigger: String,
    pub status: StrategyRunStatus,
    pub symbol: String,
    pub timeframe: String,
    pub bar_end: DateTime<Utc>,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub metadata_json: Value,
}

/// 策略发出的信号事件入库记录：event_id 主键、关联 run_id、策略名/标的/信号字符串/事件时间/payload JSON。
#[derive(Debug, Clone, PartialEq)]
pub struct SignalEventRecord {
    pub event_id: String,
    pub run_id: String,
    pub strategy_name: String,
    pub symbol: String,
    pub signal: String,
    pub ts: DateTime<Utc>,
    pub payload_json: Value,
}

/// 信号信封：包装 Signal 枚举与任意 metadata JSON，用于 translate_signal 等下游消费。
#[derive(Debug, Clone, PartialEq)]
pub struct SignalEnvelope {
    pub signal: Signal,
    pub metadata_json: Value,
}

impl SignalEnvelope {
    /// 用信号构造 envelope，`metadata_json` 初始化为空对象。
    pub fn new(signal: Signal) -> Self {
        Self {
            signal,
            metadata_json: Value::Object(Default::default()),
        }
    }
}

/// 纸面交易执行策略：fixed_cash_per_buy 每次买入固定金额（按手取整），slippage_bps 滑点 bps（1bp=0.01%）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionPolicy {
    pub fixed_cash_per_buy: Decimal,
    pub slippage_bps: u32,
}

/// translate_signal 翻译信号产生的下单意图：标的/方向/数量/价格/类型/原因，附 policy 快照 JSON 便于审计。
#[derive(Debug, Clone, PartialEq)]
pub struct OrderIntent {
    pub symbol: String,
    pub side: OrderSide,
    pub requested_quantity: i64,
    pub requested_price: Decimal,
    pub order_type: OrderType,
    pub reason: String,
    pub policy_snapshot_json: Value,
}

/// 订单全生命周期入库记录：order_id 主键、client_order_id 客户端标识、关联 run_id、状态/成交信息/版本号与 payload JSON。
#[derive(Debug, Clone, PartialEq)]
pub struct OrderRecord {
    pub order_id: String,
    pub client_order_id: String,
    pub run_id: String,
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub requested_quantity: i64,
    pub requested_price: Decimal,
    pub filled_quantity: i64,
    pub remaining_quantity: i64,
    pub avg_fill_price: Option<Decimal>,
    pub status: OrderStatus,
    pub adapter: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_transition_at: DateTime<Utc>,
    pub version: i64,
    pub payload_json: Value,
}

/// QMT 实盘任务身份四元组：task_id / client_order_id / local_submission_id / external_order_id，用于崩溃恢复关联本地与 broker 单。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct QmtLiveTaskIdentity {
    #[serde(default)]
    pub task_id: String,
    #[serde(default)]
    pub client_order_id: String,
    #[serde(default)]
    pub local_submission_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_order_id: Option<String>,
}

/// QMT 实盘最近一次查询摘要：最新状态/累计成交/均价/broker 事件类型/拒单原因/更新时间，用于运行时对账展示。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct QmtLiveLastQuerySummary {
    pub latest_status: String,
    #[serde(default)]
    pub filled_quantity: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avg_fill_price: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub broker_event_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rejection_reason: Option<String>,
    pub updated_at: String,
}

/// QMT 实盘对账状态：上次动作/上次错误/上次尝试时间，用于崩溃恢复时追踪人工或自动对账进度。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct QmtLiveReconciliationState {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_action: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_attempt_at: Option<String>,
}

/// QMT 实盘运行时元数据聚合：task_identity + last_query + reconciliation，整体序列化存入 OrderRecord.payload_json。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct QmtLiveRuntimeMetadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_identity: Option<QmtLiveTaskIdentity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_query: Option<QmtLiveLastQuerySummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reconciliation: Option<QmtLiveReconciliationState>,
}

impl QmtLiveTaskIdentity {
    /// 用入参回填缺失字段（空串视为缺失），返回新的 identity 实例。
    pub fn recover_missing_fields(
        &self,
        task_id: &str,
        client_order_id: &str,
        local_submission_id: Option<&str>,
        external_order_id: Option<&str>,
    ) -> Self {
        Self {
            task_id: if self.task_id.trim().is_empty() {
                task_id.to_string()
            } else {
                self.task_id.clone()
            },
            client_order_id: if self.client_order_id.trim().is_empty() {
                client_order_id.to_string()
            } else {
                self.client_order_id.clone()
            },
            local_submission_id: if self.local_submission_id.trim().is_empty() {
                local_submission_id.unwrap_or_default().to_string()
            } else {
                self.local_submission_id.clone()
            },
            external_order_id: self
                .external_order_id
                .clone()
                .or_else(|| external_order_id.map(|value| value.to_string())),
        }
    }
}

impl QmtLiveRuntimeMetadata {
    /// 从单字段推断完整 identity（用于崩溃恢复 / 重启场景）。
    pub fn recover_task_identity(
        &self,
        task_id: &str,
        client_order_id: &str,
        local_submission_id: Option<&str>,
        external_order_id: Option<&str>,
    ) -> Self {
        Self {
            task_identity: self.task_identity.as_ref().map(|task_identity| {
                task_identity.recover_missing_fields(
                    task_id,
                    client_order_id,
                    local_submission_id,
                    external_order_id,
                )
            }),
            last_query: self.last_query.clone(),
            reconciliation: self.reconciliation.clone(),
        }
    }
}

/// 单笔成交明细：fill_id 本地成交 ID/成交量/成交价/最近一次增量成交价量/总成交笔数/佣金/其他费用/交易所/broker fill ID。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FillDetails {
    pub fill_id: u64,
    pub fill_quantity: i64,
    pub fill_price: Decimal,
    /// Last fill price (for incremental fills)
    #[serde(default)]
    pub last_fill_price: Decimal,
    /// Last fill quantity (for incremental fills)
    #[serde(default)]
    pub last_fill_quantity: i64,
    /// Total number of fills
    #[serde(default)]
    pub total_fills: i64,
    /// Commission amount
    #[serde(default)]
    pub commission: Decimal,
    /// Other fees
    #[serde(default)]
    pub fees: Decimal,
    /// Execution venue
    #[serde(default)]
    pub venue: String,
    /// Broker's fill ID
    #[serde(default)]
    pub broker_fill_id: String,
}

/// 增量成交上下文：order_id/client_order_id/symbol/side/请求价/旧成交均价对应成交量/新成交量/可选 FillDetails/事件时间，传给 fill delta 处理器计算增量结果。
#[derive(Debug, Clone, PartialEq)]
pub struct FillDeltaContext {
    pub order_id: String,
    pub client_order_id: String,
    pub symbol: String,
    pub side: OrderSide,
    pub requested_price: Decimal,
    pub old_filled_quantity: i64,
    pub new_filled_quantity: i64,
    pub fill_details: Option<FillDetails>,
    pub event_time: DateTime<Utc>,
}

/// Fill delta 处理结果：applied 是否已落库增量、delta_quantity 本次新增成交量、trade_record_id 对应成交记录 ID（若有）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FillDeltaResult {
    pub applied: bool,
    pub delta_quantity: i64,
    pub trade_record_id: Option<String>,
}

/// 订单事件入库记录：event_id 主键、order_id/client_order_id 关联、event_type 稳定字符串、event_time、details_json 事件细节。
#[derive(Debug, Clone, PartialEq)]
pub struct OrderEventRecord {
    pub event_id: String,
    pub order_id: String,
    pub client_order_id: String,
    pub event_type: String,
    pub event_time: DateTime<Utc>,
    pub details_json: Value,
}

/// 策略 runner 检查点入库记录：checkpoint_id、策略名/模式/标的/周期、last_processed_bar/last_run_id、state_json、updated_at。
#[derive(Debug, Clone, PartialEq)]
pub struct RunnerCheckpointRecord {
    pub checkpoint_id: String,
    pub strategy_name: String,
    pub mode: String,
    pub symbol: String,
    pub timeframe: String,
    pub last_processed_bar: Option<DateTime<Utc>>,
    pub last_run_id: Option<String>,
    pub state_json: Value,
    pub updated_at: DateTime<Utc>,
}

/// 策略信号入库记录：signal_id 主键、策略实例/名称/标的/周期/bar_end、信号值/状态/审批状态、关联 run_id、metadata JSON、创建/更新时间。
#[derive(Debug, Clone, PartialEq)]
pub struct StrategySignalRecord {
    pub signal_id: String,
    pub strategy_instance_id: String,
    pub strategy_name: String,
    pub symbol: String,
    pub timeframe: String,
    pub bar_end: DateTime<Utc>,
    pub signal_value: String,
    pub signal_status: SignalStatus,
    pub approval_status: ApprovalStatus,
    pub run_id: String,
    pub metadata_json: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 执行请求入库记录：request_id 主键、signal_id 关联、目标模式/账户、请求状态、审批人、创建/更新时间、payload JSON。
#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionRequestRecord {
    pub request_id: String,
    pub signal_id: String,
    pub target_mode: String,
    pub target_account: String,
    pub request_status: ExecutionRequestStatus,
    pub approved_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub payload_json: Value,
}

/// 策略守护进程检查点入库记录：checkpoint_id、策略实例/名称/标的/周期、last_processed_bar/last_run_id、state_json、updated_at。
#[derive(Debug, Clone, PartialEq)]
pub struct StrategyDaemonCheckpointRecord {
    pub checkpoint_id: String,
    pub strategy_instance_id: String,
    pub strategy_name: String,
    pub symbol: String,
    pub timeframe: String,
    pub last_processed_bar: Option<DateTime<Utc>>,
    pub last_run_id: Option<String>,
    pub state_json: Value,
    pub updated_at: DateTime<Utc>,
}

/// 将策略信号 envelope 翻译为可执行的交易意图（含价格 / 数量 / 方向 / 时间戳）。
pub fn translate_signal(
    envelope: &SignalEnvelope,
    symbol: &str,
    market_price: Decimal,
    held_volume: Option<i64>,
    policy: &ExecutionPolicy,
) -> Result<Option<OrderIntent>> {
    match envelope.signal {
        Signal::Hold => Ok(None),
        Signal::Buy => {
            let requested_quantity = board_lot_quantity(policy.fixed_cash_per_buy, market_price)?;
            if requested_quantity <= 0 {
                return Err(QuantixError::Other(
                    "strategy paper buy 可用固定金额不足以下整手单".to_string(),
                ));
            }

            Ok(Some(OrderIntent {
                symbol: symbol.to_string(),
                side: OrderSide::Buy,
                requested_quantity,
                requested_price: apply_slippage(market_price, policy.slippage_bps, true)?,
                order_type: OrderType::Market,
                reason: "signal_buy".to_string(),
                policy_snapshot_json: serde_json::json!({
                    "fixed_cash_per_buy": policy.fixed_cash_per_buy,
                    "slippage_bps": policy.slippage_bps,
                }),
            }))
        }
        Signal::Sell => {
            let requested_quantity = held_volume.unwrap_or(0);
            if requested_quantity <= 0 {
                return Err(QuantixError::Other(
                    "strategy paper sell 当前无可卖持仓".to_string(),
                ));
            }

            Ok(Some(OrderIntent {
                symbol: symbol.to_string(),
                side: OrderSide::Sell,
                requested_quantity,
                requested_price: apply_slippage(market_price, policy.slippage_bps, false)?,
                order_type: OrderType::Market,
                reason: "signal_sell".to_string(),
                policy_snapshot_json: serde_json::json!({
                    "sell_mode": "sell_all",
                    "slippage_bps": policy.slippage_bps,
                }),
            }))
        }
    }
}

fn board_lot_quantity(cash: Decimal, price: Decimal) -> Result<i64> {
    if price <= Decimal::ZERO {
        return Err(QuantixError::Other(
            "strategy paper 市价必须大于 0".to_string(),
        ));
    }

    let raw_shares = (cash / price).floor();
    let lot_count = (raw_shares / Decimal::from(100)).floor();
    lot_count
        .to_i64()
        .map(|lots| lots * 100)
        .ok_or_else(|| QuantixError::Other("strategy paper 下单数量超出支持范围".to_string()))
}

fn apply_slippage(price: Decimal, slippage_bps: u32, is_buy: bool) -> Result<Decimal> {
    if price <= Decimal::ZERO {
        return Err(QuantixError::Other(
            "strategy paper 市价必须大于 0".to_string(),
        ));
    }

    let bps = Decimal::from(slippage_bps) / Decimal::from(10_000);
    let factor = if is_buy {
        Decimal::ONE + bps
    } else {
        Decimal::ONE - bps
    };

    Ok(price * factor)
}
