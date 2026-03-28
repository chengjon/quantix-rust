//! 龙虎榜数据获取

use crate::core::{QuantixError, Result};
use super::types::DragonTigerItem;

/// 龙虎榜数据获取器
pub struct DragonTigerFetcher {
    client: reqwest::Client,
}

impl DragonTigerFetcher {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// 获取龙虎榜数据
    pub async fn fetch(&self, code: &str, days: u32) -> Result<Vec<DragonTigerItem>> {
        let url = format!(
            "https://data.eastmoney.com/DataCenter_V3/stock2016/TradeDetail/pagesize=50,page=1,sortrule=-1,sorttype=,code={},startDate=,endDate=.js",
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

    /// 获取今日龙虎榜
    pub async fn fetch_today(&self) -> Result<Vec<DragonTigerItem>> {
        let url = "https://data.eastmoney.com/DataCenter_V3/stock2016/TradeDetail/pagesize=50,page=1,sortrule=-1,sorttype=,.js";

        let response = self.client
            .get(url)
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

impl Default for DragonTigerFetcher {
    fn default() -> Self {
        Self::new()
    }
}
