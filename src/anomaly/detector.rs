//! Anomaly detection service
//!
//! Integrates all components to provide end-to-end anomaly detection:
//! - Data fetching from East Money API
//! - Stock filtering
//! - Feature extraction
//! - Isolation Forest training
//! - Result output

use crate::anomaly::config::AnomalyConfig;
use crate::anomaly::features::{FeatureExtractor, OHLCVCandle, OHLCVSeries};
use crate::anomaly::filter::{StockFilter, StockInfo};
use crate::anomaly::forest::{AnomalyScore, IsolationForest};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

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
    pub anomalies: Vec<AnomalyScore>,
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

/// Anomaly detector
pub struct AnomalyDetector {
    config: AnomalyConfig,
    data_source: std::sync::Arc<dyn DataSource>,
}

impl AnomalyDetector {
    /// Create a new anomaly detector with configuration and data source
    pub fn new(config: AnomalyConfig, data_source: std::sync::Arc<dyn DataSource>) -> Self {
        Self {
            config,
            data_source,
        }
    }

    /// Run the complete anomaly detection pipeline
    pub async fn detect(&self) -> Result<AnomalyResult, String> {
        let start_time = chrono::Utc::now();

        info!(
            "Starting anomaly detection with period={}min",
            self.config.data.kline_period
        );

        // 1. Fetch stock list
        info!("Fetching stock list...");
        let stocks = self.data_source.get_stock_list().await?;
        info!("Fetched {} stocks", stocks.len());

        // 2. Filter stocks
        let filter = StockFilter::new(self.config.filter.clone());
        let filtered_stocks = filter.filter_stock_list(&stocks);
        info!("After filtering: {} stocks", filtered_stocks.len());

        // 3. Fetch K-line data in parallel (with concurrency limit)
        info!("Fetching K-line data...");
        let klines = self.fetch_klines_batch(&filtered_stocks).await;
        info!("Fetched K-lines for {} stocks", klines.len());

        // 4. Extract features
        info!("Extracting features...");
        let extractor = FeatureExtractor::new(self.config.features.clone());
        let (features, codes, names) = extractor.extract_all(&klines);
        info!("Extracted features for {} stocks", features.len());

        if features.is_empty() {
            return Err("No valid features extracted".to_string());
        }

        // 5. Train Isolation Forest
        info!(
            "Training Isolation Forest with {} trees...",
            self.config.forest.n_estimators
        );
        let mut forest = IsolationForest::new()
            .n_estimators(self.config.forest.n_estimators)
            .max_samples(self.config.forest.max_samples)
            .random_state(self.config.forest.random_state)
            .contamination(self.config.forest.contamination);

        forest
            .fit(&features)
            .map_err(|e| format!("Failed to fit Isolation Forest: {}", e))?;

        // 6. Find anomalies
        info!("Finding top {} anomalies...", self.config.forest.top_n);
        let mut anomalies =
            forest.find_anomalies(&features, &codes, &names, self.config.forest.top_n);

        // 7. Enrich with additional statistics
        self.enrich_anomalies(&mut anomalies, &klines);

        // 8. Filter to anomalies only if configured
        if self.config.output.anomalies_only {
            anomalies.retain(|a| a.is_anomaly);
        }

        let result = AnomalyResult {
            timestamp: start_time.to_rfc3339(),
            config: ResultConfig::from(&self.config),
            total_stocks: stocks.len(),
            filtered_stocks: stocks.len() - filtered_stocks.len(),
            valid_features: features.len(),
            anomalies,
        };

        info!(
            "Detection complete. Found {} anomalies.",
            result.anomalies.len()
        );
        Ok(result)
    }

    /// Fetch K-lines for multiple stocks with concurrency control
    async fn fetch_klines_batch(&self, stocks: &[StockInfo]) -> Vec<OHLCVSeries> {
        let period = self.config.data.kline_period;
        let adjust = self.config.data.adjust_type.clone(); // Clone to own the data
        let limit = self.config.data.kline_limit;
        let max_concurrent = self.config.data.max_concurrent;

        // Use semaphore for concurrency control
        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(max_concurrent));
        let mut handles = Vec::new();

        for stock in stocks {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let code = stock.code.clone();
            let adjust = adjust.clone(); // Clone for each task
            let ds = self.data_source.clone(); // Clone Arc for each task

            let handle = tokio::spawn(async move {
                let result = ds.get_klines(&code, period, &adjust, limit).await;
                drop(permit);
                result
            });

            handles.push((stock.code.clone(), handle));
        }

        let mut results = Vec::new();
        for (code, handle) in handles {
            match handle.await {
                Ok(Ok(series)) => results.push(series),
                Ok(Err(e)) => {
                    debug!("Failed to fetch K-lines for {}: {}", code, e);
                }
                Err(e) => {
                    warn!("Task panicked for {}: {}", code, e);
                }
            }
        }

        results
    }

    /// Enrich anomaly scores with additional statistics
    fn enrich_anomalies(&self, anomalies: &mut [AnomalyScore], klines: &[OHLCVSeries]) {
        let kline_map: HashMap<&str, &OHLCVSeries> =
            klines.iter().map(|k| (k.code.as_str(), k)).collect();

        for anomaly in anomalies.iter_mut() {
            if let Some(series) = kline_map.get(anomaly.code.as_str()) {
                // Volume ratio
                if series.len() >= 5 {
                    let today_vol = series.candles.last().map(|c| c.volume).unwrap_or(0.0);
                    let avg_5d = series.avg_volume(5);
                    if avg_5d > 0.0 {
                        anomaly.volume_ratio = Some(today_vol / avg_5d);
                    }
                }

                // Volatility
                anomaly.volatility_5 = if series.len() >= 5 {
                    let vol = series.volatility(5);
                    if vol > 0.0 { Some(vol) } else { None }
                } else {
                    None
                };

                anomaly.volatility_20 = if series.len() >= 20 {
                    let vol = series.volatility(20);
                    if vol > 0.0 { Some(vol) } else { None }
                } else {
                    None
                };

                // Latest time
                if let Some(last) = series.candles.last() {
                    anomaly.latest_time = Some(last.time.clone());
                }
            }
        }
    }

    /// Output results in the configured format
    pub fn output_results(&self, result: &AnomalyResult) {
        match self.config.output.format.as_str() {
            "json" => self.output_json(result),
            "csv" => self.output_csv(result),
            _ => self.output_cli(result),
        }
    }

    /// Output in CLI format
    fn output_cli(&self, result: &AnomalyResult) {
        println!("{}", "=".repeat(80));
        println!(
            "📊 TOP {} ANOMALOUS STOCKS (Isolation Forest)",
            result.anomalies.len()
        );
        println!("{}", "=".repeat(80));
        println!();

        for (i, anomaly) in result.anomalies.iter().enumerate() {
            let anomaly_marker = if anomaly.is_anomaly {
                "⚠️ 异常"
            } else {
                ""
            };
            println!("[{}] {} {}", i + 1, anomaly.code, anomaly.name);
            println!("  异常分数: {:.4} {}", anomaly.score, anomaly_marker);

            if let Some(time) = &anomaly.latest_time {
                println!("  最新时间: {}", time);
            }

            if let Some(vol_ratio) = anomaly.volume_ratio {
                println!("  量比: {:.2}", vol_ratio);
            }

            if let Some(vol5) = anomaly.volatility_5 {
                println!("  5周期波动率: {:.4}", vol5);
            }

            if let Some(vol20) = anomaly.volatility_20 {
                println!("  20周期波动率: {:.4}", vol20);
            }

            println!("{}", "-".repeat(40));
        }

        println!();
        println!("📌 说明:");
        println!("  - 异常分数 < 0: 异常股票，分数越低越异常");
        println!("  - 异常股票的未来价格变动通常是正常股票的2倍以上");
        println!("  - 算法不预测涨跌方向，只检测异常模式");
        println!();
        println!("📊 统计:");
        println!("  - 总股票数: {}", result.total_stocks);
        println!(
            "  - 过滤后: {} (过滤掉 {})",
            result.valid_features, result.filtered_stocks
        );
        println!("  - K线周期: {}分钟", result.config.period);
        println!("  - 树数量: {}", result.config.n_estimators);
    }

    /// Output in JSON format
    fn output_json(&self, result: &AnomalyResult) {
        match serde_json::to_string_pretty(result) {
            Ok(json) => println!("{}", json),
            Err(e) => eprintln!("Failed to serialize JSON: {}", e),
        }
    }

    /// Output in CSV format
    fn output_csv(&self, result: &AnomalyResult) {
        println!("rank,code,name,score,is_anomaly,volume_ratio,volatility_5,volatility_20");
        for (i, a) in result.anomalies.iter().enumerate() {
            println!(
                "{},{},{},{:.6},{},{:.4},{:.6},{:.6}",
                i + 1,
                a.code,
                a.name,
                a.score,
                a.is_anomaly,
                a.volume_ratio.unwrap_or(0.0),
                a.volatility_5.unwrap_or(0.0),
                a.volatility_20.unwrap_or(0.0)
            );
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_detector_with_mock_data() {
        let config = AnomalyConfig::default().with_top_n(10).with_period(15);

        let data_source = std::sync::Arc::new(MockDataSource::new(100));
        let detector = AnomalyDetector::new(config, data_source);

        let result = detector.detect().await;

        assert!(result.is_ok());
        let result = result.unwrap();

        assert!(!result.anomalies.is_empty());
        assert!(result.anomalies.len() <= 10);
        assert_eq!(result.total_stocks, 100);
    }

    #[test]
    fn test_result_config_from_anomaly_config() {
        let config = AnomalyConfig::default().with_top_n(50).with_period(5);

        let result_config = ResultConfig::from(&config);

        assert_eq!(result_config.period, 5);
        assert_eq!(result_config.top_n, 50);
    }
}
