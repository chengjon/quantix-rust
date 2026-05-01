//! Stock filtering utilities for A-share market
//!
//! Provides filters for A-share specific requirements:
//! - ST stock filtering
//! - Limit up/down detection
//! - Suspended stock detection
//! - New stock filtering
//! - Volume and volatility filtering

use super::config::FilterConfig;
use serde::{Deserialize, Serialize};

/// Stock information for filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockInfo {
    /// Stock code (e.g., "000001")
    pub code: String,
    /// Stock name
    pub name: String,
    /// Latest price
    pub price: f64,
    /// Price change percentage
    pub change_pct: f64,
    /// Trading volume
    pub volume: f64,
    /// Trading amount
    pub amount: f64,
    /// List date (optional, for new stock filtering)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_date: Option<String>,
}

impl StockInfo {
    /// Create a new stock info
    pub fn new(
        code: &str,
        name: &str,
        price: f64,
        change_pct: f64,
        volume: f64,
        amount: f64,
    ) -> Self {
        Self {
            code: code.to_string(),
            name: name.to_string(),
            price,
            change_pct,
            volume,
            amount,
            list_date: None,
        }
    }
}

/// Stock filter for applying various filtering rules
pub struct StockFilter {
    config: FilterConfig,
}

impl StockFilter {
    /// Create a new filter with configuration
    pub fn new(config: FilterConfig) -> Self {
        Self { config }
    }

    /// Create a filter with default configuration
    pub fn with_defaults() -> Self {
        Self::new(FilterConfig::default())
    }

    /// Check if a stock info passes basic filters (ST, limit, etc.)
    pub fn passes_info_filter(&self, info: &StockInfo) -> bool {
        // Filter ST stocks
        if self.config.exclude_st && self.is_st_stock(info) {
            return false;
        }

        // Filter limit up/down stocks
        if self.config.exclude_limit && self.is_at_limit(info) {
            return false;
        }

        // Filter low volume
        if info.volume < self.config.min_volume {
            return false;
        }

        true
    }

    /// Check if stock is an ST stock (Special Treatment)
    fn is_st_stock(&self, info: &StockInfo) -> bool {
        let name = info.name.to_uppercase();
        name.contains("ST") || name.contains("*ST") || name.contains("退")
    }

    /// Check if stock is at limit up or down
    fn is_at_limit(&self, info: &StockInfo) -> bool {
        // Main board: ±10%
        // ChiNext (300xxx) / STAR (688xxx): ±20%
        // ST: ±5%
        // Beijing (8xxxxx or 4xxxxx): ±30%
        let abs_change = info.change_pct.abs();

        // Check board type
        let is_chinext = info.code.starts_with("300");
        let is_star = info.code.starts_with("688");
        let is_beijing = info.code.starts_with("8") || info.code.starts_with("4");
        let is_st = self.is_st_stock(info);

        let limit = if is_st {
            4.9 // ST limit is ±5%
        } else if is_chinext || is_star {
            19.9 // ChiNext/STAR limit is ±20%
        } else if is_beijing {
            29.9 // Beijing limit is ±30%
        } else {
            9.9 // Main board limit is ±10%
        };

        abs_change >= limit
    }

    /// Filter a list of stock info, returning only valid stocks
    pub fn filter_stock_list(&self, stocks: &[StockInfo]) -> Vec<StockInfo> {
        let filtered: Vec<StockInfo> = stocks
            .iter()
            .filter(|s| self.passes_info_filter(s))
            .cloned()
            .collect();

        // Apply max_stocks limit if configured
        if self.config.max_stocks > 0 && filtered.len() > self.config.max_stocks {
            filtered.into_iter().take(self.config.max_stocks).collect()
        } else {
            filtered
        }
    }

    /// Check if a K-line series passes all filters
    ///
    /// # Arguments
    /// * `closes` - Close prices
    /// * `volumes` - Volume data
    ///
    /// # Returns
    /// * `true` if the series passes all filters
    pub fn passes_series_filter(&self, closes: &[f64], volumes: &[f64]) -> bool {
        // Check minimum candles
        if closes.len() < self.config.min_candles {
            return false;
        }

        // Check minimum volume
        let avg_vol = volumes.iter().sum::<f64>() / volumes.len() as f64;
        if avg_vol < self.config.min_volume {
            return false;
        }

        // Check minimum volatility
        let volatility = calculate_volatility(closes);
        if volatility < self.config.min_volatility {
            return false;
        }

        // Check for suspended stock
        if self.is_suspended(closes) {
            return false;
        }

        // Check for limit up/down in last candle
        if self.config.exclude_limit && self.is_limit(closes) {
            return false;
        }

        true
    }

    /// Check if stock is suspended (no trading)
    fn is_suspended(&self, closes: &[f64]) -> bool {
        if closes.len() < 5 {
            return false;
        }

        // Check if last 5 closes are identical (suspended)
        let last_closes: Vec<f64> = closes.iter().rev().take(5).cloned().collect();
        let first = last_closes[0];

        last_closes
            .iter()
            .all(|&c| (c - first).abs() < f64::EPSILON)
    }

    /// Check if the last candle is at limit
    fn is_limit(&self, closes: &[f64]) -> bool {
        if closes.len() < 2 {
            return false;
        }

        let last = closes[closes.len() - 1];
        let prev = closes[closes.len() - 2];

        if prev <= 0.0 {
            return false;
        }

        let pct_change = (last - prev) / prev * 100.0;
        pct_change.abs() >= 9.9
    }

    /// Get the filter configuration
    pub fn config(&self) -> &FilterConfig {
        &self.config
    }
}

impl Default for StockFilter {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// Calculate volatility (standard deviation of returns)
fn calculate_volatility(closes: &[f64]) -> f64 {
    if closes.len() < 2 {
        return 0.0;
    }

    // Calculate log returns
    let mut returns = Vec::with_capacity(closes.len() - 1);
    for i in 1..closes.len() {
        if closes[i - 1] > 0.0 {
            returns.push((closes[i] / closes[i - 1]).ln());
        }
    }

    if returns.is_empty() {
        return 0.0;
    }

    // Calculate standard deviation
    let mean = returns.iter().sum::<f64>() / returns.len() as f64;
    let variance = returns.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / returns.len() as f64;

    variance.sqrt()
}

/// Format large numbers for display
pub fn format_large_value(value: f64) -> String {
    if value < 1000.0 {
        format!("{:.0}", value)
    } else if value < 100_000_000.0 {
        format!("{:.2}万", value / 10000.0)
    } else {
        format!("{:.2}亿", value / 100_000_000.0)
    }
}

/// Calculate percentage change
pub fn pct_change(old: f64, new: f64) -> f64 {
    if old > 0.0 {
        (new - old) / old * 100.0
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_st_stock_detection() {
        let filter = StockFilter::with_defaults();

        let st_stock = StockInfo::new("000001", "ST某某", 5.0, 0.0, 100000.0, 500000.0);
        let normal_stock = StockInfo::new("000002", "平安银行", 15.0, 2.0, 200000.0, 3000000.0);

        assert!(!filter.passes_info_filter(&st_stock));
        assert!(filter.passes_info_filter(&normal_stock));
    }

    #[test]
    fn test_limit_detection() {
        let filter = StockFilter::with_defaults();

        let limit_up = StockInfo::new("000001", "测试股", 11.0, 10.0, 100000.0, 1100000.0);
        let normal = StockInfo::new("000002", "测试股2", 10.5, 5.0, 100000.0, 1050000.0);

        assert!(!filter.passes_info_filter(&limit_up));
        assert!(filter.passes_info_filter(&normal));
    }

    #[test]
    fn test_chinext_limit() {
        let filter = StockFilter::with_defaults();

        // ChiNext stocks have ±20% limit
        let chinext_normal = StockInfo::new("300001", "创业板", 15.0, 15.0, 100000.0, 1500000.0);
        let chinext_limit = StockInfo::new("300002", "创业板2", 15.0, 20.0, 100000.0, 1500000.0);

        assert!(filter.passes_info_filter(&chinext_normal));
        assert!(!filter.passes_info_filter(&chinext_limit));
    }

    #[test]
    fn test_suspended_detection() {
        let filter = StockFilter::with_defaults();

        // Suspended: same close for 5 bars
        let suspended_closes = vec![10.0, 10.0, 10.0, 10.0, 10.0, 10.0];
        let normal_closes = vec![10.0, 10.5, 11.0, 10.8, 11.2, 11.5];
        let volumes = vec![100000.0; 6];

        assert!(filter.is_suspended(&suspended_closes));
        assert!(!filter.is_suspended(&normal_closes));
    }

    #[test]
    fn test_format_large_value() {
        assert_eq!(format_large_value(500.0), "500");
        assert!(format_large_value(15000.0).contains("万"));
        assert!(format_large_value(150000000.0).contains("亿"));
    }

    #[test]
    fn test_max_stocks_limit() {
        let config = FilterConfig {
            max_stocks: 5,
            ..Default::default()
        };
        let filter = StockFilter::new(config);

        let stocks: Vec<StockInfo> = (0..10)
            .map(|i| {
                StockInfo::new(
                    &format!("{:06}", i),
                    &format!("股票{}", i),
                    10.0,
                    0.0,
                    50000.0,
                    500000.0,
                )
            })
            .collect();

        let filtered = filter.filter_stock_list(&stocks);
        assert_eq!(filtered.len(), 5);
    }
}
