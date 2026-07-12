//! Feature-extraction data types: [`FeatureSet`], [`OHLCVSeries`], [`OHLCVCandle`].

use serde::{Deserialize, Serialize};

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
