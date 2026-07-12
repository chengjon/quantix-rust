#![allow(clippy::should_implement_trait)]

use serde::{Deserialize, Serialize};

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
