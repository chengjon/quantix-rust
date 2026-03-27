//! 估值指标获取

use crate::core::{QuantixError, Result};
use super::types::ValuationMetrics;

/// 估值数据获取器
pub struct ValuationFetcher {
    client: reqwest::Client,
}

impl ValuationFetcher {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// 从东方财富获取估值数据
    pub async fn fetch_from_eastmoney(&self, code: &str) -> Result<ValuationMetrics> {
        let url = format!(
            "https://push2.eastmoney.com/api/qt/stock/get?secid={}&fields=f57,f58,f162,f167,f92,f173,f187,f105,f116,f117",
            Self::format_secid(code)
        );

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| QuantixError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(QuantixError::Other(format!("EastMoney API error: {}", response.status())));
        }

        // TODO: 解析响应数据
        let date = chrono::Utc::now().date_naive();
        let mut metrics = ValuationMetrics::new(code.to_string(), date);

        // 返回基础结构，实际数据需要解析 API 响应
        Ok(metrics)
    }

    /// 格式化证券代码
    fn format_secid(code: &str) -> String {
        if code.starts_with('6') {
            format!("1.{}", code)
        } else {
            format!("0.{}", code)
        }
    }
}

impl Default for ValuationFetcher {
    fn default() -> Self {
        Self::new()
    }
}
