use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;

use crate::data::models::{AdjustType, Kline};
use crate::sources::tdx_file::{FuquanFactor, TdxDayData, TdxDayRecord};

pub(super) fn from_record(record: &TdxDayRecord, factor: &FuquanFactor) -> TdxDayData {
    let change_pct = if factor.preclose > 0.0 {
        Decimal::from_f64((factor.close - factor.preclose) / factor.preclose * 100.0)
            .map(|value| value.round_dp(2))
            .unwrap_or(Decimal::ZERO)
    } else {
        Decimal::ZERO
    };

    TdxDayData {
        code: record.code_string(),
        date: record
            .naive_date()
            .unwrap_or_else(|| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()),
        open: Decimal::from_f32(record.open)
            .map(|value| value.round_dp(2))
            .unwrap_or(Decimal::ZERO),
        high: Decimal::from_f32(record.high)
            .map(|value| value.round_dp(2))
            .unwrap_or(Decimal::ZERO),
        low: Decimal::from_f32(record.low)
            .map(|value| value.round_dp(2))
            .unwrap_or(Decimal::ZERO),
        close: Decimal::from_f32(record.close)
            .map(|value| value.round_dp(2))
            .unwrap_or(Decimal::ZERO),
        volume: record.volume as i64,
        amount: Decimal::from_f32(record.amount)
            .map(|value| value.round_dp(2))
            .unwrap_or(Decimal::ZERO),
        preclose: Decimal::from_f64(factor.preclose)
            .map(|value| value.round_dp(2))
            .unwrap_or(Decimal::ZERO),
        factor: Decimal::from_f64(factor.factor)
            .map(|value| value.round_dp(6))
            .unwrap_or(Decimal::ONE),
        change_pct,
    }
}

pub(super) fn to_kline(day_data: &TdxDayData, adjust_type: AdjustType) -> Kline {
    Kline {
        code: day_data.code.clone(),
        date: day_data.date,
        open: day_data.open,
        high: day_data.high,
        low: day_data.low,
        close: day_data.close,
        volume: day_data.volume,
        amount: Some(day_data.amount),
        adjust_type,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_record() -> TdxDayRecord {
        TdxDayRecord {
            code: 600000,
            date: 20210801,
            open: 10.123,
            high: 11.456,
            low: 9.789,
            close: 10.987,
            amount: 12345.678,
            volume: 1000,
        }
    }

    fn sample_factor() -> FuquanFactor {
        FuquanFactor {
            date: NaiveDate::from_ymd_opt(2021, 8, 1).unwrap(),
            factor: 1.23456789,
            preclose: 10.0,
            close: 10.987,
            trading: true,
            xdxr: false,
        }
    }

    #[test]
    fn from_record_rounds_numeric_fields_and_change_pct() {
        let day_data = from_record(&sample_record(), &sample_factor());

        assert_eq!(day_data.code, "600000");
        assert_eq!(day_data.date, NaiveDate::from_ymd_opt(2021, 8, 1).unwrap());
        assert_eq!(day_data.open, Decimal::from_f64(10.12).unwrap());
        assert_eq!(day_data.amount, Decimal::from_f64(12345.68).unwrap());
        assert_eq!(day_data.preclose, Decimal::from_f64(10.0).unwrap());
        assert_eq!(day_data.factor, Decimal::from_f64(1.234568).unwrap());
        assert_eq!(day_data.change_pct, Decimal::from_f64(9.87).unwrap());
    }

    #[test]
    fn to_kline_preserves_ohlcv_and_wraps_amount() {
        let day_data = from_record(&sample_record(), &sample_factor());
        let kline = to_kline(&day_data, AdjustType::QFQ);

        assert_eq!(kline.code, "600000");
        assert_eq!(kline.date, NaiveDate::from_ymd_opt(2021, 8, 1).unwrap());
        assert_eq!(kline.open, Decimal::from_f64(10.12).unwrap());
        assert_eq!(kline.close, Decimal::from_f64(10.99).unwrap());
        assert_eq!(kline.volume, 1000);
        assert_eq!(kline.amount, Some(Decimal::from_f64(12345.68).unwrap()));
        assert!(matches!(kline.adjust_type, AdjustType::QFQ));
    }
}
