use super::*;

#[test]
fn parses_watchlist_add_command() {
    let cli = Cli::try_parse_from([
        "quantix",
        "watchlist",
        "add",
        "--code",
        "000001",
        "--group",
        "core",
    ])
    .unwrap();

    match cli.command {
        Commands::Watchlist(WatchlistCommands::Add { code, group }) => {
            assert_eq!(code, "000001");
            assert_eq!(group.as_deref(), Some("core"));
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_watchlist_list_command_with_filters_and_price_flag() {
    let cli = Cli::try_parse_from([
        "quantix",
        "watchlist",
        "list",
        "--group",
        "core",
        "--tag",
        "bank",
        "--with-price",
    ])
    .unwrap();

    match cli.command {
        Commands::Watchlist(WatchlistCommands::List {
            group,
            tag,
            with_price,
        }) => {
            assert_eq!(group.as_deref(), Some("core"));
            assert_eq!(tag.as_deref(), Some("bank"));
            assert!(with_price);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_watchlist_group_create_command() {
    let cli =
        Cli::try_parse_from(["quantix", "watchlist", "group", "create", "--name", "core"]).unwrap();

    match cli.command {
        Commands::Watchlist(WatchlistCommands::Group(WatchlistGroupCommands::Create { name })) => {
            assert_eq!(name, "core");
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_watchlist_history_command_with_limit() {
    let cli = Cli::try_parse_from([
        "quantix",
        "watchlist",
        "history",
        "--code",
        "000001",
        "--limit",
        "20",
    ])
    .unwrap();

    match cli.command {
        Commands::Watchlist(WatchlistCommands::History { code, limit }) => {
            assert_eq!(code.as_deref(), Some("000001"));
            assert_eq!(limit, 20);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}
