use super::*;

#[test]
fn parses_candle_pattern_command_with_explicit_reference() {
    let cli = Cli::try_parse_from([
        "quantix",
        "analyze",
        "candle-pattern",
        "--candle",
        "10,10,8,8",
        "--candle",
        "10,12,8,10",
        "--reference",
        "10",
    ])
    .unwrap();

    match cli.command {
        Commands::Analyze(AnalyzeCommands::CandlePattern {
            candle,
            reference,
            previous_close,
            ..
        }) => {
            assert_eq!(candle, vec!["10,10,8,8", "10,12,8,10"]);
            assert_eq!(reference.as_deref(), Some("10"));
            assert!(!previous_close);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_candle_pattern_command_with_previous_close_policy() {
    let cli = Cli::try_parse_from([
        "quantix",
        "analyze",
        "candle-pattern",
        "--candle",
        "10,10,10,10",
        "--candle",
        "10,12,10,12",
        "--previous-close",
    ])
    .unwrap();

    match cli.command {
        Commands::Analyze(AnalyzeCommands::CandlePattern {
            candle,
            reference,
            previous_close,
            ..
        }) => {
            assert_eq!(candle, vec!["10,10,10,10", "10,12,10,12"]);
            assert_eq!(reference, None);
            assert!(previous_close);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn rejects_candle_pattern_command_without_reference_mode() {
    let err = Cli::try_parse_from([
        "quantix",
        "analyze",
        "candle-pattern",
        "--candle",
        "10,10,8,8",
    ])
    .unwrap_err();

    assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
}

#[test]
fn parses_candle_pattern_command_with_code_source() {
    let cli = Cli::try_parse_from([
        "quantix",
        "analyze",
        "candle-pattern",
        "--code",
        "000001",
        "--type",
        "1d",
        "--limit",
        "30",
        "--previous-close",
    ])
    .unwrap();

    match cli.command {
        Commands::Analyze(AnalyzeCommands::CandlePattern {
            candle,
            code,
            start,
            end,
            r#type,
            limit,
            reference,
            previous_close,
            ..
        }) => {
            assert!(candle.is_empty());
            assert_eq!(code.as_deref(), Some("000001"));
            assert_eq!(start, None);
            assert_eq!(end, None);
            assert_eq!(r#type, "1d");
            assert_eq!(limit, 30);
            assert_eq!(reference, None);
            assert!(previous_close);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_candle_pattern_command_with_day_file_source() {
    let cli = Cli::try_parse_from([
        "quantix",
        "analyze",
        "candle-pattern",
        "--day-file",
        "/mnt/d/ProgramData/tdx_20251231/vipdoc/sh/lday/sh000001.day",
        "--limit",
        "5",
        "--previous-close",
    ])
    .unwrap();

    match cli.command {
        Commands::Analyze(AnalyzeCommands::CandlePattern {
            candle,
            code,
            day_file,
            limit,
            previous_close,
            ..
        }) => {
            assert!(candle.is_empty());
            assert_eq!(code, None);
            assert_eq!(
                day_file.as_deref(),
                Some("/mnt/d/ProgramData/tdx_20251231/vipdoc/sh/lday/sh000001.day")
            );
            assert_eq!(limit, 5);
            assert!(previous_close);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}

#[test]
fn parses_candle_pattern_command_with_tdx_root_and_market() {
    let cli = Cli::try_parse_from([
        "quantix",
        "analyze",
        "candle-pattern",
        "--code",
        "000001",
        "--tdx-root",
        "/mnt/d/ProgramData/tdx_20251231",
        "--market",
        "sz",
        "--limit",
        "5",
        "--previous-close",
    ])
    .unwrap();

    match cli.command {
        Commands::Analyze(AnalyzeCommands::CandlePattern {
            code,
            tdx_root,
            market,
            limit,
            previous_close,
            ..
        }) => {
            assert_eq!(code.as_deref(), Some("000001"));
            assert_eq!(tdx_root.as_deref(), Some("/mnt/d/ProgramData/tdx_20251231"));
            assert_eq!(market.as_deref(), Some("sz"));
            assert_eq!(limit, 5);
            assert!(previous_close);
        }
        other => panic!("unexpected command: {:?}", other),
    }
}
