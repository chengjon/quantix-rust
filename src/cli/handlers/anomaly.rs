#![allow(clippy::too_many_arguments)]

use super::*;

// ============================================================================
// 异常检测命令处理
// ============================================================================

/// 处理异常检测命令
pub async fn run_anomaly_command(cmd: AnomalyCommands) -> Result<()> {
    match cmd {
        AnomalyCommands::Run {
            top_n,
            period,
            min_volume,
            min_volatility,
            output,
            n_estimators,
            history,
            mock,
            mock_count,
        } => {
            run_anomaly_detection(
                top_n,
                period,
                min_volume,
                min_volatility,
                output,
                n_estimators,
                history,
                mock,
                mock_count,
            )
            .await
        }
    }
}

/// 运行异常检测
async fn run_anomaly_detection(
    top_n: usize,
    period: u32,
    min_volume: f64,
    min_volatility: f64,
    output: String,
    n_estimators: usize,
    history: usize,
    mock: bool,
    mock_count: usize,
) -> Result<()> {
    use crate::anomaly::{
        AnomalyConfig, AnomalyDetector, DataSource, EastMoneyAnomalySource, FeatureConfig,
        FilterConfig, ForestConfig, MockDataSource,
    };

    println!("🚀 启动异常检测...");
    println!("   K线周期: {}分钟", period);
    println!("   树数量: {}", n_estimators);
    println!("   返回数量: {}", top_n);

    // 构建配置
    let config = AnomalyConfig {
        features: FeatureConfig {
            history_to_use: history,
            ..Default::default()
        },
        filter: FilterConfig {
            min_volume,
            min_volatility,
            ..Default::default()
        },
        forest: ForestConfig {
            n_estimators,
            top_n,
            ..Default::default()
        },
        ..Default::default()
    };

    // 创建数据源
    let data_source: std::sync::Arc<dyn DataSource> = if mock {
        println!("   使用模拟数据: {} 只股票", mock_count);
        std::sync::Arc::new(MockDataSource::new(mock_count))
    } else {
        println!("   使用东方财富 API 获取实时数据");
        std::sync::Arc::new(EastMoneyAnomalySource::new())
    };

    // 创建检测器并运行
    let detector = AnomalyDetector::new(config, data_source);

    match detector.detect().await {
        Ok(result) => {
            // 根据输出格式显示结果
            match output.as_str() {
                "json" => {
                    let json = serde_json::to_string_pretty(&result)
                        .map_err(|e| QuantixError::Other(format!("JSON序列化失败: {}", e)))?;
                    println!("{}", json);
                }
                "csv" => {
                    println!(
                        "rank,code,name,score,is_anomaly,volume_ratio,volatility_5,volatility_20"
                    );
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
                _ => {
                    // CLI 格式
                    detector.output_results(&result);
                }
            }
            Ok(())
        }
        Err(e) => Err(QuantixError::Other(format!("异常检测失败: {}", e))),
    }
}
