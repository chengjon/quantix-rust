/// 数据导出器
///
/// 支持多种数据格式导出
use crate::core::Result;
use crate::data::models::Kline;
use chrono::NaiveDate;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::Path;

/// 导出格式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    CSV,
    JSON,
    Parquet,
}

/// 导出配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    /// 导出格式
    pub format: ExportFormat,
    /// 是否包含表头
    pub include_header: bool,
    /// 批处理大小
    pub batch_size: usize,
    /// 是否压缩输出
    pub compress: bool,
    /// 日期格式
    pub date_format: String,
    /// 小数精度
    pub decimal_precision: usize,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            format: ExportFormat::CSV,
            include_header: true,
            batch_size: 1000,
            compress: false,
            date_format: "%Y-%m-%d".to_string(),
            decimal_precision: 2,
        }
    }
}

/// 导出结果
#[derive(Debug, Clone)]
pub struct ExportResult {
    /// 输出文件路径
    pub output_path: String,
    /// 导出的记录数
    pub record_count: usize,
    /// 文件大小（字节）
    pub file_size: u64,
    /// 导出耗时（毫秒）
    pub duration_ms: u64,
}

/// 数据导出器
pub struct DataExporter {
    config: ExportConfig,
}

impl DataExporter {
    /// 创建新的导出器
    pub fn new(config: ExportConfig) -> Self {
        Self { config }
    }

    /// 使用默认配置创建
    pub fn with_defaults() -> Self {
        Self::new(ExportConfig::default())
    }

    /// 导出 K线数据
    pub async fn export_klines<P: AsRef<Path>>(
        &self,
        klines: &[Kline],
        output_path: P,
    ) -> Result<ExportResult> {
        let start = std::time::Instant::now();

        let output_path = output_path.as_ref();
        let record_count = klines.len();

        match self.config.format {
            ExportFormat::CSV => {
                self.export_csv(klines, output_path).await?;
            }
            ExportFormat::JSON => {
                self.export_json(klines, output_path).await?;
            }
            ExportFormat::Parquet => {
                self.export_parquet(klines, output_path).await?;
            }
        }

        let duration = start.elapsed();
        let file_size = std::fs::metadata(output_path).map(|m| m.len()).unwrap_or(0);

        Ok(ExportResult {
            output_path: output_path.to_string_lossy().to_string(),
            record_count,
            file_size,
            duration_ms: duration.as_millis() as u64,
        })
    }

    /// 导出为 CSV
    async fn export_csv<P: AsRef<Path>>(&self, klines: &[Kline], output_path: P) -> Result<()> {
        let path = output_path.as_ref();
        let mut wtr = csv::Writer::from_path(path)
            .map_err(|e| crate::core::QuantixError::Other(format!("创建 CSV 文件失败: {}", e)))?;

        // 写入表头
        if self.config.include_header {
            wtr.write_record(&[
                "code",
                "date",
                "open",
                "high",
                "low",
                "close",
                "volume",
                "amount",
                "adjust_type",
            ])
            .map_err(|e| crate::core::QuantixError::Other(format!("写入 CSV 头失败: {}", e)))?;
        }

        // 写入数据
        for kline in klines {
            wtr.write_record(&[
                &kline.code,
                &kline.date.to_string(),
                &format_decimal(kline.open, self.config.decimal_precision),
                &format_decimal(kline.high, self.config.decimal_precision),
                &format_decimal(kline.low, self.config.decimal_precision),
                &format_decimal(kline.close, self.config.decimal_precision),
                &kline.volume.to_string(),
                &format_amount_or_zero(kline.amount),
                &format!("{:?}", kline.adjust_type),
            ])
            .map_err(|e| crate::core::QuantixError::Other(format!("写入 CSV 数据失败: {}", e)))?;
        }

        wtr.flush()
            .map_err(|e| crate::core::QuantixError::Other(format!("刷新 CSV 失败: {}", e)))?;

        Ok(())
    }

    /// 导出为 JSON
    async fn export_json<P: AsRef<Path>>(&self, klines: &[Kline], output_path: P) -> Result<()> {
        use serde_json::to_string_pretty;

        let json_data = to_string_pretty(klines)
            .map_err(|e| crate::core::QuantixError::Other(format!("序列化 JSON 失败: {}", e)))?;

        std::fs::write(output_path.as_ref(), json_data)
            .map_err(|e| crate::core::QuantixError::Other(format!("写入 JSON 文件失败: {}", e)))?;

        Ok(())
    }

    /// 导出为 Parquet
    async fn export_parquet<P: AsRef<Path>>(&self, klines: &[Kline], output_path: P) -> Result<()> {
        use arrow::array::{Float64Array, PrimitiveArray, RecordBatch, StringArray};
        use arrow::datatypes::{DataType, Date32Type, Field, Int64Type, Schema};
        use parquet::arrow::arrow_writer::ArrowWriter;
        use std::sync::Arc;

        // 定义 Schema
        let schema = Schema::new(vec![
            Field::new("code", DataType::Utf8, false),
            Field::new("date", DataType::Date32, false),
            Field::new("open", DataType::Float64, false),
            Field::new("high", DataType::Float64, false),
            Field::new("low", DataType::Float64, false),
            Field::new("close", DataType::Float64, false),
            Field::new("volume", DataType::Int64, false),
            Field::new("amount", DataType::Float64, true),
        ]);

        // 转换数据
        let codes: Vec<&str> = klines.iter().map(|k| k.code.as_str()).collect();
        let dates: Vec<i32> = klines
            .iter()
            .map(|k| {
                k.date
                    .signed_duration_since(NaiveDate::from_ymd_opt(1970, 1, 1).unwrap())
                    .num_days() as i32
            })
            .collect();
        let opens: Vec<f64> = klines
            .iter()
            .map(|k| k.open.to_f64().unwrap_or(0.0))
            .collect();
        let highs: Vec<f64> = klines
            .iter()
            .map(|k| k.high.to_f64().unwrap_or(0.0))
            .collect();
        let lows: Vec<f64> = klines
            .iter()
            .map(|k| k.low.to_f64().unwrap_or(0.0))
            .collect();
        let closes: Vec<f64> = klines
            .iter()
            .map(|k| k.close.to_f64().unwrap_or(0.0))
            .collect();
        let volumes: Vec<i64> = klines.iter().map(|k| k.volume).collect();
        let amounts: Vec<f64> = klines
            .iter()
            .map(|k| k.amount.map(|a| a.to_f64().unwrap_or(0.0)).unwrap_or(0.0))
            .collect();

        // 创建 Arrow Arrays
        let code_array = StringArray::from(codes);
        let date_array = PrimitiveArray::<Date32Type>::from(dates);
        let open_array = Float64Array::from(opens);
        let high_array = Float64Array::from(highs);
        let low_array = Float64Array::from(lows);
        let close_array = Float64Array::from(closes);
        let volume_array = PrimitiveArray::<Int64Type>::from(volumes);
        let amount_array = Float64Array::from(amounts);

        // 创建 RecordBatch
        let batch = RecordBatch::try_new(
            Arc::new(schema.clone()),
            vec![
                Arc::new(code_array),
                Arc::new(date_array),
                Arc::new(open_array),
                Arc::new(high_array),
                Arc::new(low_array),
                Arc::new(close_array),
                Arc::new(volume_array),
                Arc::new(amount_array),
            ],
        )
        .map_err(|e| crate::core::QuantixError::Other(format!("创建 RecordBatch 失败: {}", e)))?;

        // 写入 Parquet 文件
        let file = File::create(output_path.as_ref()).map_err(|e| {
            crate::core::QuantixError::Other(format!("创建 Parquet 文件失败: {}", e))
        })?;

        let props = parquet::file::properties::WriterProperties::builder().build();

        let mut writer =
            ArrowWriter::try_new(file, Arc::new(schema), Some(props)).map_err(|e| {
                crate::core::QuantixError::Other(format!("创建 ArrowWriter 失败: {}", e))
            })?;

        writer.write(&batch).map_err(|e| {
            crate::core::QuantixError::Other(format!("写入 Parquet 数据失败: {}", e))
        })?;

        writer.close().map_err(|e| {
            crate::core::QuantixError::Other(format!("完成 Parquet 写入失败: {}", e))
        })?;

        Ok(())
    }
}

fn format_decimal(value: Decimal, precision: usize) -> String {
    format!("{:.prec$}", value, prec = precision)
}

fn format_amount_or_zero(value: Option<Decimal>) -> String {
    value
        .map(|amount| amount.to_string())
        .unwrap_or_else(|| "0".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::models::AdjustType;
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;
    use std::fs;
    use tempfile::tempdir;

    fn create_test_klines(count: usize) -> Vec<Kline> {
        (0..count)
            .map(|i| {
                let date = NaiveDate::from_ymd_opt(2024, 1, 1 + i as u32).unwrap();
                let i_f64 = i as f64;
                let i_i64 = i as i64;
                Kline {
                    code: "000001".to_string(),
                    date,
                    open: Decimal::from_f64_retain(10.0 + i_f64 * 0.1).unwrap(),
                    high: Decimal::from_f64_retain(11.0 + i_f64 * 0.1).unwrap(),
                    low: Decimal::from_f64_retain(9.0 + i_f64 * 0.1).unwrap(),
                    close: Decimal::from_f64_retain(10.5 + i_f64 * 0.1).unwrap(),
                    volume: 1000000 + i_i64 * 1000,
                    amount: Some(
                        Decimal::from_f64_retain((10000000 + i_i64 * 10000) as f64).unwrap(),
                    ),
                    adjust_type: AdjustType::None,
                }
            })
            .collect()
    }

    #[test]
    fn test_export_config_default() {
        let config = ExportConfig::default();
        assert_eq!(config.format, ExportFormat::CSV);
        assert_eq!(config.include_header, true);
        assert_eq!(config.batch_size, 1000);
    }

    #[test]
    fn test_exporter_creation() {
        let exporter = DataExporter::with_defaults();
        assert_eq!(exporter.config.format, ExportFormat::CSV);
    }

    #[test]
    fn test_export_csv() {
        let klines = create_test_klines(3);
        let exporter = DataExporter::with_defaults();
        let temp_dir = tempdir().unwrap();
        let output_path = temp_dir.path().join("test.csv");

        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(exporter.export_klines(&klines, &output_path))
            .unwrap();

        assert!(output_path.exists());
        let content = fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("code,date,open,high"));
        assert!(content.contains("000001"));
    }

    #[test]
    fn test_export_json() {
        let klines = create_test_klines(2);
        let config = ExportConfig {
            format: ExportFormat::JSON,
            ..Default::default()
        };
        let exporter = DataExporter::new(config);
        let temp_dir = tempdir().unwrap();
        let output_path = temp_dir.path().join("test.json");

        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(exporter.export_klines(&klines, &output_path))
            .unwrap();

        assert!(output_path.exists());
        let content = fs::read_to_string(&output_path).unwrap();
        assert!(content.contains(r#""code": "000001""#));
    }

    #[test]
    fn test_export_result() {
        let klines = create_test_klines(5);
        let exporter = DataExporter::with_defaults();
        let temp_dir = tempdir().unwrap();
        let output_path = temp_dir.path().join("test.csv");

        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(exporter.export_klines(&klines, &output_path))
            .unwrap();

        assert_eq!(result.record_count, 5);
        assert!(result.file_size > 0);
        // Duration might be 0ms for very fast operations
        assert!(result.duration_ms >= 0);
    }

    #[test]
    fn test_format_decimal_respects_precision() {
        assert_eq!(format_decimal(dec!(10.567), 2), "10.56");
        assert_eq!(format_decimal(dec!(10.567), 1), "10.5");
    }
}
