//! Metrics collection and export module
//!
//! Provides metrics collection and export capabilities:
//! - Prometheus format export
//! - JSON format export
//! - Custom metric registration
#![allow(clippy::collapsible_if)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Metric value types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MetricValue {
    Counter(i64),
    Gauge(f64),
    Histogram(HistogramValue),
}

/// Histogram metric value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramValue {
    pub count: u64,
    pub sum: f64,
    pub buckets: Vec<(f64, u64)>,
}

impl HistogramValue {
    pub fn new() -> Self {
        // Default Prometheus-style buckets
        let thresholds = vec![
            0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
        ];
        Self {
            count: 0,
            sum: 0.0,
            buckets: thresholds.into_iter().map(|t| (t, 0)).collect(),
        }
    }

    pub fn observe(&mut self, value: f64) {
        self.count += 1;
        self.sum += value;
        for (threshold, count) in &mut self.buckets {
            if value <= *threshold {
                *count += 1;
            }
        }
    }
}

impl Default for HistogramValue {
    fn default() -> Self {
        Self::new()
    }
}

/// A single metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    /// Metric name
    pub name: String,
    /// Help text describing the metric
    pub help: String,
    /// Metric type
    pub metric_type: MetricType,
    /// Labels attached to this metric
    pub labels: HashMap<String, String>,
    /// Current value
    pub value: MetricValue,
    /// When this metric was last updated
    pub updated_at: DateTime<Utc>,
}

/// Metric type
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
}

/// Metrics collector for gathering and storing metrics
#[derive(Debug)]
pub struct MetricsCollector {
    metrics: Arc<RwLock<HashMap<String, Metric>>>,
    prefix: String,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            prefix: prefix.into(),
        }
    }

    /// Create with default quantix prefix
    pub fn with_default_prefix() -> Self {
        Self::new("quantix")
    }

    /// Register a counter metric
    pub fn register_counter(
        &self,
        name: &str,
        help: &str,
        labels: HashMap<String, String>,
    ) -> String {
        let full_name = format!("{}_{}", self.prefix, name);
        let metric = Metric {
            name: full_name.clone(),
            help: help.to_string(),
            metric_type: MetricType::Counter,
            labels,
            value: MetricValue::Counter(0),
            updated_at: Utc::now(),
        };

        let mut metrics = self.metrics.write().unwrap();
        metrics.insert(full_name.clone(), metric);
        full_name
    }

    /// Register a gauge metric
    pub fn register_gauge(
        &self,
        name: &str,
        help: &str,
        labels: HashMap<String, String>,
    ) -> String {
        let full_name = format!("{}_{}", self.prefix, name);
        let metric = Metric {
            name: full_name.clone(),
            help: help.to_string(),
            metric_type: MetricType::Gauge,
            labels,
            value: MetricValue::Gauge(0.0),
            updated_at: Utc::now(),
        };

        let mut metrics = self.metrics.write().unwrap();
        metrics.insert(full_name.clone(), metric);
        full_name
    }

    /// Register a histogram metric
    pub fn register_histogram(
        &self,
        name: &str,
        help: &str,
        labels: HashMap<String, String>,
    ) -> String {
        let full_name = format!("{}_{}", self.prefix, name);
        let metric = Metric {
            name: full_name.clone(),
            help: help.to_string(),
            metric_type: MetricType::Histogram,
            labels,
            value: MetricValue::Histogram(HistogramValue::new()),
            updated_at: Utc::now(),
        };

        let mut metrics = self.metrics.write().unwrap();
        metrics.insert(full_name.clone(), metric);
        full_name
    }

    /// Increment a counter
    pub fn increment_counter(&self, name: &str, delta: i64) {
        let mut metrics = self.metrics.write().unwrap();
        if let Some(metric) = metrics.get_mut(name) {
            if let MetricValue::Counter(ref mut value) = metric.value {
                *value += delta;
                metric.updated_at = Utc::now();
            }
        }
    }

    /// Set a gauge value
    pub fn set_gauge(&self, name: &str, value: f64) {
        let mut metrics = self.metrics.write().unwrap();
        if let Some(metric) = metrics.get_mut(name) {
            if let MetricValue::Gauge(ref mut current) = metric.value {
                *current = value;
                metric.updated_at = Utc::now();
            }
        }
    }

    /// Observe a histogram value
    pub fn observe_histogram(&self, name: &str, value: f64) {
        let mut metrics = self.metrics.write().unwrap();
        if let Some(metric) = metrics.get_mut(name) {
            if let MetricValue::Histogram(ref mut hist) = metric.value {
                hist.observe(value);
                metric.updated_at = Utc::now();
            }
        }
    }

    /// Get all metrics
    pub fn get_all(&self) -> Vec<Metric> {
        let metrics = self.metrics.read().unwrap();
        metrics.values().cloned().collect()
    }

    /// Get a specific metric
    pub fn get(&self, name: &str) -> Option<Metric> {
        let metrics = self.metrics.read().unwrap();
        metrics.get(name).cloned()
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::with_default_prefix()
    }
}

impl Clone for MetricsCollector {
    fn clone(&self) -> Self {
        Self {
            metrics: Arc::clone(&self.metrics),
            prefix: self.prefix.clone(),
        }
    }
}

/// Export format for metrics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricsFormat {
    Prometheus,
    Json,
}

/// Metrics exporter for outputting metrics in various formats
pub struct MetricsExporter {
    collector: MetricsCollector,
}

impl MetricsExporter {
    pub fn new(collector: MetricsCollector) -> Self {
        Self { collector }
    }

    /// Export metrics in the specified format
    pub fn export(&self, format: MetricsFormat) -> String {
        match format {
            MetricsFormat::Prometheus => self.export_prometheus(),
            MetricsFormat::Json => self.export_json(),
        }
    }

    /// Export in Prometheus text format
    pub fn export_prometheus(&self) -> String {
        let metrics = self.collector.get_all();
        let mut output = String::new();

        for metric in metrics {
            // Help line
            output.push_str(&format!("# HELP {} {}\n", metric.name, metric.help));

            // Type line
            let type_str = match metric.metric_type {
                MetricType::Counter => "counter",
                MetricType::Gauge => "gauge",
                MetricType::Histogram => "histogram",
            };
            output.push_str(&format!("# TYPE {} {}\n", metric.name, type_str));

            // Value line(s)
            let labels_str = if metric.labels.is_empty() {
                String::new()
            } else {
                let pairs: Vec<String> = metric
                    .labels
                    .iter()
                    .map(|(k, v)| format!("{}=\"{}\"", k, v))
                    .collect();
                format!("{{{}}}", pairs.join(","))
            };

            match &metric.value {
                MetricValue::Counter(v) => {
                    output.push_str(&format!("{}{} {}\n", metric.name, labels_str, v));
                }
                MetricValue::Gauge(v) => {
                    output.push_str(&format!("{}{} {}\n", metric.name, labels_str, v));
                }
                MetricValue::Histogram(hist) => {
                    for (threshold, count) in &hist.buckets {
                        output.push_str(&format!(
                            "{}_bucket{{le=\"{}\"{}}} {}\n",
                            metric.name,
                            threshold,
                            if labels_str.is_empty() {
                                String::new()
                            } else {
                                format!(",{}", &labels_str[1..labels_str.len() - 1])
                            },
                            count
                        ));
                    }
                    output.push_str(&format!(
                        "{}_bucket{{le=\"+Inf\"{}}} {}\n",
                        metric.name,
                        if labels_str.is_empty() {
                            String::new()
                        } else {
                            format!(",{}", &labels_str[1..labels_str.len() - 1])
                        },
                        hist.count
                    ));
                    output.push_str(&format!("{}_sum{} {}\n", metric.name, labels_str, hist.sum));
                    output.push_str(&format!(
                        "{}_count{} {}\n",
                        metric.name, labels_str, hist.count
                    ));
                }
            }
            output.push('\n');
        }

        output
    }

    /// Export as JSON
    pub fn export_json(&self) -> String {
        let metrics = self.collector.get_all();
        serde_json::to_string_pretty(&metrics).unwrap_or_else(|_| "[]".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter_operations() {
        let collector = MetricsCollector::with_default_prefix();
        let name = collector.register_counter("requests_total", "Total requests", HashMap::new());

        collector.increment_counter(&name, 1);
        collector.increment_counter(&name, 5);

        let metric = collector.get(&name).unwrap();
        match metric.value {
            MetricValue::Counter(v) => assert_eq!(v, 6),
            _ => panic!("Expected counter"),
        }
    }

    #[test]
    fn test_gauge_operations() {
        let collector = MetricsCollector::with_default_prefix();
        let name = collector.register_gauge("temperature", "Current temperature", HashMap::new());

        collector.set_gauge(&name, 25.5);

        let metric = collector.get(&name).unwrap();
        match metric.value {
            MetricValue::Gauge(v) => assert!((v - 25.5).abs() < 0.001),
            _ => panic!("Expected gauge"),
        }
    }

    #[test]
    fn test_histogram_operations() {
        let collector = MetricsCollector::with_default_prefix();
        let name = collector.register_histogram(
            "request_duration",
            "Request duration in seconds",
            HashMap::new(),
        );

        collector.observe_histogram(&name, 0.1);
        collector.observe_histogram(&name, 0.5);
        collector.observe_histogram(&name, 2.0);

        let metric = collector.get(&name).unwrap();
        match metric.value {
            MetricValue::Histogram(hist) => {
                assert_eq!(hist.count, 3);
                assert!((hist.sum - 2.6).abs() < 0.001);
            }
            _ => panic!("Expected histogram"),
        }
    }

    #[test]
    fn test_prometheus_export() {
        let collector = MetricsCollector::with_default_prefix();
        collector.register_counter("test_counter", "A test counter", HashMap::new());

        let exporter = MetricsExporter::new(collector);
        let output = exporter.export(MetricsFormat::Prometheus);

        assert!(output.contains("# HELP quantix_test_counter"));
        assert!(output.contains("# TYPE quantix_test_counter counter"));
    }

    #[test]
    fn test_json_export() {
        let collector = MetricsCollector::with_default_prefix();
        collector.register_counter("test_counter", "A test counter", HashMap::new());

        let exporter = MetricsExporter::new(collector);
        let output = exporter.export(MetricsFormat::Json);

        assert!(output.contains("\"name\": \"quantix_test_counter\""));
        assert!(output.contains("\"metric_type\": \"counter\""));
    }
}
