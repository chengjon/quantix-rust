//! Feature extraction for anomaly detection
//!
//! Generates feature vectors from OHLCV data for use with Isolation Forest.
//!
//! # Features
//!
//! - **Volume Returns**: `V[t] / V[t-1]` - ratio of consecutive volumes
//! - **Log Returns**: `ln(C[t] / C[t-1])` - logarithmic price returns
//! - **EOM (Ease of Movement)**: Price movement relative to volume
//!
//! Each feature type includes slope, R², and p-value statistics.

use super::config::FeatureConfig;
use crate::anomaly::statistics::linear_regression;
use serde::{Deserialize, Serialize};
use tracing::debug;

/// Feature set for a single stock
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureSet {
    /// Stock code
    pub code: String,
    /// Stock name
    pub name: String,
    /// Feature vector
    pub features: Vec<f64>,
    /// Feature names (for debugging/interpretation)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub feature_names: Vec<String>,
    /// Additional statistics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_ratio: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volatility_5: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volatility_20: Option<f64>,
}

/// OHLCV data for feature extraction
#[derive(Debug, Clone)]
pub struct OHLCVSeries {
    /// Stock code
    pub code: String,
    /// Stock name
    pub name: String,
    /// Candles
    pub candles: Vec<OHLCVCandle>,
}

impl OHLCVSeries {
    /// Create a new series
    pub fn new(code: &str, name: &str) -> Self {
        Self {
            code: code.to_string(),
            name: name.to_string(),
            candles: Vec::new(),
        }
    }

    /// Add a candle
    pub fn add(&mut self, candle: OHLCVCandle) {
        self.candles.push(candle);
    }

    /// Get number of candles
    pub fn len(&self) -> usize {
        self.candles.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.candles.is_empty()
    }

    /// Get close prices
    pub fn closes(&self) -> Vec<f64> {
        self.candles.iter().map(|c| c.close).collect()
    }

    /// Get high prices
    pub fn highs(&self) -> Vec<f64> {
        self.candles.iter().map(|c| c.high).collect()
    }

    /// Get low prices
    pub fn lows(&self) -> Vec<f64> {
        self.candles.iter().map(|c| c.low).collect()
    }

    /// Get volumes
    pub fn volumes(&self) -> Vec<f64> {
        self.candles.iter().map(|c| c.volume).collect()
    }

    /// Calculate log returns
    pub fn log_returns(&self) -> Vec<f64> {
        let closes = self.closes();
        if closes.len() < 2 {
            return Vec::new();
        }

        let mut returns = Vec::with_capacity(closes.len() - 1);
        for i in 1..closes.len() {
            if closes[i - 1] > 0.0 {
                returns.push((closes[i] / closes[i - 1]).ln());
            } else {
                returns.push(0.0);
            }
        }
        returns
    }

    /// Calculate volume returns
    pub fn volume_returns(&self) -> Vec<f64> {
        let volumes = self.volumes();
        if volumes.len() < 2 {
            return Vec::new();
        }

        let mut returns = Vec::with_capacity(volumes.len() - 1);
        for i in 1..volumes.len() {
            if volumes[i - 1] > 0.0 {
                returns.push(volumes[i] / volumes[i - 1]);
            } else {
                returns.push(1.0);
            }
        }
        returns
    }

    /// Calculate volatility over last n periods
    pub fn volatility(&self, n: usize) -> f64 {
        let returns = self.log_returns();
        if returns.len() < n {
            return 0.0;
        }

        let recent: Vec<f64> = returns.iter().rev().take(n).cloned().collect();
        let mean = recent.iter().sum::<f64>() / recent.len() as f64;
        let variance = recent.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / recent.len() as f64;
        variance.sqrt()
    }

    /// Get average volume over last n periods
    pub fn avg_volume(&self, n: usize) -> f64 {
        let volumes = self.volumes();
        if volumes.is_empty() {
            return 0.0;
        }

        volumes.iter().rev().take(n).sum::<f64>() / n.min(volumes.len()) as f64
    }
}

/// Single OHLCV candle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OHLCVCandle {
    /// Timestamp
    pub time: String,
    /// Open price
    pub open: f64,
    /// Close price
    pub close: f64,
    /// High price
    pub high: f64,
    /// Low price
    pub low: f64,
    /// Volume
    pub volume: f64,
    /// Amount (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<f64>,
}

impl OHLCVCandle {
    /// Create a new candle
    pub fn new(time: &str, open: f64, close: f64, high: f64, low: f64, volume: f64) -> Self {
        Self {
            time: time.to_string(),
            open,
            close,
            high,
            low,
            volume,
            amount: None,
        }
    }
}

/// Feature extractor for generating ML features from OHLCV data
pub struct FeatureExtractor {
    config: FeatureConfig,
}

impl FeatureExtractor {
    /// Create a new feature extractor
    pub fn new(config: FeatureConfig) -> Self {
        Self { config }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(FeatureConfig::default())
    }

    /// Extract features from an OHLCV series
    pub fn extract(&self, series: &OHLCVSeries) -> Option<FeatureSet> {
        if series.len() < self.config.min_candles {
            return None;
        }

        let n = self.config.history_to_use;
        let mut features = Vec::new();
        let mut feature_names = Vec::new();

        // 1. Volume returns with slope stats
        if self.config.include_volume_returns {
            let vol_returns = series.volume_returns();
            if vol_returns.len() >= n {
                let recent: Vec<f64> = vol_returns.iter().rev().take(n).cloned().collect();

                // Add raw volume returns
                for (i, &val) in recent.iter().rev().enumerate() {
                    features.push(val);
                    feature_names.push(format!("vol_ret_{}", i));
                }

                // Add slope stats
                if let Some(stats) = linear_regression(&recent) {
                    features.push(stats.slope);
                    features.push(stats.r_squared);
                    features.push(stats.p_value);
                    feature_names.push("vol_ret_slope".to_string());
                    feature_names.push("vol_ret_r2".to_string());
                    feature_names.push("vol_ret_pval".to_string());
                }
            }
        }

        // 2. Log returns
        if self.config.include_log_returns {
            let log_returns = series.log_returns();
            if log_returns.len() >= n {
                let recent: Vec<f64> = log_returns.iter().rev().take(n).cloned().collect();

                for (i, &val) in recent.iter().rev().enumerate() {
                    features.push(val);
                    feature_names.push(format!("log_ret_{}", i));
                }
            }
        }

        // 3. EOM indicators for different periods
        if self.config.include_eom {
            let highs = series.highs();
            let lows = series.lows();
            let volumes = series.volumes();

            for period in &self.config.eom_periods {
                if let Ok(eom_values) = ease_of_movement(&highs, &lows, &volumes, *period) {
                    let recent: Vec<f64> =
                        eom_values.iter().filter(|v| !v.is_nan()).cloned().collect();

                    if recent.len() >= n {
                        let eom_slice: Vec<f64> = recent.iter().rev().take(n).cloned().collect();

                        if let Some(stats) = linear_regression(&eom_slice) {
                            features.push(stats.slope);
                            features.push(stats.r_squared);
                            features.push(stats.p_value);
                            feature_names.push(format!("eom_{}_slope", period));
                            feature_names.push(format!("eom_{}_r2", period));
                            feature_names.push(format!("eom_{}_pval", period));
                        }
                    }
                }
            }
        }

        // Check for NaN values
        if features.iter().any(|f| f.is_nan()) {
            debug!("Skipping {} due to NaN in features", series.code);
            return None;
        }

        if features.is_empty() {
            return None;
        }

        // Calculate additional statistics
        let volume_ratio = if series.len() >= 5 {
            let today_vol = series.candles.last().map(|c| c.volume).unwrap_or(0.0);
            let avg_5d = series.avg_volume(5);
            if avg_5d > 0.0 {
                Some(today_vol / avg_5d)
            } else {
                None
            }
        } else {
            None
        };

        let volatility_5 = if series.len() >= 5 {
            let vol = series.volatility(5);
            if vol > 0.0 { Some(vol) } else { None }
        } else {
            None
        };

        let volatility_20 = if series.len() >= 20 {
            let vol = series.volatility(20);
            if vol > 0.0 { Some(vol) } else { None }
        } else {
            None
        };

        Some(FeatureSet {
            code: series.code.clone(),
            name: series.name.clone(),
            features,
            feature_names,
            volume_ratio,
            volatility_5,
            volatility_20,
        })
    }

    /// Extract features from multiple series
    ///
    /// Returns (feature_matrix, codes, names) for valid series
    pub fn extract_all(
        &self,
        series_list: &[OHLCVSeries],
    ) -> (Vec<Vec<f64>>, Vec<String>, Vec<String>) {
        let mut features = Vec::new();
        let mut codes = Vec::new();
        let mut names = Vec::new();

        for series in series_list {
            if let Some(feature_set) = self.extract(series) {
                features.push(feature_set.features);
                codes.push(feature_set.code);
                names.push(feature_set.name);
            }
        }

        (features, codes, names)
    }
}

impl Default for FeatureExtractor {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// Calculate Ease of Movement indicator
///
/// EOM relates price change to volume, showing how much prices move
/// per unit of volume.
fn ease_of_movement(
    high: &[f64],
    low: &[f64],
    volume: &[f64],
    period: usize,
) -> Result<Vec<f64>, String> {
    if high.len() != low.len() || high.len() != volume.len() {
        return Err("high, low, volume must have same length".to_string());
    }

    let n = high.len();
    if n < 2 {
        return Err("Insufficient data for EOM calculation".to_string());
    }

    let mut emv_values: Vec<f64> = Vec::with_capacity(n);
    emv_values.push(f64::NAN);

    for i in 1..n {
        // Midpoint movement
        let midpoint_today = (high[i] + low[i]) / 2.0;
        let midpoint_yesterday = (high[i - 1] + low[i - 1]) / 2.0;
        let midpoint_move = midpoint_today - midpoint_yesterday;

        // Box ratio (scaled by volume)
        let range = high[i] - low[i];
        if range <= 0.0 || volume[i] <= 0.0 {
            emv_values.push(0.0);
            continue;
        }

        // Scale volume by 1e6 for normalization
        let box_ratio = (volume[i] / 1e6) / range;

        if box_ratio == 0.0 {
            emv_values.push(0.0);
        } else {
            emv_values.push(midpoint_move / box_ratio);
        }
    }

    // Apply SMA smoothing if period > 1
    if period > 1 {
        Ok(smooth_sma(&emv_values, period))
    } else {
        Ok(emv_values)
    }
}

/// Simple Moving Average smoothing
fn smooth_sma(data: &[f64], period: usize) -> Vec<f64> {
    let n = data.len();
    let mut result = vec![f64::NAN; n];

    // Handle edge case: period must to at least 2
    if period < 2 || n < period {
        return result;
    }

    // Use saturating_sub to avoid overflow: start_idx = i + 1 - period
    // When i = period - 1, start_idx = 0
    for i in period - 1..n {
        let start_idx = i + 1 - period; // This avoids the overflow: i+1 >= period always
        let valid_values: Vec<f64> = data[start_idx..=i]
            .iter()
            .filter(|v| !v.is_nan())
            .cloned()
            .collect();

        if !valid_values.is_empty() {
            result[i] = valid_values.iter().sum::<f64>() / valid_values.len() as f64;
        }
    }

    result
}

#[cfg(test)]
mod tests;
