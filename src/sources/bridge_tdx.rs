use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;

use crate::bridge::client::BridgeHttpClient;
use crate::core::{QuantixError, Result};
use crate::data::fetcher::Fetcher;
use crate::data::models::{AdjustType, Kline, StockInfo};
use crate::sources::tdx::StockQuote;

#[derive(Debug, Clone)]
pub struct BridgeTdxSource {
    client: BridgeHttpClient,
}

impl BridgeTdxSource {
    pub fn new(client: BridgeHttpClient) -> Self {
        Self { client }
    }

    pub async fn fetch_quotes_batch(&self, codes: &[(u16, &str)]) -> Result<Vec<StockQuote>> {
        if codes.is_empty() {
            return Ok(Vec::new());
        }

        let symbols: Vec<String> = codes
            .iter()
            .map(|(market, code)| format_symbol(*market, code))
            .collect();
        let response = self
            .client
            .fetch_tdx_quotes(&symbols)
            .await
            .map_err(map_bridge_err)?;

        response
            .quotes
            .into_iter()
            .map(|quote| {
                let raw_code = split_symbol(&quote.symbol).0.to_string();
                let market = split_symbol(&quote.symbol).1;
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
}

#[async_trait]
impl Fetcher for BridgeTdxSource {
    async fn get_stock_info(&self, _code: &str) -> Result<Option<StockInfo>> {
        Err(QuantixError::Unsupported(
            "BridgeTdxSource::get_stock_info 尚未接入真实股票信息来源".to_string(),
        ))
    }

    async fn get_kline(&self, code: &str, start: NaiveDate, end: NaiveDate) -> Result<Vec<Kline>> {
        let symbol = infer_symbol(code);
        let response = self
            .client
            .fetch_tdx_kline(
                &symbol,
                "1d",
                &start.format("%Y-%m-%d").to_string(),
                &end.format("%Y-%m-%d").to_string(),
            )
            .await
            .map_err(map_bridge_err)?;

        response
            .bars
            .into_iter()
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

    async fn check_connection(&self) -> Result<()> {
        self.client.capabilities().await.map_err(map_bridge_err)?;
        Ok(())
    }
}

fn map_bridge_err(err: crate::bridge::error::BridgeError) -> QuantixError {
    QuantixError::DataSource(format!("bridge tdx error: {err}"))
}

fn decimal_from_f64(value: f64, field: &str) -> Result<Decimal> {
    Decimal::from_f64_retain(value)
        .ok_or_else(|| QuantixError::DataParse(format!("bridge tdx 无法转换字段 {field}={value}")))
}

fn format_symbol(market: u16, code: &str) -> String {
    match market {
        1 => format!("{code}.SH"),
        _ => format!("{code}.SZ"),
    }
}

fn infer_symbol(code: &str) -> String {
    if code.starts_with('6') {
        format!("{code}.SH")
    } else {
        format!("{code}.SZ")
    }
}

fn split_symbol(symbol: &str) -> (&str, u8) {
    match symbol.split_once('.') {
        Some((code, "SH")) => (code, 1),
        Some((code, _)) => (code, 0),
        None => (symbol, 0),
    }
}
