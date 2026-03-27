//! 机构持仓数据获取

use crate::core::{QuantixError, Result};
use super::types::InstitutionHolding;

/// 机构持仓数据获取器
pub struct InstitutionFetcher {
    client: reqwest::Client,
}

impl InstitutionFetcher {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// 获取机构持仓
    pub async fn fetch_holdings(&self, code: &str) -> Result<Vec<InstitutionHolding>> {
        let url = format!(
            "https://data.eastmoney.com/dataapi/stockholder/list?code={}",
            code
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
        Ok(Vec::new())
    }
}

impl Default for InstitutionFetcher {
    fn default() -> Self {
        Self::new()
    }
}
