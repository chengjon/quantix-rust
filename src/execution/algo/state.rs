//! Algorithm State Machine
//!
//! 算法状态定义和状态机实现

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 算法状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlgoStatus {
    /// 已创建，等待启动
    Pending,
    /// 运行中
    Running,
    /// 已暂停
    Paused,
    /// 已完成
    Completed,
    /// 已取消
    Cancelled,
    /// 错误状态
    Error,
}

impl std::fmt::Display for AlgoStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlgoStatus::Pending => write!(f, "pending"),
            AlgoStatus::Running => write!(f, "running"),
            AlgoStatus::Paused => write!(f, "paused"),
            AlgoStatus::Completed => write!(f, "completed"),
            AlgoStatus::Cancelled => write!(f, "cancelled"),
            AlgoStatus::Error => write!(f, "error"),
        }
    }
}

/// 算法执行状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgoState {
    /// 算法ID
    pub algo_id: String,
    /// 当前状态
    pub status: AlgoStatus,
    /// 股票代码
    pub symbol: String,
    /// 买卖方向
    pub side: String,
    /// 目标数量
    pub target_quantity: i64,
    /// 已成交数量
    pub filled_quantity: i64,
    /// 平均成交价
    pub avg_fill_price: Decimal,
    /// 总成交金额
    pub total_amount: Decimal,
    /// 订单数量
    pub order_count: u32,
    /// 成交数量
    pub fill_count: u32,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 开始时间
    pub started_at: Option<DateTime<Utc>>,
    /// 完成时间
    pub completed_at: Option<DateTime<Utc>>,
    /// 最后更新时间
    pub updated_at: DateTime<Utc>,
    /// 错误信息
    pub error_message: Option<String>,
    /// 滑点 (基点)
    pub slippage_bps: Option<Decimal>,
}

impl AlgoState {
    /// 创建新的算法状态
    pub fn new(algo_id: String, symbol: String, side: String, target_quantity: i64) -> Self {
        let now = Utc::now();
        Self {
            algo_id,
            status: AlgoStatus::Pending,
            symbol,
            side,
            target_quantity,
            filled_quantity: 0,
            avg_fill_price: Decimal::ZERO,
            total_amount: Decimal::ZERO,
            order_count: 0,
            fill_count: 0,
            created_at: now,
            started_at: None,
            completed_at: None,
            updated_at: now,
            error_message: None,
            slippage_bps: None,
        }
    }

    /// 计算完成百分比
    pub fn completion_percent(&self) -> Decimal {
        if self.target_quantity == 0 {
            return Decimal::ZERO;
        }
        Decimal::from(self.filled_quantity) * Decimal::from(100) / Decimal::from(self.target_quantity)
    }

    /// 计算剩余数量
    pub fn remaining_quantity(&self) -> i64 {
        self.target_quantity - self.filled_quantity
    }

    /// 是否已完成
    pub fn is_finished(&self) -> bool {
        matches!(self.status, AlgoStatus::Completed | AlgoStatus::Cancelled | AlgoStatus::Error)
    }

    /// 启动算法
    pub fn start(&mut self) {
        if self.status == AlgoStatus::Pending {
            self.status = AlgoStatus::Running;
            self.started_at = Some(Utc::now());
            self.updated_at = Utc::now();
        }
    }

    /// 暂停算法
    pub fn pause(&mut self) {
        if self.status == AlgoStatus::Running {
            self.status = AlgoStatus::Paused;
            self.updated_at = Utc::now();
        }
    }

    /// 恢复算法
    pub fn resume(&mut self) {
        if self.status == AlgoStatus::Paused {
            self.status = AlgoStatus::Running;
            self.updated_at = Utc::now();
        }
    }

    /// 完成算法
    pub fn complete(&mut self) {
        if !self.is_finished() {
            self.status = AlgoStatus::Completed;
            self.completed_at = Some(Utc::now());
            self.updated_at = Utc::now();
        }
    }

    /// 取消算法
    pub fn cancel(&mut self) {
        if !self.is_finished() {
            self.status = AlgoStatus::Cancelled;
            self.completed_at = Some(Utc::now());
            self.updated_at = Utc::now();
        }
    }

    /// 设置错误
    pub fn set_error(&mut self, message: String) {
        self.status = AlgoStatus::Error;
        self.error_message = Some(message);
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// 更新成交
    pub fn update_fill(&mut self, fill_quantity: i64, fill_price: Decimal) {
        let new_total_qty = self.filled_quantity + fill_quantity;
        let new_total_amount = self.total_amount + fill_price * Decimal::from(fill_quantity);

        if new_total_qty > 0 {
            self.avg_fill_price = new_total_amount / Decimal::from(new_total_qty);
        }

        self.filled_quantity = new_total_qty;
        self.total_amount = new_total_amount;
        self.fill_count += 1;
        self.updated_at = Utc::now();
    }

    /// 记录下单
    pub fn record_order(&mut self) {
        self.order_count += 1;
        self.updated_at = Utc::now();
    }
}

/// 子订单状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildOrder {
    /// 子订单ID
    pub order_id: String,
    /// 父算法ID
    pub algo_id: String,
    /// 计划时间
    pub scheduled_time: DateTime<Utc>,
    /// 计划数量
    pub scheduled_quantity: i64,
    /// 计划价格
    pub scheduled_price: Option<Decimal>,
    /// 实际下单数量
    pub order_quantity: i64,
    /// 实际下单价格
    pub order_price: Option<Decimal>,
    /// 成交数量
    pub filled_quantity: i64,
    /// 成交均价
    pub avg_fill_price: Decimal,
    /// 状态
    pub status: ChildOrderStatus,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

/// 子订单状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChildOrderStatus {
    /// 等待下单
    Pending,
    /// 已下单
    Submitted,
    /// 部分成交
    PartiallyFilled,
    /// 完全成交
    Filled,
    /// 已取消
    Cancelled,
    /// 被拒绝
    Rejected,
}
