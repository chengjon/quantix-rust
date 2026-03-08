/// AkShare 数据源
///
/// 通过 HTTP API 获取 AkShare 数据
use async_trait::async_trait;

use crate::core::Result;
use crate::data::fetcher::Fetcher;
use crate::data::models::{Kline, StockInfo};

/// AkShare 数据源
pub struct AkShareSource {
    base_url: String,
    client: reqwest::Client,
}

impl AkShareSource {
    pub fn new(base_url: String) -> Result<Self> {
        let client = reqwest::Client::builder()
            .build()
            .map_err(|e| crate::core::error::QuantixError::Http(e))?;
        Ok(Self { base_url, client })
    }
}

#[async_trait]
impl Fetcher for AkShareSource {
    async fn get_stock_info(&self, code: &str) -> Result<Option<StockInfo>> {
        // TODO: 实现 AkShare API 调用
        tracing::warn!("AkShareSource::get_stock_info not implemented yet");
        Ok(None)
    }

    async fn get_kline(
        &self,
        code: &str,
        start: chrono::NaiveDate,
        end: chrono::NaiveDate,
    ) -> Result<Vec<Kline>> {
        // TODO: 实现 AkShare K线获取
        tracing::warn!("AkShareSource::get_kline not implemented yet");
        Ok(vec![])
    }

    async fn check_connection(&self) -> Result<()> {
        self.client
            .get(&format!("{}/health", self.base_url))
            .send()
            .await
            .map_err(|e| crate::core::error::QuantixError::Http(e))?;
        Ok(())
    }
}
