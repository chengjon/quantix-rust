use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;

use crate::data::models::{AdjustType, Kline};
use crate::sources::tdx_file::FuquanFactor;

pub(super) fn scaled_price_kline(
    kline: &Kline,
    factor: Decimal,
    adjust_type: AdjustType,
) -> Kline {
    Kline {
        open: (kline.open * factor).round_dp(2),
        high: (kline.high * factor).round_dp(2),
        low: (kline.low * factor).round_dp(2),
        close: (kline.close * factor).round_dp(2),
        adjust_type,
        ..kline.clone()
    }
}

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
    scaled_price_kline(kline, adj_dec, AdjustType::QFQ)
}

pub(super) fn apply_hfq(kline: &Kline, factor: f64) -> Kline {
    let adj_dec = Decimal::from_f64(factor).unwrap_or(Decimal::ONE);
    scaled_price_kline(kline, adj_dec, AdjustType::HFQ)
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

    #[test]
    fn scaled_price_kline_scales_ohlc_and_sets_adjust_type() {
        let factor = Decimal::from_f64(2.0).unwrap();
        let adjusted = scaled_price_kline(&sample_kline(), factor, AdjustType::QFQ);

        assert_eq!(adjusted.open, Decimal::from_f64(20.0).unwrap());
        assert_eq!(adjusted.high, Decimal::from_f64(24.0).unwrap());
        assert_eq!(adjusted.low, Decimal::from_f64(18.0).unwrap());
        assert_eq!(adjusted.close, Decimal::from_f64(22.0).unwrap());
        assert!(matches!(adjusted.adjust_type, AdjustType::QFQ));
    }
}
