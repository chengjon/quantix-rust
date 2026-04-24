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
