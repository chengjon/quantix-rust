//! Performance report struct, equity/trade records, and the trade-side enum.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 回测性能报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    /// 总收益率
    pub total_return: Decimal,
    /// 年化收益率
    pub annual_return: Decimal,
    /// 最大回撤
    pub max_drawdown: Decimal,
    /// 夏普比率
    pub sharpe_ratio: Decimal,
    /// 索提诺比率
    pub sortino_ratio: Decimal,
    /// 胜率
    pub win_rate: Decimal,
    /// 盈亏比
    pub profit_loss_ratio: Decimal,
    /// 总交易次数
    pub total_trades: usize,
    /// 盈利交易次数
    pub win_trades: usize,
    /// 亏损交易次数
    pub loss_trades: usize,
    /// 平均盈利
    pub avg_win: Decimal,
    /// 平均亏损
    pub avg_loss: Decimal,
    /// 最大盈利
    pub max_win: Decimal,
    /// 最大亏损
    pub max_loss: Decimal,
    /// 最大连续盈利次数
    pub max_consecutive_wins: usize,
    /// 最大连续亏损次数
    pub max_consecutive_losses: usize,
    /// 总手续费
    pub total_commission: Decimal,
    /// 卡玛比率
    pub calmar_ratio: Decimal,
}

impl Default for PerformanceReport {
    fn default() -> Self {
        Self {
            total_return: Decimal::ZERO,
            annual_return: Decimal::ZERO,
            max_drawdown: Decimal::ZERO,
            sharpe_ratio: Decimal::ZERO,
            sortino_ratio: Decimal::ZERO,
            win_rate: Decimal::ZERO,
            profit_loss_ratio: Decimal::ZERO,
            total_trades: 0,
            win_trades: 0,
            loss_trades: 0,
            avg_win: Decimal::ZERO,
            avg_loss: Decimal::ZERO,
            max_win: Decimal::ZERO,
            max_loss: Decimal::ZERO,
            max_consecutive_wins: 0,
            max_consecutive_losses: 0,
            total_commission: Decimal::ZERO,
            calmar_ratio: Decimal::ZERO,
        }
    }
}

/// 账户价值记录
#[derive(Debug, Clone)]
pub struct EquityPoint {
    /// 日期
    pub date: NaiveDate,
    /// 账户价值
    pub equity: Decimal,
    /// 当日收益
    pub daily_return: Decimal,
}

/// 交易记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRecord {
    /// 股票代码
    pub code: String,
    /// 交易方向
    pub side: TradeSide,
    /// 开仓日期
    pub open_date: NaiveDate,
    /// 平仓日期
    pub close_date: NaiveDate,
    /// 开仓价格
    pub open_price: Decimal,
    /// 平仓价格
    pub close_price: Decimal,
    /// 数量
    pub quantity: i64,
    /// 盈亏
    pub pnl: Decimal,
    /// 盈亏比例
    pub pnl_percent: Decimal,
    /// 手续费
    pub commission: Decimal,
}

/// 交易方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TradeSide {
    Long,
    Short,
}
