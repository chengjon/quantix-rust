//! Data export command plus the low-level CSV/Parquet writers and the
//! export-format validator.

use crate::cli::handlers::shared_support::create_clickhouse_client;
use crate::core::{QuantixError, Result};
use crate::data::models::Kline;
use crate::io::{DataExporter, ExportConfig, ExportFormat};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::Path;

/// 导出数据
pub(crate) async fn export_data(code: String, format: String, output: String) -> Result<()> {
    validate_data_export_format(&format)?;

    println!("📤 导出数据");
    println!("  代码: {}", code);
    println!("  格式: {}", format);
    println!("  输出: {}", output);

    // 创建输出目录
    fs::create_dir_all(&output)
        .map_err(|e| QuantixError::Other(format!("创建输出目录失败: {}", e)))?;

    // 连接 ClickHouse
    let client = create_clickhouse_client().await?;

    // 查询数据
    let klines = client
        .get_kline_data(
            &code,
            "1d",
            None,
            None,
            Some(10000), // 导出时使用较大的限制
        )
        .await?;

    if klines.is_empty() {
        println!("⚠️  未找到数据");
        return Ok(());
    }

    let output_path = Path::new(&output).join(format!("{}.{}", code, format));
    let progress = ProgressBar::new(3);
    let template = ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {msg}")
        .unwrap_or_else(|_| ProgressStyle::default_bar());
    progress.set_style(template);

    progress.set_message("准备导出...");

    match format.as_str() {
        "csv" => {
            progress.set_message("写入 CSV...");
            progress.inc(1);

            export_klines_to_file(&klines, "csv", &output_path).await?;

            progress.inc(1);
            progress.finish_with_message("CSV 导出完成");
        }
        "parquet" => {
            progress.set_message("写入 Parquet...");
            progress.inc(1);

            export_klines_to_file(&klines, "parquet", &output_path).await?;

            progress.inc(1);
            progress.finish_with_message("Parquet 导出完成");
        }
        _ => return unsupported_data_export_format(&format),
    }

    println!("✅ 数据已导出到: {}", output_path.display());
    Ok(())
}

pub(crate) async fn export_klines_to_file<P: AsRef<Path>>(
    klines: &[Kline],
    format: &str,
    output_path: P,
) -> Result<()> {
    match format {
        "csv" => {
            let mut wtr = csv::Writer::from_path(output_path.as_ref())
                .map_err(|e| QuantixError::Other(format!("创建 CSV 文件失败: {}", e)))?;

            wtr.write_record(["date", "open", "high", "low", "close", "volume"])
                .map_err(|e| QuantixError::Other(format!("写入 CSV 头失败: {}", e)))?;

            for kline in klines {
                wtr.write_record(&[
                    kline.date.to_string(),
                    kline.open.to_string(),
                    kline.high.to_string(),
                    kline.low.to_string(),
                    kline.close.to_string(),
                    kline.volume.to_string(),
                ])
                .map_err(|e| QuantixError::Other(format!("写入 CSV 数据失败: {}", e)))?;
            }

            wtr.flush()
                .map_err(|e| QuantixError::Other(format!("刷新 CSV 失败: {}", e)))?;
            Ok(())
        }
        "parquet" => {
            let exporter = DataExporter::new(ExportConfig {
                format: ExportFormat::Parquet,
                ..Default::default()
            });
            exporter.export_klines(klines, output_path).await?;
            Ok(())
        }
        _ => unsupported_data_export_format(format),
    }
}

fn validate_data_export_format(format: &str) -> Result<()> {
    match format {
        "csv" | "parquet" => Ok(()),
        _ => unsupported_data_export_format(format),
    }
}

fn unsupported_data_export_format(format: &str) -> Result<()> {
    Err(QuantixError::Unsupported(format!(
        "data export format 不支持: {format}；支持: csv, parquet"
    )))
}
