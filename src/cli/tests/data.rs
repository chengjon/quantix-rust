use super::*;

#[test]
fn parses_data_source_list_command() {
    let cli = Cli::try_parse_from(["quantix", "data", "source", "list"]).unwrap();

    match cli.command {
        Commands::Data(DataCommands::Source(DataSourceCommands::List { config_dir })) => {
            assert_eq!(config_dir, "config");
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_data_source_add_tdx_command() {
    let cli = Cli::try_parse_from([
        "quantix",
        "data",
        "source",
        "add",
        "--type",
        "tdx",
        "--hosts",
        "114.80.63.12,114.80.63.13",
        "--port",
        "7709",
        "--timeout",
        "5000",
    ])
    .unwrap();

    match cli.command {
        Commands::Data(DataCommands::Source(DataSourceCommands::Add {
            source_type,
            hosts,
            port,
            timeout,
            ..
        })) => {
            assert_eq!(source_type, DataSourceKind::Tdx);
            assert_eq!(hosts, vec!["114.80.63.12", "114.80.63.13"]);
            assert_eq!(port, Some(7709));
            assert_eq!(timeout, Some(5000));
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_data_source_add_akshare_command() {
    let cli = Cli::try_parse_from([
        "quantix",
        "data",
        "source",
        "add",
        "--type",
        "akshare",
        "--base-url",
        "http://localhost:8000",
        "--rate-limit",
        "120",
    ])
    .unwrap();

    match cli.command {
        Commands::Data(DataCommands::Source(DataSourceCommands::Add {
            source_type,
            base_url,
            rate_limit,
            ..
        })) => {
            assert_eq!(source_type, DataSourceKind::Akshare);
            assert_eq!(base_url.as_deref(), Some("http://localhost:8000"));
            assert_eq!(rate_limit, Some(120));
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_data_source_set_default_command() {
    let cli =
        Cli::try_parse_from(["quantix", "data", "source", "set-default", "--name", "tdx"]).unwrap();

    match cli.command {
        Commands::Data(DataCommands::Source(DataSourceCommands::SetDefault {
            config_dir,
            name,
        })) => {
            assert_eq!(config_dir, "config");
            assert_eq!(name, DataSourceKind::Tdx);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_data_source_test_command() {
    let cli =
        Cli::try_parse_from(["quantix", "data", "source", "test", "--name", "akshare"]).unwrap();

    match cli.command {
        Commands::Data(DataCommands::Source(DataSourceCommands::Test { config_dir, name })) => {
            assert_eq!(config_dir, "config");
            assert_eq!(name, DataSourceKind::Akshare);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_data_openstock_validate_fixture_command() {
    let cli = Cli::try_parse_from([
        "quantix",
        "data",
        "openstock",
        "validate-fixture",
        "--file",
        "tests/fixtures/openstock/daily_kline.json",
    ])
    .unwrap();

    match cli.command {
        Commands::Data(DataCommands::OpenStock(OpenStockCommands::ValidateFixture { file })) => {
            assert_eq!(file, "tests/fixtures/openstock/daily_kline.json");
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_data_import_fundamentals_command() {
    let cli = Cli::try_parse_from([
        "quantix",
        "data",
        "import-fundamentals",
        "--input",
        "fixtures/market_fundamentals.json",
    ])
    .unwrap();

    match cli.command {
        Commands::Data(DataCommands::ImportFundamentals { input }) => {
            assert_eq!(input, "fixtures/market_fundamentals.json");
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[cfg(test)]
mod tests_p0_15a {
    use crate::cli::handlers::openstock_handler::compute_apply;

    #[test]
    fn compute_apply_returns_false_when_apply_flag_false() {
        // U3: --apply not set → false regardless of env
        unsafe { std::env::set_var("QUANTIX_OPENSTOCK_MINUTE_APPLY", "yes") };
        assert!(!compute_apply(false));
        unsafe { std::env::remove_var("QUANTIX_OPENSTOCK_MINUTE_APPLY") };
    }

    #[test]
    fn compute_apply_reads_env_var() {
        // U2: env var must be exactly "yes" (not "true", not "1", not unset)
        unsafe { std::env::set_var("QUANTIX_OPENSTOCK_MINUTE_APPLY", "yes") };
        assert!(compute_apply(true));

        unsafe { std::env::set_var("QUANTIX_OPENSTOCK_MINUTE_APPLY", "true") };
        assert!(!compute_apply(true)); // wrong value

        unsafe { std::env::set_var("QUANTIX_OPENSTOCK_MINUTE_APPLY", "1") };
        assert!(!compute_apply(true)); // wrong value

        unsafe { std::env::remove_var("QUANTIX_OPENSTOCK_MINUTE_APPLY") };
        assert!(!compute_apply(true)); // unset
    }

    #[test]
    fn import_minute_args_validate_period_and_adjust() {
        use crate::data::models::{AdjustType, DateOrRange, MinutePeriod};
        use std::str::FromStr;

        // Mirror the parsing the handler will do.
        let period_enum = MinutePeriod::from_str("1m").expect("1m parses");
        let adjust_enum = AdjustType::from_str("none").expect("none parses");
        let dor = DateOrRange::from_cli(None, Some("2026-01-01"), Some("2026-01-05"))
            .expect("range parses");

        assert_eq!(period_enum.as_str(), "1m");
        assert_eq!(adjust_enum.as_str(), "none");
        match dor {
            DateOrRange::Range { start, end } => {
                assert_eq!(start.to_string(), "2026-01-01");
                assert_eq!(end.to_string(), "2026-01-05");
            }
            DateOrRange::Date(_) => panic!("expected Range, got Date"),
        }
    }
}
