//! 研究侧指标门面：强类型封装常用流式指标；未被封装的指标可在本模块内按同一模式扩展（约定：fin 类型不出 src/fin）。

use fin_primitives::signals::Signal as FinIndicator;
use fin_primitives::signals::indicators::{Atr, BollingerB, Ema, Rsi, Sma, StochasticK};
use rust_decimal::Decimal;

use crate::core::{QuantixError, Result};
use crate::data::models::Kline;
use crate::fin::adapter::to_bar_inputs;

/// 单指标全序列输出：`values[i]` 对应 `klines[i]`，未热身期为 `None`。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IndicatorSeries {
    pub name: String,
    pub values: Vec<Option<Decimal>>,
}

fn stream(
    name: String,
    klines: &[Kline],
    make: impl FnOnce() -> std::result::Result<Box<dyn FinIndicator>, fin_primitives::FinError>,
) -> Result<IndicatorSeries> {
    let bars = to_bar_inputs(klines);
    let mut indicator =
        make().map_err(|e| QuantixError::DataParse(format!("指标 {name} 构造失败: {e}")))?;
    let mut values = Vec::with_capacity(bars.len());
    for bar in &bars {
        let v = indicator
            .update(bar)
            .map_err(|e| QuantixError::DataParse(format!("指标 {name} 流式更新失败: {e}")))?;
        values.push(v.as_decimal());
    }
    Ok(IndicatorSeries { name, values })
}

pub fn sma(klines: &[Kline], period: usize) -> Result<IndicatorSeries> {
    stream(format!("sma{period}"), klines, || {
        Ok(Box::new(Sma::new(format!("sma{period}"), period)?))
    })
}

pub fn ema(klines: &[Kline], period: usize) -> Result<IndicatorSeries> {
    stream(format!("ema{period}"), klines, || {
        Ok(Box::new(Ema::new(format!("ema{period}"), period)?))
    })
}

pub fn rsi(klines: &[Kline], period: usize) -> Result<IndicatorSeries> {
    stream(format!("rsi{period}"), klines, || {
        Ok(Box::new(Rsi::new(format!("rsi{period}"), period)?))
    })
}

pub fn atr(klines: &[Kline], period: usize) -> Result<IndicatorSeries> {
    stream(format!("atr{period}"), klines, || {
        Ok(Box::new(Atr::new(format!("atr{period}"), period)?))
    })
}

pub fn bollinger(klines: &[Kline], period: usize, multiplier: Decimal) -> Result<IndicatorSeries> {
    stream(format!("boll{period}x{multiplier}"), klines, || {
        Ok(Box::new(BollingerB::new(
            format!("boll{period}"),
            period,
            multiplier,
        )?))
    })
}

pub fn stochastic(klines: &[Kline], period: usize) -> Result<IndicatorSeries> {
    stream(format!("stoch{period}"), klines, || {
        Ok(Box::new(StochasticK::new(
            format!("stoch{period}"),
            period,
        )?))
    })
}

/// CLI 分发：指标名 → 门面（名称与 `analyze indicators --indicators` 现有 native 命名对齐处由 handler 处理）。
pub fn resolve(name: &str, klines: &[Kline], period: usize) -> Result<IndicatorSeries> {
    match name {
        "sma" => sma(klines, period),
        "ema" => ema(klines, period),
        "rsi" => rsi(klines, period),
        "atr" => atr(klines, period),
        "boll" => bollinger(klines, period, Decimal::from(2)),
        "stoch" => stochastic(klines, period),
        other => Err(QuantixError::Unsupported(format!(
            "fin 引擎暂不支持指标: {other}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    use super::*;
    use crate::data::models::AdjustType;

    fn kline_at(day: u32, close: &str) -> Kline {
        let c = Decimal::from_str(close).unwrap();
        Kline {
            code: "600000".into(),
            date: NaiveDate::from_ymd_opt(2026, 1, day).unwrap(),
            open: c,
            high: c,
            low: c,
            close: c,
            volume: 100,
            amount: None,
            adjust_type: AdjustType::None,
        }
    }

    fn rising_series(n: u32) -> Vec<Kline> {
        (1..=n)
            .map(|i| kline_at(i.min(28).max(1), &format!("{}", i)))
            .collect()
    }

    #[test]
    fn sma3_on_ramp_last_value_is_nine() {
        let klines: Vec<Kline> = (1..=10u32).map(|i| kline_at(i, &i.to_string())).collect();
        let out = sma(&klines, 3).unwrap();
        assert_eq!(out.values.len(), 10);
        assert_eq!(out.values[1], None); // 未热身
        assert_eq!(out.values[9], Some(Decimal::from_str("9").unwrap())); // (8+9+10)/3 = 9
    }

    #[test]
    fn rsi_all_rising_is_hundred() {
        let klines = rising_series(30);
        let out = rsi(&klines, 14).unwrap();
        assert_eq!(out.values[29], Some(Decimal::from(100)));
    }

    #[test]
    fn resolve_unknown_indicator_is_unsupported() {
        let klines = rising_series(20);
        let err = resolve("kdj", &klines, 9).unwrap_err();
        assert!(matches!(err, QuantixError::Unsupported(_)));
    }
}
