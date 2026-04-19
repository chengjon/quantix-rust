//! 估值指标获取

use super::types::ValuationMetrics;
use crate::core::{QuantixError, Result};
use rust_decimal::Decimal;
use serde::Deserialize;

/// EastMoney push2 API 响应
#[derive(Debug, Deserialize)]
struct EastMoneyResponse {
    data: Option<EastMoneyStockData>,
}

#[derive(Debug, Deserialize)]
struct EastMoneyStockData {
    /// f57: 代码
    #[serde(rename = "f57")]
    _f57: Option<serde_json::Value>,
    /// f58: 名称
    #[serde(rename = "f58")]
    _f58: Option<serde_json::Value>,
    /// f116: 总市值 (元)
    f116: Option<serde_json::Value>,
    /// f117: 流通市值 (元)
    f117: Option<serde_json::Value>,
    /// f162: 市盈率(动态/TTM)
    f162: Option<serde_json::Value>,
    /// f167: 市净率
    f167: Option<serde_json::Value>,
    /// f173: ROE(%)
    f173: Option<serde_json::Value>,
    /// f187: 每股收益
    f187: Option<serde_json::Value>,
    /// f92: 净利润增长率(%)
    #[serde(rename = "f92")]
    _f92: Option<serde_json::Value>,
    /// f105: 净利润(元)
    #[serde(rename = "f105")]
    _f105: Option<serde_json::Value>,
}

/// 从 serde_json::Value 提取 f64
fn value_to_f64(v: &Option<serde_json::Value>) -> Option<f64> {
    v.as_ref().and_then(|val| match val {
        serde_json::Value::Number(n) => n.as_f64(),
        serde_json::Value::String(s) => s.parse().ok(),
        _ => None,
    })
}

/// 从 f64 转换为 Decimal (保留4位小数)
fn f64_to_decimal(val: f64) -> Option<Decimal> {
    Decimal::from_f64_retain((val * 10000.0).round() / 10000.0)
}

/// 估值数据获取器
pub struct ValuationFetcher {
    client: reqwest::Client,
}

impl ValuationFetcher {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
        }
    }

    /// 从东方财富获取估值数据
    pub async fn fetch_from_eastmoney(&self, code: &str) -> Result<ValuationMetrics> {
        let url = format!(
            "https://push2.eastmoney.com/api/qt/stock/get?secid={}&fields=f57,f58,f162,f167,f92,f173,f187,f105,f116,f117",
            Self::format_secid(code)
        );

        let response = self
            .client
            .get(&url)
            .header("Referer", "https://quote.eastmoney.com/")
            .send()
            .await
            .map_err(|e| QuantixError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(QuantixError::Other(format!(
                "EastMoney API error: {}",
                response.status()
            )));
        }

        let resp: EastMoneyResponse = response
            .json()
            .await
            .map_err(|e| QuantixError::Other(format!("解析 EastMoney 响应失败: {}", e)))?;

        let data = resp
            .data
            .ok_or_else(|| QuantixError::Other("EastMoney API 返回空数据".to_string()))?;

        // 解析估值指标
        let date = chrono::Utc::now().date_naive();
        let mut metrics = ValuationMetrics::new(code.to_string(), date);

        // 市盈率 (TTM)
        if let Some(pe) = value_to_f64(&data.f162) {
            metrics.pe_ttm = f64_to_decimal(pe);
        }

        // 市净率
        if let Some(pb) = value_to_f64(&data.f167) {
            metrics.pb = f64_to_decimal(pb);
        }

        // 总市值 (元 -> 亿元)
        if let Some(cap) = value_to_f64(&data.f116) {
            metrics.market_cap = f64_to_decimal(cap / 1e8);
        }

        // 流通市值 (元 -> 亿元)
        if let Some(fcap) = value_to_f64(&data.f117) {
            metrics.float_market_cap = f64_to_decimal(fcap / 1e8);
        }

        // ROE (%)
        if let Some(roe) = value_to_f64(&data.f173) {
            metrics.roe = f64_to_decimal(roe);
        }

        // 每股收益
        if let Some(eps) = value_to_f64(&data.f187) {
            metrics.eps = f64_to_decimal(eps);
        }

        Ok(metrics)
    }

    /// 格式化证券代码 (EastMoney secid 格式: 市场编号.代码)
    fn format_secid(code: &str) -> String {
        let code = code.trim_start_matches(|c: char| !c.is_ascii_digit());
        if code.starts_with('6') || code.starts_with('9') {
            format!("1.{}", code) // 上海
        } else {
            format!("0.{}", code) // 深圳
        }
    }
}

impl Default for ValuationFetcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_secid() {
        assert_eq!(ValuationFetcher::format_secid("600519"), "1.600519");
        assert_eq!(ValuationFetcher::format_secid("000001"), "0.000001");
        assert_eq!(ValuationFetcher::format_secid("300001"), "0.300001");
        assert_eq!(ValuationFetcher::format_secid("688001"), "1.688001");
    }

    #[test]
    fn test_parse_response() {
        let json = r#"{
            "rc": 0,
            "rt": 17,
            "data": {
                "f57": "600519",
                "f58": "贵州茅台",
                "f116": 2200000000000,
                "f117": 2200000000000,
                "f162": 23.45,
                "f167": 8.76,
                "f173": 31.5,
                "f187": 42.12,
                "f92": 13.2,
                "f105": 56000000000
            }
        }"#;

        let resp: EastMoneyResponse = serde_json::from_str(json).unwrap();
        let data = resp.data.unwrap();

        assert_eq!(value_to_f64(&data.f162), Some(23.45));
        assert_eq!(value_to_f64(&data.f167), Some(8.76));
        assert_eq!(value_to_f64(&data.f116), Some(2200000000000.0));
        assert_eq!(value_to_f64(&data.f173), Some(31.5));
    }

    #[test]
    fn test_value_to_f64() {
        assert_eq!(value_to_f64(&Some(serde_json::json!(23.45))), Some(23.45));
        assert_eq!(value_to_f64(&Some(serde_json::json!("18.2"))), Some(18.2));
        assert_eq!(value_to_f64(&Some(serde_json::json!(100))), Some(100.0));
        assert_eq!(value_to_f64(&None), None);
    }
}
