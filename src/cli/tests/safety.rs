use super::*;

#[test]
fn parses_safety_kill_switch_commands() {
    let cli = Cli::try_parse_from([
        "quantix",
        "safety",
        "kill-switch",
        "enable",
        "--reason",
        "broker instability",
    ])
    .unwrap();
    match cli.command {
        Commands::Safety(SafetyCommands::KillSwitch(SafetyKillSwitchCommands::Enable {
            reason,
        })) => {
            assert_eq!(reason, "broker instability");
        }
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from(["quantix", "safety", "kill-switch", "disable"]).unwrap();
    match cli.command {
        Commands::Safety(SafetyCommands::KillSwitch(SafetyKillSwitchCommands::Disable)) => {}
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from(["quantix", "safety", "kill-switch", "status"]).unwrap();
    match cli.command {
        Commands::Safety(SafetyCommands::KillSwitch(SafetyKillSwitchCommands::Status)) => {}
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn safety_kill_switch_enable_requires_reason() {
    let err = Cli::try_parse_from(["quantix", "safety", "kill-switch", "enable"])
        .expect_err("expected clap missing required argument");

    assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    assert!(err.to_string().contains("--reason"));
}
