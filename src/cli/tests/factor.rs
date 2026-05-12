use super::*;

#[test]
fn parses_factor_list_command() {
    let cli = Cli::try_parse_from([
        "quantix",
        "factor",
        "list",
        "--category",
        "technical",
        "--verbose",
    ])
    .unwrap();

    match cli.command {
        Commands::Factor(FactorCommands::List { category, verbose }) => {
            assert_eq!(category.as_deref(), Some("technical"));
            assert!(verbose);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_factor_compute_defaults() {
    let cli = Cli::try_parse_from([
        "quantix",
        "factor",
        "compute",
        "--input",
        "/tmp/factor-input.csv",
        "--factor",
        "rank_close",
        "delta_close_1",
        "--symbol",
        "000001.SZ",
        "600000.SH",
        "--start",
        "2026-01-01",
        "--end",
        "2026-01-10",
    ])
    .unwrap();

    match cli.command {
        Commands::Factor(FactorCommands::Compute {
            input,
            factors,
            symbols,
            start,
            end,
            format,
            output,
            skip_checks,
        }) => {
            assert_eq!(input, "/tmp/factor-input.csv");
            assert_eq!(factors, vec!["rank_close", "delta_close_1"]);
            assert_eq!(symbols, vec!["000001.SZ", "600000.SH"]);
            assert_eq!(start, "2026-01-01");
            assert_eq!(end, "2026-01-10");
            assert_eq!(format, FactorOutputFormat::Table);
            assert_eq!(output, None);
            assert!(!skip_checks);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn rejects_factor_compute_without_required_args() {
    let err = Cli::try_parse_from(["quantix", "factor", "compute"]).unwrap_err();
    assert_eq!(err.kind(), clap::error::ErrorKind::MissingRequiredArgument);
}

#[test]
fn parses_factor_evaluate_command() {
    let cli = Cli::try_parse_from([
        "quantix",
        "factor",
        "evaluate",
        "--input",
        "/tmp/factor-input.csv",
        "--factor",
        "rank_close",
        "--symbol",
        "000001.SZ",
        "600000.SH",
        "--start",
        "2026-01-01",
        "--end",
        "2026-01-10",
        "--horizon",
        "2",
        "--format",
        "json",
        "--output",
        "/tmp/icir.json",
        "--skip-checks",
    ])
    .unwrap();

    match cli.command {
        Commands::Factor(FactorCommands::Evaluate {
            input,
            factor,
            symbols,
            start,
            end,
            horizon,
            format,
            output,
            skip_checks,
        }) => {
            assert_eq!(input, "/tmp/factor-input.csv");
            assert_eq!(factor, "rank_close");
            assert_eq!(symbols, vec!["000001.SZ", "600000.SH"]);
            assert_eq!(start, "2026-01-01");
            assert_eq!(end, "2026-01-10");
            assert_eq!(horizon, 2);
            assert_eq!(format, FactorOutputFormat::Json);
            assert_eq!(output.as_deref(), Some("/tmp/icir.json"));
            assert!(skip_checks);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_factor_evaluate_csv_output() {
    let cli = Cli::try_parse_from([
        "quantix",
        "factor",
        "evaluate",
        "--input",
        "/tmp/factor-input.csv",
        "--factor",
        "rank_close",
        "--symbol",
        "000001.SZ",
        "--start",
        "2026-01-01",
        "--end",
        "2026-01-10",
        "--format",
        "csv",
        "--output",
        "/tmp/icir.csv",
    ])
    .unwrap();

    match cli.command {
        Commands::Factor(FactorCommands::Evaluate { format, output, .. }) => {
            assert_eq!(format, FactorOutputFormat::Csv);
            assert_eq!(output.as_deref(), Some("/tmp/icir.csv"));
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_factor_compute_command() {
    let cli = Cli::try_parse_from([
        "quantix",
        "factor",
        "compute",
        "--input",
        "/tmp/factor-input.csv",
        "--factor",
        "rank_close",
        "--factor",
        "delta_close_1",
        "--symbol",
        "000001.SZ",
        "--symbol",
        "600000.SH",
        "--start",
        "2026-01-01",
        "--end",
        "2026-01-10",
        "--format",
        "json",
        "--output",
        "/tmp/factors.json",
        "--skip-checks",
    ])
    .unwrap();

    match cli.command {
        Commands::Factor(FactorCommands::Compute {
            input,
            factors,
            symbols,
            start,
            end,
            format,
            output,
            skip_checks,
        }) => {
            assert_eq!(input, "/tmp/factor-input.csv");
            assert_eq!(factors, vec!["rank_close", "delta_close_1"]);
            assert_eq!(symbols, vec!["000001.SZ", "600000.SH"]);
            assert_eq!(start, "2026-01-01");
            assert_eq!(end, "2026-01-10");
            assert_eq!(format, FactorOutputFormat::Json);
            assert_eq!(output.as_deref(), Some("/tmp/factors.json"));
            assert!(skip_checks);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}
