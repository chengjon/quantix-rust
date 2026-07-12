//! Anomaly detection result types, data-source trait and mock.

use crate::anomaly::config::AnomalyConfig;
use crate::anomaly::features::{OHLCVCandle, OHLCVSeries};
use crate::anomaly::filter::StockInfo;

use serde::{Deserialize, Serialize};

/// Anomaly detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyResult {
    /// Timestamp of detection
    pub timestamp: String,
    /// Configuration used
    pub config: ResultConfig,
    /// Total stocks processed
    pub total_stocks: usize,
    /// Stocks filtered out
    pub filtered_stocks: usize,
    /// Valid feature sets extracted
    pub valid_features: usize,
    /// Anomaly scores
    pub anomalies: Vec<crate::anomaly::forest::AnomalyScore>,
}

/// Configuration summary for results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultConfig {
    /// K-line period
    pub period: u32,
    /// Number of estimators
    pub n_estimators: usize,
    /// History to use
    pub history_to_use: usize,
    /// Top N returned
    pub top_n: usize,
}

impl From<&AnomalyConfig> for ResultConfig {
    fn from(config: &AnomalyConfig) -> Self {
        Self {
            period: config.data.kline_period,
            n_estimators: config.forest.n_estimators,
            history_to_use: config.features.history_to_use,
            top_n: config.forest.top_n,
        }
    }
}

/// Data source trait for fetching stock data
#[async_trait::async_trait]
pub trait DataSource: Send + Sync {
    /// Get list of all A-share stocks
    async fn get_stock_list(&self) -> Result<Vec<StockInfo>, String>;

    /// Get K-line data for a single stock
    async fn get_klines(
        &self,
        code: &str,
        period: u32,
        adjust: &str,
        limit: usize,
    ) -> Result<OHLCVSeries, String>;
}

/// Mock data source for testing
pub struct MockDataSource {
    stocks: Vec<StockInfo>,
}

impl MockDataSource {
    /// Create a new mock data source with generated data
    pub fn new(stock_count: usize) -> Self {
        let stocks: Vec<StockInfo> = (0..stock_count)
            .map(|i| {
                let code = format!("{:06}", i);
                let name = format!("测试股票{}", i);
                StockInfo::new(
                    &code,
                    &name,
                    10.0 + i as f64 % 100.0,
                    0.0,
                    100000.0,
                    1000000.0,
                )
            })
            .collect();

        Self { stocks }
    }
}

#[async_trait::async_trait]
impl DataSource for MockDataSource {
    async fn get_stock_list(&self) -> Result<Vec<StockInfo>, String> {
        Ok(self.stocks.clone())
    }

    async fn get_klines(
        &self,
        code: &str,
        _period: u32,
        _adjust: &str,
        limit: usize,
    ) -> Result<OHLCVSeries, String> {
        let name = format!("股票{}", code);
        let mut series = OHLCVSeries::new(code, &name);

        // Generate random walk data
        let mut price = 100.0;
        for i in 0..limit {
            let change = (i as f64 % 10.0 - 5.0) * 0.01;
            price *= 1.0 + change;

            let high = price * 1.02;
            let low = price * 0.98;
            let open = price * (1.0 + (i as f64 % 3.0 - 1.5) * 0.005);
            let volume = 1000000.0 + (i as f64 % 100.0) * 10000.0;

            series.add(OHLCVCandle::new(
                &format!("2026-03-{:02} 10:00", (i % 28) + 1),
                open,
                price,
                high,
                low,
                volume,
            ));
        }

        Ok(series)
    }
}
