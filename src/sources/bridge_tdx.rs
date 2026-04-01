use async_trait::async_trait;
use chrono::NaiveDate;

use crate::bridge::client::BridgeHttpClient;
use crate::core::{QuantixError, Result};
use crate::data::fetcher::Fetcher;
use crate::data::models::{Kline, StockInfo};
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
        let response = self.client.fetch_tdx_quotes(&symbols).await.map_err(map_bridge_err)?;
        super::bridge_tdx_support::map_bridge_quotes(response.quotes)
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

        let _ = (&response.symbol, &response.period, &response.source);
        super::bridge_tdx_support::map_bridge_kline_bars(code, response.bars)
    }

    async fn check_connection(&self) -> Result<()> {
        self.client.capabilities().await.map_err(map_bridge_err)?;
        Ok(())
    }
}

fn map_bridge_err(err: crate::bridge::error::BridgeError) -> QuantixError {
    super::bridge_tdx_support::map_bridge_err(err)
}

fn format_symbol(market: u16, code: &str) -> String {
    super::bridge_tdx_support::format_symbol(market, code)
}

fn infer_symbol(code: &str) -> String {
    super::bridge_tdx_support::infer_symbol(code)
}
