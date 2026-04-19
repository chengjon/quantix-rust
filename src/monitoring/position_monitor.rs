#![allow(clippy::collapsible_if)]

/// 持仓监控模块
///
/// 实时追踪持仓状态变化
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::analysis::portfolio::Position;

#[cfg(test)]
mod tests;

/// 持仓监控配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionMonitorConfig {
    /// 启用快照记录
    pub enable_snapshot: bool,
    /// 快照间隔（秒）
    pub snapshot_interval_secs: u64,
    /// 启用盈亏监控
    pub enable_pnl_monitoring: bool,
    /// 最大持仓比例告警阈值（0-1）
    pub max_position_ratio_threshold: Decimal,
    /// 启用持仓变化通知
    pub enable_change_notification: bool,
}

impl Default for PositionMonitorConfig {
    fn default() -> Self {
        Self {
            enable_snapshot: true,
            snapshot_interval_secs: 60, // 1分钟
            enable_pnl_monitoring: true,
            max_position_ratio_threshold: dec!(0.2), // 20%
            enable_change_notification: true,
        }
    }
}

/// 持仓快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionSnapshot {
    /// 快照时间
    pub timestamp: DateTime<Utc>,
    /// 所有持仓
    pub positions: HashMap<String, PositionInfo>,
    /// 总市值
    pub total_market_value: Decimal,
    /// 总成本
    pub total_cost: Decimal,
    /// 总浮动盈亏
    pub total_pnl: Decimal,
    /// 总浮动盈亏比例
    pub total_pnl_percent: Decimal,
    /// 持仓数量
    pub position_count: usize,
}

/// 持仓信息（简化版）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionInfo {
    /// 股票代码
    pub code: String,
    /// 持仓数量
    pub quantity: i64,
    /// 平均成本
    pub avg_cost: Decimal,
    /// 当前价格
    pub current_price: Decimal,
    /// 市值
    pub market_value: Decimal,
    /// 浮动盈亏
    pub pnl: Decimal,
    /// 浮动盈亏比例
    pub pnl_percent: Decimal,
    /// 开仓日期
    pub open_date: NaiveDate,
    /// 持仓天数
    pub holding_days: i64,
}

impl From<&Position> for PositionInfo {
    fn from(pos: &Position) -> Self {
        Self {
            code: pos.code.clone(),
            quantity: pos.quantity,
            avg_cost: pos.avg_cost,
            current_price: if pos.quantity > 0 {
                pos.market_value / Decimal::from(pos.quantity)
            } else {
                Decimal::ZERO
            },
            market_value: pos.market_value,
            pnl: pos.pnl,
            pnl_percent: pos.pnl_percent,
            open_date: pos.open_date,
            holding_days: (Utc::now().naive_utc().date() - pos.open_date).num_days(),
        }
    }
}

/// 持仓变化类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PositionChangeType {
    New,
    Increased,
    Decreased,
    Closed,
    PriceUpdated,
}

/// 持仓变化事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionChangeEvent {
    /// 变化类型
    pub change_type: PositionChangeType,
    /// 股票代码
    pub code: String,
    /// 变化时间
    pub timestamp: DateTime<Utc>,
    /// 变化前持仓（如果有）
    pub before: Option<PositionInfo>,
    /// 变化后持仓
    pub after: PositionInfo,
    /// 变化数量
    pub quantity_change: i64,
    /// 市值变化
    pub value_change: Decimal,
}

/// 持仓监控器
pub struct PositionMonitor {
    /// 配置
    config: PositionMonitorConfig,
    /// 当前持仓
    current_positions: HashMap<String, PositionInfo>,
    /// 快照历史
    snapshots: Vec<PositionSnapshot>,
    /// 变化事件历史
    change_history: Vec<PositionChangeEvent>,
    /// 初始资金
    initial_capital: Decimal,
    /// 当前总权益
    current_equity: Decimal,
}

impl PositionMonitor {
    /// 创建新的持仓监控器
    pub fn new(config: PositionMonitorConfig, initial_capital: Decimal) -> Self {
        Self {
            config,
            current_positions: HashMap::new(),
            snapshots: Vec::new(),
            change_history: Vec::new(),
            initial_capital,
            current_equity: initial_capital,
        }
    }

    /// 使用默认配置创建
    pub fn with_defaults(initial_capital: Decimal) -> Self {
        Self::new(PositionMonitorConfig::default(), initial_capital)
    }

    /// 更新持仓
    pub fn update_positions(&mut self, positions: &[Position]) {
        let mut new_positions = HashMap::new();
        let mut change_events = Vec::new();

        for pos in positions {
            let info = PositionInfo::from(pos);
            new_positions.insert(pos.code.clone(), info.clone());

            // 检测变化
            if let Some(old_info) = self.current_positions.get(&pos.code).cloned() {
                let event = self.detect_position_change_direct(&old_info, &info, pos);
                if let Some(evt) = event {
                    change_events.push(evt);
                }
            } else {
                // 新持仓
                let event = PositionChangeEvent {
                    change_type: PositionChangeType::New,
                    code: pos.code.clone(),
                    timestamp: Utc::now(),
                    before: None,
                    after: info,
                    quantity_change: pos.quantity,
                    value_change: pos.market_value,
                };
                change_events.push(event);
            }
        }

        // 检测已平仓的持仓
        let old_positions = self.current_positions.clone();
        for (code, old_info) in old_positions {
            if !new_positions.contains_key(&code) {
                let event = PositionChangeEvent {
                    change_type: PositionChangeType::Closed,
                    code: code.clone(),
                    timestamp: Utc::now(),
                    before: Some(old_info.clone()),
                    after: PositionInfo {
                        code: code.clone(),
                        quantity: 0,
                        avg_cost: Decimal::ZERO,
                        current_price: Decimal::ZERO,
                        market_value: Decimal::ZERO,
                        pnl: Decimal::ZERO,
                        pnl_percent: Decimal::ZERO,
                        open_date: old_info.open_date,
                        holding_days: (Utc::now().naive_utc().date() - old_info.open_date)
                            .num_days(),
                    },
                    quantity_change: -old_info.quantity,
                    value_change: -old_info.market_value,
                };
                change_events.push(event);
            }
        }

        self.current_positions = new_positions;
        self.update_equity();

        // 记录所有变化事件
        for event in change_events {
            self.record_change_event(event);
        }
    }

    /// 检测持仓变化（直接版本，返回事件而不是记录）
    fn detect_position_change_direct(
        &self,
        before: &PositionInfo,
        after: &PositionInfo,
        _pos: &Position,
    ) -> Option<PositionChangeEvent> {
        let change_type = if before.quantity != after.quantity {
            if after.quantity > before.quantity {
                PositionChangeType::Increased
            } else {
                PositionChangeType::Decreased
            }
        } else if before.current_price != after.current_price {
            PositionChangeType::PriceUpdated
        } else {
            return None; // 无变化
        };

        Some(PositionChangeEvent {
            change_type,
            code: before.code.clone(),
            timestamp: Utc::now(),
            before: Some(before.clone()),
            after: after.clone(),
            quantity_change: after.quantity - before.quantity,
            value_change: after.market_value - before.market_value,
        })
    }

    /// 记录变化事件
    fn record_change_event(&mut self, event: PositionChangeEvent) {
        if self.config.enable_change_notification {
            self.change_history.push(event);
        }
    }

    /// 更新权益
    fn update_equity(&mut self) {
        let _total_market_value: Decimal = self
            .current_positions
            .values()
            .map(|p| p.market_value)
            .sum();

        let total_pnl: Decimal = self.current_positions.values().map(|p| p.pnl).sum();

        self.current_equity = self.initial_capital + total_pnl;
    }

    /// 创建快照
    pub fn create_snapshot(&mut self) -> PositionSnapshot {
        let positions = self.current_positions.clone();
        let total_market_value: Decimal = positions.values().map(|p| p.market_value).sum();
        let total_cost: Decimal = positions
            .values()
            .map(|p| p.avg_cost * Decimal::from(p.quantity))
            .sum();
        let total_pnl: Decimal = positions.values().map(|p| p.pnl).sum();
        let total_pnl_percent = if total_cost > Decimal::ZERO {
            (total_pnl / total_cost) * Decimal::from(100)
        } else {
            Decimal::ZERO
        };

        let snapshot = PositionSnapshot {
            timestamp: Utc::now(),
            positions,
            total_market_value,
            total_cost,
            total_pnl,
            total_pnl_percent,
            position_count: self.current_positions.len(),
        };

        if self.config.enable_snapshot {
            self.snapshots.push(snapshot.clone());
        }

        snapshot
    }

    /// 获取当前持仓
    pub fn get_current_positions(&self) -> &HashMap<String, PositionInfo> {
        &self.current_positions
    }

    /// 获取持仓信息
    pub fn get_position(&self, code: &str) -> Option<&PositionInfo> {
        self.current_positions.get(code)
    }

    /// 获取当前权益
    pub fn get_current_equity(&self) -> Decimal {
        self.current_equity
    }

    /// 获取总盈亏
    pub fn get_total_pnl(&self) -> Decimal {
        self.current_positions.values().map(|p| p.pnl).sum()
    }

    /// 获取总盈亏比例
    pub fn get_total_pnl_percent(&self) -> Decimal {
        let total_cost: Decimal = self
            .current_positions
            .values()
            .map(|p| p.avg_cost * Decimal::from(p.quantity))
            .sum();

        if total_cost > Decimal::ZERO {
            let total_pnl = self.get_total_pnl();
            (total_pnl / total_cost) * Decimal::from(100)
        } else {
            Decimal::ZERO
        }
    }

    /// 获取持仓数量
    pub fn get_position_count(&self) -> usize {
        self.current_positions.len()
    }

    /// 检查持仓比例是否超限
    pub fn check_position_ratio(&self, code: &str) -> bool {
        if let Some(pos) = self.get_position(code) {
            if self.current_equity > Decimal::ZERO {
                let ratio = pos.market_value / self.current_equity;
                return ratio > self.config.max_position_ratio_threshold;
            }
        }
        false
    }

    /// 获取变化事件
    pub fn get_change_events(&self) -> &[PositionChangeEvent] {
        &self.change_history
    }

    /// 获取最近的变化事件
    pub fn get_recent_changes(&self, count: usize) -> &[PositionChangeEvent] {
        let len = self.change_history.len().min(count);
        &self.change_history[self.change_history.len() - len..]
    }

    /// 获取快照历史
    pub fn get_snapshots(&self) -> &[PositionSnapshot] {
        &self.snapshots
    }

    /// 清空快照历史
    pub fn clear_snapshots(&mut self) {
        self.snapshots.clear();
    }

    /// 清空变化事件
    pub fn clear_change_history(&mut self) {
        self.change_history.clear();
    }
}
