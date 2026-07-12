use super::*;

use crate::anomaly::config::{AnomalyConfig, OutputConfig};
use crate::anomaly::detector::types::ResultConfig;
use crate::anomaly::forest::AnomalyScore;

fn detector_with_output(format: &str) -> AnomalyDetector {
    let config = AnomalyConfig {
        output: OutputConfig {
            format: format.to_string(),
            ..Default::default()
        },
        ..Default::default()
    };
    let data_source = std::sync::Arc::new(MockDataSource::new(1));
    AnomalyDetector::new(config, data_source)
}

fn sample_result() -> AnomalyResult {
    AnomalyResult {
        timestamp: "2026-05-24T00:00:00Z".to_string(),
        config: ResultConfig {
            period: 15,
            n_estimators: 100,
            history_to_use: 60,
            top_n: 1,
        },
        total_stocks: 1,
        filtered_stocks: 0,
        valid_features: 1,
        anomalies: vec![AnomalyScore {
            code: "000001".to_string(),
            name: "平安银行".to_string(),
            score: -0.42,
            is_anomaly: true,
            index: 0,
            volume_ratio: Some(2.5),
            volatility_5: Some(0.1234),
            volatility_20: Some(0.5678),
            latest_time: Some("2026-05-24 09:30:00".to_string()),
        }],
    }
}

#[test]
fn output_results_renders_cli_text_without_writing_stdio() {
    let detector = detector_with_output("cli");
    let output = detector.output_results(&sample_result()).unwrap();

    assert!(output.contains("TOP 1 ANOMALOUS STOCKS"));
    assert!(output.contains("000001 平安银行"));
    assert!(output.contains("异常分数: -0.4200"));
}

#[test]
fn output_results_renders_json_text() {
    let detector = detector_with_output("json");
    let output = detector.output_results(&sample_result()).unwrap();

    assert!(output.contains("\"total_stocks\": 1"));
    assert!(output.ends_with('\n'));
}

#[test]
fn output_results_renders_csv_text() {
    let detector = detector_with_output("csv");
    let output = detector.output_results(&sample_result()).unwrap();

    assert!(
        output
            .starts_with("rank,code,name,score,is_anomaly,volume_ratio,volatility_5,volatility_20")
    );
    assert!(output.contains("1,000001,平安银行,-0.420000,true,2.5000,0.123400,0.567800"));
}

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
