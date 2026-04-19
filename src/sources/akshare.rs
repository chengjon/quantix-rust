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
            .map_err(crate::core::error::QuantixError::Http)?;
        Ok(Self { base_url, client })
    }
}

#[async_trait]
impl Fetcher for AkShareSource {
    async fn get_stock_info(&self, _code: &str) -> Result<Option<StockInfo>> {
        Err(crate::core::QuantixError::Unsupported(
            "AkShareSource::get_stock_info 尚未接入真实 API".to_string(),
        ))
    }

    async fn get_kline(
        &self,
        _code: &str,
        _start: chrono::NaiveDate,
        _end: chrono::NaiveDate,
    ) -> Result<Vec<Kline>> {
        Err(crate::core::QuantixError::Unsupported(
            "AkShareSource::get_kline 尚未接入真实 API".to_string(),
        ))
    }

    async fn check_connection(&self) -> Result<()> {
        self.client
            .get(format!("{}/health", self.base_url))
            .send()
            .await
            .map_err(crate::core::error::QuantixError::Http)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::QuantixError;

    #[tokio::test]
    async fn test_akshare_get_stock_info_returns_unsupported() {
        let source = AkShareSource::new("http://localhost:8000".to_string()).unwrap();
        let err = source.get_stock_info("000001").await.unwrap_err();
        assert!(matches!(err, QuantixError::Unsupported(_)));
    }

    #[tokio::test]
    async fn test_akshare_get_kline_returns_unsupported() {
        let source = AkShareSource::new("http://localhost:8000".to_string()).unwrap();
        let err = source
            .get_kline(
                "000001",
                chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                chrono::NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            )
            .await
            .unwrap_err();

        assert!(matches!(err, QuantixError::Unsupported(_)));
    }
}
