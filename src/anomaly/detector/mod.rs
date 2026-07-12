//! Anomaly detection service
//!
//! Integrates all components to provide end-to-end anomaly detection:
//! - Data fetching from East Money API
//! - Stock filtering
//! - Feature extraction
//! - Isolation Forest training
//! - Result rendering

pub mod service;
pub mod types;

#[cfg(test)]
mod tests;

pub use service::AnomalyDetector;
pub use types::{AnomalyResult, DataSource, MockDataSource};
