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
fn parses_data_source_set_default_tdx_api_command() {
    let cli = Cli::try_parse_from([
        "quantix",
        "data",
        "source",
        "set-default",
        "--name",
        "tdx-api",
    ])
    .unwrap();

    match cli.command {
        Commands::Data(DataCommands::Source(DataSourceCommands::SetDefault {
            config_dir,
            name,
        })) => {
            assert_eq!(config_dir, "config");
            assert_eq!(name, DataSourceKind::TdxApi);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_data_tdx_api_kline_command() {
    let cli = Cli::try_parse_from([
        "quantix", "data", "tdx-api", "kline", "--code", "600000", "--type", "minute5", "--limit",
        "10",
    ])
    .unwrap();

    match cli.command {
        Commands::Data(DataCommands::TdxApi(TdxApiCommands::Kline {
            code,
            r#type,
            limit,
        })) => {
            assert_eq!(code, "600000");
            assert_eq!(r#type, "minute5");
            assert_eq!(limit, Some(10));
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
fn parses_data_tdx_api_import_klines_code_command() {
    let cli = Cli::try_parse_from([
        "quantix",
        "data",
        "tdx-api",
        "import-klines",
        "--code",
        "600000",
        "--type",
        "week",
        "--force",
    ])
    .unwrap();

    match cli.command {
        Commands::Data(DataCommands::TdxApi(TdxApiCommands::ImportKlines {
            code,
            all,
            exchange,
            r#type,
            force,
        })) => {
            assert_eq!(code.as_deref(), Some("600000"));
            assert!(!all);
            assert_eq!(exchange, None);
            assert_eq!(r#type, "week");
            assert!(force);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_data_tdx_api_import_klines_all_command() {
    let cli = Cli::try_parse_from([
        "quantix",
        "data",
        "tdx-api",
        "import-klines",
        "--all",
        "--exchange",
        "sh",
    ])
    .unwrap();

    match cli.command {
        Commands::Data(DataCommands::TdxApi(TdxApiCommands::ImportKlines {
            code,
            all,
            exchange,
            r#type,
            force,
        })) => {
            assert_eq!(code, None);
            assert!(all);
            assert_eq!(exchange.as_deref(), Some("sh"));
            assert_eq!(r#type, "day");
            assert!(!force);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn rejects_data_tdx_api_import_klines_conflicting_targets() {
    let err = Cli::try_parse_from([
        "quantix",
        "data",
        "tdx-api",
        "import-klines",
        "--code",
        "600000",
        "--all",
    ])
    .unwrap_err();

    assert!(
        err.to_string().contains("cannot be used with"),
        "unexpected error: {err}"
    );
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
