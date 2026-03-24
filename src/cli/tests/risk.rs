use super::*;
use crate::risk::{
    JsonRiskStore, RiskRule, RiskRuleType, RiskState, RiskStore, RuleValue, SqliteLiveImportStore,
};
use rust_decimal_macros::dec;
use std::fs;
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

    let cli = Cli::try_parse_from([
        "quantix",
        "risk",
        "rule",
        "set",
        "--type",
        "volatility-limit",
        "--value",
        "4%",
    ])
    .unwrap();
    match cli.command {
        Commands::Risk(RiskCommands::Rule(RiskRuleCommands::Set { rule_type, value })) => {
            assert_eq!(rule_type, "volatility-limit");
            assert_eq!(value, "4%");
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
        Commands::Risk(RiskCommands::Status {
            source,
            account,
        }) => {
            assert_eq!(source, None);
            assert_eq!(account, None);
        }
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from(["quantix", "risk", "pnl"]).unwrap();
    match cli.command {
        Commands::Risk(RiskCommands::Pnl { source, account }) => {
            assert_eq!(source, None);
            assert_eq!(account, None);
        }
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from(["quantix", "risk", "position"]).unwrap();
    match cli.command {
        Commands::Risk(RiskCommands::Position { source, account }) => {
            assert_eq!(source, None);
            assert_eq!(account, None);
        }
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from([
        "quantix",
        "risk",
        "status",
        "--source",
        "live_import",
        "--account",
        "live-001",
    ])
    .unwrap();
    match cli.command {
        Commands::Risk(RiskCommands::Status { source, account }) => {
            assert_eq!(source.as_deref(), Some("live_import"));
            assert_eq!(account.as_deref(), Some("live-001"));
        }
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from([
        "quantix",
        "risk",
        "import",
        "live-trades",
        "--account",
        "live-001",
        "--input",
        "/tmp/live.csv",
    ])
    .unwrap();
    match cli.command {
        Commands::Risk(RiskCommands::Import(RiskImportCommands::LiveTrades {
            account,
            input,
        })) => {
            assert_eq!(account, "live-001");
            assert_eq!(input, "/tmp/live.csv");
        }
        other => panic!("unexpected command: {:?}", other),
    }

    let cli = Cli::try_parse_from([
        "quantix",
        "risk",
        "rebuild",
        "live-account",
        "--account",
        "live-001",
    ])
    .unwrap();
    match cli.command {
        Commands::Risk(RiskCommands::Rebuild(RiskRebuildCommands::LiveAccount {
            account,
        })) => {
            assert_eq!(account, "live-001");
        }
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

    let err =
        Cli::try_parse_from(["quantix", "risk", "rule", "set", "--value", "20%"]).unwrap_err();
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
async fn run_risk_rule_set_volatility_limit_dispatches_to_handler() {
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
        "volatility-limit",
        "--value",
        "4%",
    ])
    .unwrap();

    cli.run().await.unwrap();

    let state = JsonRiskStore::new(risk_path)
        .load_state()
        .await
        .unwrap()
        .unwrap();
    assert_eq!(state.rules.len(), 1);
    assert_eq!(state.rules[0].rule_type, RiskRuleType::VolatilityLimit);
    assert_eq!(state.rules[0].value, RuleValue::Percentage(dec!(4)));
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

#[tokio::test]
async fn run_risk_import_rebuild_and_live_import_status_dispatch_to_handlers() {
    let _lock = env_lock();
    let _guard = RiskEnvGuard::capture();
    let dir = tempfile::tempdir().unwrap();
    let risk_path = dir.path().join("risk.json");
    let trade_path = dir.path().join("trade.json");
    let input_path = dir.path().join("live.csv");
    fs::write(
        &input_path,
        "record_type,account_id,external_id,code,side,price,volume,fee_total,business_type,amount,executed_at,occurred_at\ncash,live-001,cash-1,,,,,,deposit,100000.00,,2026-03-24T09:00:00Z\ntrade,live-001,fill-1,000001,buy,15.20,100,5.00,,,2026-03-24T09:35:00Z,\n",
    )
    .unwrap();
    unsafe {
        std::env::set_var("QUANTIX_RISK_PATH", &risk_path);
        std::env::set_var("QUANTIX_TRADE_PATH", &trade_path);
    }

    Cli::try_parse_from([
        "quantix",
        "risk",
        "import",
        "live-trades",
        "--account",
        "live-001",
        "--input",
        input_path.to_str().unwrap(),
    ])
    .unwrap()
    .run()
    .await
    .unwrap();

    Cli::try_parse_from([
        "quantix",
        "risk",
        "rebuild",
        "live-account",
        "--account",
        "live-001",
    ])
    .unwrap()
    .run()
    .await
    .unwrap();

    Cli::try_parse_from([
        "quantix",
        "risk",
        "status",
        "--source",
        "live_import",
        "--account",
        "live-001",
    ])
    .unwrap()
    .run()
    .await
    .unwrap();

    let live_import_path = risk_path.with_file_name("live_import.db");
    let store = SqliteLiveImportStore::new(&live_import_path).await.unwrap();
    let mirror = store
        .get_latest_mirror_account("live-001")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(mirror.account_id, "live-001");
    assert_eq!(mirror.positions.len(), 1);

    let risk_state = JsonRiskStore::new(&risk_path).load_state().await.unwrap();
    assert!(risk_state.is_none());
}

#[tokio::test]
async fn run_risk_live_import_status_rejects_invalid_daily_loss_rule_value_type() {
    let _lock = env_lock();
    let _guard = RiskEnvGuard::capture();
    let dir = tempfile::tempdir().unwrap();
    let risk_path = dir.path().join("risk.json");
    let trade_path = dir.path().join("trade.json");
    let input_path = dir.path().join("live.csv");
    fs::write(
        &input_path,
        "record_type,account_id,external_id,code,side,price,volume,fee_total,business_type,amount,executed_at,occurred_at\ncash,live-001,cash-1,,,,,,deposit,100000.00,,2026-03-24T09:00:00Z\ntrade,live-001,fill-1,000001,buy,15.20,100,5.00,,,2026-03-24T09:35:00Z,\n",
    )
    .unwrap();
    unsafe {
        std::env::set_var("QUANTIX_RISK_PATH", &risk_path);
        std::env::set_var("QUANTIX_TRADE_PATH", &trade_path);
    }

    Cli::try_parse_from([
        "quantix",
        "risk",
        "import",
        "live-trades",
        "--account",
        "live-001",
        "--input",
        input_path.to_str().unwrap(),
    ])
    .unwrap()
    .run()
    .await
    .unwrap();

    Cli::try_parse_from([
        "quantix",
        "risk",
        "rebuild",
        "live-account",
        "--account",
        "live-001",
    ])
    .unwrap()
    .run()
    .await
    .unwrap();

    let now = chrono::Utc::now();
    let mut state = RiskState::default();
    state.rules.push(RiskRule {
        rule_type: RiskRuleType::DailyLossLimit,
        value: RuleValue::TextList(vec!["银行".to_string()]),
        enabled: true,
        created_at: now,
        updated_at: now,
    });
    JsonRiskStore::new(&risk_path).save_state(&state).await.unwrap();

    let err = Cli::try_parse_from([
        "quantix",
        "risk",
        "status",
        "--source",
        "live_import",
        "--account",
        "live-001",
    ])
    .unwrap()
    .run()
    .await
    .unwrap_err();

    assert!(err.to_string().contains("daily-loss-limit"));
    assert!(err.to_string().contains("配置无效"));
}

#[tokio::test]
async fn run_risk_live_import_status_requires_account_flag() {
    let cli = Cli::try_parse_from([
        "quantix",
        "risk",
        "status",
        "--source",
        "live_import",
    ])
    .unwrap();

    let err = cli.run().await.unwrap_err();
    assert!(err.to_string().contains("--account"));
}
