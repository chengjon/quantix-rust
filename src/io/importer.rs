/// 数据导入器
///
/// 支持多种数据格式导入
use crate::core::Result;
use crate::data::models::{AdjustType, Kline};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use serde::Deserialize;
use std::fs::File;
use std::path::Path;

/// 导入格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportFormat {
    CSV,
    JSON,
    Parquet,
}

/// 导入配置
#[derive(Debug, Clone)]
pub struct ImportConfig {
    /// 导入格式
    pub format: ImportFormat,
    /// 是否跳过无效行
    pub skip_invalid: bool,
    /// 批处理大小
    pub batch_size: usize,
    /// 是否进行数据验证
    pub validate: bool,
    /// 日期格式（用于 CSV）
    pub date_format: String,
}

impl Default for ImportConfig {
    fn default() -> Self {
        Self {
            format: ImportFormat::CSV,
            skip_invalid: true,
            batch_size: 1000,
            validate: true,
            date_format: "%Y-%m-%d".to_string(),
        }
    }
}

/// 导入结果
#[derive(Debug, Clone)]
pub struct ImportResult {
    /// 导入的记录数
    pub record_count: usize,
    /// 跳过的记录数
    pub skipped_count: usize,
    /// 错误记录数
    pub error_count: usize,
    /// 导入耗时（毫秒）
    pub duration_ms: u64,
    /// 验证错误列表
    pub validation_errors: Vec<String>,
}

/// 数据导入器
pub struct DataImporter {
    config: ImportConfig,
}

impl DataImporter {
    /// 创建新的导入器
    pub fn new(config: ImportConfig) -> Self {
        Self { config }
    }

    /// 使用默认配置创建
    pub fn with_defaults() -> Self {
        Self::new(ImportConfig::default())
    }

    /// 导入 K线数据
    pub async fn import_klines<P: AsRef<Path>>(&self, input_path: P) -> Result<ImportResult> {
        let start = std::time::Instant::now();
        let path = input_path.as_ref();

        let (klines, skipped, errors) = match self.config.format {
            ImportFormat::CSV => self.import_csv(path)?,
            ImportFormat::JSON => self.import_json(path)?,
            ImportFormat::Parquet => self.import_parquet(path).await?,
        };

        let duration = start.elapsed();
        let record_count = klines.len();

        Ok(ImportResult {
            record_count,
            skipped_count: skipped,
            error_count: errors,
            duration_ms: duration.as_millis() as u64,
            validation_errors: Vec::new(),
        })
    }

    /// 从 CSV 导入
    fn import_csv(&self, path: &Path) -> Result<(Vec<Kline>, usize, usize)> {
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_path(path)
            .map_err(|e| crate::core::QuantixError::Other(format!("打开 CSV 文件失败: {}", e)))?;

        let mut klines = Vec::new();
        let mut skipped = 0;
        let mut errors = 0;

        for result in rdr.deserialize() {
            match result {
                Ok(row) => {
                    let csv_row: CsvKlineRow = row;
                    match self.csv_row_to_kline(&csv_row) {
                        Ok(kline) => klines.push(kline),
                        Err(e) => {
                            if self.config.skip_invalid {
                                skipped += 1;
                            } else {
                                return Err(e);
                            }
                        }
                    }
                }
                Err(e) => {
                    if self.config.skip_invalid {
                        skipped += 1;
                    } else {
                        errors += 1;
                        eprintln!("CSV 解析错误: {}", e);
                    }
                }
            }
        }

        Ok((klines, skipped, errors))
    }

    /// 从 JSON 导入
    fn import_json(&self, path: &Path) -> Result<(Vec<Kline>, usize, usize)> {
        let json_str = std::fs::read_to_string(path)
            .map_err(|e| crate::core::QuantixError::Other(format!("读取 JSON 文件失败: {}", e)))?;

        let klines: Vec<JsonKlineRow> = serde_json::from_str(&json_str)
            .map_err(|e| crate::core::QuantixError::Other(format!("解析 JSON 失败: {}", e)))?;

        let mut result = Vec::new();
        let mut skipped = 0;

        for row in klines {
            match self.json_row_to_kline(&row) {
                Ok(kline) => result.push(kline),
                Err(_) => {
                    if self.config.skip_invalid {
                        skipped += 1;
                    } else {
                        return Err(crate::core::QuantixError::Other(format!(
                            "无效的 JSON 行数据"
                        )));
                    }
                }
            }
        }

        Ok((result, skipped, 0))
    }

    /// 从 Parquet 导入
    async fn import_parquet(&self, path: &Path) -> Result<(Vec<Kline>, usize, usize)> {
        use arrow::array::*;
        use arrow::datatypes::*;
        use arrow::record_batch::RecordBatchReader;
        use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;

        let file = File::open(path).map_err(|e| {
            crate::core::QuantixError::Other(format!("打开 Parquet 文件失败: {}", e))
        })?;

        let mut reader = ParquetRecordBatchReaderBuilder::try_new(file)
            .map_err(|e| {
                crate::core::QuantixError::Other(format!("创建 ParquetReader 失败: {}", e))
            })?
            .build()
            .map_err(|e| {
                crate::core::QuantixError::Other(format!("构建 ParquetReader 失败: {}", e))
            })?;

        let mut klines = Vec::new();

        loop {
            let batch_result = reader.next_batch().map_err(|e| {
                crate::core::QuantixError::Other(format!("读取 Parquet batch 失败: {}", e))
            })?;

            let batch = match batch_result {
                Some(b) => b,
                None => break,
            };

            // 提取数据
            let codes = batch
                .column_by_name("code")
                .ok_or_else(|| crate::core::QuantixError::Other("缺少 code 列".to_string()))?
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or_else(|| crate::core::QuantixError::Other("code 列类型错误".to_string()))?;

            let dates = batch
                .column_by_name("date")
                .ok_or_else(|| crate::core::QuantixError::Other("缺少 date 列".to_string()))?
                .as_any()
                .downcast_ref::<PrimitiveArray<Date32Type>>()
                .ok_or_else(|| crate::core::QuantixError::Other("date 列类型错误".to_string()))?;

            let opens = batch
                .column_by_name("open")
                .ok_or_else(|| crate::core::QuantixError::Other("缺少 open 列".to_string()))?
                .as_any()
                .downcast_ref::<Float64Array>()
                .ok_or_else(|| crate::core::QuantixError::Other("open 列类型错误".to_string()))?;

            let closes = batch
                .column_by_name("close")
                .ok_or_else(|| crate::core::QuantixError::Other("缺少 close 列".to_string()))?
                .as_any()
                .downcast_ref::<Float64Array>()
                .ok_or_else(|| crate::core::QuantixError::Other("close 列类型错误".to_string()))?;

            let volumes = batch
                .column_by_name("volume")
                .ok_or_else(|| crate::core::QuantixError::Other("缺少 volume 列".to_string()))?
                .as_any()
                .downcast_ref::<PrimitiveArray<Int64Type>>()
                .ok_or_else(|| crate::core::QuantixError::Other("volume 列类型错误".to_string()))?;

            // 转换为 Kline
            for i in 0..batch.num_rows() {
                let date_value = dates.value(i);
                let date = NaiveDate::from_ymd_opt(1970, 1, 1)
                    .unwrap()
                    .checked_add_signed(chrono::Duration::days(date_value as i64))
                    .ok_or_else(|| crate::core::QuantixError::Other("日期转换失败".to_string()))?;

                klines.push(Kline {
                    code: codes.value(i).to_string(),
                    date,
                    open: Decimal::from_f64_retain(opens.value(i)).unwrap_or_default(),
                    high: Decimal::from_f64_retain(0.0).unwrap(), // Parquet 可能没有 high/low
                    low: Decimal::from_f64_retain(0.0).unwrap(),
                    close: Decimal::from_f64_retain(closes.value(i)).unwrap_or_default(),
                    volume: volumes.value(i),
                    amount: None,
                    adjust_type: AdjustType::None,
                });
            }
        }

        Ok((klines, 0, 0))
    }

    /// CSV 行转 Kline
    fn csv_row_to_kline(&self, row: &CsvKlineRow) -> Result<Kline> {
        let date = parse_date(&row.date, &self.config.date_format)?;

        Ok(Kline {
            code: row.code.clone(),
            date,
            open: Decimal::from_str(&row.open).map_err(|_| {
                crate::core::QuantixError::Other(format!("无效的 open 值: {}", row.open))
            })?,
            high: Decimal::from_str(&row.high).map_err(|_| {
                crate::core::QuantixError::Other(format!("无效的 high 值: {}", row.high))
            })?,
            low: Decimal::from_str(&row.low).map_err(|_| {
                crate::core::QuantixError::Other(format!("无效的 low 值: {}", row.low))
            })?,
            close: Decimal::from_str(&row.close).map_err(|_| {
                crate::core::QuantixError::Other(format!("无效的 close 值: {}", row.close))
            })?,
            volume: row.volume.parse::<i64>().map_err(|_| {
                crate::core::QuantixError::Other(format!("无效的 volume 值: {}", row.volume))
            })?,
            amount: row
                .amount
                .as_ref()
                .map(|s| {
                    Decimal::from_str(s).map_err(|_| {
                        crate::core::QuantixError::Other(format!("无效的 amount 值: {}", s))
                    })
                })
                .transpose()?,
            adjust_type: AdjustType::None,
        })
    }

    /// JSON 行转 Kline
    fn json_row_to_kline(&self, row: &JsonKlineRow) -> Result<Kline> {
        let date = NaiveDate::parse_from_str(&row.date, "%Y-%m-%d").map_err(|_| {
            crate::core::QuantixError::Other(format!("无效的日期格式: {}", row.date))
        })?;

        Ok(Kline {
            code: row.code.clone(),
            date,
            open: Decimal::from_str(&row.open).map_err(|_| {
                crate::core::QuantixError::Other(format!("无效的 open 值: {}", row.open))
            })?,
            high: row
                .high
                .as_ref()
                .map(|s| Decimal::from_str(s).unwrap_or_default())
                .unwrap_or_default(),
            low: row
                .low
                .as_ref()
                .map(|s| Decimal::from_str(s).unwrap_or_default())
                .unwrap_or_default(),
            close: Decimal::from_str(&row.close).map_err(|_| {
                crate::core::QuantixError::Other(format!("无效的 close 值: {}", row.close))
            })?,
            volume: row.volume,
            amount: None,
            adjust_type: AdjustType::None,
        })
    }
}

fn parse_date(value: &str, format: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(value, format)
        .map_err(|_| crate::core::QuantixError::Other(format!("无效的日期格式: {}", value)))
}

/// CSV K线行格式
#[derive(Debug, Deserialize)]
struct CsvKlineRow {
    code: String,
    date: String,
    open: String,
    high: String,
    low: String,
    close: String,
    volume: String,
    #[serde(default)]
    amount: Option<String>,
    #[serde(default)]
    adjust_type: Option<String>,
}

/// JSON K线行格式
#[derive(Debug, Deserialize)]
struct JsonKlineRow {
    code: String,
    date: String,
    open: String,
    #[serde(default)]
    high: Option<String>,
    #[serde(default)]
    low: Option<String>,
    close: String,
    volume: i64,
    #[serde(default)]
    amount: Option<String>,
    #[serde(default)]
    adjust_type: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_import_config_default() {
        let config = ImportConfig::default();
        assert_eq!(config.format, ImportFormat::CSV);
        assert_eq!(config.skip_invalid, true);
    }

    #[test]
    fn test_importer_creation() {
        let importer = DataImporter::with_defaults();
        assert_eq!(importer.config.format, ImportFormat::CSV);
    }

    #[test]
    fn test_import_csv() {
        let temp_dir = tempdir().unwrap();
        let csv_path = temp_dir.path().join("test.csv");

        // 创建测试 CSV 文件
        let csv_content = "code,date,open,high,low,close,volume\n\
            000001,2024-01-01,10.0,11.0,9.5,10.5,1000000\n\
            000001,2024-01-02,10.5,11.5,10.0,11.0,1100000";
        fs::write(&csv_path, csv_content).unwrap();

        let importer = DataImporter::with_defaults();
        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(importer.import_klines(&csv_path))
            .unwrap();

        assert_eq!(result.record_count, 2);
        assert_eq!(result.skipped_count, 0);
    }

    #[test]
    fn test_import_json() {
        let temp_dir = tempdir().unwrap();
        let json_path = temp_dir.path().join("test.json");

        // 创建测试 JSON 文件
        let json_content = r#"[{"code":"000001","date":"2024-01-01","open":"10.0","close":"10.5","volume":1000000}]"#;
        fs::write(&json_path, json_content).unwrap();

        let config = ImportConfig {
            format: ImportFormat::JSON,
            ..Default::default()
        };
        let importer = DataImporter::new(config);
        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(importer.import_klines(&json_path))
            .unwrap();

        assert_eq!(result.record_count, 1);
    }
}
