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

    struct DayRecordFixtureConfig {
        code: u32,
        date: u32,
        open: f32,
        high: f32,
        low: f32,
        close: f32,
        amount: f32,
        volume: u32,
    }

    impl Default for DayRecordFixtureConfig {
        fn default() -> Self {
            Self {
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
    }

    struct FuquanFactorFixtureConfig {
        date: NaiveDate,
        factor: f64,
        preclose: f64,
        close: f64,
        trading: bool,
        xdxr: bool,
    }

    impl Default for FuquanFactorFixtureConfig {
        fn default() -> Self {
            Self {
                date: NaiveDate::from_ymd_opt(2021, 8, 1).unwrap(),
                factor: 1.23456789,
                preclose: 10.0,
                close: 10.987,
                trading: true,
                xdxr: false,
            }
        }
    }

    fn build_record(config: &DayRecordFixtureConfig) -> TdxDayRecord {
        TdxDayRecord {
            code: config.code,
            date: config.date,
            open: config.open,
            high: config.high,
            low: config.low,
            close: config.close,
            amount: config.amount,
            volume: config.volume,
        }
    }

    fn build_factor(config: &FuquanFactorFixtureConfig) -> FuquanFactor {
        FuquanFactor {
            date: config.date,
            factor: config.factor,
            preclose: config.preclose,
            close: config.close,
            trading: config.trading,
            xdxr: config.xdxr,
        }
    }

    #[test]
    fn from_record_rounds_numeric_fields_and_change_pct() {
        let record_config = DayRecordFixtureConfig::default();
        let factor_config = FuquanFactorFixtureConfig::default();
        let day_data = from_record(
            &build_record(&record_config),
            &build_factor(&factor_config),
        );

        assert_eq!(day_data.code, format!("{:06}", record_config.code));
        assert_eq!(day_data.date, factor_config.date);
        assert_eq!(
            day_data.open,
            Decimal::from_f64(record_config.open as f64).unwrap().round_dp(2)
        );
        assert_eq!(
            day_data.amount,
            Decimal::from_f64(record_config.amount as f64).unwrap().round_dp(2)
        );
        assert_eq!(
            day_data.preclose,
            Decimal::from_f64(factor_config.preclose).unwrap()
        );
        assert_eq!(
            day_data.factor,
            Decimal::from_f64(factor_config.factor).unwrap().round_dp(6)
        );
        assert_eq!(
            day_data.change_pct,
            Decimal::from_f64(
                (factor_config.close - factor_config.preclose) / factor_config.preclose * 100.0
            )
            .unwrap()
            .round_dp(2)
        );
    }

    #[test]
    fn to_kline_preserves_ohlcv_and_wraps_amount() {
        let record_config = DayRecordFixtureConfig::default();
        let factor_config = FuquanFactorFixtureConfig::default();
        let day_data = from_record(
            &build_record(&record_config),
            &build_factor(&factor_config),
        );
        let kline = to_kline(&day_data, AdjustType::QFQ);

        assert_eq!(kline.code, format!("{:06}", record_config.code));
        assert_eq!(kline.date, factor_config.date);
        assert_eq!(
            kline.open,
            Decimal::from_f64(record_config.open as f64).unwrap().round_dp(2)
        );
        assert_eq!(
            kline.close,
            Decimal::from_f64(record_config.close as f64).unwrap().round_dp(2)
        );
        assert_eq!(kline.volume, record_config.volume as i64);
        assert_eq!(
            kline.amount,
            Some(Decimal::from_f64(record_config.amount as f64).unwrap().round_dp(2))
        );
        assert!(matches!(kline.adjust_type, AdjustType::QFQ));
    }
}
