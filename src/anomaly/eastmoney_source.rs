//! East Money data source for anomaly detection
//!
//! Implements the DataSource trait using East Money API

use super::detector::DataSource;
use super::features::{OHLCVCandle, OHLCVSeries};
use super::filter::StockInfo;
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use tracing::{debug, info, warn};

/// East Money API data source for anomaly detection
pub struct EastMoneyAnomalySource {
    client: Client,
    base_url: String,
}

impl EastMoneyAnomalySource {
    /// Create a new East Money data source
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .build()
            .unwrap_or_default();

        Self {
            client,
            base_url: "https://push2.eastmoney.com".to_string(),
        }
    }

    /// Create with custom base URL (for testing)
    pub fn with_base_url(base_url: &str) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .build()
            .unwrap_or_default();

        Self {
            client,
            base_url: base_url.to_string(),
        }
    }

    /// Get market code for a stock
    fn get_market_code(code: &str) -> &'static str {
        if code.starts_with('6') {
            "1" // 上海
        } else if code.starts_with('8') || code.starts_with('4') {
            "0" // 北京 (北交所)
        } else {
            "0" // 深圳
        }
    }

    /// Build secid for East Money API
    fn build_secid(code: &str) -> String {
        format!("{}.{}", Self::get_market_code(code), code)
    }
}

impl Default for EastMoneyAnomalySource {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DataSource for EastMoneyAnomalySource {
    /// Get list of all A-share stocks from East Money
    async fn get_stock_list(&self) -> Result<Vec<StockInfo>, String> {
        info!("Fetching stock list from East Money API");

        let url = format!("{}/api/qt/clist/get", self.base_url);

        // Parameters for A-share stocks
        // fs: m:0+t:6,m:0+t:80,m:1+t:2,m:1+t:23
        // - m:0+t:6 = 深圳A股
        // - m:0+t:80 = 深圳创业板
        // - m:1+t:2 = 上海A股
        // - m:1+t:23 = 上海科创板
        let params = [
            ("pn", "1"),           // Page number
            ("pz", "5000"),        // Page size (max)
            ("po", "1"),           // Sort order
            ("np", "1"),           // No pagination
            ("fltt", "2"),         // Float format
            ("invt", "2"),         // Investment type
            ("fid", "f3"),         // Sort field (change %)
            ("fs", "m:0+t:6,m:0+t:80,m:1+t:2,m:1+t:23"),
            ("fields", "f12,f14,f2,f3,f5,f6"),
        ];

        let response = self
            .client
            .get(&url)
            .query(&params)
            .header("Referer", "https://data.eastmoney.com/")
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("API returned status: {}", response.status()));
        }

        let json: EastMoneyListResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;

        let stocks: Vec<StockInfo> = json
            .data
            .diff
            .into_iter()
            .filter_map(|item| {
                // Skip stocks with no price data
                if item.price <= 0.0 {
                    return None;
                }

                Some(StockInfo {
                    code: item.code,
                    name: item.name,
                    price: item.price,
                    change_pct: item.change_pct.unwrap_or(0.0),
                    volume: item.volume.unwrap_or(0.0),
                    amount: item.amount.unwrap_or(0.0),
                    list_date: None,
                })
            })
            .collect();

        info!("Fetched {} stocks from East Money", stocks.len());
        Ok(stocks)
    }

    /// Get K-line data for a single stock
    async fn get_klines(
        &self,
        code: &str,
        period: u32,
        adjust: &str,
        limit: usize,
    ) -> Result<OHLCVSeries, String> {
        debug!("Fetching K-lines for {} (period={}min)", code, period);

        let url = format!("{}/api/qt/stock/kline/get", self.base_url);

        // Convert period to klt (K-line type)
        // 1=1min, 5=5min, 15=15min, 30=30min, 60=60min, 101=日线, 102=周线, 103=月线
        let klt = match period {
            1 => "1",
            5 => "5",
            15 => "15",
            30 => "30",
            60 => "60",
            1440 | 0 => "101", // 日线
            _ => "15",          // Default to 15min
        };

        // Adjust type: qfq=前复权, hfq=后复权, none=不复权
        let fqt = match adjust.to_lowercase().as_str() {
            "qfq" | "前复权" => "1",
            "hfq" | "后复权" => "2",
            _ => "0", // 不复权
        };

        let secid = Self::build_secid(code);

        let params = [
            ("secid", secid.as_str()),
            ("fields1", "f1,f2,f3,f4,f5,f6"),
            ("fields2", "f51,f52,f53,f54,f55,f56,f57"),
            ("klt", klt),
            ("fqt", fqt),
            ("end", "20500101"),
            ("lmt", &limit.to_string()),
        ];

        let response = self
            .client
            .get(&url)
            .query(&params)
            .header("Referer", "https://quote.eastmoney.com/")
            .send()
            .await
            .map_err(|e| format!("HTTP request failed for {}: {}", code, e))?;

        if !response.status().is_success() {
            return Err(format!("API returned status {} for {}", response.status(), code));
        }

        let json: EastMoneyKlineResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse K-line JSON for {}: {}", code, e))?;

        // Parse K-line data
        let name = json.data.name.clone();
        let mut series = OHLCVSeries::new(code, &name);

        if let Some(klines) = json.data.klines {
            for kline_str in klines {
                // Parse: "time,open,close,high,low,volume,amount,amplitude"
                let parts: Vec<&str> = kline_str.split(',').collect();
                if parts.len() >= 6 {
                    let time = parts[0].to_string();
                    let open = parts[1].parse().unwrap_or(0.0);
                    let close = parts[2].parse().unwrap_or(0.0);
                    let high = parts[3].parse().unwrap_or(0.0);
                    let low = parts[4].parse().unwrap_or(0.0);
                    let volume = parts[5].parse().unwrap_or(0.0);

                    if open > 0.0 && close > 0.0 {
                        series.add(OHLCVCandle::new(&time, open, close, high, low, volume));
                    }
                }
            }
        }

        debug!("Fetched {} candles for {}", series.len(), code);
        Ok(series)
    }
}

// ============================================================================
// API Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
struct EastMoneyListResponse {
    data: EastMoneyListData,
}

#[derive(Debug, Deserialize)]
struct EastMoneyListData {
    diff: Vec<EastMoneyStockItem>,
}

#[derive(Debug, Deserialize)]
struct EastMoneyStockItem {
    #[serde(rename = "f12")]
    code: String,
    #[serde(rename = "f14")]
    name: String,
    #[serde(rename = "f2")]
    price: f64,
    #[serde(rename = "f3")]
    change_pct: Option<f64>,
    #[serde(rename = "f5")]
    volume: Option<f64>,
    #[serde(rename = "f6")]
    amount: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct EastMoneyKlineResponse {
    data: EastMoneyKlineData,
}

#[derive(Debug, Deserialize)]
struct EastMoneyKlineData {
    #[serde(rename = "code")]
    _code: Option<String>,
    #[serde(rename = "market")]
    _market: Option<i32>,
    #[serde(rename = "name")]
    name: String,
    #[serde(rename = "klines")]
    klines: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_market_code() {
        assert_eq!(EastMoneyAnomalySource::get_market_code("600000"), "1");
        assert_eq!(EastMoneyAnomalySource::get_market_code("000001"), "0");
        assert_eq!(EastMoneyAnomalySource::get_market_code("300001"), "0");
        assert_eq!(EastMoneyAnomalySource::get_market_code("688001"), "1");
        assert_eq!(EastMoneyAnomalySource::get_market_code("830001"), "0");
        assert_eq!(EastMoneyAnomalySource::get_market_code("430001"), "0");
    }

    #[test]
    fn test_build_secid() {
        assert_eq!(
            EastMoneyAnomalySource::build_secid("600000"),
            "1.600000"
        );
        assert_eq!(
            EastMoneyAnomalySource::build_secid("000001"),
            "0.000001"
        );
    }

    #[test]
    fn test_source_creation() {
        let source = EastMoneyAnomalySource::new();
        assert_eq!(source.base_url, "https://push2.eastmoney.com");
    }
}
