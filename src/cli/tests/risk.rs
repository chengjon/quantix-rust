use super::*;
use crate::risk::{JsonRiskStore, RiskStore, RiskRuleType, RuleValue};
use rust_decimal_macros::dec;
use std::sync::{Mutex, OnceLock};

fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|err| err.into_inner())
}

struct RiskEnvGuard {
    risk_path: Option<String>,
    trade_path: Option<String>,
}

impl RiskEnvGuard {
    fn capture() -> Self {
        Self {
            risk_path: std::env::var("QUANTIX_RISK_PATH").ok(),
            trade_path: std::env::var("QUANTIX_TRADE_PATH").ok(),
        }
    }
}

impl Drop for RiskEnvGuard {
    fn drop(&mut self) {
        match &self.risk_path {
            Some(value) => unsafe { std::env::set_var("QUANTIX_RISK_PATH", value) },
            None => unsafe { std::env::remove_var("QUANTIX_RISK_PATH") },
        }

        match &self.trade_path {
            Some(value) => unsafe { std::env::set_var("QUANTIX_TRADE_PATH", value) },
            None => unsafe { std::env::remove_var("QUANTIX_TRADE_PATH") },
        }
    }
}

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
        "industry-blocklist",
        "--value",
        "银行,地产",
    ])
    .unwrap();
    match cli.command {
        Commands::Risk(RiskCommands::Rule(RiskRuleCommands::Set { rule_type, value })) => {
            assert_eq!(rule_type, "industry-blocklist");
            assert_eq!(value, "银行,地产");
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

    let cli = Cli::try_parse_from(["quantix", "risk", "pnl"]).unwrap();
    match cli.command {
        Commands::Risk(RiskCommands::Pnl) => {}
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from(["quantix", "risk", "position"]).unwrap();
    match cli.command {
        Commands::Risk(RiskCommands::Position) => {}
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from(["quantix", "risk", "log", "--limit", "5"]).unwrap();
    match cli.command {
        Commands::Risk(RiskCommands::Log {
            limit,
            date,
            event_type,
        }) => {
            assert_eq!(limit, 5);
            assert_eq!(date, None);
            assert_eq!(event_type, None);
        }
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from(["quantix", "risk", "log", "--date", "2026-03-12"]).unwrap();
    match cli.command {
        Commands::Risk(RiskCommands::Log {
            limit,
            date,
            event_type,
        }) => {
            assert_eq!(limit, 20);
            assert_eq!(date.as_deref(), Some("2026-03-12"));
            assert_eq!(event_type, None);
        }
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from(["quantix", "risk", "log", "--type", "rule-set"]).unwrap();
    match cli.command {
        Commands::Risk(RiskCommands::Log {
            limit,
            date,
            event_type,
        }) => {
            assert_eq!(limit, 20);
            assert_eq!(date, None);
            assert_eq!(event_type.as_deref(), Some("rule-set"));
        }
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from([
        "quantix",
        "risk",
        "log",
        "--date",
        "2026-03-12",
        "--type",
        "buy-lock-released",
        "--limit",
        "3",
    ])
    .unwrap();
    match cli.command {
        Commands::Risk(RiskCommands::Log {
            limit,
            date,
            event_type,
        }) => {
            assert_eq!(limit, 3);
            assert_eq!(date.as_deref(), Some("2026-03-12"));
            assert_eq!(event_type.as_deref(), Some("buy-lock-released"));
        }
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from(["quantix", "risk", "lock", "release"]).unwrap();
    match cli.command {
        Commands::Risk(RiskCommands::Lock(RiskLockCommands::Release)) => {}
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

#[tokio::test]
async fn run_risk_rule_set_dispatches_to_handler() {
    let _lock = env_lock();
    let _guard = RiskEnvGuard::capture();
    let dir = tempfile::tempdir().unwrap();
    let risk_path = dir.path().join("risk.json");
    let trade_path = dir.path().join("trade.json");
    unsafe {
        std::env::set_var("QUANTIX_RISK_PATH", &risk_path);
        std::env::set_var("QUANTIX_TRADE_PATH", &trade_path);
    }

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

    cli.run().await.unwrap();

    let state = JsonRiskStore::new(risk_path)
        .load_state()
        .await
        .unwrap()
        .unwrap();
    assert_eq!(state.rules.len(), 1);
    assert_eq!(state.rules[0].rule_type, RiskRuleType::PositionLimit);
    assert_eq!(state.rules[0].value, RuleValue::Percentage(dec!(20)));
}

#[tokio::test]
async fn run_risk_rule_set_industry_blocklist_dispatches_to_handler() {
    let _lock = env_lock();
    let _guard = RiskEnvGuard::capture();
    let dir = tempfile::tempdir().unwrap();
    let risk_path = dir.path().join("risk.json");
    let trade_path = dir.path().join("trade.json");
    unsafe {
        std::env::set_var("QUANTIX_RISK_PATH", &risk_path);
        std::env::set_var("QUANTIX_TRADE_PATH", &trade_path);
    }

    let cli = Cli::try_parse_from([
        "quantix",
        "risk",
        "rule",
        "set",
        "--type",
        "industry-blocklist",
        "--value",
        "银行,地产",
    ])
    .unwrap();

    cli.run().await.unwrap();

    let state = JsonRiskStore::new(risk_path)
        .load_state()
        .await
        .unwrap()
        .unwrap();
    assert_eq!(state.rules.len(), 1);
    assert_eq!(state.rules[0].rule_type, RiskRuleType::IndustryBlocklist);
    assert_eq!(
        state.rules[0].value,
        RuleValue::TextList(vec!["银行".to_string(), "地产".to_string()])
    );
}
