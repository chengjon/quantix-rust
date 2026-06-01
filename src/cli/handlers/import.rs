use super::*;
use chrono::{SecondsFormat, Utc};
use serde::Deserialize;

use crate::core::{CliRuntime, QuantixError, Result};

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

async fn run_import_market_manifest(
    manifest: &str,
    dataset_version: &str,
    artifact_type: &str,
    schema_version: Option<&str>,
    artifact_hash: Option<&str>,
    verify_artifact_file: bool,
    comparison_options: MarketManifestComparisonOptions<'_>,
    regression_report_output: Option<&str>,
    evidence_output: Option<&str>,
    consumer_build_commit: Option<&str>,
    database_target: &str,
) -> Result<()> {
    let resolved = resolve_import_market_manifest_artifact_with_options(
        manifest,
        dataset_version,
        artifact_type,
        schema_version,
        artifact_hash,
        verify_artifact_file,
    )?;

    write_import_market_manifest_report_outputs(
        &resolved,
        manifest,
        dataset_version,
        artifact_type,
        schema_version,
        artifact_hash,
        verify_artifact_file,
        comparison_options,
        regression_report_output,
        evidence_output,
        consumer_build_commit,
        database_target,
    )
    .await?;

    let output = resolved.to_pretty_json().map_err(QuantixError::Other)?;
    println!("{output}");

    Ok(())
}

pub fn resolve_import_market_manifest_artifact(
    manifest: &str,
    dataset_version: &str,
    artifact_type: &str,
    schema_version: Option<&str>,
    artifact_hash: Option<&str>,
) -> Result<crate::miniqmt_market::ResolvedMarketArtifact> {
    resolve_import_market_manifest_artifact_with_options(
        manifest,
        dataset_version,
        artifact_type,
        schema_version,
        artifact_hash,
        false,
    )
}

pub fn resolve_import_market_manifest_artifact_with_options(
    manifest: &str,
    dataset_version: &str,
    artifact_type: &str,
    schema_version: Option<&str>,
    artifact_hash: Option<&str>,
    verify_artifact_file: bool,
) -> Result<crate::miniqmt_market::ResolvedMarketArtifact> {
    let mut request =
        crate::miniqmt_market::MarketArtifactRequest::new(dataset_version, artifact_type);
    if let Some(schema_version) = schema_version {
        request = request.require_schema_version(schema_version);
    }
    if let Some(artifact_hash) = artifact_hash {
        request = request.require_artifact_hash(artifact_hash);
    }

    let mut resolved = request
        .resolve_from_path(manifest)
        .map_err(QuantixError::DataParse)?;

    if verify_artifact_file {
        resolved
            .verify_artifact_file_hash(manifest)
            .map_err(QuantixError::DataParse)?;
    }

    Ok(resolved)
}

async fn write_import_market_manifest_report_outputs(
    resolved: &crate::miniqmt_market::ResolvedMarketArtifact,
    manifest: &str,
    dataset_version: &str,
    artifact_type: &str,
    schema_version: Option<&str>,
    artifact_hash: Option<&str>,
    verify_artifact_file: bool,
    comparison_options: MarketManifestComparisonOptions<'_>,
    regression_report_output: Option<&str>,
    evidence_output: Option<&str>,
    consumer_build_commit: Option<&str>,
    database_target: &str,
) -> Result<()> {
    if regression_report_output.is_none() && evidence_output.is_none() {
        return Ok(());
    }

    let run_at = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    let consumer_build_commit = consumer_build_commit.unwrap_or("unknown");
    let comparison_sources = usize::from(comparison_options.reference_artifact.is_some())
        + usize::from(comparison_options.source_of_truth_summary.is_some())
        + usize::from(comparison_options.clickhouse.is_enabled());
    if comparison_sources > 1 {
        return Err(QuantixError::DataParse(
            "--comparison-reference-artifact, --comparison-source-of-truth-summary, and --comparison-clickhouse-* options are mutually exclusive"
                .to_string(),
        ));
    }
    let comparison = match (
        comparison_options.reference_artifact,
        comparison_options.source_of_truth_summary,
        comparison_options.clickhouse.is_enabled(),
    ) {
        (Some(_), Some(_), _) | (Some(_), _, true) | (_, Some(_), true) => {
            return Err(QuantixError::DataParse(
                "--comparison-reference-artifact, --comparison-source-of-truth-summary, and --comparison-clickhouse-* options are mutually exclusive"
                    .to_string(),
            ));
        }
        (Some(reference_artifact), None, false) => {
            if !verify_artifact_file {
                return Err(QuantixError::DataParse(
                    "--comparison-reference-artifact requires --verify-artifact-file".to_string(),
                ));
            }
            Some(
                crate::miniqmt_market::QuantixRegressionComparison::from_local_reference_artifact(
                    resolved,
                    reference_artifact,
                )
                .map_err(QuantixError::DataParse)?,
            )
        }
        (None, Some(source_of_truth_summary), false) => {
            if !verify_artifact_file {
                return Err(QuantixError::DataParse(
                    "--comparison-source-of-truth-summary requires --verify-artifact-file"
                        .to_string(),
                ));
            }
            Some(
                crate::miniqmt_market::QuantixRegressionComparison::from_source_of_truth_summary(
                    resolved,
                    source_of_truth_summary,
                )
                .map_err(QuantixError::DataParse)?,
            )
        }
        (None, None, true) => {
            if !verify_artifact_file {
                return Err(QuantixError::DataParse(
                    "--comparison-clickhouse-* options require --verify-artifact-file".to_string(),
                ));
            }
            Some(
                load_direct_clickhouse_read_only_comparison(
                    resolved,
                    comparison_options.clickhouse,
                )
                .await?,
            )
        }
        (None, None, false) => None,
    };
    let report =
        crate::miniqmt_market::QuantixRegressionReport::from_resolved_artifact_with_comparison(
            resolved,
            crate::miniqmt_market::QuantixRegressionContext {
                source_command: build_import_market_manifest_source_command(
                    manifest,
                    dataset_version,
                    artifact_type,
                    schema_version,
                    artifact_hash,
                    verify_artifact_file,
                    comparison_options,
                    regression_report_output,
                    evidence_output,
                    consumer_build_commit,
                    database_target,
                ),
                run_at: run_at.clone(),
                consumer_build_commit: consumer_build_commit.to_string(),
                database_target: database_target.to_string(),
                writes_performed: false,
            },
            comparison,
        );

    let Some(regression_report_output) = regression_report_output else {
        return Err(QuantixError::DataParse(
            "--evidence-output requires --regression-report-output".to_string(),
        ));
    };

    write_json_file(
        regression_report_output,
        &report.to_pretty_json().map_err(QuantixError::Other)?,
    )?;

    if let Some(evidence_output) = evidence_output {
        if !verify_artifact_file {
            return Err(QuantixError::DataParse(
                "--evidence-output requires --verify-artifact-file".to_string(),
            ));
        }
        if consumer_build_commit == "unknown" {
            return Err(QuantixError::DataParse(
                "--evidence-output requires --consumer-build-commit or QUANTIX_CONSUMER_BUILD_COMMIT"
                    .to_string(),
            ));
        }

        let raw_report = crate::miniqmt_market::raw_report_reference(regression_report_output)
            .map_err(QuantixError::DataParse)?;
        let evidence = crate::miniqmt_market::QuantixRegressionEvidence::from_report(
            &report, raw_report, run_at,
        )
        .map_err(QuantixError::DataParse)?;
        write_json_file(
            evidence_output,
            &evidence.to_pretty_json().map_err(QuantixError::Other)?,
        )?;
    }

    Ok(())
}

async fn load_direct_clickhouse_read_only_comparison(
    resolved: &crate::miniqmt_market::ResolvedMarketArtifact,
    options: MarketManifestClickHouseComparisonOptions<'_>,
) -> Result<crate::miniqmt_market::QuantixRegressionComparison> {
    let url = options
        .url
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            QuantixError::DataParse(
                "--comparison-clickhouse-url is required when ClickHouse comparison is enabled"
                    .to_string(),
            )
        })?;
    let table = options
        .table
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            QuantixError::DataParse(
                "--comparison-clickhouse-table is required when ClickHouse comparison is enabled"
                    .to_string(),
            )
        })?;

    let table = validate_clickhouse_table_identifier(table)?;
    let dataset_column =
        validate_clickhouse_column_identifier(options.dataset_version_column, "dataset version")?;
    let symbol_column = validate_clickhouse_column_identifier(options.symbol_column, "symbol")?;
    let date_column = validate_clickhouse_column_identifier(options.date_column, "date")?;
    let dataset_version_literal = clickhouse_string_literal(&resolved.dataset_version);
    let client =
        crate::db::ClickHouseClient::new(url, options.database, options.user, options.password)
            .await?;

    let row_count_sql = format!(
        "SELECT count() AS row_count FROM {table} WHERE {dataset_column} = '{dataset_version_literal}'"
    );
    let row_counts: Vec<ClickHouseRowCount> = client.query_json(&row_count_sql).await?;
    let row_count = row_counts.first().map(|row| row.row_count).ok_or_else(|| {
        QuantixError::DataParse("ClickHouse row-count comparison returned no rows".to_string())
    })?;

    let symbol_sql = format!(
        "SELECT DISTINCT {symbol_column} AS value FROM {table} WHERE {dataset_column} = '{dataset_version_literal}' ORDER BY value LIMIT 5"
    );
    let sample_symbols = query_clickhouse_values(&client, &symbol_sql).await?;

    let date_sql = format!(
        "SELECT DISTINCT {date_column} AS value FROM {table} WHERE {dataset_column} = '{dataset_version_literal}' ORDER BY value LIMIT 5"
    );
    let sample_dates = query_clickhouse_values(&client, &date_sql).await?;

    let summary = crate::miniqmt_market::QuantixClickHouseReadOnlySummary {
        database: options.database.to_string(),
        table,
        dataset_version: resolved.dataset_version.clone(),
        row_count,
        sample_symbols,
        sample_dates,
    };

    crate::miniqmt_market::QuantixRegressionComparison::from_clickhouse_read_only_summary(
        resolved, summary,
    )
    .map_err(QuantixError::DataParse)
}

async fn query_clickhouse_values(
    client: &crate::db::ClickHouseClient,
    sql: &str,
) -> Result<Vec<String>> {
    let rows: Vec<ClickHouseValue> = client.query_json(sql).await?;
    Ok(rows.into_iter().map(|row| row.value).collect())
}

fn validate_clickhouse_table_identifier(identifier: &str) -> Result<String> {
    let trimmed = identifier.trim();
    if trimmed.is_empty() {
        return Err(QuantixError::DataParse(
            "ClickHouse table identifier must not be empty".to_string(),
        ));
    }
    if trimmed
        .split('.')
        .all(is_safe_clickhouse_identifier_segment)
    {
        Ok(trimmed.to_string())
    } else {
        Err(QuantixError::DataParse(format!(
            "unsafe ClickHouse table identifier: {identifier}"
        )))
    }
}

fn validate_clickhouse_column_identifier(identifier: &str, role: &str) -> Result<String> {
    let trimmed = identifier.trim();
    if is_safe_clickhouse_identifier_segment(trimmed) {
        Ok(trimmed.to_string())
    } else {
        Err(QuantixError::DataParse(format!(
            "unsafe ClickHouse {role} column identifier: {identifier}"
        )))
    }
}

fn is_safe_clickhouse_identifier_segment(segment: &str) -> bool {
    !segment.is_empty()
        && segment
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn clickhouse_string_literal(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\'', "\\'")
}

fn write_json_file(path: &str, content: &str) -> Result<()> {
    let path = std::path::Path::new(path);
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, content)?;
    Ok(())
}

fn build_import_market_manifest_source_command(
    manifest: &str,
    dataset_version: &str,
    artifact_type: &str,
    schema_version: Option<&str>,
    artifact_hash: Option<&str>,
    verify_artifact_file: bool,
    comparison_options: MarketManifestComparisonOptions<'_>,
    regression_report_output: Option<&str>,
    evidence_output: Option<&str>,
    consumer_build_commit: &str,
    database_target: &str,
) -> String {
    let mut parts = vec![
        "quantix".to_string(),
        "import".to_string(),
        "market-manifest".to_string(),
        "--manifest".to_string(),
        manifest.to_string(),
        "--dataset-version".to_string(),
        dataset_version.to_string(),
        "--artifact-type".to_string(),
        artifact_type.to_string(),
    ];

    if let Some(schema_version) = schema_version {
        parts.extend(["--schema-version".to_string(), schema_version.to_string()]);
    }
    if let Some(artifact_hash) = artifact_hash {
        parts.extend(["--artifact-hash".to_string(), artifact_hash.to_string()]);
    }
    if verify_artifact_file {
        parts.push("--verify-artifact-file".to_string());
    }
    if let Some(comparison_reference_artifact) = comparison_options.reference_artifact {
        parts.extend([
            "--comparison-reference-artifact".to_string(),
            comparison_reference_artifact.to_string(),
        ]);
    }
    if let Some(comparison_source_of_truth_summary) = comparison_options.source_of_truth_summary {
        parts.extend([
            "--comparison-source-of-truth-summary".to_string(),
            comparison_source_of_truth_summary.to_string(),
        ]);
    }
    if comparison_options.clickhouse.is_enabled() {
        if let Some(url) = comparison_options.clickhouse.url {
            parts.extend(["--comparison-clickhouse-url".to_string(), url.to_string()]);
        }
        parts.extend([
            "--comparison-clickhouse-database".to_string(),
            comparison_options.clickhouse.database.to_string(),
            "--comparison-clickhouse-user".to_string(),
            comparison_options.clickhouse.user.to_string(),
        ]);
        if !comparison_options.clickhouse.password.is_empty() {
            parts.extend([
                "--comparison-clickhouse-password".to_string(),
                "<redacted>".to_string(),
            ]);
        }
        if let Some(table) = comparison_options.clickhouse.table {
            parts.extend([
                "--comparison-clickhouse-table".to_string(),
                table.to_string(),
            ]);
        }
        parts.extend([
            "--comparison-clickhouse-dataset-version-column".to_string(),
            comparison_options
                .clickhouse
                .dataset_version_column
                .to_string(),
            "--comparison-clickhouse-symbol-column".to_string(),
            comparison_options.clickhouse.symbol_column.to_string(),
            "--comparison-clickhouse-date-column".to_string(),
            comparison_options.clickhouse.date_column.to_string(),
        ]);
    }
    if let Some(regression_report_output) = regression_report_output {
        parts.extend([
            "--regression-report-output".to_string(),
            regression_report_output.to_string(),
        ]);
    }
    if let Some(evidence_output) = evidence_output {
        parts.extend(["--evidence-output".to_string(), evidence_output.to_string()]);
    }
    parts.extend([
        "--consumer-build-commit".to_string(),
        consumer_build_commit.to_string(),
        "--database-target".to_string(),
        database_target.to_string(),
    ]);

    parts.join(" ")
}

async fn run_import_from_image(file: &str, model: &str) -> Result<()> {
    let provider = crate::import::ImageVisionProvider::parse(model)?;
    let extractor = crate::import::ImageExtractor::with_provider(provider);
    let result = extractor.extract_from_file(file).await?;

    println!("📷 图片股票识别");
    println!("   文件: {}", file);
    println!("   模型: {}", model);
    println!();

    if result.items.is_empty() {
        if !result.errors.is_empty() {
            println!("❌ {}", result.errors[0]);
        } else {
            println!("❌ 未从图片中识别到股票信息");
        }
        return Ok(());
    }

    println!("✅ 识别到 {} 只股票:", result.items.len());
    println!();
    println!("{:<10} {:<12} {:<8} {}", "代码", "名称", "置信度", "来源");
    println!("{}", "-".repeat(50));
    for item in &result.items {
        let code = item.code.as_deref().unwrap_or("-");
        let name = item.name.as_deref().unwrap_or("-");
        println!(
            "{:<10} {:<12} {:.0}%     {:?}",
            code,
            name,
            item.confidence * 100.0,
            item.source
        );
    }

    Ok(())
}

async fn run_import_from_csv(file: &str) -> Result<()> {
    println!("📄 CSV 导入");
    println!("   文件: {}", file);
    println!();

    let parser = crate::import::CsvParser::with_defaults();
    let result = parser.parse_file(file)?;

    if result.items.is_empty() {
        println!("❌ 未从 CSV 中解析到股票信息");
        if !result.errors.is_empty() {
            println!();
            println!("解析错误:");
            for err in &result.errors {
                println!("   - {}", err);
            }
        }
        return Ok(());
    }

    println!(
        "✅ 解析完成: {} 只股票 (共 {} 行, 跳过 {} 行)",
        result.parsed_count, result.total_input_lines, result.skipped_count
    );
    println!();
    println!("{:<10} {:<12} {}", "代码", "名称", "置信度");
    println!("{}", "-".repeat(35));
    for item in &result.items {
        let code = item.code.as_deref().unwrap_or("-");
        let name = item.name.as_deref().unwrap_or("-");
        println!("{:<10} {:<12} {:.0}%", code, name, item.confidence * 100.0);
    }

    Ok(())
}

async fn run_import_from_excel(file: &str, sheet: Option<&str>) -> Result<()> {
    println!("📊 Excel 导入");
    println!("   文件: {}", file);
    if let Some(sheet) = sheet {
        println!("   Sheet: {}", sheet);
    }
    println!();

    let parser = crate::import::ExcelParser::with_defaults();
    let result = parser.parse_file(file, sheet)?;

    if result.items.is_empty() {
        println!("❌ 未从 Excel 中解析到股票信息");
        if !result.errors.is_empty() {
            println!();
            println!("解析错误:");
            for err in &result.errors {
                println!("   - {}", err);
            }
        }
        return Ok(());
    }

    println!(
        "✅ 解析完成: {} 只股票 (共 {} 行, 跳过 {} 行)",
        result.parsed_count, result.total_input_lines, result.skipped_count
    );
    println!();
    println!("{:<10} {:<12} {}", "代码", "名称", "置信度");
    println!("{}", "-".repeat(35));
    for item in &result.items {
        let code = item.code.as_deref().unwrap_or("-");
        let name = item.name.as_deref().unwrap_or("-");
        println!("{:<10} {:<12} {:.0}%", code, name, item.confidence * 100.0);
    }

    Ok(())
}

async fn run_import_from_clipboard() -> Result<()> {
    println!("📋 剪贴板导入");
    println!();

    let clipboard_content = get_clipboard_content()?;

    if clipboard_content.is_empty() {
        println!("❌ 剪贴板为空");
        return Ok(());
    }

    println!("📝 剪贴板内容 (前 200 字符):");
    let preview: String = clipboard_content.chars().take(200).collect();
    println!("   {}", preview);
    println!();

    let parser = crate::import::TextParser::with_defaults();
    let result = parser.parse(&clipboard_content, crate::import::ImportSource::Clipboard);

    if result.items.is_empty() {
        println!("❌ 未从剪贴板内容中解析到股票信息");
        return Ok(());
    }

    println!("✅ 解析到 {} 只股票:", result.items.len());
    println!();
    println!("{:<10} {:<12} {}", "代码", "名称", "置信度");
    println!("{}", "-".repeat(35));
    for item in &result.items {
        let code = item.code.as_deref().unwrap_or("-");
        let name = item.name.as_deref().unwrap_or("-");
        println!("{:<10} {:<12} {:.0}%", code, name, item.confidence * 100.0);
    }

    Ok(())
}

async fn run_import_from_text(text: &str) -> Result<()> {
    println!("📝 文本导入");
    println!("   输入: {}", text);
    println!();

    let parser = crate::import::TextParser::with_defaults();
    let result = parser.parse(text, crate::import::ImportSource::Text);

    if result.items.is_empty() {
        println!("❌ 未从文本中解析到股票信息");
        if !result.errors.is_empty() {
            println!();
            println!("解析错误:");
            for err in &result.errors {
                println!("   - {}", err);
            }
        }
        return Ok(());
    }

    println!("✅ 解析到 {} 只股票:", result.items.len());
    println!();
    println!("{:<10} {:<12} {}", "代码", "名称", "置信度");
    println!("{}", "-".repeat(35));
    for item in &result.items {
        let code = item.code.as_deref().unwrap_or("-");
        let name = item.name.as_deref().unwrap_or("-");
        println!("{:<10} {:<12} {:.0}%", code, name, item.confidence * 100.0);
    }

    Ok(())
}

async fn run_import_resolve(input: &str) -> Result<()> {
    println!("🔍 股票代码/名称解析");
    println!("   输入: {}", input);
    println!();

    let resolver = crate::import::CodeResolver::new();

    match resolver.resolve(input) {
        Some(result) => {
            println!("✅ 解析成功:");
            println!("   代码: {}", result.code);
            if let Some(name) = &result.name {
                println!("   名称: {}", name);
            }
            println!("   匹配方式: {:?}", result.match_method);
            println!("   置信度: {:.0}%", result.confidence * 100.0);
        }
        None => {
            println!("❌ 无法解析: {}", input);
            println!();
            println!("💡 提示:");
            println!("   - 输入6位数字代码 (如 000001)");
            println!("   - 输入股票名称 (如 平安银行)");
            println!("   - 输入拼音首字母 (如 PAYH)");
        }
    }

    Ok(())
}

/// 获取剪贴板内容
fn get_clipboard_content() -> Result<String> {
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = std::process::Command::new("xclip")
            .args(["-selection", "clipboard", "-o"])
            .output()
        {
            if output.status.success() {
                return Ok(String::from_utf8_lossy(&output.stdout).to_string());
            }
        }
        if let Ok(output) = std::process::Command::new("xsel")
            .args(["--clipboard", "--output"])
            .output()
        {
            if output.status.success() {
                return Ok(String::from_utf8_lossy(&output.stdout).to_string());
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = std::process::Command::new("pbpaste").output() {
            if output.status.success() {
                return Ok(String::from_utf8_lossy(&output.stdout).to_string());
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(output) = std::process::Command::new("powershell")
            .args(["-command", "Get-Clipboard"])
            .output()
        {
            if output.status.success() {
                return Ok(String::from_utf8_lossy(&output.stdout).to_string());
            }
        }
    }

    Err(QuantixError::Other("无法读取剪贴板内容，请确保已安装 xclip/xsel (Linux)、pbpaste (macOS) 或 PowerShell (Windows)".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::import::excel_parser::tests::write_minimal_xlsx;

    #[tokio::test]
    async fn import_from_excel_uses_real_parser() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("watchlist.xlsx");
        write_minimal_xlsx(
            &path,
            "positions",
            &[
                &["代码", "名称"],
                &["000001", "平安银行"],
                &["600036", "招商银行"],
            ],
        )
        .unwrap();

        run_import_from_excel(path.to_str().unwrap(), Some("positions"))
            .await
            .expect("from-excel should parse a valid workbook instead of returning Unsupported");
    }
}
