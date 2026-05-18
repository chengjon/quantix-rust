//! 龙虎榜数据获取

use super::types::{BrokerActivity, DragonTigerItem};
use crate::core::{QuantixError, Result};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::Deserialize;

/// EastMoney 龙虎榜 API 响应
#[derive(Debug, Deserialize)]
struct DragonTigerResponse {
    result: Option<DragonTigerResult>,
}

#[derive(Debug, Deserialize)]
struct DragonTigerResult {
    data: Option<Vec<DragonTigerItemRaw>>,
}

#[derive(Debug, Deserialize)]
struct DragonTigerItemRaw {
    /// 股票代码
    #[serde(rename = "SCode")]
    s_code: Option<String>,
    /// 股票名称
    #[serde(rename = "SName")]
    s_name: Option<String>,
    /// 交易日期
    #[serde(rename = "TradeDate")]
    trade_date: Option<String>,
    /// 收盘价
    #[serde(rename = "ClosePrice")]
    close_price: Option<serde_json::Value>,
    /// 涨跌幅
    #[serde(rename = "ChangePct")]
    change_pct: Option<serde_json::Value>,
    /// 上榜原因
    #[serde(rename = "CReason")]
    reason: Option<String>,
    /// 买入金额
    #[serde(rename = "BuyAmount")]
    buy_amount: Option<serde_json::Value>,
    /// 卖出金额
    #[serde(rename = "SellAmount")]
    sell_amount: Option<serde_json::Value>,
    /// 净买入
    #[serde(rename = "NetBuy")]
    net_buy: Option<serde_json::Value>,
}

fn value_to_f64(v: &Option<serde_json::Value>) -> Option<f64> {
    v.as_ref().and_then(|val| match val {
        serde_json::Value::Number(n) => n.as_f64(),
        serde_json::Value::String(s) => s.parse().ok(),
        _ => None,
    })
}

fn to_decimal(val: f64) -> Option<Decimal> {
    Decimal::from_f64_retain(val)
}

fn parse_date(s: &Option<String>) -> chrono::NaiveDate {
    s.as_ref()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .unwrap_or_else(|| chrono::Utc::now().date_naive())
}

/// 龙虎榜数据获取器
pub struct DragonTigerFetcher {
    client: reqwest::Client,
}

impl DragonTigerFetcher {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
        }
    }

    /// 获取龙虎榜数据
    pub async fn fetch(&self, code: &str, _days: u32) -> Result<Vec<DragonTigerItem>> {
        let url = format!(
            "https://data.eastmoney.com/DataCenter_V3/stock2016/TradeDetail/pagesize=50,page=1,sortrule=-1,sorttype=,code={},startDate=,endDate=.js",
            code
        );

        let response = self
            .client
            .get(&url)
            .header("Referer", "https://data.eastmoney.com/")
            .send()
            .await
            .map_err(|e| QuantixError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(QuantixError::Other(format!(
                "EastMoney API error: {}",
                response.status()
            )));
        }

        let resp: DragonTigerResponse = response
            .json()
            .await
            .map_err(|e| QuantixError::Other(format!("解析龙虎榜响应失败: {}", e)))?;

        let items = resp.result.and_then(|r| r.data).unwrap_or_default();

        let result: Vec<DragonTigerItem> = items
            .into_iter()
            .filter_map(|item| {
                let code = item.s_code.clone().unwrap_or_default();
                let name = item.s_name.clone().unwrap_or_else(|| code.clone());
                let close = value_to_f64(&item.close_price)?;
                let change = value_to_f64(&item.change_pct).unwrap_or(0.0);
                let buy = value_to_f64(&item.buy_amount).unwrap_or(0.0) / 10000.0;
                let sell = value_to_f64(&item.sell_amount).unwrap_or(0.0) / 10000.0;
                let net = buy - sell;

                Some(DragonTigerItem {
                    code,
                    name,
                    trade_date: parse_date(&item.trade_date),
                    close_price: to_decimal(close).unwrap_or(Decimal::ZERO),
                    change_pct: to_decimal(change).unwrap_or(Decimal::ZERO),
                    reason: item.reason.unwrap_or_default(),
                    buy_amount: to_decimal(buy).unwrap_or(Decimal::ZERO),
                    sell_amount: to_decimal(sell).unwrap_or(Decimal::ZERO),
                    net_buy: to_decimal(net).unwrap_or(Decimal::ZERO),
                    top_buyers: Vec::new(),
                    top_sellers: Vec::new(),
                })
            })
            .collect();

        Ok(result)
    }

    /// 获取今日龙虎榜
    pub async fn fetch_today(&self) -> Result<Vec<DragonTigerItem>> {
        let url = "https://data.eastmoney.com/DataCenter_V3/stock2016/TradeDetail/pagesize=50,page=1,sortrule=-1,sorttype=,.js";

        let response = self
            .client
            .get(url)
            .header("Referer", "https://data.eastmoney.com/")
            .send()
            .await
            .map_err(|e| QuantixError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(QuantixError::Other(format!(
                "EastMoney API error: {}",
                response.status()
            )));
        }

        let resp: DragonTigerResponse = response
            .json()
            .await
            .map_err(|e| QuantixError::Other(format!("解析龙虎榜响应失败: {}", e)))?;

        let items = resp.result.and_then(|r| r.data).unwrap_or_default();

        let result: Vec<DragonTigerItem> = items
            .into_iter()
            .filter_map(|item| {
                let code = item.s_code.clone().unwrap_or_default();
                let name = item.s_name.clone().unwrap_or_else(|| code.clone());
                let close = value_to_f64(&item.close_price)?;
                let change = value_to_f64(&item.change_pct).unwrap_or(0.0);
                let buy = value_to_f64(&item.buy_amount).unwrap_or(0.0) / 10000.0;
                let sell = value_to_f64(&item.sell_amount).unwrap_or(0.0) / 10000.0;

                Some(DragonTigerItem {
                    code,
                    name,
                    trade_date: parse_date(&item.trade_date),
                    close_price: to_decimal(close).unwrap_or(Decimal::ZERO),
                    change_pct: to_decimal(change).unwrap_or(Decimal::ZERO),
                    reason: item.reason.unwrap_or_default(),
                    buy_amount: to_decimal(buy).unwrap_or(Decimal::ZERO),
                    sell_amount: to_decimal(sell).unwrap_or(Decimal::ZERO),
                    net_buy: to_decimal(buy - sell).unwrap_or(Decimal::ZERO),
                    top_buyers: Vec::new(),
                    top_sellers: Vec::new(),
                })
            })
            .collect();

        Ok(result)
    }
}

impl Default for DragonTigerFetcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_date() {
        let d = parse_date(&Some("2024-12-31".to_string()));
        assert_eq!(d, chrono::NaiveDate::from_ymd_opt(2024, 12, 31).unwrap());
    }

    #[test]
    fn test_parse_date_fallback() {
        let d = parse_date(&None);
        assert!(d <= chrono::Utc::now().date_naive());
    }

    #[test]
    fn test_parse_empty_response() {
        let json = r#"{"result": null}"#;
        let resp: DragonTigerResponse = serde_json::from_str(json).unwrap();
        assert!(resp.result.is_none());
    }

    #[test]
    fn test_parse_items() {
        let json = r#"{
            "result": {
                "data": [
                    {
                        "SCode": "600519",
                        "SName": "贵州茅台",
                        "TradeDate": "2024-12-20",
                        "ClosePrice": 1500.5,
                        "ChangePct": 5.23,
                        "CReason": "日涨幅偏离值达7%",
                        "BuyAmount": 580000000,
                        "SellAmount": 320000000,
                        "NetBuy": 260000000
                    }
                ]
            }
        }"#;

        let resp: DragonTigerResponse = serde_json::from_str(json).unwrap();
        let data = resp.result.unwrap().data.unwrap();
        assert_eq!(data.len(), 1);
        assert_eq!(data[0].s_code.as_deref(), Some("600519"));
        assert_eq!(data[0].s_name.as_deref(), Some("贵州茅台"));
        assert_eq!(value_to_f64(&data[0].close_price), Some(1500.5));
    }
}
