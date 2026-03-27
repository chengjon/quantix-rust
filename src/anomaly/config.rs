//! Configuration for anomaly detection
//!
//! Provides configuration structures for the anomaly detection pipeline.

use serde::{Deserialize, Serialize};

/// Main configuration for anomaly detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyConfig {
    /// Feature extraction configuration
    pub features: FeatureConfig,
    /// Stock filtering configuration
    pub filter: FilterConfig,
    /// Isolation Forest configuration
    pub forest: ForestConfig,
    /// Data source configuration
    pub data: DataConfig,
    /// Output configuration
    pub output: OutputConfig,
}

impl Default for AnomalyConfig {
    fn default() -> Self {
        Self {
            features: FeatureConfig::default(),
            filter: FilterConfig::default(),
            forest: ForestConfig::default(),
            data: DataConfig::default(),
            output: OutputConfig::default(),
        }
    }
}

/// Feature extraction configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureConfig {
    /// Number of historical bars to use for feature calculation
    pub history_to_use: usize,
    /// EOM (Ease of Movement) periods to calculate
    pub eom_periods: Vec<usize>,
    /// Whether to include volume returns
    pub include_volume_returns: bool,
    /// Whether to include log returns
    pub include_log_returns: bool,
    /// Whether to include EOM features
    pub include_eom: bool,
    /// Minimum candles required for feature extraction
    pub min_candles: usize,
}

impl Default for FeatureConfig {
    fn default() -> Self {
        Self {
            history_to_use: 7,
            eom_periods: vec![5, 10, 20],
            include_volume_returns: true,
            include_log_returns: true,
            include_eom: true,
            min_candles: 50,
        }
    }
}

/// Stock filtering configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterConfig {
    /// Minimum average volume (in lots/shou)
    pub min_volume: f64,
    /// Minimum volatility (standard deviation)
    pub min_volatility: f64,
    /// Exclude ST stocks
    pub exclude_st: bool,
    /// Exclude stocks at limit up/down
    pub exclude_limit: bool,
    /// Minimum listing days for new stocks
    pub min_listing_days: usize,
    /// Minimum number of candles required
    pub min_candles: usize,
    /// Maximum stocks to process (0 = no limit)
    pub max_stocks: usize,
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            min_volume: 10000.0,
            min_volatility: 0.03,
            exclude_st: true,
            exclude_limit: true,
            min_listing_days: 60,
            min_candles: 50,
            max_stocks: 0,
        }
    }
}

/// Isolation Forest configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForestConfig {
    /// Number of trees in the forest
    pub n_estimators: usize,
    /// Maximum samples per tree
    pub max_samples: usize,
    /// Random seed for reproducibility
    pub random_state: u64,
    /// Expected proportion of anomalies (for threshold)
    pub contamination: f64,
    /// Number of top anomalies to return
    pub top_n: usize,
}

impl Default for ForestConfig {
    fn default() -> Self {
        Self {
            n_estimators: 100,
            max_samples: 256,
            random_state: 42,
            contamination: 0.1,
            top_n: 20,
        }
    }
}

/// Data source configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataConfig {
    /// K-line period in minutes (1, 5, 15, 30, 60)
    pub kline_period: u32,
    /// Number of K-lines to fetch
    pub kline_limit: usize,
    /// Adjustment type: "qfq" (forward), "hfq" (backward), "none"
    pub adjust_type: String,
    /// Request timeout in seconds
    pub timeout_secs: u64,
    /// Maximum concurrent requests
    pub max_concurrent: usize,
}

impl Default for DataConfig {
    fn default() -> Self {
        Self {
            kline_period: 15,
            kline_limit: 100,
            adjust_type: "qfq".to_string(),
            timeout_secs: 30,
            max_concurrent: 10,
        }
    }
}

/// Output configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Output format: "cli", "json", "csv"
    pub format: String,
    /// Whether to include detailed statistics
    pub verbose: bool,
    /// Whether to output only anomalies (score < 0)
    pub anomalies_only: bool,
    /// Custom output file path (optional)
    pub output_file: Option<String>,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            format: "cli".to_string(),
            verbose: false,
            anomalies_only: false,
            output_file: None,
        }
    }
}

impl AnomalyConfig {
    /// Create a new configuration with custom top_n
    pub fn with_top_n(mut self, top_n: usize) -> Self {
        self.forest.top_n = top_n;
        self
    }

    /// Create a new configuration with custom kline period
    pub fn with_period(mut self, period: u32) -> Self {
        self.data.kline_period = period;
        self
    }

    /// Create a new configuration with custom output format
    pub fn with_format(mut self, format: &str) -> Self {
        self.output.format = format.to_string();
        self
    }

    /// Load configuration from a TOML file
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        toml::from_str(&content).map_err(|e| format!("Failed to parse config: {}", e))
    }

    /// Save configuration to a TOML file
    pub fn to_file(&self, path: &str) -> Result<(), String> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        std::fs::write(path, content)
            .map_err(|e| format!("Failed to write config file: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AnomalyConfig::default();

        assert_eq!(config.features.history_to_use, 7);
        assert_eq!(config.filter.min_volume, 10000.0);
        assert_eq!(config.forest.n_estimators, 100);
        assert_eq!(config.data.kline_period, 15);
        assert_eq!(config.output.format, "cli");
    }

    #[test]
    fn test_config_builders() {
        let config = AnomalyConfig::default()
            .with_top_n(50)
            .with_period(5)
            .with_format("json");

        assert_eq!(config.forest.top_n, 50);
        assert_eq!(config.data.kline_period, 5);
        assert_eq!(config.output.format, "json");
    }
}
