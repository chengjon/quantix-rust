use super::*;
use chrono::{SecondsFormat, Utc};
use serde::Deserialize;

use crate::core::{QuantixError, Result};

mod build_command;
mod clipboard;
mod manifest;
mod sources;

#[allow(unused_imports)]
pub use build_command::*;
#[allow(unused_imports)]
pub use clipboard::*;
#[allow(unused_imports)]
pub use manifest::*;
#[allow(unused_imports)]
pub use sources::*;

// ============================================================
// 导入命令
// ============================================================

#[derive(Debug, Clone, Copy)]
struct MarketManifestComparisonOptions<'a> {
    reference_artifact: Option<&'a str>,
    source_of_truth_summary: Option<&'a str>,
    clickhouse: MarketManifestClickHouseComparisonOptions<'a>,
}

#[derive(Debug, Clone, Copy)]
struct MarketManifestClickHouseComparisonOptions<'a> {
    url: Option<&'a str>,
    database: &'a str,
    user: &'a str,
    password: &'a str,
    table: Option<&'a str>,
    dataset_version_column: &'a str,
    symbol_column: &'a str,
    date_column: &'a str,
}

impl MarketManifestClickHouseComparisonOptions<'_> {
    fn is_enabled(&self) -> bool {
        self.url.is_some() || self.table.is_some()
    }
}

#[derive(Debug, Deserialize)]
struct ClickHouseRowCount {
    row_count: u64,
}

#[derive(Debug, Deserialize)]
struct ClickHouseValue {
    value: String,
}

/// 处理导入命令
pub async fn run_import_command(cmd: ImportCommands) -> Result<()> {
    match cmd {
        ImportCommands::FromImage { file, model } => run_import_from_image(&file, &model).await,
        ImportCommands::FromCsv { file } => run_import_from_csv(&file).await,
        ImportCommands::FromExcel { file, sheet } => {
            run_import_from_excel(&file, sheet.as_deref()).await
        }
        ImportCommands::FromClipboard => run_import_from_clipboard().await,
        ImportCommands::FromText { text } => run_import_from_text(&text).await,
        ImportCommands::Resolve { input } => run_import_resolve(&input).await,
        ImportCommands::MarketManifest {
            manifest,
            dataset_version,
            artifact_type,
            schema_version,
            artifact_hash,
            verify_artifact_file,
            comparison_reference_artifact,
            comparison_source_of_truth_summary,
            comparison_clickhouse_url,
            comparison_clickhouse_database,
            comparison_clickhouse_user,
            comparison_clickhouse_password,
            comparison_clickhouse_table,
            comparison_clickhouse_dataset_version_column,
            comparison_clickhouse_symbol_column,
            comparison_clickhouse_date_column,
            regression_report_output,
            evidence_output,
            consumer_build_commit,
            database_target,
        } => {
            run_import_market_manifest(
                &manifest,
                &dataset_version,
                &artifact_type,
                schema_version.as_deref(),
                artifact_hash.as_deref(),
                verify_artifact_file,
                MarketManifestComparisonOptions {
                    reference_artifact: comparison_reference_artifact.as_deref(),
                    source_of_truth_summary: comparison_source_of_truth_summary.as_deref(),
                    clickhouse: MarketManifestClickHouseComparisonOptions {
                        url: comparison_clickhouse_url.as_deref(),
                        database: &comparison_clickhouse_database,
                        user: &comparison_clickhouse_user,
                        password: &comparison_clickhouse_password,
                        table: comparison_clickhouse_table.as_deref(),
                        dataset_version_column: &comparison_clickhouse_dataset_version_column,
                        symbol_column: &comparison_clickhouse_symbol_column,
                        date_column: &comparison_clickhouse_date_column,
                    },
                },
                regression_report_output.as_deref(),
                evidence_output.as_deref(),
                consumer_build_commit.as_deref(),
                &database_target,
            )
            .await
        }
    }
}
