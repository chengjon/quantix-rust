//! Order reconciliation module for execution system
//!
//! Provides open-order scanning, account reconciliation, and state repair capabilities.
#![allow(clippy::should_implement_trait)]
//!
//! # Features
//!
//! - **Open Order Scanner**: Scan and query open orders across all adapters
//! - **Account Reconciliation**: Compare local state with broker state
//! - **State Repair**: Handle inconsistencies between local and broker states
//! - **Recovery**: Recover from Unknown order states

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::bridge::error::BridgeError;
use crate::bridge::models::BridgeBrokerEventType;
use crate::core::QuantixError;
use crate::execution::models::{
    OrderRecord, OrderStatus, QmtLiveLastQuerySummary, QmtLiveReconciliationState,
    QmtLiveRuntimeMetadata, QmtLiveTaskIdentity,
};
use crate::execution::qmt_task_submit_service::{QmtTaskResolvedResult, QmtTaskSubmitService};
use crate::execution::runtime_store::StrategyRuntimeStore;

mod order_state;
mod qmt_live;
mod scanner;
mod service_core;

#[allow(unused_imports)]
pub use order_state::*;
#[allow(unused_imports)]
pub use qmt_live::*;
#[allow(unused_imports)]
pub use scanner::*;
#[allow(unused_imports)]
pub use service_core::*;

/// 单次对账汇总：reconciled_at 执行时间、total_open_orders 扫描的未终结订单数、matched_orders 匹配数、mismatched_orders 状态不一致数、recovered_orders 从 Unknown 恢复数、failed_orders 失败数、duration_ms 耗时毫秒。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationSummary {
    /// When this reconciliation was performed
    pub reconciled_at: DateTime<Utc>,
    /// Total open orders scanned
    pub total_open_orders: usize,
    /// Orders that matched broker state
    pub matched_orders: usize,
    /// Orders with state discrepancies
    pub mismatched_orders: usize,
    /// Orders recovered from Unknown state
    pub recovered_orders: usize,
    /// Orders that failed to reconcile
    pub failed_orders: usize,
    /// Time taken in milliseconds
    pub duration_ms: u64,
}

/// 单订单对账结果：order_id/client_order_id 订单标识、symbol 标的、local_status 本地状态、broker_status broker 状态（可空）、action 采取的动作、success 是否成功、error 失败信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderReconciliationResult {
    /// Order ID
    pub order_id: String,
    /// Client order ID
    pub client_order_id: String,
    /// Symbol
    pub symbol: String,
    /// Local status before reconciliation
    pub local_status: OrderStatus,
    /// Broker status (if available)
    pub broker_status: Option<OrderStatus>,
    /// Action taken
    pub action: ReconciliationAction,
    /// Whether reconciliation succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// 对账动作：NoAction 一致无需动作、StateUpdated 本地状态已同步 broker、Recovered 从 Unknown 恢复、MarkedFailed 超时标记失败、Cancelled 因不一致已撤单、ManualIntervention 需人工介入。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReconciliationAction {
    /// No action needed - states match
    NoAction,
    /// Updated local state to match broker
    StateUpdated,
    /// Order was in Unknown state and recovered
    Recovered,
    /// Order was marked as failed due to timeout
    MarkedFailed,
    /// Order was cancelled due to discrepancy
    Cancelled,
    /// Manual intervention required
    ManualIntervention,
}

impl ReconciliationAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NoAction => "no_action",
            Self::StateUpdated => "state_updated",
            Self::Recovered => "recovered",
            Self::MarkedFailed => "marked_failed",
            Self::Cancelled => "cancelled",
            Self::ManualIntervention => "manual_intervention",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "no_action" => Some(Self::NoAction),
            "state_updated" => Some(Self::StateUpdated),
            "recovered" => Some(Self::Recovered),
            "marked_failed" => Some(Self::MarkedFailed),
            "cancelled" => Some(Self::Cancelled),
            "manual_intervention" => Some(Self::ManualIntervention),
            _ => None,
        }
    }
}

/// 挂单扫描器：基于 StrategyRuntimeStore 查找未终结订单，识别过期订单（stale）与 Unknown 订单。阈值由 with_thresholds 配置，默认 stale 1h / unknown 5min。
pub struct OpenOrderScanner {
    store: StrategyRuntimeStore,
    /// Maximum age in seconds for an order to be considered "stale"
    stale_order_threshold_seconds: i64,
    /// Maximum age in seconds for an Unknown order before marking failed
    unknown_timeout_seconds: i64,
}

/// 挂单扫描汇总：total_open 总挂单数、by_status 按状态计数、stale_count 过期数、unknown_count Unknown 数、stale_threshold_seconds 过期阈值、unknown_timeout_seconds Unknown 超时阈值。
pub struct OpenOrderSummary {
    /// Total number of open orders
    pub total_open: usize,
    /// Count by status
    pub by_status: HashMap<String, usize>,
    /// Number of stale orders
    pub stale_count: usize,
    /// Number of orders in Unknown state
    pub unknown_count: usize,
    /// Stale threshold in seconds
    pub stale_threshold_seconds: i64,
    /// Unknown timeout in seconds
    pub unknown_timeout_seconds: i64,
}

/// 对账服务：扫描挂单 + 比对本地与 broker 状态 + 修复不一致。可选注入 QmtTaskSubmitService 用于 qmt_live 订单的恢复。
pub struct ReconciliationService {
    store: StrategyRuntimeStore,
    scanner: OpenOrderScanner,
    qmt_submit_service: Option<QmtTaskSubmitService>,
}

/// 对账报告：summary 总览统计、results 逐单结果明细。
pub struct ReconciliationReport {
    /// Summary statistics
    pub summary: ReconciliationSummary,
    /// Individual order results
    pub results: Vec<OrderReconciliationResult>,
}

#[cfg(test)]
mod tests;
