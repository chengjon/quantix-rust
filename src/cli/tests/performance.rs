use super::*;

#[test]
fn parses_performance_report_command() {
    let cli = Cli::try_parse_from(["quantix", "performance", "report", "--id", "bt-001"]).unwrap();

    match cli.command {
        Commands::Performance(PerformanceCommands::Report { id }) => {
            assert_eq!(id, "bt-001");
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_performance_list_command() {
    let cli = Cli::try_parse_from(["quantix", "performance", "list"]).unwrap();

    match cli.command {
        Commands::Performance(PerformanceCommands::List) => {}
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_performance_compare_command() {
    let cli = Cli::try_parse_from([
        "quantix",
        "performance",
        "compare",
        "--id",
        "bt-001",
        "--id",
        "bt-002",
    ])
    .unwrap();

    match cli.command {
        Commands::Performance(PerformanceCommands::Compare { ids }) => {
            assert_eq!(ids, vec!["bt-001", "bt-002"]);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}
