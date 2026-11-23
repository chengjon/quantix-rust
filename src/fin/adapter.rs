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
/// Price 约束严格 >0（停牌 0 价行被拒是预期校验）；tick_count 占位 1（日线无 tick 聚合语义）。
/// 失败时错误附 code/date 定位上下文，FinError display 保留于消息尾部。
pub fn to_ohlcv_bar(kline: &Kline) -> Result<OhlcvBar> {
    let with_ctx = |e: fin_primitives::FinError| {
        QuantixError::DataParse(format!(
            "Kline→OhlcvBar 失败 code={} date={}: {e}",
            kline.code, kline.date
        ))
    };
    let symbol = Symbol::new(kline.code.as_str()).map_err(with_ctx)?;
    let (ts_open, ts_close) = session_timestamps(kline.date)?;
    OhlcvBar::new(
        symbol,
        Price::new(kline.open).map_err(with_ctx)?,
        Price::new(kline.high).map_err(with_ctx)?,
        Price::new(kline.low).map_err(with_ctx)?,
        Price::new(kline.close).map_err(with_ctx)?,
        Quantity::try_from(Decimal::from(kline.volume)).map_err(with_ctx)?,
        ts_open,
        ts_close,
        1,
    )
    .map_err(with_ctx)
}

pub fn to_ohlcv_bars(klines: &[Kline]) -> Result<Vec<OhlcvBar>> {
    klines.iter().map(to_ohlcv_bar).collect()
}

/// `OhlcvBar` → `Kline`（walk-forward 闭包内重放 quantix 同步策略用）。
/// 复权类型丢失（置 None）；amount/tick_count 无 fin 对应物（None/丢弃）。
/// volume 为非整数 Decimal 时报错（日线成交量恒为整数；`to_i64` 对小数是截断语义，
/// 必须先显式校验 integrality）。
pub fn ohlcv_to_kline(bar: &OhlcvBar) -> Result<Kline> {
    let volume_dec = bar.volume.value();
    if !volume_dec.is_integer() {
        return Err(QuantixError::DataParse(format!(
            "volume 非整数: {volume_dec}"
        )));
    }
    let volume = volume_dec
        .to_i64()
        .ok_or_else(|| QuantixError::DataParse(format!("volume 超出 i64 范围: {volume_dec}")))?;
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
        assert_eq!(
            b,
            BarInput {
                open: Decimal::from_str("12.34").unwrap(),
                high: Decimal::from_str("12.50").unwrap(),
                low: Decimal::from_str("12.10").unwrap(),
                close: Decimal::from_str("12.34").unwrap(),
                volume: Decimal::from(1000),
            }
        );
    }

    #[test]
    fn ohlcv_bar_roundtrip_preserves_price_and_date() {
        let k = make_kline(6, "10.00", "10.50", "9.80");
        let bar = to_ohlcv_bar(&k).unwrap();
        let back = ohlcv_to_kline(&bar).unwrap();
        assert_eq!(back.code, "600000");
        assert_eq!(back.date, NaiveDate::from_ymd_opt(2026, 1, 6).unwrap());
        assert_eq!(back.open, Decimal::from_str("10.00").unwrap());
        assert_eq!(back.high, Decimal::from_str("10.50").unwrap());
        assert_eq!(back.low, Decimal::from_str("9.80").unwrap());
        assert_eq!(back.close, Decimal::from_str("10.00").unwrap());
        assert_eq!(back.volume, 1000);
    }

    #[test]
    fn ohlcv_bar_rejects_high_below_low() {
        let k = make_kline(7, "10.00", "9.00", "9.50"); // high < low
        let err = to_ohlcv_bar(&k).unwrap_err();
        assert!(
            matches!(err, QuantixError::DataParse(_)),
            "应收敛为 DataParse: {err:?}"
        );
        let text = err.to_string();
        assert!(text.contains("code=600000"), "应含 code 上下文: {text}");
        assert!(
            text.contains("invariant"),
            "应保留 FinError display: {text}"
        );
    }

    #[test]
    fn to_ohlcv_bar_rejects_non_positive_price_with_context() {
        let k = make_kline(8, "0", "0", "0"); // 停牌 0 价行：Price 拒 <=0
        let err = to_ohlcv_bar(&k).unwrap_err();
        assert!(
            matches!(err, QuantixError::DataParse(_)),
            "应收敛为 DataParse: {err:?}"
        );
        let text = err.to_string();
        assert!(text.contains("code=600000"), "应含 code 上下文: {text}");
        assert!(text.contains("date=2026-01-08"), "应含 date 上下文: {text}");
        assert!(
            text.contains("Price must be positive"),
            "应保留 FinError display: {text}"
        );
    }

    #[test]
    fn ohlcv_to_kline_rejects_non_integer_volume() {
        let bar = OhlcvBar::new(
            Symbol::new("600000").unwrap(),
            Price::new(Decimal::from_str("10.00").unwrap()).unwrap(),
            Price::new(Decimal::from_str("10.50").unwrap()).unwrap(),
            Price::new(Decimal::from_str("9.80").unwrap()).unwrap(),
            Price::new(Decimal::from_str("10.00").unwrap()).unwrap(),
            Quantity::try_from(Decimal::from_str("1000.5").unwrap()).unwrap(),
            NanoTimestamp::new(0),
            NanoTimestamp::new(0),
            1,
        )
        .unwrap();
        let err = ohlcv_to_kline(&bar).unwrap_err();
        assert!(matches!(err, QuantixError::DataParse(_)));
        assert!(err.to_string().contains("volume 非整数"));
    }

    #[test]
    fn to_ohlcv_bars_converts_batch_in_order() {
        let ks = [
            make_kline(5, "10.00", "10.50", "9.80"),
            make_kline(6, "11.00", "11.50", "10.80"),
        ];
        let bars = to_ohlcv_bars(&ks).unwrap();
        assert_eq!(bars.len(), 2);
        assert_eq!(bars[0].close.value(), Decimal::from_str("10.00").unwrap());
        assert_eq!(bars[1].close.value(), Decimal::from_str("11.00").unwrap());
    }
}
