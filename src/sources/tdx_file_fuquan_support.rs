use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;

use crate::data::models::{AdjustType, Kline};
use crate::sources::tdx_file::FuquanFactor;

pub(super) fn get_latest_factor(factors: &[FuquanFactor]) -> Option<(f64, f64)> {
    factors.last().map(|factor| (factor.close, factor.factor))
}

pub(super) fn apply_qfq(
    kline: &Kline,
    factor: f64,
    latest_factor: f64,
) -> Kline {
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

pub(super) fn apply_hfq(kline: &Kline, factor: f64) -> Kline {
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

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;

    fn sample_kline() -> Kline {
        Kline {
            code: "000001".to_string(),
            date: NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
            open: Decimal::from_f64(10.0).unwrap(),
            high: Decimal::from_f64(12.0).unwrap(),
            low: Decimal::from_f64(9.0).unwrap(),
            close: Decimal::from_f64(11.0).unwrap(),
            volume: 1000,
            amount: None,
            adjust_type: AdjustType::None,
        }
    }

    #[test]
    fn apply_qfq_scales_prices_by_latest_over_factor() {
        let adjusted = apply_qfq(&sample_kline(), 2.0, 4.0);
        assert_eq!(adjusted.open, Decimal::from_f64(20.0).unwrap());
        assert_eq!(adjusted.close, Decimal::from_f64(22.0).unwrap());
        assert!(matches!(adjusted.adjust_type, AdjustType::QFQ));
    }

    #[test]
    fn apply_hfq_scales_prices_by_factor() {
        let adjusted = apply_hfq(&sample_kline(), 1.5);
        assert_eq!(adjusted.open, Decimal::from_f64(15.0).unwrap());
        assert_eq!(adjusted.close, Decimal::from_f64(16.5).unwrap());
        assert!(matches!(adjusted.adjust_type, AdjustType::HFQ));
    }
}
