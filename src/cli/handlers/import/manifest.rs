use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) async fn run_import_market_manifest(
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

#[allow(clippy::too_many_arguments)]
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
