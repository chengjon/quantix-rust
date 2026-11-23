//! 研究侧回撤风险门面（落地主线任务「回撤控制」的分析能力）。

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
    pub peak: Decimal,
    pub current_equity: Decimal,
    pub duration_bars: usize,
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
    }

    #[test]
    fn never_in_drawdown_when_rising() {
        let report = drawdown_report(&[d(100), d(101), d(102)]).unwrap();
        assert!(!report.in_drawdown);
        assert_eq!(report.worst_pct, Decimal::ZERO);
    }
}
