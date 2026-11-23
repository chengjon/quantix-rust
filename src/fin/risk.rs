//! 研究侧回撤风险门面（落地主线任务「回撤控制」的分析能力）。
//!
//! 口径对比：本门面回撤为百分数 0-100（`current_pct`/`worst_pct`）；fin
//! `BacktestResult.max_drawdown` 为 0-1 小数——两套口径勿混用。

use fin_primitives::risk::DrawdownTracker;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::core::{QuantixError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawdownReport {
    /// 当前回撤百分数（0-100）
    pub current_pct: Decimal,
    /// 最大回撤百分数（0-100）
    pub worst_pct: Decimal,
    /// 序列期内净值最高点（首元素为初始净值，但不封顶——后续 update 可上移 peak）
    pub peak: Decimal,
    /// 最新净值
    pub current_equity: Decimal,
    /// 距最近一次新峰的 update 次数（0 = 最近一次 update 即新峰）
    pub duration_bars: usize,
    /// 当前是否处于回撤中（current_equity < peak）
    pub in_drawdown: bool,
}

/// 逐 bar 净值序列 → 回撤报告。序列按时间升序，首元素为初始净值。
pub fn drawdown_report(equity: &[Decimal]) -> Result<DrawdownReport> {
    let Some((&initial, rest)) = equity.split_first() else {
        return Err(QuantixError::Config("净值序列为空".into()));
    };
    let mut tracker = DrawdownTracker::new(initial);
    for &value in rest {
        tracker.update(value);
    }
    Ok(DrawdownReport {
        current_pct: tracker.current_drawdown_pct(),
        worst_pct: tracker.worst_drawdown_pct(),
        peak: tracker.peak(),
        current_equity: tracker.current_equity(),
        duration_bars: tracker.drawdown_duration(),
        in_drawdown: tracker.in_drawdown(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    fn d(v: i64) -> Decimal {
        Decimal::from(v)
    }

    #[test]
    fn empty_equity_is_config_error() {
        let err = drawdown_report(&[]).unwrap_err();
        assert!(matches!(err, QuantixError::Config(_)));
    }

    #[test]
    fn drawdown_percentages_are_exact() {
        let report = drawdown_report(&[d(100), d(110), d(99)]).unwrap();
        assert_eq!(report.peak, d(110));
        assert_eq!(report.current_equity, d(99));
        // (110-99)/110*100 = 10.00（除法精确）
        assert_eq!(report.current_pct, Decimal::from_str("10").unwrap());
        assert_eq!(report.worst_pct, Decimal::from_str("10").unwrap());
        assert!(report.in_drawdown);
        // update(110) 归零 since_peak，update(99) 低于峰值 1 次 → duration = 1
        assert_eq!(report.duration_bars, 1);
    }

    /// 0 起始序列（首元素为 0）：初始净值不封顶 peak——update(5) 将 peak 上移至 5，
    /// 回撤 = (5-3)/5×100 = 40（百分数口径精确）；持续 1 根、处于回撤中。
    #[test]
    fn zero_start_series_tracks_peak_growth() {
        let report = drawdown_report(&[d(0), d(5), d(3)]).unwrap();
        assert_eq!(report.peak, d(5));
        assert_eq!(report.current_equity, d(3));
        assert_eq!(report.current_pct, Decimal::from_str("40").unwrap());
        assert_eq!(report.worst_pct, Decimal::from_str("40").unwrap());
        assert_eq!(report.duration_bars, 1);
        assert!(report.in_drawdown);
    }

    /// peak=0 守卫钉（上游 risk/mod.rs:239 `if peak == 0 return ZERO`）：全零序列的
    /// 两次 update 均走 else 分支（updates_since_peak 逐次累计 → duration=2），pct 经守卫
    /// 安全取 0——不 panic、无除零。注：`in_drawdown=true 且 pct=0` 在上游不可能
    /// （pct=0 ⇒ peak=0 ⇒ current=0=peak ⇒ 不在回撤），守卫的实际表现即本测试钉住的三元组。
    #[test]
    fn all_zero_series_guard_yields_zero_pct_without_panic() {
        let report = drawdown_report(&[d(0), d(0), d(0)]).unwrap();
        assert_eq!(report.peak, d(0));
        assert_eq!(report.current_pct, Decimal::ZERO);
        assert_eq!(report.worst_pct, Decimal::ZERO);
        assert_eq!(report.duration_bars, 2);
        assert!(!report.in_drawdown);
    }

    /// 单元素序列：首元素即初始净值、无 update 循环——全零回撤、duration=0、不在回撤中。
    #[test]
    fn single_element_series_has_no_drawdown() {
        let report = drawdown_report(&[d(100)]).unwrap();
        assert_eq!(report.peak, d(100));
        assert_eq!(report.current_equity, d(100));
        assert_eq!(report.current_pct, Decimal::ZERO);
        assert_eq!(report.worst_pct, Decimal::ZERO);
        assert_eq!(report.duration_bars, 0);
        assert!(!report.in_drawdown);
    }

    #[test]
    fn never_in_drawdown_when_rising() {
        let report = drawdown_report(&[d(100), d(101), d(102)]).unwrap();
        assert!(!report.in_drawdown);
        assert_eq!(report.worst_pct, Decimal::ZERO);
    }
}
