//! Kline ↔ fin-primitives 类型适配。
//!
//! 正向：`Kline` → `BarInput`（指标流）/ `OhlcvBar`（回测），fallible（Symbol/Price 构造与 OHLC 校验）。
//! 反向：`OhlcvBar` → `Kline`（walk-forward 闭包内重放 quantix 策略）。

use chrono::NaiveDate;
use fin_primitives::ohlcv::OhlcvBar;
use fin_primitives::signals::BarInput;
use fin_primitives::types::{NanoTimestamp, Price, Quantity, Symbol};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

use crate::core::{QuantixError, Result};
use crate::data::models::{AdjustType, Kline};

/// 日线开盘/收盘时刻（A 股）：09:30 / 15:00 CST（= 01:30 / 07:00 UTC）。
fn session_timestamps(date: NaiveDate) -> Result<(NanoTimestamp, NanoTimestamp)> {
    let open = date
        .and_hms_opt(1, 30, 0)
        .and_then(|t| t.and_utc().timestamp_nanos_opt())
        .ok_or_else(|| QuantixError::DataParse(format!("无效日期: {date}")))?;
    let close = date
        .and_hms_opt(7, 0, 0)
        .and_then(|t| t.and_utc().timestamp_nanos_opt())
        .ok_or_else(|| QuantixError::DataParse(format!("无效日期: {date}")))?;
    Ok((NanoTimestamp::new(open), NanoTimestamp::new(close)))
}

/// `Kline` → 指标流输入（无校验，volume i64 → Decimal）。
pub fn to_bar_input(kline: &Kline) -> BarInput {
    BarInput {
        open: kline.open,
        high: kline.high,
        low: kline.low,
        close: kline.close,
        volume: Decimal::from(kline.volume),
    }
}

pub fn to_bar_inputs(klines: &[Kline]) -> Vec<BarInput> {
    klines.iter().map(to_bar_input).collect()
}

/// `Kline` → `OhlcvBar`（fallible：Symbol/Price 构造 + OHLC 不变量校验）。
pub fn to_ohlcv_bar(kline: &Kline) -> Result<OhlcvBar> {
    let symbol = Symbol::new(kline.code.as_str()).map_err(QuantixError::Fin)?;
    let (ts_open, ts_close) = session_timestamps(kline.date)?;
    OhlcvBar::new(
        symbol,
        Price::new(kline.open).map_err(QuantixError::Fin)?,
        Price::new(kline.high).map_err(QuantixError::Fin)?,
        Price::new(kline.low).map_err(QuantixError::Fin)?,
        Price::new(kline.close).map_err(QuantixError::Fin)?,
        Quantity::try_from(Decimal::from(kline.volume)).map_err(QuantixError::Fin)?,
        ts_open,
        ts_close,
        1,
    )
    .map_err(QuantixError::Fin)
}

pub fn to_ohlcv_bars(klines: &[Kline]) -> Result<Vec<OhlcvBar>> {
    klines.iter().map(to_ohlcv_bar).collect()
}

/// `OhlcvBar` → `Kline`（walk-forward 闭包内重放 quantix 同步策略用）。
/// 复权类型丢失（置 None）；amount/tick_count 无 fin 对应物（None/丢弃）。
/// volume 为非整数 Decimal 时报错（日线成交量恒为整数）。
pub fn ohlcv_to_kline(bar: &OhlcvBar) -> Result<Kline> {
    let volume =
        bar.volume.value().to_i64().ok_or_else(|| {
            QuantixError::DataParse(format!("volume 非整数: {}", bar.volume.value()))
        })?;
    let date = chrono::DateTime::from_timestamp_nanos(bar.ts_open.nanos()).date_naive();
    Ok(Kline {
        code: bar.symbol.as_str().to_string(),
        date,
        open: bar.open.value(),
        high: bar.high.value(),
        low: bar.low.value(),
        close: bar.close.value(),
        volume,
        amount: None,
        adjust_type: AdjustType::None,
    })
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    fn make_kline(day: u32, close: &str, high: &str, low: &str) -> Kline {
        let c = Decimal::from_str(close).unwrap();
        Kline {
            code: "600000".into(),
            date: NaiveDate::from_ymd_opt(2026, 1, day).unwrap(),
            open: c,
            high: Decimal::from_str(high).unwrap(),
            low: Decimal::from_str(low).unwrap(),
            close: c,
            volume: 1000,
            amount: None,
            adjust_type: AdjustType::None,
        }
    }

    #[test]
    fn bar_input_maps_all_fields_with_volume_i64_to_decimal() {
        let k = make_kline(5, "12.34", "12.50", "12.10");
        let b = to_bar_input(&k);
        assert_eq!(b.close, Decimal::from_str("12.34").unwrap());
        assert_eq!(b.volume, Decimal::from(1000));
    }

    #[test]
    fn ohlcv_bar_roundtrip_preserves_price_and_date() {
        let k = make_kline(6, "10.00", "10.50", "9.80");
        let bar = to_ohlcv_bar(&k).unwrap();
        let back = ohlcv_to_kline(&bar).unwrap();
        assert_eq!(back.code, "600000");
        assert_eq!(back.date, NaiveDate::from_ymd_opt(2026, 1, 6).unwrap());
        assert_eq!(back.close, Decimal::from_str("10.00").unwrap());
        assert_eq!(back.volume, 1000);
    }

    #[test]
    fn ohlcv_bar_rejects_high_below_low() {
        let k = make_kline(7, "10.00", "9.00", "9.50"); // high < low
        let err = to_ohlcv_bar(&k).unwrap_err();
        assert!(matches!(err, QuantixError::Fin(_)));
    }
}
