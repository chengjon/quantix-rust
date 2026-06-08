//! 机构持仓数据获取

use super::types::InstitutionHolding;
use crate::core::{QuantixError, Result};
use rust_decimal::Decimal;
use serde::Deserialize;

/// EastMoney 机构持仓 API 响应
#[derive(Debug, Deserialize)]
struct HoldingApiResponse {
    result: Option<HoldingResult>,
}

#[derive(Debug, Deserialize)]
struct HoldingResult {
    data: Option<Vec<HoldingItem>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct HoldingItem {
    /// 机构名称
    holder_name: Option<String>,
    /// 持股数量
    hold_num: Option<serde_json::Value>,
    /// 占流通股比例(%)
    hold_ratio: Option<serde_json::Value>,
    /// 持股市值
    hold_value: Option<serde_json::Value>,
    /// 变动比例(%)
    change_ratio: Option<serde_json::Value>,
    /// 报告期
    #[allow(dead_code)]
    end_date: Option<String>,
    /// 机构类型
    holder_type: Option<serde_json::Value>,
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

fn holder_type_label(t: Option<&serde_json::Value>) -> String {
    t.and_then(|v| v.as_i64())
        .map(|n| match n {
            1 => "基金",
            2 => "QFII",
            3 => "社保",
            4 => "券商",
            5 => "保险",
            6 => "信托",
            _ => "其他",
        })
        .unwrap_or("其他")
        .to_string()
}

fn change_direction(change: Option<f64>) -> String {
    match change {
        Some(c) if c > 0.0 => "增持".to_string(),
        Some(c) if c < 0.0 => "减持".to_string(),
        _ => "不变".to_string(),
    }
}

/// 机构持仓数据获取器
pub struct InstitutionFetcher {
    client: reqwest::Client,
}

impl InstitutionFetcher {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
        }
    }

    /// 获取机构持仓
    pub async fn fetch_holdings(&self, code: &str) -> Result<Vec<InstitutionHolding>> {
        let secid = Self::format_secid(code);
        let url = format!(
            "https://data.eastmoney.com/dataapi/stockholder/list?code={}&type=0&num=20",
            secid
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

        let resp: HoldingApiResponse = response
            .json()
            .await
            .map_err(|e| QuantixError::Other(format!("解析机构持仓响应失败: {}", e)))?;

        let items = resp.result.and_then(|r| r.data).unwrap_or_default();

        let report_date = chrono::Utc::now().date_naive();
        let code_str = code.to_string();

        let holdings: Vec<InstitutionHolding> = items
            .into_iter()
            .map(|item| {
                let change_pct = value_to_f64(&item.change_ratio);
                InstitutionHolding {
                    code: code_str.clone(),
                    institution_name: item.holder_name.unwrap_or_else(|| "-".to_string()),
                    institution_type: holder_type_label(item.holder_type.as_ref()),
                    shares: Decimal::from_f64_retain(
                        value_to_f64(&item.hold_num).unwrap_or(0.0) / 10000.0,
                    )
                    .unwrap_or(Decimal::ZERO),
                    market_value: value_to_f64(&item.hold_value)
                        .and_then(|v| to_decimal(v / 10000.0)),
                    float_ratio: value_to_f64(&item.hold_ratio).and_then(to_decimal),
                    change_direction: change_direction(change_pct),
                    change_shares: change_pct.and_then(to_decimal),
                    report_date,
                }
            })
            .collect();

        Ok(holdings)
    }

    fn format_secid(code: &str) -> String {
        let code = code.trim_start_matches(|c: char| !c.is_ascii_digit());
        if code.starts_with('6') || code.starts_with('9') {
            format!("1.{}", code)
        } else {
            format!("0.{}", code)
        }
    }
}

impl Default for InstitutionFetcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_holder_type_label() {
        assert_eq!(holder_type_label(Some(&serde_json::json!(1))), "基金");
        assert_eq!(holder_type_label(Some(&serde_json::json!(2))), "QFII");
        assert_eq!(holder_type_label(Some(&serde_json::json!(3))), "社保");
        assert_eq!(holder_type_label(None), "其他");
    }

    #[test]
    fn test_change_direction() {
        assert_eq!(change_direction(Some(5.0)), "增持");
        assert_eq!(change_direction(Some(-3.0)), "减持");
        assert_eq!(change_direction(None), "不变");
        assert_eq!(change_direction(Some(0.0)), "不变");
    }

    #[test]
    fn test_parse_empty_response() {
        let json = r#"{"result": null}"#;
        let resp: HoldingApiResponse = serde_json::from_str(json).unwrap();
        assert!(resp.result.is_none());
    }

    #[test]
    fn test_parse_holdings() {
        let json = r#"{
            "result": {
                "data": [
                    {
                        "HolderName": "华夏大盘精选",
                        "HoldNum": 5000000,
                        "HoldRatio": 0.39,
                        "HoldValue": 950000000,
                        "ChangeRatio": 10.5,
                        "EndDate": "2024-12-31",
                        "HolderType": 1
                    }
                ]
            }
        }"#;

        let resp: HoldingApiResponse = serde_json::from_str(json).unwrap();
        let data = resp.result.unwrap().data.unwrap();
        assert_eq!(data.len(), 1);
        assert_eq!(data[0].holder_name.as_deref(), Some("华夏大盘精选"));
        assert_eq!(value_to_f64(&data[0].hold_num), Some(5000000.0));
    }
}
