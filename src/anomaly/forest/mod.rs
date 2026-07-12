//! Isolation Forest implementation for anomaly detection
//!
//! The Isolation Forest algorithm isolates anomalies by randomly selecting
//! a feature and a split value. Anomalies are isolated faster (shorter path
//! length) than normal points.
//!
//! # Algorithm
//!
//! 1. Build `n_estimators` isolation trees
//! 2. Each tree randomly selects features and split values
//! 3. Anomalies have shorter average path lengths
//! 4. Score = 2^(-E[h(x)] / c(n)) - 0.5
//!
//! # References
//!
//! - Liu, Fei Tony, Kai Ming Ting, and Zhi-Hua Zhou. "Isolation forest."
//!   2008 Eighth IEEE International Conference on Data Mining.

pub mod model;
pub mod tree;

#[cfg(test)]
mod tests;

pub use model::{AnomalyScore, IsolationForest, avg_path_length};
