use chrono::NaiveDate;
use rust_decimal::Decimal;

use crate::bridge::models::{BridgeKlineBarPayload, BridgeQuotePayload};
use crate::core::{QuantixError, Result};
use crate::data::models::{AdjustType, Kline};
use crate::sources::tdx::StockQuote;

pub(super) fn map_bridge_err(err: crate::bridge::error::BridgeError) -> QuantixError {
    QuantixError::DataSource(format!("bridge tdx error: {err}"))
}

pub(super) fn map_bridge_quotes(quotes: Vec<BridgeQuotePayload>) -> Result<Vec<StockQuote>> {
    quotes
        .into_iter()
        .map(|quote| {
            let raw_code = split_symbol(&quote.symbol).0.to_string();
            let market = split_symbol(&quote.symbol).1;
            let _ = (&quote.bid, &quote.ask, &quote.timestamp, &quote.source);
            Ok(StockQuote::from_tdx(
                raw_code,
                quote.name,
                quote.last,
                quote.pre_close,
                quote.open,
                quote.high,
                quote.low,
                quote.volume as f64,
                quote.turnover,
                market,
            ))
        })
        .collect()
}

pub(super) fn map_bridge_kline_bars(
    code: &str,
    bars: Vec<BridgeKlineBarPayload>,
) -> Result<Vec<Kline>> {
    bars.into_iter()
        .map(|bar| {
            let date = NaiveDate::parse_from_str(&bar.datetime, "%Y-%m-%d").map_err(|err| {
                QuantixError::DataParse(format!("bridge tdx kline 日期解析失败: {err}"))
            })?;
            Ok(Kline {
                code: code.to_string(),
                date,
                open: decimal_from_f64(bar.open, "open")?,
                high: decimal_from_f64(bar.high, "high")?,
                low: decimal_from_f64(bar.low, "low")?,
                close: decimal_from_f64(bar.close, "close")?,
                volume: bar.volume,
                amount: Some(decimal_from_f64(bar.turnover, "turnover")?),
                adjust_type: AdjustType::None,
            })
        })
        .collect()
}

pub(super) fn decimal_from_f64(value: f64, field: &str) -> Result<Decimal> {
    Decimal::from_f64_retain(value)
        .ok_or_else(|| QuantixError::DataParse(format!("bridge tdx 无法转换字段 {field}={value}")))
}

pub(super) fn format_symbol(market: u16, code: &str) -> String {
    match market {
        1 => format!("{code}.SH"),
        _ => format!("{code}.SZ"),
    }
}

pub(super) fn infer_symbol(code: &str) -> String {
    if code.starts_with('6') {
        format!("{code}.SH")
    } else {
        format!("{code}.SZ")
    }
}

pub(super) fn split_symbol(symbol: &str) -> (&str, u8) {
    match symbol.split_once('.') {
        Some((code, "SH")) => (code, 1),
        Some((code, _)) => (code, 0),
        None => (symbol, 0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symbol_helpers_round_trip_sh_and_sz_codes() {
        assert_eq!(format_symbol(1, "600000"), "600000.SH");
        assert_eq!(format_symbol(0, "000001"), "000001.SZ");
        assert_eq!(infer_symbol("600000"), "600000.SH");
        assert_eq!(infer_symbol("000001"), "000001.SZ");
        assert_eq!(split_symbol("600000.SH"), ("600000", 1));
        assert_eq!(split_symbol("000001.SZ"), ("000001", 0));
        assert_eq!(split_symbol("300750"), ("300750", 0));
    }

    #[test]
    fn map_bridge_kline_bars_parses_expected_fields() {
        let bars = vec![BridgeKlineBarPayload {
            datetime: "2026-04-01".to_string(),
            open: 10.0,
            high: 12.0,
            low: 9.5,
            close: 11.2,
            volume: 12345,
            turnover: 99887.5,
        }];

        let rows = map_bridge_kline_bars("000001", bars).unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].code, "000001");
        assert_eq!(rows[0].date, NaiveDate::from_ymd_opt(2026, 4, 1).unwrap());
        assert_eq!(rows[0].volume, 12345);
        assert!(matches!(rows[0].adjust_type, AdjustType::None));
    }
}
