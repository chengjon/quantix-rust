use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn build_import_market_manifest_source_command(
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
