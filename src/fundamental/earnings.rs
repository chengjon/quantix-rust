//! 财报数据获取

use super::types::EarningsReport;
use crate::core::{QuantixError, Result};
use rust_decimal::Decimal;
use serde::Deserialize;

/// EastMoney 财报分析 API 响应
#[derive(Debug, Deserialize)]
struct EarningsApiResponse {
    data: Option<EarningsApiData>,
}

#[derive(Debug, Deserialize)]
struct EarningsApiData {
    /// f57: 代码
    #[serde(rename = "f57")]
    _f57: Option<serde_json::Value>,
    /// f58: 名称
    #[serde(rename = "f58")]
    _f58: Option<serde_json::Value>,
    /// f162: 市盈率(动)
    #[serde(rename = "f162")]
    _f162: Option<serde_json::Value>,
    /// f167: 市净率
    #[serde(rename = "f167")]
    _f167: Option<serde_json::Value>,
    /// f183: 总营收 (元)
    f183: Option<serde_json::Value>,
    /// f184: 总营收同比增长(%)
    f184: Option<serde_json::Value>,
    /// f185: 净利润同比(%)
    f185: Option<serde_json::Value>,
    /// f186: 毛利率(%)
    f186: Option<serde_json::Value>,
    /// f187: 净利润 (元)
    f187: Option<serde_json::Value>,
    /// f188: 净利润增长率(%)
    #[serde(rename = "f188")]
    _f188: Option<serde_json::Value>,
    /// f189: 未分配利润
    #[serde(rename = "f189")]
    _f189: Option<serde_json::Value>,
}

/// 从 serde_json::Value 提取 f64
fn value_to_f64(v: &Option<serde_json::Value>) -> Option<f64> {
    v.as_ref().and_then(|val| match val {
        serde_json::Value::Number(n) => n.as_f64(),
        serde_json::Value::String(s) => s.parse().ok(),
        _ => None,
    })
}

/// f64 -> Decimal
fn to_decimal(val: f64) -> Option<Decimal> {
    Decimal::from_f64_retain(val)
}

/// 财报数据获取器
pub struct EarningsFetcher {
    client: reqwest::Client,
}

impl EarningsFetcher {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
        }
    }

    /// 获取最新财报
    pub async fn fetch_latest(&self, code: &str) -> Result<EarningsReport> {
        let url = format!(
            "https://push2.eastmoney.com/api/qt/stock/get?secid={}&fields=f57,f58,f162,f167,f183,f184,f185,f186,f187,f188,f189",
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

        let resp: EarningsApiResponse = response
            .json()
            .await
            .map_err(|e| QuantixError::Other(format!("解析财报响应失败: {}", e)))?;

        let data = resp
            .data
            .ok_or_else(|| QuantixError::Other("EastMoney 财报 API 返回空数据".to_string()))?;

        let date = chrono::Utc::now().date_naive();
        let mut report = EarningsReport::new(code.to_string(), date, "最新".to_string());

        // 总营收 (元 -> 亿元)
        if let Some(v) = value_to_f64(&data.f183) {
            report.revenue = to_decimal(v / 1e8);
        }

        // 营收同比增长(%)
        if let Some(v) = value_to_f64(&data.f184) {
            report.revenue_yoy = to_decimal(v);
        }

        // 净利润 (元 -> 亿元)
        if let Some(v) = value_to_f64(&data.f187) {
            report.net_profit = to_decimal(v / 1e8);
        }

        // 净利润同比增长(%)
        if let Some(v) = value_to_f64(&data.f185) {
            report.net_profit_yoy = to_decimal(v);
        }

        // 毛利率(%)
        if let Some(v) = value_to_f64(&data.f186) {
            report.gross_margin = to_decimal(v);
        }

        Ok(report)
    }

    /// 获取历史财报 (简化版，返回最新一季)
    pub async fn fetch_history(&self, code: &str, _years: u32) -> Result<Vec<EarningsReport>> {
        let report = self.fetch_latest(code).await?;
        Ok(vec![report])
    }

    /// 格式化证券代码
    fn format_secid(code: &str) -> String {
        let code = code.trim_start_matches(|c: char| !c.is_ascii_digit());
        if code.starts_with('6') || code.starts_with('9') {
            format!("1.{}", code)
        } else {
            format!("0.{}", code)
        }
    }
}

impl Default for EarningsFetcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_secid() {
        assert_eq!(EarningsFetcher::format_secid("600519"), "1.600519");
        assert_eq!(EarningsFetcher::format_secid("000001"), "0.000001");
        assert_eq!(EarningsFetcher::format_secid("300001"), "0.300001");
    }

    #[test]
    fn test_parse_earnings_response() {
        let json = r#"{
            "rc": 0,
            "rt": 17,
            "data": {
                "f57": "600519",
                "f58": "贵州茅台",
                "f183": 1200000000000,
                "f184": 16.5,
                "f185": 13.2,
                "f186": 91.5,
                "f187": 56000000000,
                "f188": 13.2,
                "f189": 200000000000
            }
        }"#;

        let resp: EarningsApiResponse = serde_json::from_str(json).unwrap();
        let data = resp.data.unwrap();

        assert_eq!(value_to_f64(&data.f183), Some(1200000000000.0));
        assert_eq!(value_to_f64(&data.f184), Some(16.5));
        assert_eq!(value_to_f64(&data.f186), Some(91.5));
    }

    #[test]
    fn test_empty_response() {
        let json = r#"{"rc": 0, "rt": 17}"#;
        let resp: EarningsApiResponse = serde_json::from_str(json).unwrap();
        assert!(resp.data.is_none());
    }
}
