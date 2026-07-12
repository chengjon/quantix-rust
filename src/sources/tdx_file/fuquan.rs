use crate::core::Result;
use crate::data::models::{AdjustType, Kline};
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};

use super::TdxDayRecord;
use super::gbbq::TdxGbbqRecord;

fn fallback_date() -> chrono::NaiveDate {
    chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap_or_default()
}

/// 复权因子
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuquanFactor {
    /// 日期
    pub date: chrono::NaiveDate,
    /// 复权因子
    pub factor: f64,
    /// 前收盘价
    pub preclose: f64,
    /// 收盘价
    pub close: f64,
    /// 是否为交易日
    pub trading: bool,
    /// 是否为除权日
    pub xdxr: bool,
}

/// 复权类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FuquanType {
    /// 不复权
    None,
    /// 前复权
    QFQ,
    /// 后复权
    HFQ,
}

/// 复权计算器
pub struct FuquanCalculator;

impl FuquanCalculator {
    /// 计算复权因子（使用涨跌幅算法）
    ///
    /// 算法说明:
    /// - 基于每日涨跌幅连续计算复权因子
    /// - factor = factor * (close / preclose)
    /// - 除权日需要调整前收盘价
    pub fn calculate(
        days: &[TdxDayRecord],
        gbbqs: Option<&[TdxGbbqRecord]>,
    ) -> Result<Vec<FuquanFactor>> {
        if days.is_empty() {
            return Ok(Vec::new());
        }

        let mut factors = Vec::with_capacity(days.len());
        let mut preclose = days[0].close as f64;
        let mut factor = 1.0;

        let mut gbbq_iter = gbbqs.map(|g| g.iter()).unwrap_or([].iter()).peekable();

        let mut current_xdxr = gbbq_iter.peek().copied();

        for day in days {
            let close = day.close as f64;

            // 检查是否有除权事件
            let mut xdxr = false;
            if let Some(xdxr_record) = current_xdxr {
                if day.date == xdxr_record.date {
                    // 除权日
                    let [new_preclose, _, _] =
                        xdxr_record.compute_pre_pct(day.close, preclose, true);
                    preclose = new_preclose;
                    xdxr = true;

                    // 移动到下一个除权记录
                    gbbq_iter.next();
                    current_xdxr = gbbq_iter.peek().copied();
                } else if day.date > xdxr_record.date {
                    // 跳过已经过的除权日（非交易日）
                    gbbq_iter.next();
                    current_xdxr = gbbq_iter.peek().copied();
                }
            }

            // 计算复权因子
            factor *= close / preclose;
            preclose = close;

            factors.push(FuquanFactor {
                date: day.naive_date().unwrap_or_else(fallback_date),
                factor,
                preclose: close,
                close,
                trading: true,
                xdxr,
            });
        }

        Ok(factors)
    }

    /// 获取最新的复权因子状态（用于增量更新）
    pub fn get_latest_factor(factors: &[FuquanFactor]) -> Option<(f64, f64)> {
        factors.last().map(|f| (f.close, f.factor))
    }

    /// 应用前复权
    pub fn apply_qfq(kline: &Kline, factor: f64, latest_factor: f64) -> Kline {
        let adj_factor = latest_factor / factor;
        let adj_dec = Decimal::from_f64(adj_factor).unwrap_or(Decimal::ONE);
        Kline {
            open: (kline.open * adj_dec).round_dp(2),
            high: (kline.high * adj_dec).round_dp(2),
            low: (kline.low * adj_dec).round_dp(2),
            close: (kline.close * adj_dec).round_dp(2),
            adjust_type: AdjustType::QFQ,
            ..kline.clone()
        }
    }

    /// 应用后复权
    pub fn apply_hfq(kline: &Kline, factor: f64) -> Kline {
        let adj_dec = Decimal::from_f64(factor).unwrap_or(Decimal::ONE);
        Kline {
            open: (kline.open * adj_dec).round_dp(2),
            high: (kline.high * adj_dec).round_dp(2),
            low: (kline.low * adj_dec).round_dp(2),
            close: (kline.close * adj_dec).round_dp(2),
            adjust_type: AdjustType::HFQ,
            ..kline.clone()
        }
    }
}
