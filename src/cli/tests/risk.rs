use super::*;

#[test]
fn parses_risk() {
    let cli = Cli::try_parse_from([
        "quantix",
        "risk",
        "rule",
        "set",
        "--type",
        "position-limit",
        "--value",
        "20%",
    ])
    .unwrap();
    match cli.command {
        Commands::Risk(RiskCommands::Rule(RiskRuleCommands::Set { rule_type, value })) => {
            assert_eq!(rule_type, "position-limit");
            assert_eq!(value, "20%");
        }
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from([
        "quantix",
        "risk",
        "rule",
        "set",
        "--type",
        "daily-loss-limit",
        "--value",
        "50000",
    ])
    .unwrap();
    match cli.command {
        Commands::Risk(RiskCommands::Rule(RiskRuleCommands::Set { rule_type, value })) => {
            assert_eq!(rule_type, "daily-loss-limit");
            assert_eq!(value, "50000");
        }
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from(["quantix", "risk", "rule", "list"]).unwrap();
    match cli.command {
        Commands::Risk(RiskCommands::Rule(RiskRuleCommands::List)) => {}
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from([
        "quantix",
        "risk",
        "rule",
        "enable",
        "--type",
        "position-limit",
    ])
    .unwrap();
    match cli.command {
        Commands::Risk(RiskCommands::Rule(RiskRuleCommands::Enable { rule_type })) => {
            assert_eq!(rule_type, "position-limit");
        }
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from([
        "quantix",
        "risk",
        "rule",
        "disable",
        "--type",
        "daily-loss-limit",
    ])
    .unwrap();
    match cli.command {
        Commands::Risk(RiskCommands::Rule(RiskRuleCommands::Disable { rule_type })) => {
            assert_eq!(rule_type, "daily-loss-limit");
        }
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from(["quantix", "risk", "status"]).unwrap();
    match cli.command {
        Commands::Risk(RiskCommands::Status) => {}
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_risk_rejects_missing_value_or_type() {
    let err = Cli::try_parse_from(["quantix", "risk", "rule", "set", "--type", "position-limit"])
        .unwrap_err();
    assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    assert!(err.to_string().contains("--value"));

    let err = Cli::try_parse_from(["quantix", "risk", "rule", "set", "--value", "20%"])
        .unwrap_err();
    assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    assert!(err.to_string().contains("--type"));
}
