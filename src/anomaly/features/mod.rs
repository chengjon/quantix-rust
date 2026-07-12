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

pub mod extractor;
pub mod series;

#[cfg(test)]
mod tests;

pub use extractor::FeatureExtractor;
pub use series::{FeatureSet, OHLCVCandle, OHLCVSeries};
