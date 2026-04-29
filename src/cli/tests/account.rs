use super::*;

#[test]
fn parses_account_commands_with_qmt_live_primary_wording_and_live_alias() {
    let cli = Cli::try_parse_from([
        "quantix",
        "account",
        "register",
        "--id",
        "acct-qmt",
        "--account-type",
        "qmt_live",
        "--capital",
        "200000",
        "--adapter",
        "broker",
    ])
    .unwrap();
    match cli.command {
        Commands::Account(AccountCommands::Register {
            id,
            account_type,
            capital,
            adapter,
        }) => {
            assert_eq!(id, "acct-qmt");
            assert_eq!(account_type, "qmt_live");
            assert_eq!(capital, 200000.0);
            assert_eq!(adapter, "broker");
        }
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from([
        "quantix",
        "account",
        "list",
        "--account-type",
        "live",
        "--enabled-only",
    ])
    .unwrap();
    match cli.command {
        Commands::Account(AccountCommands::List {
            account_type,
            enabled_only,
        }) => {
            assert_eq!(account_type.as_deref(), Some("live"));
            assert!(enabled_only);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn account_register_help_mentions_qmt_live_primary_wording_and_live_alias() {
    let err = Cli::try_parse_from(["quantix", "account", "register", "--help"])
        .expect_err("expected clap help");

    assert_eq!(err.kind(), ErrorKind::DisplayHelp);

    let help = err.to_string();
    assert!(help.contains("账户类型: paper | mock_live | qmt_live"));
    assert!(help.contains("兼容 live 别名"));
}
