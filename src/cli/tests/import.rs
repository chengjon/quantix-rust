use super::*;

#[test]
fn parses_import_resolve_command() {
    let cli = Cli::try_parse_from(["quantix", "import", "resolve", "--input", "000001"]).unwrap();

    match cli.command {
        Commands::Import(ImportCommands::Resolve { input }) => {
            assert_eq!(input, "000001");
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_import_market_manifest_command() {
    let cli = Cli::try_parse_from([
        "quantix",
        "import",
        "market-manifest",
        "--manifest",
        "/tmp/manifest.json",
        "--dataset-version",
        "cn-a-share-daily@2026-05-12",
        "--artifact-type",
        "parquet",
        "--schema-version",
        "kline-daily-v1",
        "--artifact-hash",
        "sha256:artifact",
        "--verify-artifact-file",
        "--regression-report-output",
        "/tmp/quantix-regression.json",
        "--evidence-output",
        "/tmp/quantix-regression.evidence.json",
        "--consumer-build-commit",
        "abc123",
        "--database-target",
        "dry-run-only",
    ])
    .unwrap();

    match cli.command {
        Commands::Import(ImportCommands::MarketManifest {
            manifest,
            dataset_version,
            artifact_type,
            schema_version,
            artifact_hash,
            verify_artifact_file,
            regression_report_output,
            evidence_output,
            consumer_build_commit,
            database_target,
            ..
        }) => {
            assert_eq!(manifest, "/tmp/manifest.json");
            assert_eq!(dataset_version, "cn-a-share-daily@2026-05-12");
            assert_eq!(artifact_type, "parquet");
            assert_eq!(schema_version.as_deref(), Some("kline-daily-v1"));
            assert_eq!(artifact_hash.as_deref(), Some("sha256:artifact"));
            assert!(verify_artifact_file);
            assert_eq!(
                regression_report_output.as_deref(),
                Some("/tmp/quantix-regression.json")
            );
            assert_eq!(
                evidence_output.as_deref(),
                Some("/tmp/quantix-regression.evidence.json")
            );
            assert_eq!(consumer_build_commit.as_deref(), Some("abc123"));
            assert_eq!(database_target, "dry-run-only");
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_import_from_excel_command() {
    let cli = Cli::try_parse_from([
        "quantix",
        "import",
        "from-excel",
        "--file",
        "/tmp/watchlist.xlsx",
        "--sheet",
        "positions",
    ])
    .unwrap();

    match cli.command {
        Commands::Import(ImportCommands::FromExcel { file, sheet }) => {
            assert_eq!(file, "/tmp/watchlist.xlsx");
            assert_eq!(sheet.as_deref(), Some("positions"));
        }
        other => panic!("unexpected command: {:?}", other),
    }
}
