//! Anomaly detection service — the core `AnomalyDetector` pipeline and output
//! renderers.

use crate::anomaly::config::AnomalyConfig;
use crate::anomaly::features::{FeatureExtractor, OHLCVSeries};
use crate::anomaly::filter::{StockFilter, StockInfo};
use crate::anomaly::forest::{AnomalyScore, IsolationForest};

use std::collections::HashMap;
use std::fmt::Write as _;
use tracing::{debug, info, warn};

use super::types::{AnomalyResult, DataSource, ResultConfig};

macro_rules! append_line {
    ($output:expr) => {{
        let _ = writeln!($output);
    }};
    ($output:expr, $($arg:tt)*) => {{
        let _ = writeln!($output, $($arg)*);
    }};
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
            let permit = match semaphore.clone().acquire_owned().await {
                Ok(permit) => permit,
                Err(e) => {
                    warn!("Failed to acquire K-line fetch permit: {}", e);
                    continue;
                }
            };
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

    /// Render results in the configured format.
    pub fn output_results(&self, result: &AnomalyResult) -> serde_json::Result<String> {
        match self.config.output.format.as_str() {
            "json" => self.output_json(result),
            "csv" => Ok(self.output_csv(result)),
            _ => Ok(self.output_cli(result)),
        }
    }

    /// Render in CLI format.
    fn output_cli(&self, result: &AnomalyResult) -> String {
        let mut output = String::new();

        append_line!(&mut output, "{}", "=".repeat(80));
        append_line!(
            &mut output,
            "📊 TOP {} ANOMALOUS STOCKS (Isolation Forest)",
            result.anomalies.len()
        );
        append_line!(&mut output, "{}", "=".repeat(80));
        append_line!(&mut output);

        for (i, anomaly) in result.anomalies.iter().enumerate() {
            let anomaly_marker = if anomaly.is_anomaly {
                "⚠️ 异常"
            } else {
                ""
            };
            append_line!(&mut output, "[{}] {} {}", i + 1, anomaly.code, anomaly.name);
            append_line!(
                &mut output,
                "  异常分数: {:.4} {}",
                anomaly.score,
                anomaly_marker
            );

            if let Some(time) = &anomaly.latest_time {
                append_line!(&mut output, "  最新时间: {}", time);
            }

            if let Some(vol_ratio) = anomaly.volume_ratio {
                append_line!(&mut output, "  量比: {:.2}", vol_ratio);
            }

            if let Some(vol5) = anomaly.volatility_5 {
                append_line!(&mut output, "  5周期波动率: {:.4}", vol5);
            }

            if let Some(vol20) = anomaly.volatility_20 {
                append_line!(&mut output, "  20周期波动率: {:.4}", vol20);
            }

            append_line!(&mut output, "{}", "-".repeat(40));
        }

        append_line!(&mut output);
        append_line!(&mut output, "📌 说明:");
        append_line!(&mut output, "  - 异常分数 < 0: 异常股票，分数越低越异常");
        append_line!(
            &mut output,
            "  - 异常股票的未来价格变动通常是正常股票的2倍以上"
        );
        append_line!(&mut output, "  - 算法不预测涨跌方向，只检测异常模式");
        append_line!(&mut output);
        append_line!(&mut output, "📊 统计:");
        append_line!(&mut output, "  - 总股票数: {}", result.total_stocks);
        append_line!(
            &mut output,
            "  - 过滤后: {} (过滤掉 {})",
            result.valid_features,
            result.filtered_stocks
        );
        append_line!(&mut output, "  - K线周期: {}分钟", result.config.period);
        append_line!(&mut output, "  - 树数量: {}", result.config.n_estimators);

        output
    }

    /// Render in JSON format.
    fn output_json(&self, result: &AnomalyResult) -> serde_json::Result<String> {
        let mut output = serde_json::to_string_pretty(result)?;
        output.push('\n');
        Ok(output)
    }

    /// Render in CSV format.
    fn output_csv(&self, result: &AnomalyResult) -> String {
        let mut output = String::new();
        append_line!(
            &mut output,
            "rank,code,name,score,is_anomaly,volume_ratio,volatility_5,volatility_20"
        );
        for (i, a) in result.anomalies.iter().enumerate() {
            append_line!(
                &mut output,
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
        output
    }
}
