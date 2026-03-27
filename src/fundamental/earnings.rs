//! 财报数据获取

use crate::core::{QuantixError, Result};
use super::types::EarningsReport;

/// 财报数据获取器
pub struct EarningsFetcher {
    client: reqwest::Client,
}

impl EarningsFetcher {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// 获取最新财报
    pub async fn fetch_latest(&self, code: &str) -> Result<EarningsReport> {
        let url = format!(
            "https://emweb.eastmoney.com/PC_HSF10/NewFinanceAnalysis/ZYZBAjaxNew?type=web&code={}",
            Self::format_code(code)
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
        let report = EarningsReport::new(code.to_string(), date, "最新".to_string());

        Ok(report)
    }

    /// 获取历史财报
    pub async fn fetch_history(&self, code: &str, years: u32) -> Result<Vec<EarningsReport>> {
        let mut reports = Vec::new();

        for _ in 0..years {
            if let Ok(report) = self.fetch_latest(code).await {
                reports.push(report);
            }
        }

        Ok(reports)
    }

    /// 格式化代码
    fn format_code(code: &str) -> String {
        if code.starts_with('6') {
            format!("sh{}", code)
        } else {
            format!("sz{}", code)
        }
    }
}

impl Default for EarningsFetcher {
    fn default() -> Self {
        Self::new()
    }
}
