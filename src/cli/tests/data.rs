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
    use super::*;
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

    /// REQ-PERSIST-006 scenario C: missing both --start and --end is rejected
    /// by DateOrRange::from_cli before any client is constructed.
    #[test]
    fn import_minute_klines_rejects_missing_start_and_end() {
        use crate::data::models::DateOrRange;

        let err =
            DateOrRange::from_cli(None, None, None).expect_err("missing start+end must error");
        let msg = format!("{err}");
        assert!(
            msg.contains("at least one of --date or (--start, --end)"),
            "expected from_cli error mentioning required flags, got: {msg}"
        );
    }

    /// REQ-PERSIST-006 scenario B: invalid period value must fail enum parse,
    /// producing a Config error before any OpenStockClient is constructed.
    #[test]
    fn import_minute_klines_rejects_invalid_period() {
        use crate::data::models::MinutePeriod;
        use std::str::FromStr;

        let err = MinutePeriod::from_str("7m").expect_err("7m must not parse");
        // handler wraps this in QuantixError::Config("--period: ...")
        let msg = format!("{err}");
        assert!(
            !msg.is_empty(),
            "MinutePeriod::from_str error must be non-empty"
        );

        // Sanity: valid periods still parse (negative-case confirmation)
        assert!(MinutePeriod::from_str("1m").is_ok());
        assert!(MinutePeriod::from_str("5m").is_ok());
        assert!(MinutePeriod::from_str("15m").is_ok());
        assert!(MinutePeriod::from_str("30m").is_ok());
        assert!(MinutePeriod::from_str("60m").is_ok());
    }

    /// REQ-PERSIST-007 scenario B: ImportMinuteShare variant must NOT accept
    /// --period or --adjust flags (clap rejects as unknown argument).
    #[test]
    fn import_minute_share_rejects_period_and_adjust_flags() {
        // Direct parse: --period must be rejected
        let res = Cli::try_parse_from([
            "quantix",
            "data",
            "openstock",
            "import-minute-share",
            "--code",
            "sh600000",
            "--start",
            "2026-01-01",
            "--end",
            "2026-01-31",
            "--period",
            "5m",
        ]);
        assert!(
            res.is_err(),
            "--period must be rejected by import-minute-share"
        );

        // Same for --adjust
        let res = Cli::try_parse_from([
            "quantix",
            "data",
            "openstock",
            "import-minute-share",
            "--code",
            "sh600000",
            "--start",
            "2026-01-01",
            "--end",
            "2026-01-31",
            "--adjust",
            "qfq",
        ]);
        assert!(
            res.is_err(),
            "--adjust must be rejected by import-minute-share"
        );

        // Sanity: ImportMinuteShare variant has only 4 fields (code, start, end, apply)
        let ok = Cli::try_parse_from([
            "quantix",
            "data",
            "openstock",
            "import-minute-share",
            "--code",
            "sh600000",
            "--start",
            "2026-01-01",
            "--end",
            "2026-01-31",
        ])
        .expect("valid invocation without --period/--adjust must parse");
        match ok.command {
            Commands::Data(DataCommands::OpenStock(OpenStockCommands::ImportMinuteShare {
                code,
                start,
                end,
                apply,
            })) => {
                assert_eq!(code, "sh600000");
                assert_eq!(start.as_deref(), Some("2026-01-01"));
                assert_eq!(end.as_deref(), Some("2026-01-31"));
                assert!(!apply, "apply must default to false");
            }
            other => panic!("unexpected command: {:?}", other),
        }
    }

    /// REQ-PERSIST-009 invariant: when compute_apply returns false, the
    /// handler must not even construct an OpenStockClient — much less a
    /// ClickHouseClient. We can't unit-test the handler body without
    /// starting a wiremock, but we CAN assert the env+flag combinations
    /// that the dry-run branch keys on. This pins INV-CLI-2 ("no
    /// ClickHouse credentials required for dry-run") at the boundary.
    #[test]
    fn import_minute_dry_run_gates_known_env_combinations() {
        // The four combinations compute_apply can return:
        //  (apply=false, env=*)    → false (U3 already covered)
        //  (apply=true,  env="yes") → true  (U2 already covered)
        //  (apply=true,  env=unset) → false
        //  (apply=true,  env="no")  → false
        unsafe { std::env::remove_var("QUANTIX_OPENSTOCK_MINUTE_APPLY") };
        assert!(
            !compute_apply(true),
            "env unset + apply=true must produce dry-run (no CH write)"
        );

        unsafe { std::env::set_var("QUANTIX_OPENSTOCK_MINUTE_APPLY", "no") };
        assert!(
            !compute_apply(true),
            "env='no' + apply=true must produce dry-run"
        );

        unsafe { std::env::remove_var("QUANTIX_OPENSTOCK_MINUTE_APPLY") };
    }
}
